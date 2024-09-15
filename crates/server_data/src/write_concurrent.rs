//! Write commands that can be run concurrently also with synchronous
//! write commands.

use std::{collections::HashMap, fmt, fmt::Debug, sync::Arc};

use axum::body::BodyDataStream;
use config::Config;
use futures::Future;
use model::{AccountId, AccountIdInternal, ContentProcessingId, IteratorSessionId, IteratorSessionIdInternal, ProfileLink, ReceivedLikesIteratorSessionId, ReceivedLikesIteratorSessionIdInternal};
use tokio::sync::{Mutex, OwnedMutexGuard, RwLock};

use super::{
    cache::{CacheError, DatabaseCache},
    file::utils::FileDir,
    index::LocationIndexIteratorHandle,
    IntoDataError,
};
use crate::{
    cache::received_likes::ReceivedLikesIteratorState, content_processing::NewContentInfo, db_manager::RouterDatabaseWriteHandle, result::Result, DataError
};

const PROFILE_ITERATOR_PAGE_SIZE: usize = 25;

pub type OutputFuture<R> = Box<dyn Future<Output = R> + Send + 'static>;

pub enum ConcurrentWriteAction<R> {
    Image {
        handle: ConcurrentWriteContentHandle,
        action: Box<dyn FnOnce(ConcurrentWriteContentHandle) -> OutputFuture<R> + Send + 'static>,
    },
}

pub struct AccountHandle;

#[derive(Default, Clone)]
pub struct AccountWriteLockManager {
    locks: Arc<RwLock<HashMap<AccountId, Arc<Mutex<AccountHandle>>>>>,
}

impl fmt::Debug for AccountWriteLockManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccountWriteLockManager").finish()
    }
}

impl AccountWriteLockManager {
    pub async fn lock_account(&self, a: AccountId) -> OwnedMutexGuard<AccountHandle> {
        let mutex = {
            let mut write_lock = self.locks.write().await;
            if let Some(mutex) = write_lock.get(&a) {
                mutex.clone()
            } else {
                let mutex = Arc::new(Mutex::new(AccountHandle));
                write_lock.insert(a, mutex.clone());
                mutex
            }
        };
        mutex.lock_owned().await
    }
}

#[derive(Debug)]
pub struct ConcurrentWriteCommandHandle {
    write: Arc<RouterDatabaseWriteHandle>,
    /// Content upload queue
    content_upload_queue: Arc<tokio::sync::Semaphore>,
    /// Profile index write queue
    profile_index_queue: Arc<tokio::sync::Semaphore>,
    account_write_locks: AccountWriteLockManager,
}

impl ConcurrentWriteCommandHandle {
    pub fn new(write: RouterDatabaseWriteHandle, config: &Config) -> Self {
        Self {
            write: write.into(),
            content_upload_queue: tokio::sync::Semaphore::new(config.queue_limits().content_upload)
                .into(),
            profile_index_queue: tokio::sync::Semaphore::new(num_cpus::get()).into(),
            account_write_locks: AccountWriteLockManager::default(),
        }
    }

    pub async fn accquire(&self, account: AccountId) -> ConcurrentWriteSelectorHandle {
        let lock = self.account_write_locks.lock_account(account).await;

        ConcurrentWriteSelectorHandle {
            write: self.write.clone(),
            content_upload_queue: self.content_upload_queue.clone(),
            profile_index_queue: self.profile_index_queue.clone(),
            _account_write_lock: lock,
        }
    }
}

pub struct ConcurrentWriteSelectorHandle {
    write: Arc<RouterDatabaseWriteHandle>,
    content_upload_queue: Arc<tokio::sync::Semaphore>,
    profile_index_queue: Arc<tokio::sync::Semaphore>,
    _account_write_lock: OwnedMutexGuard<AccountHandle>,
}

impl fmt::Debug for ConcurrentWriteSelectorHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConcurrentWriteSelectorHandle").finish()
    }
}

impl ConcurrentWriteSelectorHandle {
    pub async fn accquire_image<
        R,
        A: FnOnce(ConcurrentWriteContentHandle) -> OutputFuture<R> + Send + 'static,
    >(
        self,
        action: A,
    ) -> ConcurrentWriteAction<R> {
        let permit = self
            .content_upload_queue
            .clone()
            .acquire_owned()
            .await
            // Code does not call close method of Semaphore, so this should not
            // panic.
            .expect("Semaphore was closed. This should not happen.");

        let handle = ConcurrentWriteContentHandle {
            write: self.write,
            _permit: permit,
            _account_write_lock: self._account_write_lock,
        };

        ConcurrentWriteAction::Image {
            handle,
            action: Box::new(action),
        }
    }

    pub async fn profile_blocking(
        self,
    ) -> ConcurrentWriteProfileHandleBlocking {
        let permit = self
            .profile_index_queue
            .clone()
            .acquire_owned()
            .await
            // Code does not call close method of Semaphore, so this should not
            // panic.
            .expect("Semaphore was closed. This should not happen.");

        ConcurrentWriteProfileHandleBlocking {
            write: self.write,
            _permit: permit,
            _account_write_lock: self._account_write_lock,
        }
    }
}

pub struct ConcurrentWriteContentHandle {
    write: Arc<RouterDatabaseWriteHandle>,
    _permit: tokio::sync::OwnedSemaphorePermit,
    _account_write_lock: OwnedMutexGuard<AccountHandle>,
}

impl fmt::Debug for ConcurrentWriteContentHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConcurrentWriteImageHandle").finish()
    }
}

impl ConcurrentWriteContentHandle {
    pub async fn save_to_tmp(
        &self,
        id: AccountIdInternal,
        stream: BodyDataStream,
    ) -> Result<NewContentInfo, DataError> {
        self.write
            .user_write_commands_account()
            .save_to_tmp(id, stream)
            .await
    }
}

pub struct ConcurrentWriteProfileHandleBlocking {
    write: Arc<RouterDatabaseWriteHandle>,
    _permit: tokio::sync::OwnedSemaphorePermit,
    _account_write_lock: OwnedMutexGuard<AccountHandle>,
}

impl fmt::Debug for ConcurrentWriteProfileHandleBlocking {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConcurrentWriteProfileHandleBlocking").finish()
    }
}

impl ConcurrentWriteProfileHandleBlocking {
    pub fn next_profiles(
        &self,
        id: AccountIdInternal,
        iterator_id: IteratorSessionId,
    ) -> Result<Option<Vec<ProfileLink>>, DataError> {
        self.write
            .user_write_commands_account()
            .next_profiles(id, iterator_id)
    }

    pub fn reset_profile_iterator(&self, id: AccountIdInternal) -> Result<IteratorSessionIdInternal, DataError> {
        self.write
            .user_write_commands_account()
            .reset_profile_iterator(id)
    }

    pub fn next_received_likes_iterator_state(
        &self,
        id: AccountIdInternal,
        iterator_id: ReceivedLikesIteratorSessionId,
    ) -> Result<Option<ReceivedLikesIteratorState>, DataError> {
        self.write
            .user_write_commands_account()
            .next_received_likes_iterator_state(id, iterator_id)
    }

    pub fn reset_received_likes_iterator(&self, id: AccountIdInternal) -> Result<ReceivedLikesIteratorSessionIdInternal, DataError> {
        self.write
            .user_write_commands_account()
            .reset_received_likes_iterator(id)
    }
}

/// Commands that can run concurrently with other write commands, but which have
/// limitation that one account can execute only one command at a time.
/// It possible to run this and normal write command concurrently for
/// one account.
pub struct WriteCommandsConcurrent<'a> {
    cache: &'a DatabaseCache,
    file_dir: &'a FileDir,
    location: LocationIndexIteratorHandle<'a>,
}

impl<'a> WriteCommandsConcurrent<'a> {
    pub fn new(
        cache: &'a DatabaseCache,
        file_dir: &'a FileDir,
        location: LocationIndexIteratorHandle<'a>,
    ) -> Self {
        Self {
            cache,
            file_dir,
            location,
        }
    }

    pub async fn save_to_tmp(
        &self,
        id: AccountIdInternal,
        stream: BodyDataStream,
    ) -> Result<NewContentInfo, DataError> {
        let content_id = ContentProcessingId::new_random_id();

        // Clear tmp dir in case previous content writing failed and there is no
        // content ID in the database about it.
        self.file_dir
            .tmp_dir(id.as_id())
            .remove_contents_if_exists()
            .await?;

        let tmp_raw_img = self
            .file_dir
            .raw_content_upload(id.as_id(), content_id.to_content_id());
        tmp_raw_img.save_stream(stream).await?;

        let tmp_img = self
            .file_dir
            .processed_content_upload(id.as_id(), content_id.to_content_id());

        Ok(NewContentInfo {
            processing_id: content_id,
            tmp_raw_img,
            tmp_img,
        })
    }

    /// Returns None if profile iterator session ID is
    /// invalid.
    pub fn next_profiles(
        &self,
        id: AccountIdInternal,
        iterator_id_from_client: IteratorSessionId,
    ) -> Result<Option<Vec<ProfileLink>>, DataError> {
        let (location, query_maker_filters, iterator_id_current) = self
            .cache
            .read_cache_blocking(id.as_id(), |e| {
                let p = e.profile.as_ref().ok_or(CacheError::FeatureNotEnabled)?;
                error_stack::Result::<_, CacheError>::Ok((p.location.clone(), p.filters(), p.profile_iterator_session_id))
            })
            .into_data_error(id)??;

        let iterator_id_current: Option<IteratorSessionId> =
            iterator_id_current.map(|v| v.into());
        if iterator_id_current != Some(iterator_id_from_client) {
            return Ok(None);
        }

        let (mut next_state, profiles) = self
            .location
            .next_profiles(location.current_iterator, &query_maker_filters);

        let (next_state, profiles) = if let Some(mut profiles) = profiles {
            loop {
                if profiles.len() >= PROFILE_ITERATOR_PAGE_SIZE {
                    break (next_state, profiles);
                } else {
                    let (new_next_state, new_profiles) = self
                        .location
                        .next_profiles(next_state, &query_maker_filters);
                    next_state = new_next_state;

                    if let Some(new_profiles) = new_profiles {
                        profiles.extend(new_profiles);
                    } else {
                        break (next_state, profiles);
                    }
                }
            }
        } else {
            (next_state, vec![])
        };

        self.cache
            .write_cache_blocking(id.as_id(), |e| {
                if let Some(p) = e.profile.as_mut() {
                    p.location.current_iterator = next_state;
                }
                Ok(())
            })
            .into_data_error(id)?;

        Ok(Some(profiles))
    }

    pub fn reset_profile_iterator(
        &self,
        id: AccountIdInternal,
    ) -> Result<IteratorSessionIdInternal, DataError> {
        self.cache
            .write_cache_blocking(id.as_id(), |e| {
                let new_id = IteratorSessionIdInternal::create_random();
                if let Some(p) = e.profile.as_mut() {
                    let next_state = self
                        .location
                        .reset_iterator(p.location.current_iterator, p.location.current_position);
                    p.location.current_iterator = next_state;
                    p.profile_iterator_session_id = Some(new_id);
                }
                Ok(new_id)
            })
            .into_data_error(id)
    }

    pub fn next_received_likes_iterator_state(
        &self,
        id: AccountIdInternal,
        iterator_session_id: ReceivedLikesIteratorSessionId,
    ) -> Result<Option<ReceivedLikesIteratorState>, DataError> {
        self.cache
            .write_cache_blocking(id.as_id(), |e| {
                if let Some(c) = e.chat.as_mut() {
                    Ok(c.received_likes_iterator.get_and_increment(iterator_session_id))
                } else {
                    Err(CacheError::FeatureNotEnabled.report())
                }
            })
            .into_data_error(id)
    }

    pub fn reset_received_likes_iterator(
        &self,
        id: AccountIdInternal,
    ) -> Result<ReceivedLikesIteratorSessionIdInternal, DataError> {
        self.cache
            .write_cache_blocking(id.as_id(), |e| {
                if let Some(c) = e.chat.as_mut() {
                    Ok(c.received_likes_iterator.reset())
                } else {
                    Err(CacheError::FeatureNotEnabled.report())
                }
            })
            .into_data_error(id)
    }
}

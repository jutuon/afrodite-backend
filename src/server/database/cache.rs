use std::{collections::HashMap, future::Future, hash::Hash, sync::Arc};

use async_trait::async_trait;
use tokio::sync::{Mutex, MutexGuard, RwLock};
use tokio_stream::StreamExt;
use tracing::info;

use crate::{
    api::model::{Account, AccountIdInternal, AccountIdLight, AccountSetup, ApiKey, Profile},
    config::Config,
};

use error_stack::{Result, ResultExt};
use serde::Serialize;

use super::{
    current::read::SqliteReadCommands,
    sqlite::{SqliteDatabaseError, SqliteSelectJson},
    write::AccountWriteLock,
    DatabaseError,
};

#[derive(thiserror::Error, Debug)]
pub enum CacheError {
    #[error("Key already exists")]
    AlreadyExists,

    #[error("Key not exists")]
    KeyNotExists,

    #[error("Data is not in cache")]
    NotInCache,

    #[error("Cache init error")]
    Init,
}

pub struct AccountEntry {
    pub lock: Arc<Mutex<AccountWriteLock>>,
    pub cache: RwLock<CacheEntry>,
}

pub struct DatabaseCache {
    /// Accounts which are logged in.
    api_keys: RwLock<HashMap<ApiKey, Arc<AccountEntry>>>,
    /// All accounts registered in the service.
    accounts: RwLock<HashMap<AccountIdLight, Arc<AccountEntry>>>,
}

impl DatabaseCache {
    pub async fn new(read: SqliteReadCommands<'_>, config: &Config) -> Result<Self, CacheError> {
        let cache = Self {
            api_keys: RwLock::new(HashMap::new()),
            accounts: RwLock::new(HashMap::new()),
        };

        // Load data from database to memory.
        info!("Starting to load data from database to memory");

        let mut accounts = read.account_ids_stream();

        while let Some(id) = accounts.next().await {
            let id = id.change_context(CacheError::Init)?;
            cache.insert_account_if_not_exists(id).await?;
        }

        let read_account = cache.accounts.read().await;
        let ids = read_account.values();
        for lock_and_cache in ids {
            let mut entry = lock_and_cache.cache.write().await;
            let internal_id = entry.account_id_internal;

            let api_key = read
                .api_key(entry.account_id_internal)
                .await
                .change_context(CacheError::Init)?;

            if let Some(key) = api_key {
                let mut write_api_keys = cache.api_keys.write().await;
                if write_api_keys.contains_key(&key) {
                    return Err(CacheError::AlreadyExists.into()).change_context(CacheError::Init);
                } else {
                    write_api_keys.insert(key, lock_and_cache.clone());
                }
            }

            if config.components().account {
                let account = Account::select_json(internal_id, &read)
                    .await
                    .change_context(CacheError::Init)?;
                entry.account = Some(account.clone().into())
            }

            if config.components().profile {
                let profile = Profile::select_json(internal_id, &read)
                    .await
                    .change_context(CacheError::Init)?;
                entry.profile = Some(profile.clone().into());
            }
        }

        info!("Loading to memory complete");

        drop(read_account);
        Ok(cache)
    }

    pub async fn insert_account_if_not_exists(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), CacheError> {
        let mut data = self.accounts.write().await;
        if data.get(&id.as_light()).is_none() {
            let lock = Arc::new(Mutex::new(AccountWriteLock));
            let value = RwLock::new(CacheEntry::new(id));
            data.insert(id.as_light(), AccountEntry { lock, cache: value }.into());
            Ok(())
        } else {
            Err(CacheError::AlreadyExists.into())
        }
    }

    pub async fn update_api_key(
        &self,
        id: AccountIdLight,
        api_key: ApiKey,
    ) -> Result<(), CacheError> {
        let cache_entry = self
            .accounts
            .read()
            .await
            .get(&id)
            .ok_or(CacheError::KeyNotExists)?
            .clone();

        let mut api_key_guard = self.api_keys.write().await;
        if api_key_guard.get(&api_key).is_none() {
            api_key_guard.insert(api_key, cache_entry);
            Ok(())
        } else {
            Err(CacheError::AlreadyExists.into())
        }
    }

    pub async fn delete_api_key(&self, api_key: ApiKey) -> Result<(), CacheError> {
        let mut guard = self.api_keys.write().await;
        guard.remove(&api_key).ok_or(CacheError::KeyNotExists)?;
        Ok(())
    }

    pub async fn api_key_exists(&self, api_key: &ApiKey) -> Option<AccountIdInternal> {
        let api_key_guard = self.api_keys.read().await;
        if let Some(entry) = api_key_guard.get(api_key) {
            Some(entry.cache.read().await.account_id_internal)
        } else {
            None
        }
    }

    pub async fn api_key_exists_with_account_lock(
        &self,
        api_key: &ApiKey,
    ) -> Option<(AccountIdInternal, Arc<Mutex<AccountWriteLock>>)> {
        let api_key_guard = self.api_keys.read().await;
        if let Some(entry) = api_key_guard.get(api_key) {
            let id = entry.cache.read().await.account_id_internal;
            Some((id, entry.lock.clone()))
        } else {
            None
        }
    }

    pub async fn to_account_id_internal(
        &self,
        id: AccountIdLight,
    ) -> Result<AccountIdInternal, CacheError> {
        let guard = self.accounts.read().await;
        let data = guard
            .get(&id)
            .ok_or(CacheError::KeyNotExists)?
            .cache
            .read()
            .await
            .account_id_internal;
        Ok(data)
    }

    pub async fn read_cache<T>(
        &self,
        id: AccountIdLight,
        cache_operation: impl Fn(&CacheEntry) -> T,
    ) -> Result<T, CacheError> {
        let guard = self.accounts.read().await;
        let cache_entry = guard
            .get(&id)
            .ok_or(CacheError::KeyNotExists)?
            .cache
            .read()
            .await;
        Ok(cache_operation(&cache_entry))
    }

    pub async fn write_cache<T>(
        &self,
        id: AccountIdLight,
        cache_operation: impl Fn(&mut CacheEntry) -> T,
    ) -> Result<T, CacheError> {
        let guard = self.accounts.read().await;
        let mut cache_entry = guard
            .get(&id)
            .ok_or(CacheError::KeyNotExists)?
            .cache
            .write()
            .await;
        Ok(cache_operation(&mut cache_entry))
    }

    pub async fn account(&self, id: AccountIdLight) -> Result<Account, CacheError> {
        let guard = self.accounts.read().await;
        let data = guard
            .get(&id)
            .ok_or(CacheError::KeyNotExists)?
            .cache
            .read()
            .await
            .account
            .as_ref()
            .map(|data| data.as_ref().clone())
            .ok_or(CacheError::NotInCache)?;

        Ok(data)
    }

    pub async fn profile(&self, id: AccountIdLight) -> Result<Profile, CacheError> {
        let guard = self.accounts.read().await;
        let data = guard
            .get(&id)
            .ok_or(CacheError::KeyNotExists)?
            .cache
            .read()
            .await
            .profile
            .as_ref()
            .map(|data| data.as_ref().clone())
            .ok_or(CacheError::NotInCache)?;

        Ok(data)
    }

    pub async fn update_profile(
        &self,
        id: AccountIdLight,
        profile: Profile,
    ) -> Result<(), CacheError> {
        let mut data = self.accounts.write().await;
        data.get_mut(&id)
            .ok_or(CacheError::KeyNotExists)?
            .cache
            .write()
            .await
            .profile
            .as_mut()
            .ok_or(CacheError::NotInCache)
            .map(|data| *data.as_mut() = profile)?;
        Ok(())
    }

    pub async fn update_account(
        &self,
        id: AccountIdLight,
        data: Account,
    ) -> Result<(), CacheError> {
        let mut write_guard = self.accounts.write().await;
        write_guard
            .get_mut(&id)
            .ok_or(CacheError::KeyNotExists)?
            .cache
            .write()
            .await
            .account
            .as_mut()
            .ok_or(CacheError::NotInCache)
            .map(|current_data| *current_data.as_mut() = data)?;
        Ok(())
    }

    pub async fn get_write_lock_simple(
        &self,
        id: AccountIdLight,
    ) -> Result<Arc<Mutex<AccountWriteLock>>, CacheError> {
        let guard = self.accounts.read().await;
        let lock = guard.get(&id).ok_or(CacheError::KeyNotExists)?.lock.clone();

        Ok(lock)
    }
}

pub struct CacheEntry {
    pub account_id_internal: AccountIdInternal,
    pub profile: Option<Box<Profile>>,
    pub account: Option<Box<Account>>,
}

impl CacheEntry {
    pub fn new(account_id_internal: AccountIdInternal) -> Self {
        Self {
            profile: None,
            account: None,
            account_id_internal,
        }
    }
}

#[async_trait]
pub trait ReadCacheJson: Sized + Send {
    const CACHED_JSON: bool = false;

    async fn read_from_cache(
        id: AccountIdLight,
        cache: &DatabaseCache,
    ) -> Result<Self, CacheError> {
        Err(CacheError::NotInCache.into())
    }
}

impl ReadCacheJson for AccountSetup {}

#[async_trait]
impl ReadCacheJson for Account {
    const CACHED_JSON: bool = true;

    async fn read_from_cache(
        id: AccountIdLight,
        cache: &DatabaseCache,
    ) -> Result<Self, CacheError> {
        let data_in_cache = cache
            .read_cache(id, |entry| {
                entry
                    .account
                    .as_ref()
                    .map(|account| account.as_ref().clone())
            })
            .await?;
        data_in_cache.ok_or(CacheError::NotInCache.into())
    }
}

#[async_trait]
impl ReadCacheJson for Profile {
    const CACHED_JSON: bool = true;

    async fn read_from_cache(
        id: AccountIdLight,
        cache: &DatabaseCache,
    ) -> Result<Self, CacheError> {
        let data_in_cache = cache
            .read_cache(id, |entry| {
                entry.profile.as_ref().map(|data| data.as_ref().clone())
            })
            .await?;
        data_in_cache.ok_or(CacheError::NotInCache.into())
    }
}

#[async_trait]
pub trait WriteCacheJson: Sized + Send + ReadCacheJson {
    async fn write_to_cache(
        &self,
        id: AccountIdLight,
        cache: &DatabaseCache,
    ) -> Result<(), CacheError> {
        Err(CacheError::NotInCache.into())
    }
}

impl WriteCacheJson for AccountSetup {}

#[async_trait]
impl WriteCacheJson for Account {
    async fn write_to_cache(
        &self,
        id: AccountIdLight,
        cache: &DatabaseCache,
    ) -> Result<(), CacheError> {
        cache
            .write_cache(id, |entry| {
                entry
                    .account
                    .as_mut()
                    .map(|data| *data.as_mut() = self.clone())
            })
            .await
            .map(|_| ())
    }
}

#[async_trait]
impl WriteCacheJson for Profile {
    async fn write_to_cache(
        &self,
        id: AccountIdLight,
        cache: &DatabaseCache,
    ) -> Result<(), CacheError> {
        cache
            .write_cache(id, |entry| {
                entry
                    .profile
                    .as_mut()
                    .map(|data| *data.as_mut() = self.clone())
            })
            .await
            .map(|_| ())
    }
}

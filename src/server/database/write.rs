
use std::{time::Duration};


use axum::extract::BodyStream;
use error_stack::{Report, Result, ResultExt};
use serde::Serialize;

use tokio_stream::StreamExt;


use crate::{
    api::{
        media::data::Moderation,
        model::{
            Account, AccountIdInternal, AccountIdLight, AccountSetup, ApiKey, ContentId,
            NewModerationRequest, Profile,
        },
    },
    config::Config,
    server::database::{DatabaseError},
    utils::{ErrorConversion},
};

use super::{
    cache::{DatabaseCache, WriteCacheJson},
    current::write::CurrentDataWriteCommands,
    file::{file::ImageSlot, utils::FileDir},
    history::write::HistoryWriteCommands,
    sqlite::{CurrentDataWriteHandle, HistoryUpdateJson, HistoryWriteHandle, SqliteUpdateJson},
    utils::GetReadWriteCmd,
};

#[derive(Debug, Clone)]
pub enum WriteCmd {
    AccountId(AccountIdLight),
    Profile(AccountIdInternal),
    ApiKey(AccountIdInternal),
    AccountState(AccountIdInternal),
    AccountSetup(AccountIdInternal),
    MediaModerationRequest(AccountIdInternal),
    MediaModeration(AccountIdInternal),
}

impl std::fmt::Display for WriteCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Write command: {:?}", self))
    }
}

#[derive(Debug, Clone)]
pub struct HistoryWrite(pub WriteCmd);

impl std::fmt::Display for HistoryWrite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("History write command: {:?}", self))
    }
}

#[derive(Debug, Clone)]
pub struct CacheWrite(pub WriteCmd);

impl std::fmt::Display for CacheWrite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Cache write command: {:?}", self))
    }
}

// TODO: If one commands does multiple writes to database, move writes to happen
// in a transaction.

// TODO: When server starts, check that latest history data matches with current
// data.

/// One Account can do only one write command at a time.
pub struct AccountWriteLock;

/// Globally synchronous write commands.
pub struct WriteCommands<'a> {
    current_write: &'a CurrentDataWriteHandle,
    history_write: &'a HistoryWriteHandle,
    cache: &'a DatabaseCache,
    file_dir: &'a FileDir,
}

impl<'a> WriteCommands<'a> {
    pub fn new(
        current_write: &'a CurrentDataWriteHandle,
        history_write: &'a HistoryWriteHandle,
        cache: &'a DatabaseCache,
        file_dir: &'a FileDir,
    ) -> Self {
        Self {
            current_write,
            history_write,
            cache,
            file_dir,
        }
    }

    pub async fn register(
        id_light: AccountIdLight,
        config: &Config,
        current_data_write: CurrentDataWriteHandle,
        history_wirte: HistoryWriteHandle,
        cache: &DatabaseCache,
    ) -> Result<AccountIdInternal, DatabaseError> {

        let current = CurrentDataWriteCommands::new(&current_data_write);
        let history = HistoryWriteCommands::new(&history_wirte);

        let account = Account::default();
        let account_setup = AccountSetup::default();
        let profile = Profile::default();

        // TODO: Use transactions here. One for current and other for history.

        let id = current
            .store_account_id(id_light)
            .await
            .with_info_lazy(|| WriteCmd::AccountId(id_light))?;

        history
            .store_account_id(id)
            .await
            .with_info_lazy(|| HistoryWrite(WriteCmd::AccountId(id_light)))?;

        cache
            .insert_account_if_not_exists(id)
            .await
            .with_info_lazy(|| CacheWrite(WriteCmd::AccountId(id_light)))?;

        current
            .store_api_key(id, None)
            .await
            .with_info_lazy(|| WriteCmd::ApiKey(id))?;

        if config.components().account {
            current
                .store_account(id, &account)
                .await
                .with_write_cmd_info::<Account>(id)?;

            history
                .store_account(id, &account)
                .await
                .with_history_write_cmd_info::<Account>(id)?;

            cache
                .write_cache(id.as_light(), |cache| {
                    cache.account = Some(account.clone().into())
                })
                .await
                .with_history_write_cmd_info::<Account>(id)?;

            current
                .store_account_setup(id, &account_setup)
                .await
                .with_write_cmd_info::<AccountSetup>(id)?;

            history
                .store_account_setup(id, &account_setup)
                .await
                .with_history_write_cmd_info::<AccountSetup>(id)?;
        }

        if config.components().profile {
            current
                .store_profile(id, &profile)
                .await
                .with_write_cmd_info::<Profile>(id)?;

            history
                .store_profile(id, &profile)
                .await
                .with_history_write_cmd_info::<Profile>(id)?;

            cache
                .write_cache(id.as_light(), |cache| {
                    cache.profile = Some(profile.clone().into())
                })
                .await
                .with_history_write_cmd_info::<Profile>(id)?;
        }

        Ok(id)
    }

    pub async fn set_new_api_key(
        &self,
        id: AccountIdInternal,
        key: ApiKey,
    ) -> Result<(), DatabaseError> {
        self.current()
            .update_api_key(id, Some(&key))
            .await
            .with_info_lazy(|| WriteCmd::AccountId(id.as_light()))?;

        self.cache
            .update_api_key(id.as_light(), key)
            .await
            .with_info_lazy(|| WriteCmd::AccountId(id.as_light()))
    }

    pub async fn update_json<
        T: GetReadWriteCmd
            + Serialize
            + Clone
            + Send
            + SqliteUpdateJson
            + HistoryUpdateJson
            + WriteCacheJson
            + Sync
            + 'static,
    >(
        &mut self,
        id: AccountIdInternal,
        data: &T,
    ) -> Result<(), DatabaseError> {
        data.update_json(id, &self.current())
            .await
            .with_write_cmd_info::<T>(id)?;

        data.history_update_json(id, &self.history())
            .await
            .with_history_write_cmd_info::<T>(id)?;

        if T::CACHED_JSON {
            data.write_to_cache(id.as_light(), &self.cache)
                .await
                .with_cache_write_cmd_info::<T>(id)
        } else {
            Ok(())
        }
    }

    pub async fn set_moderation_request(
        &self,
        account_id: AccountIdInternal,
        request: NewModerationRequest,
    ) -> Result<(), DatabaseError> {
        self.current()
            .media()
            .create_new_moderation_request(account_id, request)
            .await
            .with_info_lazy(|| WriteCmd::MediaModerationRequest(account_id))
    }

    pub async fn moderation_get_list_and_create_new_if_necessary(
        self,
        account_id: AccountIdInternal,
    ) -> Result<Vec<Moderation>, DatabaseError> {
        self.current()
            .media()
            .moderation_get_list_and_create_new_if_necessary(account_id)
            .await
            .with_info_lazy(|| WriteCmd::MediaModeration(account_id))
    }

    /// Completes previous sava_to_tmp.
    pub async fn save_to_slot(
        &self,
        id: AccountIdInternal,
        content_id: ContentId,
        slot: ImageSlot,
    ) -> Result<(), DatabaseError> {
        // Remove previous slot image.
        let current_content_in_slot = self
            .current_write
            .read()
            .media()
            .get_content_id_from_slot(id, slot)
            .await
            .change_context(DatabaseError::Sqlite)?;
        if let Some(current_id) = current_content_in_slot {
            let path = self
                .file_dir
                .image_content(id.as_light(), current_id.as_content_id());
            path.remove_if_exists()
                .await
                .change_context(DatabaseError::File)?;
            self.current()
                .media()
                .delete_image_from_slot(id, slot)
                .await
                .change_context(DatabaseError::Sqlite)?;
        }

        let transaction = self
            .current()
            .media()
            .store_content_id_to_slot(id, content_id, slot)
            .await
            .change_context(DatabaseError::Sqlite)?;

        let file_operations = || {
            async {
                // Move image from tmp to image dir
                let raw_img = self
                    .file_dir
                    .unprocessed_image_upload(id.as_light(), content_id);
                let processed_content_path = self.file_dir.image_content(id.as_light(), content_id);
                raw_img
                    .move_to(&processed_content_path)
                    .await
                    .change_context(DatabaseError::File)?;

                Ok::<(), Report<DatabaseError>>(())
            }
        };

        match file_operations().await {
            Ok(()) => transaction
                .commit()
                .await
                .change_context(DatabaseError::Sqlite),
            Err(e) => {
                match transaction
                    .rollback()
                    .await
                    .change_context(DatabaseError::Sqlite)
                {
                    Ok(()) => Err(e),
                    Err(another_error) => Err(another_error.attach(e)),
                }
            }
        }
    }

    fn current(&self) -> CurrentDataWriteCommands {
        CurrentDataWriteCommands::new(&self.current_write)
    }

    fn history(&self) -> HistoryWriteCommands {
        HistoryWriteCommands::new(&self.history_write)
    }
}

/// Commands that can run concurrently with other write commands, but which have
/// limitation that one account can execute only one command at a time.
/// It possible to run this and normal write command concurrently for
/// one account.
pub struct WriteCommandsAccount<'a> {
    current_write: &'a CurrentDataWriteHandle,
    history_write: &'a HistoryWriteHandle,
    cache: &'a DatabaseCache,
    file_dir: &'a FileDir,
}

impl<'a> WriteCommandsAccount<'a> {
    pub fn new(
        current_write: &'a CurrentDataWriteHandle,
        history_write: &'a HistoryWriteHandle,
        cache: &'a DatabaseCache,
        file_dir: &'a FileDir,
    ) -> Self {
        Self {
            current_write,
            history_write,
            cache,
            file_dir,
        }
    }

    pub async fn save_to_tmp(
        &self,
        id: AccountIdInternal,
        stream: BodyStream,
    ) -> Result<ContentId, DatabaseError> {
        let content_id = ContentId::new_random_id();

        // Clear tmp dir if previous image writing failed and there is no
        // content ID in the database about it.
        self.file_dir
            .tmp_dir(id.as_light())
            .remove_contents_if_exists()
            .await
            .change_context(DatabaseError::File)?;

        let raw_img = self
            .file_dir
            .unprocessed_image_upload(id.as_light(), content_id);
        raw_img
            .save_stream(stream)
            .await
            .change_context(DatabaseError::File)?;

        // TODO: image safety checks and processing

        Ok(content_id)
    }

    fn current(&self) -> CurrentDataWriteCommands {
        CurrentDataWriteCommands::new(&self.current_write)
    }

    fn history(&self) -> HistoryWriteCommands {
        HistoryWriteCommands::new(&self.history_write)
    }
}

use std::{fmt::Debug, marker::PhantomData};

use database::{
    current::read::{CurrentSyncReadCommands, SqliteReadCommands},
    diesel::{DieselCurrentReadHandle, DieselDatabaseError, DieselConnection},
    sqlite::{SqlxReadHandle},
};
use error_stack::{Result, ResultExt};
use model::{
    AccountIdInternal, AccountId, ContentId, MediaContentInternal, ModerationRequest,
};
use tokio_util::io::ReaderStream;
use utils::{IntoReportExt, IntoReportFromString};

use self::{
    account::ReadCommandsAccount, account_admin::ReadCommandsAccountAdmin, chat::ReadCommandsChat,
    chat_admin::ReadCommandsChatAdmin, media::ReadCommandsMedia,
    media_admin::ReadCommandsMediaAdmin, profile::ReadCommandsProfile,
    profile_admin::ReadCommandsProfileAdmin,
};
use super::{
    cache::{DatabaseCache},
    file::utils::FileDir,
    DatabaseError, IntoDataError,
};
use crate::utils::{ErrorConversion};

macro_rules! define_read_commands {
    ($struct_name:ident) => {
        pub struct $struct_name<'a> {
            cmds: ReadCommands<'a>,
        }

        impl<'a> $struct_name<'a> {
            pub fn new(cmds: ReadCommands<'a>) -> Self {
                Self { cmds }
            }

            #[allow(dead_code)]
            fn db(&self) -> &database::current::read::SqliteReadCommands<'_> {
                &self.cmds.db
            }

            #[allow(dead_code)]
            fn cache(&self) -> &DatabaseCache {
                &self.cmds.cache
            }

            #[allow(dead_code)]
            fn files(&self) -> &FileDir {
                &self.cmds.files
            }

            #[track_caller]
            pub async fn db_read<
                T: FnOnce(
                        database::current::read::CurrentSyncReadCommands<&mut database::diesel::DieselConnection>,
                    )
                        -> error_stack::Result<R, database::diesel::DieselDatabaseError>
                    + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, crate::data::DatabaseError> {
                self.cmds.db_read(cmd).await
            }

            #[track_caller]
            pub async fn read_cache<T, Id: Into<model::AccountId>>(
                &self,
                id: Id,
                cache_operation: impl Fn(&crate::data::cache::CacheEntry) -> T,
            ) -> error_stack::Result<T, crate::data::DatabaseError> {
                use error_stack::ResultExt;
                self.cache()
                    .read_cache(id, cache_operation)
                    .await
                    .change_context(crate::data::DatabaseError::Cache)
            }
        }
    };
}

pub mod account;
pub mod account_admin;
pub mod chat;
pub mod chat_admin;
pub mod media;
pub mod media_admin;
pub mod profile;
pub mod profile_admin;

pub struct ReadCommands<'a> {
    db: SqliteReadCommands<'a>,
    diesel_current_read: &'a DieselCurrentReadHandle,
    cache: &'a DatabaseCache,
    files: &'a FileDir,
}

impl<'a> ReadCommands<'a> {
    pub fn new(
        sqlite: &'a SqlxReadHandle,
        cache: &'a DatabaseCache,
        files: &'a FileDir,
        diesel_current_read: &'a DieselCurrentReadHandle,
    ) -> Self {
        Self {
            db: SqliteReadCommands::new(sqlite),
            diesel_current_read,
            cache,
            files,
        }
    }

    pub fn account(self) -> ReadCommandsAccount<'a> {
        ReadCommandsAccount::new(self)
    }

    pub fn account_admin(self) -> ReadCommandsAccountAdmin<'a> {
        ReadCommandsAccountAdmin::new(self)
    }

    pub fn media(self) -> ReadCommandsMedia<'a> {
        ReadCommandsMedia::new(self)
    }

    pub fn media_admin(self) -> ReadCommandsMediaAdmin<'a> {
        ReadCommandsMediaAdmin::new(self)
    }

    pub fn profile(self) -> ReadCommandsProfile<'a> {
        ReadCommandsProfile::new(self)
    }

    pub fn profile_admin(self) -> ReadCommandsProfileAdmin<'a> {
        ReadCommandsProfileAdmin::new(self)
    }

    pub fn chat(self) -> ReadCommandsChat<'a> {
        ReadCommandsChat::new(self)
    }

    pub fn chat_admin(self) -> ReadCommandsChatAdmin<'a> {
        ReadCommandsChatAdmin::new(self)
    }

    pub async fn image_stream(
        &self,
        account_id: AccountId,
        content_id: ContentId,
    ) -> Result<ReaderStream<tokio::fs::File>, DatabaseError> {
        self.files
            .image_content(account_id, content_id)
            .read_stream()
            .await
            .into_data_error((account_id, content_id))
    }

    pub async fn all_account_media(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<Vec<MediaContentInternal>, DatabaseError> {
        self.db_read(move |mut cmds| cmds.media().get_account_media(account_id))
            .await
    }

    pub async fn moderation_request(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<Option<ModerationRequest>, DatabaseError> {
        self.db_read(move |mut cmds| cmds.media().moderation_request(account_id))
            .await
            .map(|r| r.map(|request| request.into_request()))
    }

    pub async fn profile_visibility(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<Option<bool>, DatabaseError> {
        self.cache
            .read_cache(account_id.as_light(), |e| {
                e.profile.as_ref().map(|p| p.public).flatten()
            })
            .await
            .change_context(DatabaseError::Cache)
    }

    #[track_caller]
    pub async fn db_read<
        T: FnOnce(CurrentSyncReadCommands<&mut DieselConnection>) -> Result<R, DieselDatabaseError> + Send + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> Result<R, DatabaseError> {
        let conn = self
            .diesel_current_read
            .pool()
            .get()
            .await
            .into_error(DieselDatabaseError::GetConnection)
            .change_context(DatabaseError::Diesel)?;

        conn.interact(move |conn| cmd(CurrentSyncReadCommands::new(conn)))
            .await
            .into_error_string(DieselDatabaseError::Execute)
            .change_context(DatabaseError::Diesel)?
            .change_context(DatabaseError::Diesel)
    }
}

use error_stack::Result;
use model::{AccountIdInternal, AccountState};
use time::OffsetDateTime;
use tokio_stream::{Stream, StreamExt};
use utils::IntoReportExt;

use super::{
    super::sqlite::{SqliteDatabaseError},
    HistoryData,
};

use self::{
    account::{HistoryReadAccount, HistorySyncReadAccount},
    account_admin::HistorySyncReadAccountAdmin,
    chat::{HistoryReadChat, HistorySyncReadChat},
    chat_admin::HistorySyncReadChatAdmin,
    media::{HistoryReadMedia, HistorySyncReadMedia},
    media_admin::HistorySyncReadMediaAdmin,
    profile::{HistoryReadProfile, HistorySyncReadProfile},
    profile_admin::HistorySyncReadProfileAdmin,
};

use crate::{diesel::{DieselConnection, ConnectionProvider}, sqlite::SqlxReadHandle};

macro_rules! define_read_commands {
    ($struct_name:ident, $sync_name:ident) => {
        pub struct $struct_name<'a> {
            cmds: &'a crate::history::read::HistoryReadCommands<'a>,
        }

        impl<'a> $struct_name<'a> {
            pub fn new(cmds: &'a crate::history::read::HistoryReadCommands<'a>) -> Self {
                Self { cmds }
            }

            pub fn pool(&self) -> &'a sqlx::SqlitePool {
                self.cmds.handle.pool()
            }
        }

        pub struct $sync_name<C: crate::diesel::ConnectionProvider> {
            cmds: C,
        }

        impl<C: crate::diesel::ConnectionProvider> $sync_name<C> {
            pub fn new(cmds: C) -> Self {
                Self { cmds }
            }

            pub fn conn(&mut self) -> &mut crate::diesel::DieselConnection {
                self.cmds.conn()
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


pub struct HistoryReadCommands<'a> {
    handle: &'a SqlxReadHandle,
}

impl<'a> HistoryReadCommands<'a> {
    pub fn new(handle: &'a SqlxReadHandle) -> Self {
        Self { handle }
    }
}

pub struct HistorySyncReadCommands<C: ConnectionProvider> {
    conn: C,
}

impl<C: ConnectionProvider> HistorySyncReadCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    pub fn into_account(self) -> HistorySyncReadAccount<C> {
        HistorySyncReadAccount::new(self.conn)
    }

    pub fn into_account_admin(self) -> HistorySyncReadAccountAdmin<C> {
        HistorySyncReadAccountAdmin::new(self.conn)
    }

    pub fn into_media(self) -> HistorySyncReadMedia<C> {
        HistorySyncReadMedia::new(self.conn)
    }

    pub fn into_media_admin(self) -> HistorySyncReadMediaAdmin<C> {
        HistorySyncReadMediaAdmin::new(self.conn)
    }

    pub fn into_profile(self) -> HistorySyncReadProfile<C> {
        HistorySyncReadProfile::new(self.conn)
    }

    pub fn into_profile_admin(self) -> HistorySyncReadProfileAdmin<C> {
        HistorySyncReadProfileAdmin::new(self.conn)
    }

    pub fn into_chat(self) -> HistorySyncReadChat<C> {
        HistorySyncReadChat::new(self.conn)
    }

    pub fn into_chat_admin(self) -> HistorySyncReadChatAdmin<C> {
        HistorySyncReadChatAdmin::new(self.conn)
    }
}

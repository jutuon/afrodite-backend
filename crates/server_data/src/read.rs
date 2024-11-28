use crate::db_manager::{InternalReading, ReadAccessProvider};

use self::common::ReadCommandsCommon;

pub mod common;

pub trait GetReadCommandsCommon<'a> {
    fn common(self) -> ReadCommandsCommon<'a>;
}

impl <'a, C: ReadAccessProvider<'a>> GetReadCommandsCommon<'a> for C {
    fn common(self) -> ReadCommandsCommon<'a> {
        ReadCommandsCommon::new(self.handle())
    }
}

pub trait DbReadCommon {
    async fn db_read<
        T: FnOnce(
                database::DbReadMode<'_>,
            ) -> error_stack::Result<R, database::DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, database::DieselDatabaseError>;
}

impl <I: InternalReading> DbReadCommon for I {
    async fn db_read<
        T: FnOnce(
                database::DbReadMode<'_>,
            ) -> error_stack::Result<R, database::DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, database::DieselDatabaseError> {
        self.db_read_raw(cmd).await
    }
}

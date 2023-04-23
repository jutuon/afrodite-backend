
use async_trait::async_trait;
use error_stack::Result;
use tokio_stream::{Stream, StreamExt};


use crate::server::database::current::SqliteReadCommands;
use crate::server::database::sqlite::{SqliteDatabaseError, SqliteReadHandle, SqliteSelectJson};
use crate::api::account::data::AccountSetup;

use crate::api::model::{
    *
};

use crate::utils::{IntoReportExt};

use crate::read_json;


#[async_trait]
impl SqliteSelectJson for Profile {
    async fn select_json(
        id: AccountIdInternal,
        read: &SqliteReadCommands,
    ) -> Result<Self, SqliteDatabaseError> {
        read_json!(
            read,
            id,
            r#"
            SELECT json_text
            FROM Profile
            WHERE account_row_id = ?
            "#,
            json_text
        )
    }
}

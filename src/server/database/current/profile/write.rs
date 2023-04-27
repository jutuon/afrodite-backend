


use async_trait::async_trait;
use error_stack::Result;
use tokio_stream::{Stream, StreamExt};


use crate::server::database::current::CurrentDataWriteCommands;
use crate::server::database::sqlite::{SqliteDatabaseError, SqliteReadHandle, SqliteSelectJson, SqliteUpdateJson};
use crate::api::account::data::AccountSetup;

use crate::api::model::{
    *
};

use crate::utils::{IntoReportExt};

use crate::insert_or_update_json;






#[async_trait]
impl SqliteUpdateJson for Profile {
    async fn update_json(
        &self,
        id: AccountIdInternal,
        write: &CurrentDataWriteCommands,
    ) -> Result<(), SqliteDatabaseError> {
        insert_or_update_json!(
            write,
            r#"
            UPDATE Profile
            SET json_text = ?
            WHERE account_row_id = ?
            "#,
            self,
            id
        )
    }
}
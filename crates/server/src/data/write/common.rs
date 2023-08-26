use std::net::SocketAddr;

use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, AuthPair};

use crate::data::{DataError, IntoDataError};

define_write_commands!(WriteCommandsCommon);

impl WriteCommandsCommon<'_> {
    pub async fn set_new_auth_pair(
        &self,
        id: AccountIdInternal,
        pair: AuthPair,
        address: Option<SocketAddr>,
    ) -> Result<(), DataError> {
        let current_access_token = self
            .db_read(move |mut cmds| cmds.account().access_token(id))
            .await?;

        let access = pair.access.clone();
        self.db_write(move |mut cmds| cmds.account().access_token(id, Some(access)))
            .await?;
        self.db_write(move |mut cmds| cmds.account().refresh_token(id, Some(pair.refresh)))
            .await?;

        self.cache()
            .update_access_token_and_connection(
                id.as_id(),
                current_access_token,
                pair.access,
                address,
            )
            .await
            .into_data_error(id)
    }

    /// Remove current connection address, access and refresh tokens.
    pub async fn logout(&self, id: AccountIdInternal) -> Result<(), DataError> {
        self.db_write(move |mut cmds| cmds.account().refresh_token(id, None))
            .await?;

        self.end_connection_session(id, true).await?;

        Ok(())
    }

    /// Remove current connection address and access token.
    pub async fn end_connection_session(
        &self,
        id: AccountIdInternal,
        remove_access_token: bool,
    ) -> Result<(), DataError> {
        let current_access_token = if remove_access_token {
            self.db_read(move |mut cmds| cmds.account().access_token(id))
                .await?
        } else {
            None
        };

        self.cache()
            .delete_access_token_and_connection(id.as_id(), current_access_token)
            .await
            .into_data_error(id)?;

        self.db_write(move |mut cmds| cmds.account().access_token(id, None))
            .await?;

        Ok(())
    }
}

use axum::{extract::State, Extension};
use model::AccountIdInternal;
use obfuscate_api_macro::obfuscate_api;
use server_api::{create_open_api_router, db_write_multiple, S};
use server_data::write::GetWriteCommandsCommon;
use server_data_account::write::GetWriteCommandsAccount;
use simple_backend::create_counters;
use utoipa_axum::router::OpenApiRouter;

use super::super::utils::StatusCode;
use crate::app::WriteData;

#[obfuscate_api]
const PATH_POST_LOGOUT: &str = "/account_api/logout";

#[utoipa::path(
    post,
    path = PATH_POST_LOGOUT,
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_logout(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_logout.incr();

    db_write_multiple!(state, move |cmds| {
        cmds.common().logout(account_id).await?;
        cmds.account_chat_utils()
            .remove_fcm_device_token_and_pending_notification_token(account_id)
            .await
    })?;

    Ok(())
}

pub fn logout_router(s: S) -> OpenApiRouter {
    create_open_api_router!(s, post_logout,)
}

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_LOGOUT_COUNTERS_LIST,
    post_logout,
);

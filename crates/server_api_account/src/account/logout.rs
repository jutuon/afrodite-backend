
use axum::{extract::State, Extension};
use model::AccountIdInternal;
use obfuscate_api_macro::obfuscate_api;
use server_api::{create_open_api_router, db_write};
use server_data::write::GetWriteCommandsCommon;
use simple_backend::create_counters;
use utoipa_axum::router::OpenApiRouter;

use super::super::utils::StatusCode;
use crate::app::{StateBase, WriteData};

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
pub async fn post_logout<S: WriteData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_logout.incr();

    db_write!(state, move |cmds| cmds.common().logout(
        account_id,
    ))?;

    Ok(())
}

pub fn logout_router<S: StateBase + WriteData>(s: S) -> OpenApiRouter {
    create_open_api_router!(
        s,
        post_logout::<S>,
    )
}

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_LOGOUT_COUNTERS_LIST,
    post_logout,
);
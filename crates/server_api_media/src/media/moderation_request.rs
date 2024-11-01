use axum::{extract::State, Extension};
use model::{AccountIdInternal, CurrentModerationRequest, ModerationRequestContent};
use obfuscate_api_macro::obfuscate_api;
use server_api::create_open_api_router;
use server_data_media::{read::GetReadMediaCommands, write::GetWriteCommandsMedia};
use simple_backend::create_counters;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    app::{ReadData, StateBase, WriteData},
    db_write,
    utils::{Json, StatusCode},
};

#[obfuscate_api]
const PATH_MODERATION_REQUEST: &str = "/media_api/moderation/request";

/// Get current moderation request.
///
#[utoipa::path(
    get,
    path = PATH_MODERATION_REQUEST,
    responses(
        (status = 200, description = "Get moderation request was successfull.", body = CurrentModerationRequest),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_moderation_request<S: ReadData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<CurrentModerationRequest>, StatusCode> {
    MEDIA.get_moderation_request.incr();

    let request = state.read().media().moderation_request(account_id).await?;

    let request = CurrentModerationRequest { request };

    Ok(request.into())
}

/// Create new or override old moderation request.
///
/// Make sure that moderation request has content IDs which points to your own
/// image slots.
///
#[utoipa::path(
    put,
    path = PATH_MODERATION_REQUEST,
    request_body(content = ModerationRequestContent),
    responses(
        (status = 200, description = "Sending or updating new image moderation request was successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error or request content was invalid."),
    ),
    security(("access_token" = [])),
)]
pub async fn put_moderation_request<S: WriteData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(moderation_request): Json<ModerationRequestContent>,
) -> Result<(), StatusCode> {
    MEDIA.put_moderation_request.incr();

    db_write!(state, move |cmds| {
        cmds.media()
            .create_or_update_moderation_request(account_id, moderation_request)
    })
}

/// Delete current moderation request which is not yet in moderation.
#[utoipa::path(
    delete,
    path = PATH_MODERATION_REQUEST,
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn delete_moderation_request<S: WriteData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<(), StatusCode> {
    MEDIA.delete_moderation_request.incr();

    db_write!(state, move |cmds| {
        cmds.media()
            .delete_moderation_request_not_yet_in_moderation(account_id)
    })
}

pub fn moderation_request_router<S: StateBase + WriteData + ReadData>(s: S) -> OpenApiRouter {
    create_open_api_router!(
        s,
        get_moderation_request::<S>,
        put_moderation_request::<S>,
        delete_moderation_request::<S>,
    )
}

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_MODERATION_REQUEST_COUNTERS_LIST,
    get_moderation_request,
    put_moderation_request,
    delete_moderation_request,
);

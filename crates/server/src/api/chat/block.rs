use axum::{extract::State, Extension, Router};
use model::{AccountId, AccountIdInternal, ReceivedBlocksPage, SentBlocksPage};
use simple_backend::create_counters;

use super::super::{
    db_write,
    utils::{Json, StatusCode},
};
use crate::app::{EventManagerProvider, GetAccounts, ReadData, WriteData};

pub const PATH_POST_BLOCK_PROFILE: &str = "/chat_api/block_profile";

/// Block profile
#[utoipa::path(
    post,
    path = "/chat_api/block_profile",
    request_body(content = AccountId),
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_block_profile<S: GetAccounts + WriteData + EventManagerProvider>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(requested_profile): Json<AccountId>,
) -> Result<(), StatusCode> {
    CHAT.post_block_profile.incr();

    let requested_profile = state.accounts().get_internal_id(requested_profile).await?;

    db_write!(state, move |cmds| {
        cmds.chat().block_profile(id, requested_profile)
    })?;

    state
        .event_manager()
        .send_notification(
            requested_profile,
            model::NotificationEvent::ReceivedBlocksChanged,
        )
        .await?;

    Ok(())
}

pub const PATH_POST_UNBLOCK_PROFILE: &str = "/chat_api/unblock_profile";

/// Unblock profile
#[utoipa::path(
    post,
    path = "/chat_api/unblock_profile",
    request_body(content = AccountId),
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_unblock_profile<S: GetAccounts + WriteData + EventManagerProvider>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(requested_profile): Json<AccountId>,
) -> Result<(), StatusCode> {
    CHAT.post_unblock_profile.incr();

    let requested_profile = state.accounts().get_internal_id(requested_profile).await?;

    db_write!(state, move |cmds| {
        cmds.chat().delete_like_or_block(id, requested_profile)
    })?;

    state
        .event_manager()
        .send_notification(
            requested_profile,
            model::NotificationEvent::ReceivedBlocksChanged,
        )
        .await?;

    Ok(())
}

pub const PATH_GET_SENT_BLOCKS: &str = "/chat_api/sent_blocks";

/// Get list of sent blocks
#[utoipa::path(
    get,
    path = "/chat_api/sent_blocks",
    responses(
        (status = 200, description = "Success.", body = SentBlocksPage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_sent_blocks<S: ReadData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<SentBlocksPage>, StatusCode> {
    CHAT.get_sent_blocks.incr();

    let page = state.read().chat().all_sent_blocks(id).await?;
    Ok(page.into())
}

// TODO: Add some block query info, so that server can send sync received blocks
//       list command to client.

pub const PATH_GET_RECEIVED_BLOCKS: &str = "/chat_api/received_blocks";

/// Get list of received blocks
#[utoipa::path(
    get,
    path = "/chat_api/received_blocks",
    responses(
        (status = 200, description = "Success.", body = ReceivedBlocksPage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_received_blocks<S: ReadData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<ReceivedBlocksPage>, StatusCode> {
    CHAT.get_received_blocks.incr();

    let page = state.read().chat().all_received_blocks(id).await?;
    Ok(page.into())
}

pub fn block_router(s: crate::app::S) -> Router {
    use axum::routing::{get, post};

    use crate::app::S;

    Router::new()
        .route(PATH_POST_BLOCK_PROFILE, post(post_block_profile::<S>))
        .route(PATH_POST_UNBLOCK_PROFILE, post(post_unblock_profile::<S>))
        .route(PATH_GET_SENT_BLOCKS, get(get_sent_blocks::<S>))
        .route(PATH_GET_RECEIVED_BLOCKS, get(get_received_blocks::<S>))
        .with_state(s)
}

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_BLOCK_COUNTERS_LIST,
    post_block_profile,
    post_unblock_profile,
    get_sent_blocks,
    get_received_blocks,
);
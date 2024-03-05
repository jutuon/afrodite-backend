

use std::net::SocketAddr;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use axum_extra::TypedHeader;
use model::{
    AccessToken, AccountIdInternal, AccountSyncVersion, AuthPair, BackendVersion, ChatStateRaw, EventToClient, EventToClientInternal, RefreshToken, SpecialEventToClient, SyncCheckDataType, SyncCheckResult, SyncDataVersionFromClient, SyncVersionFromClient, SyncVersionUtils
};
use simple_backend::{create_counters, event, web_socket::WebSocketManager};
use simple_backend_utils::IntoReportFromString;
use tracing::{error, info};
pub use utils::api::PATH_CONNECT;

use crate::{
    app::{BackendVersionProvider, GetAccessTokens, ReadData, WriteData},
    api::utils::{AccessTokenHeader, Json, StatusCode},
};
use crate::{app::GetConfig, db_write, result::{Result, WrappedContextExt, WrappedResultExt}};

use super::WebSocketError;

pub async fn send_new_messages_event_if_needed<S: WriteData + ReadData + GetConfig>(
    state: &S,
    socket: &mut WebSocket,
    id: AccountIdInternal,
) -> Result<(), WebSocketError> {
    if state.config().components().chat {
        let pending_messages = state
            .read()
            .chat()
            .all_pending_messages(id)
            .await
            .change_context(WebSocketError::DatabasePendingMessagesQuery)?;

        if !pending_messages.messages.is_empty() {
            send_event(
                socket,
                EventToClientInternal::NewMessageReceived
            ).await?;
        }
    }

    Ok(())
}

pub async fn sync_data_with_client_if_needed<S: WriteData + ReadData + GetConfig>(
    state: &S,
    socket: &mut WebSocket,
    id: AccountIdInternal,
    sync_versions: Vec<SyncDataVersionFromClient>,
) -> Result<(), WebSocketError> {
    let chat_state = state
        .read()
        .chat()
        .chat_state(id)
        .await
        .change_context(WebSocketError::DatabaseChatStateQuery)?;

    for version in sync_versions {
        match version.data_type {
            SyncCheckDataType::Account =>
                if state.config().components().account {
                    handle_account_data_sync(
                        state,
                        socket,
                        id,
                        version.version,
                    ).await?;
                }
            SyncCheckDataType::ReveivedBlocks =>
                if state.config().components().chat {
                    handle_chat_state_version_check(
                        state,
                        socket,
                        id,
                        version.version,
                        chat_state.clone(),
                        |s| &mut s.received_blocks_sync_version,
                        EventToClientInternal::ReceivedBlocksChanged,
                    ).await?;
                }
            SyncCheckDataType::ReveivedLikes =>
                if state.config().components().chat {
                    handle_chat_state_version_check(
                        state,
                        socket,
                        id,
                        version.version,
                        chat_state.clone(),
                        |s| &mut s.received_likes_sync_version,
                        EventToClientInternal::ReceivedLikesChanged,
                    ).await?;
                }
            SyncCheckDataType::SentBlocks =>
                if state.config().components().chat {
                    handle_chat_state_version_check(
                        state,
                        socket,
                        id,
                        version.version,
                        chat_state.clone(),
                        |s| &mut s.sent_blocks_sync_version,
                        EventToClientInternal::SentBlocksChanged,
                    ).await?;
                }
            SyncCheckDataType::SentLikes =>
                if state.config().components().chat {
                    handle_chat_state_version_check(
                        state,
                        socket,
                        id,
                        version.version,
                        chat_state.clone(),
                        |s| &mut s.sent_likes_sync_version,
                        EventToClientInternal::SentLikesChanged,
                    ).await?;
                }
            SyncCheckDataType::Matches =>
                if state.config().components().chat {
                    handle_chat_state_version_check(
                        state,
                        socket,
                        id,
                        version.version,
                        chat_state.clone(),
                        |s| &mut s.matches_sync_version,
                        EventToClientInternal::MatchesChanged,
                    ).await?;
                }
            SyncCheckDataType::AvailableProfileAttributes =>
                if state.config().components().profile {
                    handle_profile_attributes_sync_version_check(
                        state,
                        socket,
                        id,
                        version.version,
                    ).await?;
                }
        }
    }

    Ok(())
}

async fn handle_account_data_sync<S: WriteData + ReadData>(
    state: &S,
    socket: &mut WebSocket,
    id: AccountIdInternal,
    sync_version: SyncVersionFromClient,
) -> Result<(), WebSocketError> {
    let account = state
        .read()
        .common()
        .account(id)
        .await
        .change_context(WebSocketError::DatabaseAccountStateQuery)?;

    let account = match account.sync_version().check_is_sync_required(sync_version) {
        SyncCheckResult::DoNothing => return Ok(()),
        SyncCheckResult::ResetVersionAndSync => {
            state.write(
                move |cmds| async move {
                    cmds.account().reset_syncable_account_data_version(id).await
                },
            )
                .await
                .change_context(WebSocketError::AccountDataVersionResetFailed)?;

            state
                .read()
                .common()
                .account(id)
                .await
                .change_context(WebSocketError::DatabaseAccountStateQuery)?
        }
        SyncCheckResult::Sync => account,
    };

    send_event(
        socket,
        EventToClientInternal::AccountStateChanged(
            account.state()
        )
    ).await?;

    send_event(
        socket,
        EventToClientInternal::AccountCapabilitiesChanged(
            account.capablities().clone()
        )
    ).await?;

    send_event(
        socket,
        EventToClientInternal::ProfileVisibilityChanged(
            account.profile_visibility()
        )
    ).await?;

    // This must be the last to make sure that client has
    // reveived all sync data.
    send_event(
        socket,
        SpecialEventToClient::AccountSyncVersionChanged(
            account.sync_version()
        )
    ).await?;

    Ok(())
}

async fn handle_chat_state_version_check<S: WriteData + ReadData, T: SyncVersionUtils>(
    state: &S,
    socket: &mut WebSocket,
    id: AccountIdInternal,
    sync_version: SyncVersionFromClient,
    mut chat_state: ChatStateRaw,
    getter: impl Fn(&mut ChatStateRaw) -> &mut T + Send + 'static,
    event: EventToClientInternal,
) -> Result<(), WebSocketError> {
    let check_this_version = getter(&mut chat_state);
    match check_this_version.check_is_sync_required(sync_version) {
        SyncCheckResult::DoNothing => return Ok(()),
        SyncCheckResult::ResetVersionAndSync => {
            state.write(
                move |cmds| async move {
                    cmds.chat().modify_chat_state(id, move |s| {
                        let version_to_be_reseted = getter(s);
                        *version_to_be_reseted = Default::default();
                    }).await
                },
            )
                .await
                .change_context(WebSocketError::ChatDataVersionResetFailed)?;
        }
        SyncCheckResult::Sync => (),
    };

    send_event(socket, event).await?;

    Ok(())
}

async fn handle_profile_attributes_sync_version_check<S: WriteData + ReadData>(
    state: &S,
    socket: &mut WebSocket,
    id: AccountIdInternal,
    sync_version: SyncVersionFromClient,
) -> Result<(), WebSocketError> {
    let current = state
        .read()
        .profile()
        .profile_state(id)
        .await
        .change_context(WebSocketError::DatabaseProfileStateQuery)?
        .profile_attributes_sync_version;
    match current.check_is_sync_required(sync_version) {
        SyncCheckResult::DoNothing => return Ok(()),
        SyncCheckResult::ResetVersionAndSync => {
            state.write(
                move |cmds| async move {
                    cmds.profile().reset_profile_attributes_sync_version(id).await
                },
            )
                .await
                .change_context(WebSocketError::ProfileAttributesSyncVersionResetFailed)?;
        }
        SyncCheckResult::Sync => (),
    };

    send_event(socket, EventToClientInternal::AvailableProfileAttributesChanged).await?;

    Ok(())
}

async fn send_event(
    socket: &mut WebSocket,
    event: impl Into<EventToClient>,
) -> Result<(), WebSocketError> {
    let event: EventToClient = event.into();
    let event = serde_json::to_string(&event).change_context(WebSocketError::Serialize)?;
    socket
        .send(Message::Text(event))
        .await
        .change_context(WebSocketError::Send)?;

    Ok(())
}
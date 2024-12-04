use axum::extract::ws::{Message, WebSocket};
use config::Config;
use model_chat::{
    AccountIdInternal, ChatStateRaw, EventToClient, EventToClientInternal, SpecialEventToClient,
    SyncCheckDataType, SyncCheckResult, SyncDataVersionFromClient, SyncVersionFromClient,
    SyncVersionUtils,
};
use server_common::websocket::WebSocketError;
use server_data::{
    db_manager::RouterDatabaseReadHandle,
    read::GetReadCommandsCommon,
    result::{Result, WrappedResultExt},
    write_commands::WriteCommandRunnerHandle,
};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use server_data_media::{read::GetReadMediaCommands, write::GetWriteCommandsMedia};
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile};

pub async fn reset_pending_notification(
    config: &Config,
    write_handle: &WriteCommandRunnerHandle,
    id: AccountIdInternal,
) -> Result<(), WebSocketError> {
    if config.components().chat {
        write_handle
            .write(move |cmds| async move {
                cmds.chat()
                    .push_notifications()
                    .reset_pending_notification(id)
                    .await
            })
            .await
            .change_context(WebSocketError::DatabasePendingNotificationReset)?;
    }

    Ok(())
}

pub async fn send_new_messages_event_if_needed(
    config: &Config,
    read_handle: &RouterDatabaseReadHandle,
    socket: &mut WebSocket,
    id: AccountIdInternal,
) -> Result<(), WebSocketError> {
    if config.components().chat {
        let pending_messages = read_handle
            .chat()
            .all_pending_messages(id)
            .await
            .change_context(WebSocketError::DatabasePendingMessagesQuery)?;

        if !pending_messages.is_empty() {
            send_event(socket, EventToClientInternal::NewMessageReceived).await?;
        }
    }

    Ok(())
}

pub async fn sync_data_with_client_if_needed(
    config: &Config,
    read_handle: &RouterDatabaseReadHandle,
    write_handle: &WriteCommandRunnerHandle,
    socket: &mut WebSocket,
    id: AccountIdInternal,
    sync_versions: Vec<SyncDataVersionFromClient>,
) -> Result<(), WebSocketError> {
    let chat_state = read_handle
        .chat()
        .chat_state(id)
        .await
        .change_context(WebSocketError::DatabaseChatStateQuery)?;

    for version in sync_versions {
        match version.data_type {
            SyncCheckDataType::Account => {
                if config.components().account {
                    handle_account_data_sync(
                        read_handle,
                        write_handle,
                        socket,
                        id,
                        version.version,
                    )
                    .await?;
                }
            }
            SyncCheckDataType::ReveivedBlocks => {
                if config.components().chat {
                    handle_chat_state_version_check(
                        write_handle,
                        socket,
                        id,
                        version.version,
                        chat_state.clone(),
                        |s| &mut s.received_blocks_sync_version,
                        EventToClientInternal::ReceivedBlocksChanged,
                    )
                    .await?;
                }
            }
            SyncCheckDataType::ReveivedLikes => {
                if config.components().chat {
                    handle_chat_state_version_check(
                        write_handle,
                        socket,
                        id,
                        version.version,
                        chat_state.clone(),
                        |s| &mut s.received_likes_sync_version,
                        EventToClientInternal::ReceivedLikesChanged,
                    )
                    .await?;
                }
            }
            SyncCheckDataType::SentBlocks => {
                if config.components().chat {
                    handle_chat_state_version_check(
                        write_handle,
                        socket,
                        id,
                        version.version,
                        chat_state.clone(),
                        |s| &mut s.sent_blocks_sync_version,
                        EventToClientInternal::SentBlocksChanged,
                    )
                    .await?;
                }
            }
            SyncCheckDataType::SentLikes => {
                if config.components().chat {
                    handle_chat_state_version_check(
                        write_handle,
                        socket,
                        id,
                        version.version,
                        chat_state.clone(),
                        |s| &mut s.sent_likes_sync_version,
                        EventToClientInternal::SentLikesChanged,
                    )
                    .await?;
                }
            }
            SyncCheckDataType::Matches => {
                if config.components().chat {
                    handle_chat_state_version_check(
                        write_handle,
                        socket,
                        id,
                        version.version,
                        chat_state.clone(),
                        |s| &mut s.matches_sync_version,
                        EventToClientInternal::MatchesChanged,
                    )
                    .await?;
                }
            }
            SyncCheckDataType::AvailableProfileAttributes => {
                if config.components().profile {
                    handle_profile_attributes_sync_version_check(
                        read_handle,
                        write_handle,
                        socket,
                        id,
                        version.version,
                    )
                    .await?;
                }
            }
            SyncCheckDataType::Profile => {
                if config.components().profile {
                    handle_profile_sync_version_check(
                        read_handle,
                        write_handle,
                        socket,
                        id,
                        version.version,
                    )
                    .await?;
                }
            }
            SyncCheckDataType::News => {
                if config.components().account {
                    handle_news_count_sync_version_check(
                        read_handle,
                        write_handle,
                        socket,
                        id,
                        version.version,
                    )
                    .await?;
                }
            }
            SyncCheckDataType::ProfileContent => {
                if config.components().account {
                    handle_profile_content_sync_version_check(
                        read_handle,
                        write_handle,
                        socket,
                        id,
                        version.version,
                    )
                    .await?;
                }
            }
        }
    }

    Ok(())
}

async fn handle_account_data_sync(
    read_handle: &RouterDatabaseReadHandle,
    write_handle: &WriteCommandRunnerHandle,
    socket: &mut WebSocket,
    id: AccountIdInternal,
    sync_version: SyncVersionFromClient,
) -> Result<(), WebSocketError> {
    let account = read_handle
        .common()
        .account(id)
        .await
        .change_context(WebSocketError::DatabaseAccountStateQuery)?;

    let account = match account.sync_version().check_is_sync_required(sync_version) {
        SyncCheckResult::DoNothing => return Ok(()),
        SyncCheckResult::ResetVersionAndSync => {
            write_handle
                .write(move |cmds| async move {
                    cmds.account().reset_syncable_account_data_version(id).await
                })
                .await
                .change_context(WebSocketError::AccountDataVersionResetFailed)?;

            read_handle
                .common()
                .account(id)
                .await
                .change_context(WebSocketError::DatabaseAccountStateQuery)?
        }
        SyncCheckResult::Sync => account,
    };

    send_event(
        socket,
        EventToClientInternal::AccountStateChanged(account.state()),
    )
    .await?;

    send_event(
        socket,
        EventToClientInternal::AccountPermissionsChanged(account.permissions().clone()),
    )
    .await?;

    send_event(
        socket,
        EventToClientInternal::ProfileVisibilityChanged(account.profile_visibility()),
    )
    .await?;

    // This must be the last to make sure that client has
    // reveived all sync data.
    send_event(
        socket,
        SpecialEventToClient::AccountSyncVersionChanged(account.sync_version()),
    )
    .await?;

    Ok(())
}

async fn handle_chat_state_version_check<T: SyncVersionUtils>(
    write_handle: &WriteCommandRunnerHandle,
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
        SyncCheckResult::ResetVersionAndSync => write_handle
            .write(move |cmds| async move {
                cmds.chat()
                    .modify_chat_state(id, move |s| {
                        let version_to_be_reseted = getter(s);
                        *version_to_be_reseted = Default::default();
                    })
                    .await
            })
            .await
            .change_context(WebSocketError::ChatDataVersionResetFailed)?,
        SyncCheckResult::Sync => (),
    };

    send_event(socket, event).await?;

    Ok(())
}

async fn handle_profile_attributes_sync_version_check(
    read_handle: &RouterDatabaseReadHandle,
    write_handle: &WriteCommandRunnerHandle,
    socket: &mut WebSocket,
    id: AccountIdInternal,
    sync_version: SyncVersionFromClient,
) -> Result<(), WebSocketError> {
    let current = read_handle
        .profile()
        .profile_state(id)
        .await
        .change_context(WebSocketError::DatabaseProfileStateQuery)?
        .profile_attributes_sync_version;
    match current.check_is_sync_required(sync_version) {
        SyncCheckResult::DoNothing => return Ok(()),
        SyncCheckResult::ResetVersionAndSync => write_handle
            .write(move |cmds| async move {
                cmds.profile()
                    .reset_profile_attributes_sync_version(id)
                    .await
            })
            .await
            .change_context(WebSocketError::ProfileAttributesSyncVersionResetFailed)?,
        SyncCheckResult::Sync => (),
    };

    send_event(
        socket,
        EventToClientInternal::AvailableProfileAttributesChanged,
    )
    .await?;

    Ok(())
}

async fn handle_profile_sync_version_check(
    read_handle: &RouterDatabaseReadHandle,
    write_handle: &WriteCommandRunnerHandle,
    socket: &mut WebSocket,
    id: AccountIdInternal,
    sync_version: SyncVersionFromClient,
) -> Result<(), WebSocketError> {
    let current = read_handle
        .profile()
        .profile_state(id)
        .await
        .change_context(WebSocketError::DatabaseProfileStateQuery)?
        .profile_sync_version;
    match current.check_is_sync_required(sync_version) {
        SyncCheckResult::DoNothing => return Ok(()),
        SyncCheckResult::ResetVersionAndSync => write_handle
            .write(move |cmds| async move { cmds.profile().reset_profile_sync_version(id).await })
            .await
            .change_context(WebSocketError::ProfileSyncVersionResetFailed)?,
        SyncCheckResult::Sync => (),
    };

    send_event(socket, EventToClientInternal::ProfileChanged).await?;

    Ok(())
}

async fn handle_news_count_sync_version_check(
    read_handle: &RouterDatabaseReadHandle,
    write_handle: &WriteCommandRunnerHandle,
    socket: &mut WebSocket,
    id: AccountIdInternal,
    sync_version: SyncVersionFromClient,
) -> Result<(), WebSocketError> {
    let current = read_handle
        .account()
        .news()
        .unread_news_count(id)
        .await
        .change_context(WebSocketError::DatabaseNewsCountQuery)?
        .v;
    match current.check_is_sync_required(sync_version) {
        SyncCheckResult::DoNothing => return Ok(()),
        SyncCheckResult::ResetVersionAndSync => write_handle
            .write(move |cmds| async move {
                cmds.account()
                    .news()
                    .reset_news_count_sync_version(id)
                    .await
            })
            .await
            .change_context(WebSocketError::NewsCountSyncVersionResetFailed)?,
        SyncCheckResult::Sync => (),
    };

    send_event(socket, EventToClientInternal::NewsChanged).await?;

    Ok(())
}


async fn handle_profile_content_sync_version_check(
    read_handle: &RouterDatabaseReadHandle,
    write_handle: &WriteCommandRunnerHandle,
    socket: &mut WebSocket,
    id: AccountIdInternal,
    sync_version: SyncVersionFromClient,
) -> Result<(), WebSocketError> {
    let current = read_handle
        .media()
        .profile_content_sync_version(id)
        .await
        .change_context(WebSocketError::DatabaseProfileContentSyncVersionQuery)?;
    match current.check_is_sync_required(sync_version) {
        SyncCheckResult::DoNothing => return Ok(()),
        SyncCheckResult::ResetVersionAndSync => write_handle
            .write(move |cmds| async move {
                cmds.media()
                    .reset_profile_content_sync_version(id)
                    .await
            })
            .await
            .change_context(WebSocketError::ProfileContentnSyncVersionResetFailed)?,
        SyncCheckResult::Sync => (),
    };

    send_event(socket, EventToClientInternal::ProfileContentChanged).await?;

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
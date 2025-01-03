use model::{
    AccountIdInternal, InitialContentModerationCompletedResult, NewReceivedLikesCountResult, PendingNotification, PendingNotificationFlags, PendingNotificationWithData
};
use server_data::db_manager::RouterDatabaseReadHandle;
use server_data_account::read::GetReadCommandsAccount;
use server_data_chat::read::GetReadChatCommands;
use server_data_media::read::GetReadMediaCommands;

pub async fn get_push_notification_data(
    read_handle: &RouterDatabaseReadHandle,
    id: AccountIdInternal,
    notification_value: PendingNotification,
) -> PendingNotificationWithData {
    let flags = PendingNotificationFlags::from(notification_value);
    let sender_info = if flags.contains(PendingNotificationFlags::NEW_MESSAGE) {
        read_handle
            .chat()
            .all_pending_message_sender_account_ids(id)
            .await
            .ok()
    } else {
        None
    };

    let received_likes_info = if flags.contains(PendingNotificationFlags::RECEIVED_LIKES_CHANGED) {
        read_handle
            .chat()
            .chat_state(id)
            .await
            .ok()
            .map(|chat_state| NewReceivedLikesCountResult {
                v: chat_state.received_likes_sync_version,
                c: chat_state.new_received_likes_count,
            })
    } else {
        None
    };

    let initial_content_accepted = if flags.contains(PendingNotificationFlags::INITIAL_CONTENT_MODERATION_COMPLETED) {
        read_handle
            .media()
            .profile_content_moderated_as_accepted(id)
            .await
            .ok()
            .map(|accepted| InitialContentModerationCompletedResult {
                accepted
            })
    } else {
        None
    };

    let unread_news_count = if flags.contains(PendingNotificationFlags::NEWS_CHANGED) {
        read_handle
            .account()
            .news()
            .unread_news_count(id)
            .await
            .ok()
    } else {
        None
    };

    PendingNotificationWithData {
        value: notification_value,
        new_message_received_from: sender_info,
        received_likes_changed: received_likes_info,
        initial_content_moderation_completed: initial_content_accepted,
        news_changed: unread_news_count,
    }
}

mod push_notifications;

use database_chat::current::write::chat::ChatStateChanges;
use error_stack::ResultExt;
use model::{AccountIdInternal, ChatStateRaw, MessageNumber, PendingMessageId, SyncVersionUtils};
use server_data::{
    define_server_data_write_commands, result::Result, write::WriteCommandsProvider, DataError,
    DieselDatabaseError,
};
use simple_backend_utils::ContextExt;

use self::push_notifications::WriteCommandsChatPushNotifications;

define_server_data_write_commands!(WriteCommandsChat);
define_db_transaction_command!(WriteCommandsChat);

impl<C: WriteCommandsProvider> WriteCommandsChat<C> {
    pub fn push_notifications(self) -> WriteCommandsChatPushNotifications<C> {
        WriteCommandsChatPushNotifications::new(self.cmds)
    }
}

impl<C: WriteCommandsProvider> WriteCommandsChat<C> {
    pub async fn modify_chat_state(
        &mut self,
        id: AccountIdInternal,
        action: impl Fn(&mut ChatStateRaw) + Send + 'static,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat().modify_chat_state(id, action)?;
            Ok(())
        })
    }

    /// Like or match a profile.
    ///
    /// Returns Ok only if the state change happened.
    pub async fn like_or_match_profile(
        &mut self,
        id_like_sender: AccountIdInternal,
        id_like_receiver: AccountIdInternal,
    ) -> Result<SenderAndReceiverStateChanges, DataError> {
        db_transaction!(self, move |mut cmds| {
            let interaction = cmds
                .chat()
                .interaction()
                .get_or_create_account_interaction(id_like_sender, id_like_receiver)?;

            let updated = if interaction.is_like()
                && interaction.account_id_sender == Some(id_like_sender.into_db_id())
                && interaction.account_id_receiver == Some(id_like_receiver.into_db_id())
            {
                return Err(DieselDatabaseError::AlreadyDone.report());
            } else if interaction.is_like()
                && interaction.account_id_sender == Some(id_like_receiver.into_db_id())
                && interaction.account_id_receiver == Some(id_like_sender.into_db_id())
            {
                interaction
                    .clone()
                    .try_into_match()
                    .change_context(DieselDatabaseError::NotAllowed)?
            } else if interaction.is_match() {
                return Err(DieselDatabaseError::AlreadyDone.report());
            } else {
                interaction
                    .clone()
                    .try_into_like(id_like_sender, id_like_receiver)
                    .change_context(DieselDatabaseError::NotAllowed)?
            };
            cmds.chat()
                .interaction()
                .update_account_interaction(updated.clone())?;

            let sender = cmds.chat().modify_chat_state(id_like_sender, |s| {
                if interaction.is_empty() {
                    s.sent_likes_sync_version.increment_if_not_max_value_mut();
                } else if interaction.is_like() {
                    s.matches_sync_version.increment_if_not_max_value_mut();
                }
            })?;

            let receiver = cmds.chat().modify_chat_state(id_like_receiver, |s| {
                if interaction.is_empty() {
                    s.received_likes_sync_version
                        .increment_if_not_max_value_mut();
                } else if interaction.is_like() {
                    s.matches_sync_version.increment_if_not_max_value_mut();
                }
            })?;

            Ok(SenderAndReceiverStateChanges { sender, receiver })
        })
    }

    /// Delete a like or block.
    ///
    /// Returns Ok only if the state change happened.
    pub async fn delete_like_or_block(
        &mut self,
        id_sender: AccountIdInternal,
        id_receiver: AccountIdInternal,
    ) -> Result<SenderAndReceiverStateChanges, DataError> {
        db_transaction!(self, move |mut cmds| {
            let interaction = cmds
                .chat()
                .interaction()
                .get_or_create_account_interaction(id_sender, id_receiver)?;

            if interaction.is_empty() {
                return Err(DieselDatabaseError::AlreadyDone.report());
            }
            if interaction.account_id_sender != Some(id_sender.into_db_id()) {
                return Err(DieselDatabaseError::NotAllowed.report());
            }
            let updated = interaction
                .clone()
                .try_into_empty()
                .change_context(DieselDatabaseError::NotAllowed)?;
            cmds.chat()
                .interaction()
                .update_account_interaction(updated)?;

            let sender = cmds.chat().modify_chat_state(id_sender, |s| {
                if interaction.is_like() {
                    s.sent_likes_sync_version.increment_if_not_max_value_mut();
                } else if interaction.is_blocked() {
                    s.sent_blocks_sync_version.increment_if_not_max_value_mut();
                }
            })?;

            let receiver = cmds.chat().modify_chat_state(id_receiver, |s| {
                if interaction.is_like() {
                    s.received_likes_sync_version
                        .increment_if_not_max_value_mut();
                } else if interaction.is_blocked() {
                    s.received_blocks_sync_version
                        .increment_if_not_max_value_mut();
                }
            })?;

            Ok(SenderAndReceiverStateChanges { sender, receiver })
        })
    }

    /// Block a profile.
    ///
    /// Returns Ok only if the state change happened.
    pub async fn block_profile(
        &mut self,
        id_block_sender: AccountIdInternal,
        id_block_receiver: AccountIdInternal,
    ) -> Result<SenderAndReceiverStateChanges, DataError> {
        db_transaction!(self, move |mut cmds| {
            let interaction = cmds
                .chat()
                .interaction()
                .get_or_create_account_interaction(id_block_sender, id_block_receiver)?;

            if interaction.is_blocked() {
                return Err(DieselDatabaseError::AlreadyDone.report());
            }
            let updated = interaction
                .clone()
                .try_into_block(id_block_sender, id_block_receiver)
                .change_context(DieselDatabaseError::NotAllowed)?;
            cmds.chat()
                .interaction()
                .update_account_interaction(updated)?;

            let sender = cmds.chat().modify_chat_state(id_block_sender, |s| {
                s.sent_blocks_sync_version.increment_if_not_max_value_mut();
                if interaction.is_like() {
                    s.sent_likes_sync_version.increment_if_not_max_value_mut();
                    s.received_likes_sync_version
                        .increment_if_not_max_value_mut();
                } else if interaction.is_match() {
                    s.matches_sync_version.increment_if_not_max_value_mut();
                }
            })?;

            let receiver = cmds.chat().modify_chat_state(id_block_receiver, |s| {
                s.received_blocks_sync_version
                    .increment_if_not_max_value_mut();
                if interaction.is_like() {
                    s.sent_likes_sync_version.increment_if_not_max_value_mut();
                    s.received_likes_sync_version
                        .increment_if_not_max_value_mut();
                } else if interaction.is_match() {
                    s.matches_sync_version.increment_if_not_max_value_mut();
                }
            })?;

            Ok(SenderAndReceiverStateChanges { sender, receiver })
        })
    }

    // TODO(prod): Change SQLite settings that delete is overwriting.

    /// Delete these pending messages which the receiver has received
    pub async fn delete_pending_message_list(
        &mut self,
        message_receiver: AccountIdInternal,
        messages: Vec<PendingMessageId>,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .message()
                .delete_pending_message_list(message_receiver, messages)
        })
    }

    /// Update message number which my account has viewed from the sender
    pub async fn update_message_number_of_latest_viewed_message(
        &self,
        id_my_account: AccountIdInternal,
        id_message_sender: AccountIdInternal,
        new_message_number: MessageNumber,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            let mut interaction = cmds
                .read()
                .chat()
                .interaction()
                .account_interaction(id_my_account, id_message_sender)?
                .ok_or(DieselDatabaseError::NotFound.report())?;

            // Prevent marking future messages as viewed
            if new_message_number.message_number > interaction.message_counter {
                return Err(DieselDatabaseError::NotAllowed.report());
            }

            // Who is sender and receiver in the interaction data depends
            // on who did the first like
            let modify_number = if interaction.account_id_sender == Some(id_my_account.into_db_id())
            {
                interaction.sender_latest_viewed_message.as_mut()
            } else {
                interaction.receiver_latest_viewed_message.as_mut()
            };

            if let Some(number) = modify_number {
                *number = new_message_number;
            } else {
                return Err(DieselDatabaseError::NotAllowed.report());
            }

            cmds.chat()
                .interaction()
                .update_account_interaction(interaction)?;

            Ok(())
        })
    }

    /// Insert a new pending message if sender and receiver are a match
    pub async fn insert_pending_message_if_match(
        &mut self,
        sender: AccountIdInternal,
        receiver: AccountIdInternal,
        message: String,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .message()
                .insert_pending_message_if_match(sender, receiver, message)
        })
    }
}

pub struct SenderAndReceiverStateChanges {
    pub sender: ChatStateChanges,
    pub receiver: ChatStateChanges,
}
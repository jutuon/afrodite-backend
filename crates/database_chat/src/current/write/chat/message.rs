use database::{define_current_write_commands, ConnectionProvider, DieselDatabaseError};
use diesel::{delete, insert_into, prelude::*, update};
use error_stack::Result;
use model::{AccountIdInternal, AccountInteractionState, ClientId, ClientLocalId, MessageNumber, NewPendingMessageValues, PendingMessageIdInternal, SentMessageId, UnixTime};
use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteChatMessage, CurrentSyncWriteChatMessage);

impl<C: ConnectionProvider> CurrentSyncWriteChatMessage<C> {
    pub fn add_receiver_acknowledgement_and_delete_if_also_sender_has_acknowledged(
        &mut self,
        message_receiver: AccountIdInternal,
        messages: Vec<PendingMessageIdInternal>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::pending_messages::dsl::*;

        for message in messages {
            update(pending_messages)
                .filter(message_number.eq(message.mn))
                .filter(account_id_sender.eq(message.sender.as_db_id()))
                .filter(account_id_receiver.eq(message_receiver.as_db_id()))
                .set(receiver_acknowledgement.eq(true))
                .execute(self.conn())
                .into_db_error(message_receiver)?;

            delete(pending_messages)
                .filter(message_number.eq(message.mn))
                .filter(account_id_sender.eq(message.sender.as_db_id()))
                .filter(account_id_receiver.eq(message_receiver.as_db_id()))
                .filter(sender_acknowledgement.eq(true))
                .filter(receiver_acknowledgement.eq(true))
                .execute(self.conn())
                .into_db_error(message_receiver)?;
        }

        Ok(())
    }

    pub fn add_sender_acknowledgement_and_delete_if_also_receiver_has_acknowledged(
        &mut self,
        message_sender: AccountIdInternal,
        messages: Vec<SentMessageId>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::pending_messages::dsl::*;

        for message in messages {
            update(pending_messages)
                .filter(sender_client_id.eq(message.c))
                .filter(sender_client_local_id.eq(message.l))
                .filter(account_id_sender.eq(message_sender.as_db_id()))
                .set(sender_acknowledgement.eq(true))
                .execute(self.conn())
                .into_db_error(message_sender)?;

            delete(pending_messages)
                .filter(sender_client_id.eq(message.c))
                .filter(sender_client_local_id.eq(message.l))
                .filter(account_id_sender.eq(message_sender.as_db_id()))
                .filter(sender_acknowledgement.eq(true))
                .filter(receiver_acknowledgement.eq(true))
                .execute(self.conn())
                .into_db_error(message_sender)?;
        }

        Ok(())
    }

    pub fn insert_pending_message_if_match_and_not_blocked(
        &mut self,
        sender: AccountIdInternal,
        receiver: AccountIdInternal,
        message: Vec<u8>,
        client_id_value: ClientId,
        client_local_id_value: ClientLocalId
    ) -> Result<NewPendingMessageValues, DieselDatabaseError> {
        use model::schema::{account_interaction, pending_messages::dsl::*};
        let time = UnixTime::current_time();
        let interaction = self
            .cmds()
            .chat()
            .interaction()
            .get_or_create_account_interaction(sender, receiver)?;
        // Skip message number 0, so that latest viewed message number
        // does not have that message already viewed.
        let new_message_number = MessageNumber::new(interaction.message_counter + 1);

        if interaction.state_number != AccountInteractionState::Match || interaction.is_blocked() {
            return Err(DieselDatabaseError::NotAllowed.into());
        }

        update(account_interaction::table.find(interaction.id))
            .set((
                account_interaction::message_counter.eq(new_message_number),
            ))
            .execute(self.conn())
            .into_db_error((sender, receiver, new_message_number))?;

        insert_into(pending_messages)
            .values((
                account_id_sender.eq(sender.as_db_id()),
                account_id_receiver.eq(receiver.as_db_id()),
                unix_time.eq(time),
                message_number.eq(new_message_number),
                message_bytes.eq(message),
                sender_client_id.eq(client_id_value),
                sender_client_local_id.eq(client_local_id_value),
            ))
            .execute(self.conn())
            .into_db_error((sender, receiver, new_message_number))?;

        Ok(NewPendingMessageValues {
            unix_time: time,
            message_number: new_message_number,
        })
    }
}

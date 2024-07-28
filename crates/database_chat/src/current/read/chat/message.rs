use database::{define_current_read_commands, ConnectionProvider, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::Result;
use model::{
    AccountId, AccountIdInternal, PendingMessage, PendingMessageId, PendingMessageInternal
};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadChatMessage, CurrentSyncReadChatMessage);

impl<C: ConnectionProvider> CurrentSyncReadChatMessage<C> {
    pub fn all_pending_messages(
        &mut self,
        id_message_receiver: AccountIdInternal,
    ) -> Result<Vec<PendingMessage>, DieselDatabaseError> {
        use crate::schema::{account_id, pending_messages::dsl::*};

        let value: Vec<(AccountId, PendingMessageInternal)> = pending_messages
            .inner_join(
                account_id::table.on(account_id_sender.assume_not_null().eq(account_id::id)),
            )
            .filter(account_id_receiver.eq(id_message_receiver.as_db_id()))
            .select((account_id::uuid, PendingMessageInternal::as_select()))
            .load(self.conn())
            .into_db_error(())?;

        let messages = value
            .into_iter()
            .map(|(sender_uuid, msg)| PendingMessage {
                id: PendingMessageId {
                    account_id_sender: sender_uuid,
                    message_number: msg.message_number,
                },
                unix_time: msg.unix_time,
                message: msg.message_text,
            })
            .collect();

        Ok(messages)
    }

    pub fn all_pending_message_sender_account_ids(
        &mut self,
        id_message_receiver: AccountIdInternal,
    ) -> Result<Vec<AccountId>, DieselDatabaseError> {
        use crate::schema::{account_id, pending_messages::dsl::*};

        let mut account_id_vec: Vec<AccountId> = pending_messages
            .inner_join(
                account_id::table.on(account_id_sender.assume_not_null().eq(account_id::id)),
            )
            .filter(account_id_receiver.eq(id_message_receiver.as_db_id()))
            .select(account_id::uuid)
            .order_by(account_id::id)
            .load(self.conn())
            .into_db_error(())?;

        account_id_vec.dedup();

        Ok(account_id_vec)
    }

    pub fn pending_message_count(
        &mut self,
        id_message_sender: AccountIdInternal,
        id_message_receiver: AccountIdInternal,
    ) -> Result<i64, DieselDatabaseError> {
        use crate::schema::pending_messages::dsl::*;

        pending_messages
            .filter(account_id_sender.eq(id_message_sender.as_db_id()))
            .filter(account_id_receiver.eq(id_message_receiver.as_db_id()))
            .count()
            .get_result(self.conn())
            .into_db_error(())
    }
}

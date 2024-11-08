use database::{define_current_write_commands, DieselDatabaseError};
use diesel::{prelude::*, update, ExpressionMethods};
use error_stack::Result;
use model::{AccountIdInternal, ProfileTextModerationRejectedReasonCategory, ProfileTextModerationRejectedReasonDetails, ProfileTextModerationState};

use super::ConnectionProvider;
use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteProfileAdminProfileText, CurrentWriteProfileAdminProfileText);

impl<C: ConnectionProvider> CurrentWriteProfileAdminProfileText<C> {
    pub fn moderate_profile_text(
        &mut self,
        moderator_id: AccountIdInternal,
        text_owner_id: AccountIdInternal,
        accepted: bool,
        rejected_category: Option<ProfileTextModerationRejectedReasonCategory>,
        rejected_details: Option<ProfileTextModerationRejectedReasonDetails>,
    ) -> Result<ProfileTextModerationState, DieselDatabaseError> {
        use model::schema::profile_state;

        let moderator_is_bot = self.read().common().state().other_shared_state(moderator_id)?.is_bot_account;

        let next_state = if accepted {
            if moderator_is_bot {
                ProfileTextModerationState::AcceptedByBot
            } else {
                ProfileTextModerationState::AcceptedByHuman
            }
        } else if moderator_is_bot {
            ProfileTextModerationState::RejectedByBot
        } else {
            ProfileTextModerationState::RejectedByHuman
        };

        update(profile_state::table)
            .filter(profile_state::account_id.eq(text_owner_id.as_db_id()))
            .set((
                profile_state::profile_text_moderation_state.eq(next_state),
                profile_state::profile_text_moderation_rejected_reason_category.eq(rejected_category),
                profile_state::profile_text_moderation_rejected_reason_details.eq(rejected_details),
                profile_state::profile_text_moderation_moderator_account_id.eq(moderator_id.as_db_id()),
            ))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(next_state)
    }
}
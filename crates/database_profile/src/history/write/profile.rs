use database::define_history_write_commands;
// use diesel::{insert_into, prelude::*, ExpressionMethods};
// use error_stack::{Result, ResultExt};
// use model::{AccountIdInternal, Profile};
// use simple_backend_utils::current_unix_time;

// use crate::IntoDatabaseError;

define_history_write_commands!(HistoryWriteProfile);

impl HistoryWriteProfile<'_> {
    // pub fn insert_profile(
    //     &mut self,
    //     account_id_internal: AccountIdInternal,
    //     profile: &Profile,
    // ) -> Result<(), DieselDatabaseError> {
    //     use model::schema::history_profile::dsl::*;

    //     // TODO: remove?

    //     let text =
    //         serde_json::to_string(profile).change_context(DieselDatabaseError::SerdeSerialize)?;
    //     let time = current_unix_time();

    //     insert_into(history_profile)
    //         .values((
    //             account_id.eq(account_id_internal.as_db_id()),
    //             unix_time.eq(time),
    //             json_text.eq(text),
    //         ))
    //         .execute(self.conn())
    //         .into_db_error((account_id_internal, account_id_internal))?;
    //     Ok(())
    // }
}

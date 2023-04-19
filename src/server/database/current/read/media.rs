use std::collections::HashSet;



use error_stack::Result;
use tokio_stream::{StreamExt};

use super::super::super::sqlite::{SqliteDatabaseError, SqliteReadHandle};

use crate::api::media::data::{
    ContentIdInternal, ContentState, Moderation, ModerationId, ModerationRequestId,
    ModerationRequestQueueNumber, ModerationRequestState,
};
use crate::api::model::{
    AccountIdInternal, ContentId, ModerationRequest,
    NewModerationRequest,
};
use crate::server::database::file::file::ImageSlot;



use crate::utils::{IntoReportExt};

macro_rules! read_json {
    ($self:expr, $id:expr, $sql:literal, $str_field:ident) => {{
        let id = $id.row_id();
        sqlx::query!($sql, id)
            .fetch_one($self.handle.pool())
            .await
            .into_error(SqliteDatabaseError::Execute)
            .and_then(|data| {
                serde_json::from_str(&data.$str_field)
                    .into_error(SqliteDatabaseError::SerdeDeserialize)
            })
    }};
}

pub struct CurrentReadMediaCommands<'a> {
    handle: &'a SqliteReadHandle,
}

impl<'a> CurrentReadMediaCommands<'a> {
    pub fn new(handle: &'a SqliteReadHandle) -> Self {
        Self { handle }
    }

    pub async fn get_content_id_from_slot(
        &self,
        slot_owner: AccountIdInternal,
        slot: ImageSlot,
    ) -> Result<Option<ContentIdInternal>, SqliteDatabaseError> {
        let required_state = ContentState::InSlot as i64;
        let required_slot = slot as i64;
        let request = sqlx::query_as!(
            ContentIdInternal,
            r#"
            SELECT content_row_id, content_id as "content_id: _"
            FROM MediaContent
            WHERE account_row_id = ? AND moderation_state = ? AND slot_number = ?
            "#,
            slot_owner.account_row_id,
            required_state,
            required_slot,
        )
        .fetch_optional(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Fetch)?;

        Ok(request)
    }

    /// Validate moderation request content.
    ///
    /// Returns `Err(SqliteDatabaseError::ModerationRequestContentInvalid)` if the
    /// content is invalid.
    pub async fn content_validate_moderation_request_content(
        &self,
        content_owner: AccountIdInternal,
        request_content: &NewModerationRequest,
    ) -> Result<(), SqliteDatabaseError> {
        let requested_content_set: HashSet<ContentId> = request_content.content().collect();

        let required_state = ContentState::InSlot as i64;
        let request = sqlx::query_as!(
            ContentId,
            r#"
            SELECT content_id as "content_id: _"
            FROM MediaContent
            WHERE account_row_id = ? AND moderation_state = ?
            "#,
            content_owner.account_row_id,
            required_state,
        )
        .fetch_all(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Fetch)?;

        let database_content_set: HashSet<ContentId> = request.into_iter().collect();

        if requested_content_set == database_content_set {
            Ok(())
        } else {
            Err(SqliteDatabaseError::ModerationRequestContentInvalid.into())
        }
    }

    pub async fn current_moderation_request(
        &self,
        request_creator: AccountIdInternal,
    ) -> Result<Option<ModerationRequest>, SqliteDatabaseError> {
        let account_row_id = request_creator.row_id();
        let request = sqlx::query!(
            r#"
            SELECT request_row_id, queue_number, json_text
            FROM MediaModerationRequest
            WHERE account_row_id = ?
            "#,
            account_row_id,
        )
        .fetch_optional(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Fetch)?;

        let request = match request {
            None => return Ok(None),
            Some(r) => r,
        };

        let moderation_states = sqlx::query!(
            r#"
            SELECT state_number, json_text
            FROM MediaModeration
            WHERE request_row_id = ?
            "#,
            request.request_row_id,
        )
        .fetch_all(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Fetch)?;

        let (state, data) = match moderation_states.first() {
            None => (ModerationRequestState::Waiting, &request.json_text),
            Some(first) => {
                let accepted = moderation_states
                    .iter()
                    .find(|r| r.state_number == ModerationRequestState::Accepted as i64);
                let denied = moderation_states
                    .iter()
                    .find(|r| r.state_number == ModerationRequestState::Denied as i64);

                if let Some(accepted) = accepted {
                    (ModerationRequestState::Accepted, &accepted.json_text)
                } else if let Some(denied) = denied {
                    (ModerationRequestState::Denied, &denied.json_text)
                } else {
                    (ModerationRequestState::InProgress, &first.json_text)
                }
            }
        };

        let data: NewModerationRequest =
            serde_json::from_str(data).into_error(SqliteDatabaseError::SerdeDeserialize)?;

        Ok(Some(ModerationRequest::new(
            request.request_row_id,
            request_creator.as_light(),
            state,
            data,
        )))
    }

    pub async fn get_moderation_request_content(
        &self,
        id: ModerationRequestId,
    ) -> Result<(NewModerationRequest, ModerationRequestQueueNumber), SqliteDatabaseError> {
        let request = sqlx::query!(
            r#"
            SELECT json_text, queue_number
            FROM MediaModerationRequest
            WHERE request_row_id = ?
            "#,
            id.request_row_id,
        )
        .fetch_one(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Fetch)?;

        let data: NewModerationRequest = serde_json::from_str(&request.json_text)
            .into_error(SqliteDatabaseError::SerdeDeserialize)?;

        Ok((
            data,
            ModerationRequestQueueNumber {
                number: request.queue_number,
            },
        ))
    }

    pub async fn get_in_progress_moderations(
        &self,
        moderator_id: AccountIdInternal,
    ) -> Result<Vec<Moderation>, SqliteDatabaseError> {
        let account_row_id = moderator_id.row_id();
        let state_in_progress = ModerationRequestState::InProgress as i64;
        let data = sqlx::query!(
            r#"
            SELECT request_row_id, json_text
            FROM MediaModeration
            WHERE account_row_id = ? AND state_number = ?
            "#,
            account_row_id,
            state_in_progress,
        )
        .fetch_all(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Fetch)?;

        let mut new_data = vec![];
        for r in data.into_iter() {
            let data: NewModerationRequest = serde_json::from_str(&r.json_text)
                .into_error(SqliteDatabaseError::SerdeDeserialize)?;

            let moderation = Moderation {
                moderator_id: moderator_id.as_light(),
                request_id: ModerationRequestId {
                    request_row_id: r.request_row_id,
                },
                content: data,
            };
            new_data.push(moderation);
        }

        Ok(new_data)
    }

    pub async fn get_next_active_moderation_request(
        &self,
        sub_queue: i64,
    ) -> Result<Option<ModerationRequestId>, SqliteDatabaseError> {
        let data = sqlx::query!(
            r#"
            SELECT request_row_id
            FROM MediaModerationQueueNumber
                INNER JOIN MediaModerationRequest ON MediaModerationRequest.queue_number = MediaModerationQueueNumber.queue_number
            WHERE sub_queue = ?
            ORDER BY MediaModerationQueueNumber.queue_number ASC
            LIMIT 1
            "#,
            sub_queue,
        )
        .fetch_optional(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Fetch)?;

        let request_row_id = match data.map(|r| r.request_row_id).flatten() {
            None => return Ok(None),
            Some(id) => id,
        };

        Ok(Some(ModerationRequestId { request_row_id }))
    }

    pub async fn moderation(
        &self,
        moderation: ModerationId,
    ) -> Result<NewModerationRequest, SqliteDatabaseError> {
        let account_row_id = moderation.account_id.row_id();
        let content_to_be_moderated = sqlx::query!(
            r#"
            SELECT json_text
            FROM MediaModeration
            WHERE account_row_id = ? AND request_row_id = ?
            "#,
            account_row_id,
            moderation.request_id.request_row_id,
        )
        .fetch_one(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Fetch)?;

        let data: NewModerationRequest = serde_json::from_str(&content_to_be_moderated.json_text)
            .into_error(SqliteDatabaseError::SerdeDeserialize)?;

        Ok(data)
    }
}

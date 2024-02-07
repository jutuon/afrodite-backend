use std::fmt::Debug;

use api_client::{
    apis::media_api::{get_content_slot_state, put_moderation_request}, manual_additions::put_content_to_content_slot_fixed, models::{ContentId, ContentProcessingStateType, MediaContentType, ModerationRequestContent}
};
use async_trait::async_trait;
use error_stack::{Result, ResultExt};

use super::{super::super::client::TestError, BotAction, BotState};
use crate::bot::utils::image::ImageProvider;

#[derive(Debug, Default)]
pub struct MediaState {
    slots: [Option<ContentId>; 3],
}

impl MediaState {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug)]
pub struct SendImageToSlot {
    pub slot: i32,
    pub random: bool,
    pub copy_to_slot: Option<i32>,
    /// Add mark to the image
    pub mark_copied_image: bool,
}

impl SendImageToSlot {
    /// Slot 0 will be used as secure capture every time
    pub const fn slot(slot: i32) -> Self {
        Self {
            slot,
            random: false,
            copy_to_slot: None,
            mark_copied_image: false,
        }
    }
}

#[async_trait]
impl BotAction for SendImageToSlot {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let img_data = if self.random {
            if let Some(dir) = &state.config.images_man {
                ImageProvider::random_image_from_directory(&dir)
                    .unwrap_or_else(|e| {
                        // Image loading failed
                        tracing::error!("{e:?}");
                        Some(ImageProvider::random_jpeg_image())
                    })
                    // No images available
                    .unwrap_or(ImageProvider::random_jpeg_image())
            } else {
                ImageProvider::random_jpeg_image()
            }
        } else {
            ImageProvider::jpeg_image()
        };

        let content_id =
            put_content_to_content_slot_fixed(
                state.api.media(),
                self.slot,
                if self.slot == 0 { true } else { false }, // secure capture
                MediaContentType::JpegImage,
                img_data.clone()
            )
                .await
                .change_context(TestError::ApiRequest)?;

        let content_id = loop {
            let slot_state = get_content_slot_state(state.api.media(), self.slot)
                .await
                .change_context(TestError::ApiRequest)?;

            match slot_state.state {
                ContentProcessingStateType::Empty |
                ContentProcessingStateType::Failed => return Err(TestError::ApiRequest.report()),
                ContentProcessingStateType::Processing |
                ContentProcessingStateType::InQueue =>
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await,
                ContentProcessingStateType::Completed =>
                    break slot_state.content_id.flatten().expect("Content ID is missing"),
            }
        };

        state.media.slots[self.slot as usize] = Some(*content_id);

        let img_data = if self.mark_copied_image {
            ImageProvider::mark_jpeg_image(&img_data).unwrap_or_else(|e| {
                tracing::error!("{e:?}");
                img_data
            })
        } else {
            img_data
        };

        if let Some(slot) = self.copy_to_slot {
            let content_id = put_content_to_content_slot_fixed(
                state.api.media(),
                slot,
                if slot == 0 { true } else { false }, // secure capture
                MediaContentType::JpegImage,
                img_data
            )
                .await
                .change_context(TestError::ApiRequest)?;

            let content_id = loop {
                let slot_state = get_content_slot_state(state.api.media(), self.slot)
                    .await
                    .change_context(TestError::ApiRequest)?;

                match slot_state.state {
                    ContentProcessingStateType::Empty |
                    ContentProcessingStateType::Failed => return Err(TestError::ApiRequest.report()),
                    ContentProcessingStateType::Processing |
                    ContentProcessingStateType::InQueue =>
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await,
                    ContentProcessingStateType::Completed =>
                        break slot_state.content_id.flatten().expect("Content ID is missing"),
                }
            };

            state.media.slots[slot as usize] = Some(*content_id);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct MakeModerationRequest {
    pub camera: bool,
}

#[async_trait]
impl BotAction for MakeModerationRequest {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let mut content_ids: Vec<Option<Box<ContentId>>> = vec![];

        if self.camera {
            content_ids.push(
                Box::new(state.media.slots[0].clone().unwrap_or(
                    ContentId {
                        content_id: uuid::Uuid::new_v4(),
                    },
                ))
                    .into()
            );
        }

        content_ids.push(
            state.media.slots[1]
                .clone()
                .map(|id| Box::new(id))
                .unwrap_or(Box::new(ContentId {
                    content_id: uuid::Uuid::new_v4(),
                }))
                .into()
        );

        content_ids.push(
            state.media.slots[2].clone().map(|id| Box::new(id))
        );

        let new = ModerationRequestContent {
            content0: content_ids[0].clone().expect("Content ID is missing"),
            content1: content_ids[1].clone().into(),
            content2: content_ids[2].clone().into(),
            content3: None,
            content4: None,
            content5: None,
            content6: None,
        };

        put_moderation_request(state.api.media(), new)
            .await
            .change_context(TestError::ApiRequest)?;
        Ok(())
    }
}

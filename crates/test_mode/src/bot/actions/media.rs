use std::{fmt::Debug, path::PathBuf};

use api_client::{
    apis::media_api::{
        get_content_slot_state, put_moderation_request, put_pending_profile_content,
        put_pending_security_content_info,
    },
    manual_additions::put_content_to_content_slot_fixed,
    models::{
        ContentId, ContentProcessingStateType, MediaContentType, ModerationRequestContent,
        SetProfileContent,
    },
};
use async_trait::async_trait;
use config::bot_config_file::{BotConfigFile, BotInstanceConfig, Gender};
use error_stack::{Result, ResultExt};

use super::{super::super::client::TestError, BotAction, BotState};
use crate::bot::utils::image::ImageProvider;

#[derive(Debug, Default)]
pub struct MediaState {
    /// Max slot count and one extra to allow current
    /// content sending code work when testing that sending
    /// content to the extra slot will make an error.
    slots: [Option<ContentId>; 8],
}

impl MediaState {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug)]
pub struct SendImageToSlot {
    pub slot: i32,
    pub random_if_not_defined_in_config: bool,
    pub copy_to_slot: Option<i32>,
    /// Add mark to the image
    pub mark_copied_image: bool,
}

impl SendImageToSlot {
    /// Slot 0 will be used as secure capture every time
    pub const fn slot(slot: i32) -> Self {
        Self {
            slot,
            random_if_not_defined_in_config: false,
            copy_to_slot: None,
            mark_copied_image: false,
        }
    }

    async fn send_to_slot(&self, state: &mut BotState) -> Result<(), TestError> {
        let img_data = if self.random_if_not_defined_in_config {
            let img_path = if let Some(bot) = state.get_bot_config() {
                img_for_bot(bot, &state.bot_config_file)
            } else if let Some(dir) = &state.bot_config_file.man_image_dir {
                if !state.bot_config_file.bot.iter().any(|v| v.image.is_some()) {
                    ImageProvider::random_image_from_directory(dir)
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            };

            match img_path {
                Ok(Some(img_path)) => std::fs::read(img_path).unwrap_or_else(|e| {
                    tracing::error!("{e:?}");
                    ImageProvider::random_jpeg_image()
                }),
                Ok(None) => ImageProvider::random_jpeg_image(),
                Err(e) => {
                    tracing::error!("{e:?}");
                    ImageProvider::random_jpeg_image()
                }
            }
        } else {
            ImageProvider::random_jpeg_image()
        };

        let _ = put_content_to_content_slot_fixed(
            state.api.media(),
            self.slot,
            self.slot == 0, // secure capture
            MediaContentType::JpegImage,
            img_data.clone(),
        )
        .await
        .change_context(TestError::ApiRequest)?;

        async fn wait_for_content_id(
            slot: i32,
            state: &mut BotState,
        ) -> Result<ContentId, TestError> {
            let event_waiting_result = state
                .wait_event(|e| match e.content_processing_state_changed.as_ref() {
                    Some(Some(content_processing_state)) => {
                        content_processing_state.new_state.state
                            == ContentProcessingStateType::Completed
                    }
                    _ => false,
                })
                .await;

            match event_waiting_result {
                Ok(()) => (),
                Err(e) if e.current_context() == &TestError::EventReceivingTimeout => (),
                Err(e) => return Err(e),
            }

            loop {
                let slot_state = get_content_slot_state(state.api.media(), slot)
                    .await
                    .change_context(TestError::ApiRequest)?;

                match slot_state.state {
                    ContentProcessingStateType::Empty | ContentProcessingStateType::Failed => {
                        return Err(TestError::ApiRequest.report())
                    }
                    ContentProcessingStateType::Processing
                    | ContentProcessingStateType::InQueue => {
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await
                    }
                    ContentProcessingStateType::Completed => {
                        return Ok(*slot_state
                            .cid
                            .flatten()
                            .expect("Content ID is missing"))
                    }
                }
            }
        }

        let content_id = wait_for_content_id(self.slot, state).await?;
        state.media.slots[self.slot as usize] = Some(content_id);

        let img_data = if self.mark_copied_image {
            ImageProvider::mark_jpeg_image(&img_data).unwrap_or_else(|e| {
                tracing::error!("{e:?}");
                img_data
            })
        } else {
            img_data
        };

        if let Some(slot) = self.copy_to_slot {
            let _ = put_content_to_content_slot_fixed(
                state.api.media(),
                slot,
                slot == 0, // slot 0 is for secure capture
                MediaContentType::JpegImage,
                img_data,
            )
            .await
            .change_context(TestError::ApiRequest)?;

            let content_id = wait_for_content_id(slot, state).await?;
            state.media.slots[slot as usize] = Some(content_id);
        }

        Ok(())
    }
}

fn img_for_bot(
    bot: &BotInstanceConfig,
    config: &BotConfigFile,
) -> std::result::Result<Option<PathBuf>, std::io::Error> {
    if let Some(image) = bot.get_img(config) {
        Ok(Some(image))
    } else {
        let dir = match bot.img_dir_gender() {
            Gender::Man => config.man_image_dir.clone(),
            Gender::Woman => config.woman_image_dir.clone(),
        };
        if let Some(dir) = dir {
            ImageProvider::random_image_from_directory(&dir)
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl BotAction for SendImageToSlot {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let events_enabled = state.are_events_enabled();
        if !events_enabled {
            state.enable_events();
        }
        let result = self.send_to_slot(state).await;
        if !events_enabled {
            state.disable_events();
        }
        result
    }
}

#[derive(Debug)]
pub struct MakeModerationRequest {
    pub slots_to_request: &'static [usize],
}

#[async_trait]
impl BotAction for MakeModerationRequest {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let mut content_ids: Vec<Option<Box<ContentId>>> = vec![];

        for i in self.slots_to_request {
            content_ids.push(
                match state.media.slots[*i] {
                    Some(content_id) => Box::new(content_id),
                    None => return Err(
                        TestError::MissingValue
                            .report()
                            .attach_printable(format!("Content ID is not set to index {i}"))
                    )
                }
                .into(),
            );
        }

        let new = ModerationRequestContent {
            c0: content_ids[0].clone().expect("Content ID is missing"),
            c1: content_ids.get(1).cloned(),
            c2: content_ids.get(2).cloned(),
            c3: None,
            c4: None,
            c5: None,
            c6: None,
        };

        put_moderation_request(state.api.media(), new)
            .await
            .change_context(TestError::ApiRequest)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct SetPendingContent {
    pub security_content_slot_i: Option<usize>,
    pub content_0_slot_i: Option<usize>,
}

#[async_trait]
impl BotAction for SetPendingContent {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        if let Some(i) = self.security_content_slot_i {
            let content_id = state.media.slots[i].unwrap();
            put_pending_security_content_info(state.api.media(), content_id)
                .await
                .change_context(TestError::ApiRequest)?;
        }

        if let Some(i) = self.content_0_slot_i {
            let content_id = state.media.slots[i].unwrap();
            let bot_info = state.get_bot_config();

            let info = SetProfileContent {
                c0: content_id.into(),
                grid_crop_size: bot_info.and_then(|v| v.grid_crop_size).into(),
                grid_crop_x: bot_info.and_then(|v| v.grid_crop_x).into(),
                grid_crop_y: bot_info.and_then(|v| v.grid_crop_y).into(),
                ..SetProfileContent::default()
            };
            put_pending_profile_content(state.api.media(), info)
                .await
                .change_context(TestError::ApiRequest)?;
        }

        Ok(())
    }
}

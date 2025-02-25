use std::{fmt::Debug, time::Instant};

use api_client::{apis::profile_admin_api, models::ProfileTextModerationRejectedReasonDetails};
use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequest},
    Client,
};
use async_trait::async_trait;
use config::bot_config_file::{LlmModerationConfig, ModerationAction, ProfileTextModerationConfig};
use error_stack::{Result, ResultExt};
use tracing::error;
use unicode_segmentation::UnicodeSegmentation;

use super::{BotAction, BotState, EmptyPage, ModerationResult};
use crate::client::{ApiClient, TestError};

#[derive(Debug)]
pub struct ProfileTextModerationState {
    moderation_started: Option<Instant>,
    client: Option<Client<OpenAIConfig>>,
}

#[derive(Debug)]
pub struct AdminBotProfileTextModerationLogic;

impl AdminBotProfileTextModerationLogic {
    async fn moderate_one_page(
        api: &ApiClient,
        config: &ProfileTextModerationConfig,
        state: &mut ProfileTextModerationState,
    ) -> Result<Option<EmptyPage>, TestError> {
        let list = profile_admin_api::get_profile_text_pending_moderation_list(api.profile(), true)
            .await
            .change_context(TestError::ApiRequest)?;

        if list.values.is_empty() {
            return Ok(Some(EmptyPage));
        }

        for request in list.values {
            // Allow texts with only single visible character
            if config.accept_single_visible_character && request.text.graphemes(true).count() == 1 {
                // Ignore errors as the user might have changed the text to
                // another one or it is already moderated.
                let _ = profile_admin_api::post_moderate_profile_text(
                    api.profile(),
                    api_client::models::PostModerateProfileText {
                        id: request.id.clone(),
                        text: request.text.clone(),
                        accept: true,
                        rejected_category: None,
                        rejected_details: None,
                        move_to_human: None,
                    },
                )
                .await;

                continue;
            }

            let r = if let Some(llm_config) = &config.llm {
                let r = Self::llm_profile_text_moderation(
                    &request.text,
                    llm_config,
                    state,
                ).await?;

                match r {
                    ProfileTextModerationResult::StopModerationSesssion => return Ok(Some(EmptyPage)),
                    ProfileTextModerationResult::Decision(r) => r,
                }
            } else {
                match config.default_action {
                    ModerationAction::Accept => ModerationResult::accept(),
                    ModerationAction::Reject => ModerationResult::reject(None),
                    ModerationAction::MoveToHuman => ModerationResult::move_to_human(),
                }
            };

            // Ignore errors as the user might have changed the text to
            // another one or it is already moderated.
            let _ = profile_admin_api::post_moderate_profile_text(
                api.profile(),
                api_client::models::PostModerateProfileText {
                    id: request.id.clone(),
                    text: request.text.clone(),
                    accept: r.accept,
                    rejected_category: None,
                    rejected_details: r.rejected_details.map(|v| Some(Box::new(ProfileTextModerationRejectedReasonDetails::new(v)))),
                    move_to_human: if r.move_to_human {
                        Some(Some(true))
                    } else {
                        None
                    },
                },
            )
            .await;
        }

        Ok(None)
    }

    async fn llm_profile_text_moderation(
        profile_text: &str,
        config: &LlmModerationConfig,
        state: &mut ProfileTextModerationState,
    ) -> Result<ProfileTextModerationResult, TestError> {
        let client = state.client.get_or_insert_with(||
            Client::with_config(
                OpenAIConfig::new()
                    .with_api_base(config.openai_api_url.to_string())
                    .with_api_key(""),
            )
        );

        let expected_response_lowercase = config.expected_response.to_lowercase();
        let profile_text_paragraph = profile_text.lines().collect::<Vec<&str>>().join(" ");

        let user_text = config.user_text_template.replace(
            ProfileTextModerationConfig::TEMPLATE_FORMAT_ARGUMENT,
            &profile_text_paragraph,
        );

        // Hide warning about max_tokens as Ollama does not yet
        // support max_completion_tokens.
        #[allow(deprecated)]
        let r = client
            .chat()
            .create(CreateChatCompletionRequest {
                messages: vec![
                    ChatCompletionRequestMessage::System(config.system_text.clone().into()),
                    ChatCompletionRequestMessage::User(user_text.into()),
                ],
                model: config.model.clone(),
                temperature: Some(0.0),
                seed: Some(0),
                max_completion_tokens: Some(config.max_tokens),
                max_tokens: Some(config.max_tokens),
                ..Default::default()
            })
            .await;
        let response = match r.map(|r| r.choices.into_iter().next()) {
            Ok(Some(r)) => match r.message.content {
                Some(response) => response,
                None => {
                    error!("Profile text moderation error: no response content from LLM");
                    return Ok(ProfileTextModerationResult::StopModerationSesssion);
                }
            },
            Ok(None) => {
                error!("Profile text moderation error: no response from LLM");
                return Ok(ProfileTextModerationResult::StopModerationSesssion);
            }
            Err(e) => {
                error!("Profile text moderation error: {}", e);
                return Ok(ProfileTextModerationResult::StopModerationSesssion);
            }
        };

        let response_lowercase = response.trim().to_lowercase();
        let response_first_line = response_lowercase.lines().next().unwrap_or_default();
        let accepted = response_lowercase.starts_with(&expected_response_lowercase)
            || response_first_line.contains(&expected_response_lowercase);
        let rejected_details = if !accepted && config.debug_show_llm_output_when_rejected {
            Some(response)
        } else {
            None
        };

        let move_to_human = !accepted && config.move_rejected_to_human_moderation;

        Ok(ProfileTextModerationResult::Decision(ModerationResult {
            accept: accepted,
            rejected_details,
            move_to_human,
        }))
    }
}

#[async_trait]
impl BotAction for AdminBotProfileTextModerationLogic {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let Some(config) = &state.bot_config_file.profile_text_moderation else {
            return Ok(());
        };

        let moderation_state =
            state
                .admin
                .profile_text
                .get_or_insert_with(|| ProfileTextModerationState {
                    moderation_started: None,
                    client: None,
                });

        let start_time = Instant::now();

        if let Some(previous) = moderation_state.moderation_started {
            if start_time.duration_since(previous).as_secs()
                < config.moderation_session_min_seconds.into()
            {
                return Ok(());
            }
        }

        moderation_state.moderation_started = Some(start_time);

        loop {
            if let Some(EmptyPage) = Self::moderate_one_page(
                &state.api,
                config,
                moderation_state,
            )
            .await?
            {
                break;
            }

            let current_time = Instant::now();
            if current_time.duration_since(start_time).as_secs()
                > config.moderation_session_max_seconds.into()
            {
                return Ok(());
            }
        }

        Ok(())
    }
}

enum ProfileTextModerationResult {
    StopModerationSesssion,
    Decision(ModerationResult),
}

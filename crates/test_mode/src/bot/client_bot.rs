//! Bots for fake clients

use std::{
    fmt::Debug,
    iter::Peekable,
    time::{Duration, Instant},
};

use api_client::{
    apis::{account_api::get_account_state, profile_api::post_profile},
    models::{AccountState, ProfileUpdate},
};
use async_trait::async_trait;
use error_stack::{Result, ResultExt};
use tokio::time::sleep;

use super::{
    actions::{
        account::{AssertAccountState, Login, Register, SetAccountSetup, SetProfileVisibility},
        media::SendImageToSlot,
        profile::{GetProfile, UpdateLocationRandom},
        BotAction, RunActions, RunActionsIf,
    },
    BotState, BotStruct, TaskState,
};
use crate::{
    action_array,
    bot::actions::{
        account::CompleteAccountSetup, admin::ModerateMediaModerationRequest,
        media::MakeModerationRequest, ActionArray,
    },
    client::TestError,
};

pub struct ClientBot {
    state: BotState,
    actions: Peekable<Box<dyn Iterator<Item = &'static dyn BotAction> + Send + Sync>>,
}

impl Debug for ClientBot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ClientBot").finish()
    }
}

impl ClientBot {
    pub fn new(state: BotState) -> Self {
        let admin_bot = state
            .config
            .bot_mode()
            .map(|bot_config| state.bot_id < bot_config.admins)
            .unwrap_or(false);

        let iter = if admin_bot {
            // Admin bot

            let setup = [
                &Register as &dyn BotAction,
                &Login,
                &DoInitialSetupIfNeeded { admin: true },
            ];
            const MODERATE_INITIAL: ModerateMediaModerationRequest =
                ModerateMediaModerationRequest::moderate_initial_content();
            const MODERATE_ADDITIONAL: ModerateMediaModerationRequest =
                ModerateMediaModerationRequest::moderate_additional_content();
            let action_loop = [
                &ActionsBeforeIteration as &dyn BotAction,
                &MODERATE_INITIAL,
                &MODERATE_ADDITIONAL,
                &ActionsAfterIteration,
            ];
            let iter = setup.into_iter().chain(action_loop.into_iter().cycle());

            Box::new(iter) as Box<dyn Iterator<Item = &'static dyn BotAction> + Send + Sync>
        } else {
            // User bot

            let setup = [
                &Register as &dyn BotAction,
                &Login,
                &DoInitialSetupIfNeeded { admin: false },
                &UpdateLocationRandom(None),
                &ChangeBotProfileText,
                &SetProfileVisibility(true),
            ];
            let action_loop = [
                &ActionsBeforeIteration as &dyn BotAction,
                &GetProfile,
                &RunActionsIf(
                    action_array!(UpdateLocationRandom(None), ChangeBotProfileText,),
                    || rand::random::<f32>() < 0.2,
                ),
                // TODO: Toggle the profile visiblity in the future?
                &RunActionsIf(action_array!(SetProfileVisibility(true)), || {
                    rand::random::<f32>() < 0.5
                }),
                &RunActionsIf(action_array!(SetProfileVisibility(false)), || {
                    rand::random::<f32>() < 0.1
                }),
                &ActionsAfterIteration,
            ];
            let iter = setup.into_iter().chain(action_loop.into_iter().cycle());

            Box::new(iter) as Box<dyn Iterator<Item = &'static dyn BotAction> + Send + Sync>
        };

        Self {
            state,
            actions: iter.peekable(),
        }
    }
}

#[async_trait]
impl BotStruct for ClientBot {
    fn peek_action_and_state(&mut self) -> (Option<&'static dyn BotAction>, &mut BotState) {
        (self.actions.peek().copied(), &mut self.state)
    }
    fn next_action(&mut self) {
        self.actions.next();
    }
    fn state(&self) -> &BotState {
        &self.state
    }
}

#[derive(Debug)]
pub struct DoInitialSetupIfNeeded {
    admin: bool,
}

#[async_trait]
impl BotAction for DoInitialSetupIfNeeded {
    async fn excecute_impl_task_state(
        &self,
        state: &mut BotState,
        task_state: &mut TaskState,
    ) -> Result<(), TestError> {
        let account_state = get_account_state(state.api.account())
            .await
            .change_context(TestError::ApiRequest)?;

        if account_state.state == AccountState::InitialSetup {
            let name = format!("Bot {}", state.bot_id);
            let email = format!("bot{}@example.com", state.bot_id);
            if self.admin {
                SetAccountSetup::admin()
            } else {
                SetAccountSetup {
                    name: Some(&name),
                    email: Some(&email),
                }
            }
            .excecute_impl_task_state(state, task_state)
            .await?;

            const ACTIONS: ActionArray = action_array!(
                SendImageToSlot {
                    slot: 1,
                    random: true,
                    copy_to_slot: Some(0),
                    mark_copied_image: true,
                },
                MakeModerationRequest { slot_0_secure_capture: true },
                CompleteAccountSetup,
                AssertAccountState(AccountState::Normal),
            );
            RunActions(ACTIONS)
                .excecute_impl_task_state(state, task_state)
                .await?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct ChangeBotProfileText;

#[async_trait]
impl BotAction for ChangeBotProfileText {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let profile = ProfileUpdate::new(format!(
            "Hello! My location is\n{:#?}",
            state.previous_value.location()
        ));
        post_profile(state.api.profile(), profile)
            .await
            .change_context(TestError::ApiRequest)?;
        Ok(())
    }
}

#[derive(Debug)]
struct ActionsBeforeIteration;

#[async_trait]
impl BotAction for ActionsBeforeIteration {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        if !state.config.no_sleep() {
            sleep(Duration::from_millis(1000)).await;
        }

        state.benchmark.action_duration = Instant::now();

        Ok(())
    }
}

#[derive(Debug)]
struct ActionsAfterIteration;

#[async_trait]
impl BotAction for ActionsAfterIteration {
    async fn excecute_impl(&self, _state: &mut BotState) -> Result<(), TestError> {
        Ok(())
    }
}

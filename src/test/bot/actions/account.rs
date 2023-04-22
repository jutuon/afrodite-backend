
use std::fmt::{Debug, Display};

use api_client::{apis::{account_api::{post_register, post_login, post_account_setup, get_account_state}, profile_api::{post_profile, get_profile, get_default_profile}}, models::{Profile, account_setup, AccountSetup, AccountState}};
use async_trait::async_trait;
use nalgebra::U8;

use error_stack::{Result, FutureExt, ResultExt};

use tracing::{error, log::warn};

use super::{super::super::client::{ApiClient, TestError}, BotAction};

use crate::{
    api::model::AccountId,
    config::args::{Test, TestMode},
    utils::IntoReportExt, test::bot::utils::{name::NameProvider, assert::bot_assert_eq},
};

use super::BotState;

#[derive(Debug)]
pub struct Register;

#[async_trait]
impl BotAction for Register {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        if state.id.is_some() {
            return Ok(());
        }

        let id = post_register(state.api.account())
            .await
            .into_error(TestError::ApiRequest)?;
        state.id = Some(id);
        Ok(())
    }
}

#[derive(Debug)]
pub struct Login;

#[async_trait]
impl BotAction for Login {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        if state.api.is_api_key_available() {
            return Ok(());
        }
        let key = post_login(state.api.account(), state.id()?)
            .await
            .into_error(TestError::ApiRequest)?;

        state.api.set_api_key(key);
        Ok(())
    }
}

#[derive(Debug)]
pub struct RequireAccountState(pub AccountState);

#[async_trait]
impl BotAction for RequireAccountState {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let state = get_account_state(state.api.account())
            .await
            .into_error(TestError::ApiRequest)?;

        bot_assert_eq(state.state, self.0)
    }
}

#[derive(Debug)]
pub struct SetAccountSetup;

#[async_trait]
impl BotAction for SetAccountSetup {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let setup = AccountSetup {
            name: NameProvider::men_first_name().to_string(),
        };
        post_account_setup(state.api.account(), setup)
            .await
            .into_error(TestError::ApiRequest)?;

        Ok(())
    }
}

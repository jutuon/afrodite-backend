use std::fmt::Debug;

use async_trait::async_trait;
use error_stack::{Result, ResultExt};

use super::{super::super::client::TestError, BotAction, BotState};

#[derive(Debug)]
pub struct TestWebSocket;

#[async_trait]
impl BotAction for TestWebSocket {
    async fn excecute_impl(&self, _state: &mut BotState) -> Result<(), TestError> {
        // TODO: get new refresh token and API key
        Ok(())
    }
}

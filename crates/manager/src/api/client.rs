
use manager_api::{protocol::RequestSenderCmds, ClientError};
use manager_model::{JsonRpcRequest, JsonRpcResponse, ManagerInstanceName};

use crate::server::app::S;

use super::server::json_rpc::handle_rpc_request;

use error_stack::{Result, ResultExt};

pub struct LocalOrRemoteApiClient<'a> {
    request_receiver: ManagerInstanceName,
    state: &'a S,
}

impl<'a> LocalOrRemoteApiClient<'a> {
    pub fn new(
        request_receiver: ManagerInstanceName,
        state: &'a S,
    ) -> Self {
        Self {
            request_receiver,
            state,
        }
    }
}

impl RequestSenderCmds for LocalOrRemoteApiClient<'_> {
    fn request_receiver_name(&self) -> ManagerInstanceName {
        self.request_receiver.clone()
    }
    async fn send_request(
        self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, ClientError> {
        handle_rpc_request(request, None, self.state)
            .await
            .change_context(ClientError::JsonRpc)
    }
}

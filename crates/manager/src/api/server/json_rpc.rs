

use std::net::SocketAddr;
use manager_api::ClientConfig;
use manager_api::ManagerClient;
use manager_model::JsonRpcRequest;
use manager_model::JsonRpcRequestType;
use manager_model::JsonRpcResponse;
use scheduled_task::RpcScheduledTask;
use secure_storage::RpcSecureStorage;
use software::RpcSoftware;
use system_info::RpcSystemInfo;
use task::RpcTask;
use tracing::info;
use crate::api::GetConfig;

use manager_api::protocol::{ConnectionUtilsRead, ConnectionUtilsWrite};

use tracing::error;

use crate::server::app::S;

use error_stack::{Result, ResultExt};
use super::ClientConnectionReadWrite;
use super::ServerError;

pub mod software;
pub mod secure_storage;
pub mod system_info;
pub mod task;
pub mod scheduled_task;


#[derive(thiserror::Error, Debug)]
pub enum JsonRpcError {
    #[error("Secure storage encryption key not found")]
    SecureStorageEncryptionKeyNotFound,
    #[error("Secure storage encryption key reading failed")]
    SecureStorageEncryptionKeyRead,
    #[error("System info error")]
    SystemInfo,
    #[error("Task manager error")]
    TaskManager,
    #[error("Scheduled task manager error")]
    ScheduledTaskManager,
    #[error("Update manager error")]
    UpdateManager,
}


pub async fn handle_json_rpc<
    C: ClientConnectionReadWrite,
>(
    mut c: C,
    address: SocketAddr,
    state: S,
) -> Result<(), ServerError> {
    let request = c.receive_json_rpc_request()
        .await
        .change_context(ServerError::JsonRpcRequestReceivingFailed)?;

    let response = handle_request(
        request,
        address,
        &state,
    ).await?;

    c.send_json_rpc_response(response)
        .await
        .change_context(ServerError::JsonRpcResponseSendingFailed)?;

    Ok(())
}

async fn handle_request(
    request: JsonRpcRequest,
    address: SocketAddr,
    state: &S,
) -> Result<JsonRpcResponse, ServerError> {
    if request.receiver == state.config().manager_name() {
        info!("Running RPC {:?} from {}", &request.request, address);
        handle_request_type(
            request.request,
            state,
        )
            .await
            .change_context(ServerError::JsonRpcFailed)
    } else if let Some(m) = state.config().find_remote_manager(&request.receiver)  {
        info!("Forwarding RPC {:?} from {}", &request.request, address);
        let config = ClientConfig {
            url: m.url.clone(),
            root_certificate: state.config().root_certificate(),
            api_key: state.config().api_key().to_string(),
        };
        let client = ManagerClient::connect(config)
            .await
            .change_context(ServerError::Client)?;
        let response = client.send_request(request)
            .await
            .change_context(ServerError::Client)?;
        Ok(response)
    } else {
        Ok(JsonRpcResponse::request_receiver_not_found())
    }
}

pub async fn handle_request_type(
    request: JsonRpcRequestType,
    state: &S,
) -> Result<JsonRpcResponse, JsonRpcError> {
    match request {
        JsonRpcRequestType::GetManagerInstanceNames =>
            state.rpc_get_manager_instance_names().await,
        JsonRpcRequestType::GetSecureStorageEncryptionKey(name) =>
            state.rpc_get_secure_storage_encryption_key(
                name,
            ).await,
        JsonRpcRequestType::GetSystemInfo =>
            state.rpc_get_system_info().await,
        JsonRpcRequestType::GetSoftwareUpdateStatus =>
            state.rpc_get_software_update_status().await,
        JsonRpcRequestType::TriggerSoftwareUpdateTask(task) =>
            state.rpc_trigger_update_manager_related_action(
                task,
            ).await,
        JsonRpcRequestType::TriggerManualTask(task) =>
            state.rpc_trigger_manual_task(task).await,
        JsonRpcRequestType::ScheduleTask(task, notify_backend) =>
            state.rpc_schedule_task(task, notify_backend.notify_backend).await,
        JsonRpcRequestType::UnscheduleTask(task) =>
            state.rpc_unschedule_task(task).await,
        JsonRpcRequestType::GetScheduledTasksStatus =>
            state.rpc_get_scheduled_tasks_status().await,
    }
}

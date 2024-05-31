use std::net::SocketAddr;

use config::{file::ConfigFileError, file_dynamic::ConfigFileDynamic, Config};
use error_stack::{Result, ResultExt};
use futures::Future;
use model::{
    AccessToken, AccountId, AccountIdInternal, AccountState, BackendConfig, BackendVersion,
    Capabilities, EmailAddress, PendingNotificationFlags, PushNotificationStateInfo,
    SignInWithInfo,
};
pub use server_api::app::*;
use server_api::{db_write_raw, internal_api::InternalApiClient, utils::StatusCode};
use server_common::push_notifications::{PushNotificationError, PushNotificationStateProvider};
use server_data::{
    content_processing::ContentProcessingManagerData,
    event::EventManagerWithCacheReference,
    read::ReadCommandsContainer,
    write_commands::WriteCmds,
    write_concurrent::{ConcurrentWriteAction, ConcurrentWriteSelectorHandle},
    DataError,
};
use server_data_chat::write::GetWriteCommandsChat;
use simple_backend::{
    app::{GetManagerApi, GetSimpleBackendConfig, GetTileMap, PerfCounterDataProvider, SignInWith},
    manager_client::ManagerApiManager,
    map::TileMapManager,
    perf::PerfCounterManagerData,
    sign_in_with::SignInWithManager,
};
use simple_backend_config::SimpleBackendConfig;

use super::S;

// Server common

impl EventManagerProvider for S {
    fn event_manager(&self) -> EventManagerWithCacheReference<'_> {
        EventManagerWithCacheReference::new(self.database.cache(), &self.push_notification_sender)
    }
}

impl GetAccounts for S {
    async fn get_internal_id(&self, id: AccountId) -> Result<AccountIdInternal, DataError> {
        self.database
            .account_id_manager()
            .get_internal_id(id)
            .await
            .map_err(|e| e.into_report())
    }
}

impl ReadDynamicConfig for S {
    async fn read_config(&self) -> error_stack::Result<BackendConfig, ConfigFileError> {
        let config = tokio::task::spawn_blocking(ConfigFileDynamic::load_from_current_dir)
            .await
            .change_context(ConfigFileError::LoadConfig)??;

        Ok(config.backend_config)
    }
}

impl BackendVersionProvider for S {
    fn backend_version(&self) -> BackendVersion {
        BackendVersion {
            backend_code_version: self
                .simple_backend_config()
                .backend_code_version()
                .to_string(),
            backend_version: self
                .simple_backend_config()
                .backend_semver_version()
                .to_string(),
            protocol_version: "1.0.0".to_string(),
        }
    }
}

impl GetConfig for S {
    fn config(&self) -> &Config {
        &self.config
    }
}

impl WriteDynamicConfig for S {
    async fn write_config(
        &self,
        config: BackendConfig,
    ) -> error_stack::Result<(), ConfigFileError> {
        tokio::task::spawn_blocking(move || {
            if let Some(bots) = config.bots {
                ConfigFileDynamic::edit_bot_config_from_current_dir(bots)?
            }

            Result::<(), ConfigFileError>::Ok(())
        })
        .await
        .change_context(ConfigFileError::LoadConfig)??;

        Ok(())
    }
}

impl PushNotificationStateProvider for S {
    async fn get_push_notification_state_info_and_add_notification_value(
        &self,
        account_id: AccountIdInternal,
        flags: PendingNotificationFlags,
    ) -> Result<PushNotificationStateInfo, PushNotificationError> {
        db_write_raw!(self, move |cmds| {
            cmds.chat()
                .push_notifications()
                .get_push_notification_state_info_and_add_notification_value(
                    account_id,
                    flags.into(),
                )
                .await
        })
        .await
        .map_err(|e| e.into_report())
        .change_context(PushNotificationError::SettingPushNotificationSentFlagFailed)
    }

    async fn enable_push_notification_sent_flag(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<(), PushNotificationError> {
        db_write_raw!(self, move |cmds| {
            cmds.chat()
                .push_notifications()
                .enable_push_notification_sent_flag(account_id)
                .await
        })
        .await
        .map_err(|e| e.into_report())
        .change_context(PushNotificationError::SettingPushNotificationSentFlagFailed)
    }

    async fn remove_device_token(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<(), PushNotificationError> {
        db_write_raw!(self, move |cmds| {
            cmds.chat()
                .push_notifications()
                .remove_device_token(account_id)
                .await
        })
        .await
        .map_err(|e| e.into_report())
        .change_context(PushNotificationError::RemoveDeviceTokenFailed)
    }
}

// Server data

impl WriteData for S {
    async fn write<
        CmdResult: Send + 'static,
        Cmd: Future<Output = server_common::result::Result<CmdResult, DataError>> + Send + 'static,
        GetCmd: FnOnce(WriteCmds) -> Cmd + Send + 'static,
    >(
        &self,
        cmd: GetCmd,
    ) -> server_common::result::Result<CmdResult, DataError> {
        self.write_queue.write(cmd).await
    }

    // async fn write<
    //     CmdResult: Send + 'static,
    //     Cmd: Future<Output = server_common::result::Result<CmdResult, DataError>> + Send,
    //     GetCmd,
    // >(
    //     &self,
    //     write_cmd: GetCmd,
    // ) -> server_common::result::Result<CmdResult, DataError> where GetCmd: FnOnce(SyncWriteHandleRef<'_>) -> Cmd + Send + 'static,  {
    //     self.write_queue.write_with_ref_handle(write_cmd).await
    // }

    async fn write_concurrent<
        CmdResult: Send + 'static,
        Cmd: Future<Output = ConcurrentWriteAction<CmdResult>> + Send + 'static,
        GetCmd: FnOnce(ConcurrentWriteSelectorHandle) -> Cmd + Send + 'static,
    >(
        &self,
        account: AccountId,
        cmd: GetCmd,
    ) -> server_common::result::Result<CmdResult, DataError> {
        self.write_queue.concurrent_write(account, cmd).await
    }
}

impl ReadData for S {
    fn read(&self) -> ReadCommandsContainer {
        ReadCommandsContainer::new(self.database.read())
    }
}

// Server API

impl StateBase for S {}

impl GetInternalApi for S {
    fn internal_api_client(&self) -> &InternalApiClient {
        &self.internal_api
    }
}

impl GetAccessTokens for S {
    async fn access_token_exists(&self, token: &AccessToken) -> Option<AccountIdInternal> {
        self.database
            .access_token_manager()
            .access_token_exists(token)
            .await
    }

    async fn access_token_and_connection_exists(
        &self,
        token: &AccessToken,
        connection: SocketAddr,
    ) -> Option<(AccountIdInternal, Capabilities, AccountState)> {
        self.database
            .access_token_manager()
            .access_token_and_connection_exists(token, connection)
            .await
    }
}

impl ContentProcessingProvider for S {
    fn content_processing(&self) -> &ContentProcessingManagerData {
        &self.content_processing
    }
}

impl DemoModeManagerProvider for S {
    async fn stage0_login(
        &self,
        password: model::DemoModePassword,
    ) -> error_stack::Result<model::DemoModeLoginResult, DataError> {
        self.demo_mode.stage0_login(password).await
    }

    async fn stage1_login(
        &self,
        password: model::DemoModePassword,
        token: model::DemoModeLoginToken,
    ) -> error_stack::Result<model::DemoModeConfirmLoginResult, DataError> {
        self.demo_mode.stage1_login(password, token).await
    }

    async fn demo_mode_token_exists(
        &self,
        token: &model::DemoModeToken,
    ) -> error_stack::Result<model::DemoModeId, DataError> {
        self.demo_mode.demo_mode_token_exists(token).await
    }

    async fn accessible_accounts_if_token_valid<
        S: StateBase + GetConfig + GetAccounts + ReadData,
    >(
        &self,
        state: &S,
        token: &model::DemoModeToken,
    ) -> server_common::result::Result<Vec<model::AccessibleAccount>, DataError> {
        let info = self
            .demo_mode
            .accessible_accounts_if_token_valid(token)
            .await?;
        info.with_extra_info(state).await
    }
}

impl RegisteringCmd for S {
    async fn register_impl(
        &self,
        sign_in_with: SignInWithInfo,
        email: Option<EmailAddress>,
    ) -> std::result::Result<AccountIdInternal, StatusCode> {
        server_api_all::register::register_impl(self, sign_in_with, email).await
    }
}

impl ValidateModerationRequest for S {
    async fn media_check_moderation_request_for_account(
        &self,
        account_id: AccountIdInternal,
    ) -> server_common::result::Result<(), server_common::internal_api::InternalApiError> {
        server_api_media::internal_api::media_check_moderation_request_for_account(self, account_id)
            .await
    }
}

// Simple backend

impl SignInWith for S {
    fn sign_in_with_manager(&self) -> &SignInWithManager {
        &self.simple_backend_state.sign_in_with
    }
}

impl GetManagerApi for S {
    fn manager_api(&self) -> ManagerApiManager {
        ManagerApiManager::new(&self.simple_backend_state.manager_api)
    }
}

impl GetSimpleBackendConfig for S {
    fn simple_backend_config(&self) -> &SimpleBackendConfig {
        &self.simple_backend_state.config
    }
}

impl GetTileMap for S {
    fn tile_map(&self) -> &TileMapManager {
        &self.simple_backend_state.tile_map
    }
}

impl PerfCounterDataProvider for S {
    fn perf_counter_data(&self) -> &PerfCounterManagerData {
        &self.simple_backend_state.perf_data
    }
}
use std::net::SocketAddr;

use axum::extract::ws::WebSocket;
use model::{
    AccessToken, AccessibleAccount, AccountIdInternal, AccountState, Permissions, DemoModeConfirmLoginResult, DemoModeId, DemoModeLoginResult, DemoModeLoginToken, DemoModePassword, DemoModeToken, EmailAddress, PublicKeyIdAndVersion, SignInWithInfo, SyncDataVersionFromClient
};
use server_common::internal_api::InternalApiError;
pub use server_data::app::*;
use server_data::{content_processing::ContentProcessingManagerData, DataError};

use crate::{common::WebSocketError, internal_api::InternalApiClient, utils::StatusCode};

pub trait GetInternalApi {
    fn internal_api_client(&self) -> &InternalApiClient;
}

pub trait GetAccessTokens {
    fn access_token_exists(
        &self,
        token: &AccessToken,
    ) -> impl std::future::Future<Output = Option<AccountIdInternal>> + Send;

    /// Check that token and current connection IP and port matches
    /// with WebSocket connection.
    fn access_token_and_connection_exists(
        &self,
        token: &AccessToken,
        connection: SocketAddr,
    ) -> impl std::future::Future<Output = Option<(AccountIdInternal, Permissions, AccountState)>> + Send;
}

pub trait ContentProcessingProvider {
    fn content_processing(&self) -> &ContentProcessingManagerData;
}

pub trait StateBase: Send + Sync + Clone + 'static {}

pub trait ValidateModerationRequest: GetConfig + ReadData + GetInternalApi {
    fn media_check_moderation_request_for_account(
        &self,
        account_id: AccountIdInternal,
    ) -> impl std::future::Future<Output = server_common::result::Result<(), InternalApiError>> + Send;
}

pub trait CompleteInitialSetupCmd: ReadData + WriteData + GetInternalApi + GetConfig + ValidateModerationRequest {
    fn complete_initial_setup(
        &self,
        account_id: AccountIdInternal,
    ) -> impl std::future::Future<Output = std::result::Result<(), StatusCode>> + Send;
}

pub trait RegisteringCmd: WriteData {
    fn register_impl(
        &self,
        sign_in_with: SignInWithInfo,
        email: Option<EmailAddress>,
    ) -> impl std::future::Future<Output = Result<AccountIdInternal, StatusCode>> + Send;
}

pub trait DemoModeManagerProvider: StateBase {
    fn stage0_login(
        &self,
        password: DemoModePassword,
    ) -> impl std::future::Future<Output = error_stack::Result<DemoModeLoginResult, DataError>> + Send;

    fn stage1_login(
        &self,
        password: DemoModePassword,
        token: DemoModeLoginToken,
    ) -> impl std::future::Future<Output = error_stack::Result<DemoModeConfirmLoginResult, DataError>>
           + Send;

    fn demo_mode_token_exists(
        &self,
        token: &DemoModeToken,
    ) -> impl std::future::Future<Output = error_stack::Result<DemoModeId, DataError>> + Send;

    fn accessible_accounts_if_token_valid<S: StateBase + GetConfig + GetAccounts + ReadData>(
        &self,
        state: &S,
        token: &DemoModeToken,
    ) -> impl std::future::Future<
        Output = server_common::result::Result<Vec<AccessibleAccount>, DataError>,
    > + Send;
}

pub trait ConnectionTools: StateBase + WriteData + ReadData + GetConfig {
    fn reset_pending_notification(
        &self,
        id: AccountIdInternal,
    ) -> impl std::future::Future<Output = server_common::result::Result<(), WebSocketError>> + Send;

    fn send_new_messages_event_if_needed(
        &self,
        socket: &mut WebSocket,
        id: AccountIdInternal,
    ) -> impl std::future::Future<Output = server_common::result::Result<(), WebSocketError>> + Send;

    fn sync_data_with_client_if_needed(
        &self,
        socket: &mut WebSocket,
        id: AccountIdInternal,
        sync_versions: Vec<SyncDataVersionFromClient>,
    ) -> impl std::future::Future<Output = server_common::result::Result<(), WebSocketError>> + Send;
}

pub trait ResetPushNotificationTokens: StateBase + WriteData {
    fn reset_push_notification_tokens(
        &self,
        id: AccountIdInternal,
    ) -> impl std::future::Future<Output = server_common::result::Result<(), DataError>> + Send;
}

pub trait IsMatch: StateBase + ReadData {
    /// Account interaction is in match state and there is no one or two way block.
    fn is_match(
        &self,
        account0: AccountIdInternal,
        account1: AccountIdInternal,
    ) -> impl std::future::Future<Output = server_common::result::Result<bool, DataError>> + Send;
}

pub trait UpdateUnlimitedLikes: StateBase + WriteData {
    fn update_unlimited_likes(
        &self,
        id: AccountIdInternal,
        unlimited_likes: bool,
    ) -> impl std::future::Future<Output = server_common::result::Result<(), DataError>> + Send;
}

pub trait LatestPublicKeysInfo: StateBase + WriteData {
    fn latest_public_keys_info(
        &self,
        id: AccountIdInternal,
    ) -> impl std::future::Future<Output = server_common::result::Result<Vec<PublicKeyIdAndVersion>, DataError>> + Send;
}

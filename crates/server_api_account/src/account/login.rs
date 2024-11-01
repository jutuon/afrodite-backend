use axum::extract::State;
use model::{
    AccessToken, AccountId, AuthPair, EmailAddress, GoogleAccountId, LoginResult, RefreshToken,
    SignInWithInfo, SignInWithLoginInfo,
};
use obfuscate_api_macro::obfuscate_api;
use server_api::{app::{LatestPublicKeysInfo, RegisteringCmd, ResetPushNotificationTokens}, db_write};
use server_data::write::GetWriteCommandsCommon;
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::{app::SignInWith, create_counters};

use crate::{
    app::{GetAccessTokens, GetAccounts, GetConfig, ReadData, WriteData},
    utils::{Json, StatusCode},
};

pub async fn login_impl<S: ReadData + WriteData + GetAccounts + ResetPushNotificationTokens + LatestPublicKeysInfo>(
    id: AccountId,
    state: S,
) -> Result<LoginResult, StatusCode> {
    let id = state.get_internal_id(id).await?;
    let email = state.read().account().account_data(id).await?;
    let latest_public_keys = state.latest_public_keys_info(id).await?;

    let access = AccessToken::generate_new();
    let refresh = RefreshToken::generate_new();
    let account = AuthPair { access, refresh };
    let account_clone = account.clone();

    state.reset_push_notification_tokens(id).await?;

    db_write!(state, move |cmds| cmds.common().set_new_auth_pair(
        id,
        account_clone,
        None
    ))?;

    // TODO(microservice): microservice support

    let result = LoginResult {
        account: Some(account),
        profile: None,
        media: None,
        aid: Some(id.as_id()),
        email: email.email,
        latest_public_keys,
        error_unsupported_client: false,
    };
    Ok(result)
}

#[obfuscate_api]
pub const PATH_SIGN_IN_WITH_LOGIN: &str = "/account_api/sign_in_with_login";

/// Start new session with sign in with Apple or Google. Creates new account if
/// it does not exists.
#[utoipa::path(
    post,
    path = PATH_SIGN_IN_WITH_LOGIN,
    security(),
    request_body = SignInWithLoginInfo,
    responses(
        (status = 200, description = "Login or account creation successful.", body = LoginResult),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn post_sign_in_with_login<
    S: GetAccessTokens + WriteData + ReadData + GetAccounts + SignInWith + GetConfig + RegisteringCmd + ResetPushNotificationTokens + LatestPublicKeysInfo,
>(
    State(state): State<S>,
    Json(tokens): Json<SignInWithLoginInfo>,
) -> Result<Json<LoginResult>, StatusCode> {
    ACCOUNT.post_sign_in_with_login.incr();

    if tokens.client_info.is_unsupported_client() {
        return Ok(LoginResult::error_unsupported_client().into());
    }

    if let Some(google) = tokens.google_token {
        let info = state
            .sign_in_with_manager()
            .validate_google_token(google)
            .await?;

        let email: EmailAddress = info
            .email
            .try_into()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let google_id = GoogleAccountId(info.id);
        let already_existing_account = state
            .read()
            .account()
            .google_account_id_to_account_id(google_id.clone())
            .await?;

        if let Some(already_existing_account) = already_existing_account {
            db_write!(state, move |cmds| cmds
                .account()
                .email()
                .account_email(already_existing_account, email))?;

            login_impl(already_existing_account.as_id(), state)
                .await
                .map(|d| d.into())
        } else {
            let id = state
                .register_impl(
                    SignInWithInfo {
                        google_account_id: Some(google_id),
                    },
                    Some(email),
                )
                .await?;
            login_impl(id.as_id(), state).await.map(|d| d.into())
        }
    } else if let Some(apple) = tokens.apple_token {
        let _info = state
            .sign_in_with_manager()
            .validate_apple_token(apple)
            .await?;

        // if validate_sign_in_with_apple_token(apple).await.unwrap() {
        //     let key = AccessToken::generate_new();
        //     Ok(key.into())
        // } else {
        //     Err(StatusCode::INTERNAL_SERVER_ERROR)
        // }
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_LOGIN_COUNTERS_LIST,
    post_sign_in_with_login,
);

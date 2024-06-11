use axum::{
    extract::{Path, Query, State},
    Extension, Router,
};
use model::{
    AccountId, AccountIdInternal, AccountState, Capabilities, GetProfileContentQueryParams, GetProfileContentResult, PendingProfileContent, ProfileContent, SetProfileContent
};
use server_api::app::IsMatch;
use server_data::read::GetReadCommandsCommon;
use server_data_media::{read::GetReadMediaCommands, write::GetWriteCommandsMedia};
use simple_backend::create_counters;

use crate::{
    app::{GetAccounts, ReadData, StateBase, WriteData},
    db_write,
    utils::{Json, StatusCode},
};

pub const PATH_GET_PROFILE_CONTENT_INFO: &str = "/media_api/profile_content_info/:account_id";

/// Get current profile content for selected profile.
///
/// # Access
///
/// ## Own profile
/// Unrestricted access.
///
/// ## Other profiles
/// Normal account state required.
///
/// ## Private other profiles
/// If the profile is a match, then the profile can be accessed if query
/// parameter `is_match` is set to `true`.
///
/// If the profile is not a match, then capability `admin_view_all_profiles`
/// is required.
#[utoipa::path(
    get,
    path = "/media_api/profile_content_info/{account_id}",
    params(AccountId, GetProfileContentQueryParams),
    responses(
        (status = 200, description = "Get profile content info.", body = GetProfileContentResult),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_content_info<S: ReadData + GetAccounts + IsMatch>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Extension(account_state): Extension<AccountState>,
    Extension(capabilities): Extension<Capabilities>,
    Path(requested_profile): Path<AccountId>,
    Query(params): Query<GetProfileContentQueryParams>,
) -> Result<Json<GetProfileContentResult>, StatusCode> {
    MEDIA.get_profile_content_info.incr();

    let requested_profile = state.get_internal_id(requested_profile).await?;

    let read_profile_action = || async {
        let internal = state
            .read()
            .media()
            .current_account_media(requested_profile)
            .await?;

        let info: ProfileContent = internal.clone().into();

        match params.version() {
            Some(param_version) if param_version == internal.profile_content_version_uuid =>
                Ok(GetProfileContentResult::current_version_latest_response(internal.profile_content_version_uuid).into()),
            _ => Ok(GetProfileContentResult::content_with_version(info, internal.profile_content_version_uuid).into()),
        }
    };

    if account_id.as_id() == requested_profile.as_id() {
        return read_profile_action().await;
    }

    if account_state != AccountState::Normal {
        return Ok(GetProfileContentResult::empty().into());
    }

    let visibility = state
        .read()
        .common()
        .account(requested_profile)
        .await?
        .profile_visibility()
        .is_currently_public();

    if visibility ||
        capabilities.admin_view_all_profiles ||
        (params.allow_get_content_if_match() && state.is_match(account_id, requested_profile).await?)
    {
        read_profile_action().await
    } else {
        Ok(GetProfileContentResult::empty().into())
    }
}

pub const PATH_PUT_PROFILE_CONTENT: &str = "/media_api/profile_content";

/// Set new profile content for current account.
///
/// # Restrictions
/// - All content must be moderated as accepted.
/// - All content must be owned by the account.
/// - All content must be images.
#[utoipa::path(
    put,
    path = "/media_api/profile_content",
    request_body(content = SetProfileContent),
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn put_profile_content<S: WriteData>(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Json(new): Json<SetProfileContent>,
) -> Result<(), StatusCode> {
    MEDIA.put_profile_content.incr();

    db_write!(state, move |cmds| cmds
        .media()
        .update_profile_content(api_caller_account_id, new))
}

pub const PATH_GET_PENDING_PROFILE_CONTENT_INFO: &str =
    "/media_api/pending_profile_content_info/:account_id";

/// Get pending profile content for selected profile
#[utoipa::path(
    get,
    path = "/media_api/pending_profile_content_info/{account_id}",
    params(AccountId),
    responses(
        (status = 200, description = "Successful.", body = PendingProfileContent),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_pending_profile_content_info<S: ReadData + GetAccounts>(
    State(state): State<S>,
    Path(account_id): Path<AccountId>,
    Extension(_api_caller_account_id): Extension<AccountIdInternal>,
) -> Result<Json<PendingProfileContent>, StatusCode> {
    MEDIA.get_pending_profile_content_info.incr();

    // TODO: access restrictions

    let internal_id = state.get_internal_id(account_id).await?;

    let internal_current_media = state
        .read()
        .media()
        .current_account_media(internal_id)
        .await?;

    let info: PendingProfileContent = internal_current_media.into();
    Ok(info.into())
}

pub const PATH_PUT_PENDING_PROFILE_CONTENT: &str = "/media_api/pending_profile_content";

/// Set new pending profile content for current account.
/// Server will switch to pending content when next moderation request is
/// accepted.
///
/// # Restrictions
/// - All content must not be moderated as rejected.
/// - All content must be owned by the account.
/// - All content must be images.
#[utoipa::path(
    put,
    path = "/media_api/pending_profile_content",
    request_body(content = SetProfileContent),
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn put_pending_profile_content<S: WriteData>(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Json(new): Json<SetProfileContent>,
) -> Result<(), StatusCode> {
    MEDIA.put_pending_profile_content.incr();

    db_write!(state, move |cmds| cmds
        .media()
        .update_or_delete_pending_profile_content(
            api_caller_account_id,
            Some(new)
        ))
}

pub const PATH_DELETE_PENDING_PROFILE_CONTENT: &str = "/media_api/pending_profile_content";

/// Delete new pending profile content for current account.
/// Server will not switch to pending content when next moderation request is
/// accepted.
#[utoipa::path(
    delete,
    path = "/media_api/pending_profile_content",
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn delete_pending_profile_content<S: WriteData>(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
) -> Result<(), StatusCode> {
    MEDIA.delete_pending_profile_content.incr();

    db_write!(state, move |cmds| cmds
        .media()
        .update_or_delete_pending_profile_content(
            api_caller_account_id,
            None
        ))
}

pub fn profile_content_router<S: StateBase + WriteData + ReadData + GetAccounts + IsMatch>(s: S) -> Router {
    use axum::routing::{delete, get, put};

    Router::new()
        .route(
            PATH_GET_PROFILE_CONTENT_INFO,
            get(get_profile_content_info::<S>),
        )
        .route(PATH_PUT_PROFILE_CONTENT, put(put_profile_content::<S>))
        .route(
            PATH_GET_PENDING_PROFILE_CONTENT_INFO,
            get(get_pending_profile_content_info::<S>),
        )
        .route(
            PATH_PUT_PENDING_PROFILE_CONTENT,
            put(put_pending_profile_content::<S>),
        )
        .route(
            PATH_DELETE_PENDING_PROFILE_CONTENT,
            delete(delete_pending_profile_content::<S>),
        )
        .with_state(s)
}

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_PROFILE_CONTENT_COUNTERS_LIST,
    get_profile_content_info,
    put_profile_content,
    get_pending_profile_content_info,
    put_pending_profile_content,
    delete_pending_profile_content,
);

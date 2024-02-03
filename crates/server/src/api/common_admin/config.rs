use axum::{extract::State, Extension, Router};
use model::{AccountIdInternal, BackendConfig, Capabilities};
use simple_backend::create_counters;
use tracing::info;

use crate::{
    api::utils::{Json, StatusCode},
    app::{ReadData, ReadDynamicConfig, WriteDynamicConfig},
};

pub const PATH_GET_BACKEND_CONFIG: &str = "/common_api/backend_config";

/// Get dynamic backend config.
///
/// # Capabilities
/// Requires admin_server_maintenance_view_backend_settings.
#[utoipa::path(
    get,
    path = "/common_api/backend_config",
    params(),
    responses(
        (status = 200, description = "Get was successfull.", body = BackendConfig),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_backend_config<S: ReadDynamicConfig>(
    State(state): State<S>,
    Extension(api_caller_capabilities): Extension<Capabilities>,
) -> Result<Json<BackendConfig>, StatusCode> {
    COMMON_ADMIN.get_backend_config.incr();

    if api_caller_capabilities.admin_server_maintenance_view_backend_config {
        let config = state.read_config().await?;
        Ok(config.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub const PATH_POST_BACKEND_CONFIG: &str = "/common_api/backend_config";

/// Save dynamic backend config.
///
/// # Capabilities
/// Requires admin_server_maintenance_save_backend_settings.
#[utoipa::path(
    post,
    path = "/common_api/backend_config",
    params(),
    request_body(content = BackendConfig),
    responses(
        (status = 200, description = "Save was successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_backend_config<S: ReadData + WriteDynamicConfig>(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Extension(api_caller_capabilities): Extension<Capabilities>,
    Json(backend_config): Json<BackendConfig>,
) -> Result<(), StatusCode> {
    COMMON_ADMIN.post_backend_config.incr();

    if api_caller_capabilities.admin_server_maintenance_save_backend_config {
        info!(
            "Saving dynamic backend config, account: {}, settings: {:#?}",
            api_caller_account_id.as_uuid(),
            backend_config
        );
        state.write_config(backend_config).await?;

        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub fn config_router(s: crate::app::S) -> Router {
    use axum::routing::{get, post};

    use crate::app::S;

    Router::new()
        .route(PATH_GET_BACKEND_CONFIG, get(get_backend_config::<S>))
        .route(PATH_POST_BACKEND_CONFIG, post(post_backend_config::<S>))
        .with_state(s)
}

create_counters!(
    CommonAdminCounters,
    COMMON_ADMIN,
    COMMON_ADMIN_CONFIG_COUNTERS_LIST,
    get_backend_config,
    post_backend_config,
);
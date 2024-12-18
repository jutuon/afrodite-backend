use axum::{
    extract::{Query, State},
    Extension,
};
use model::Permissions;
use obfuscate_api_macro::obfuscate_api;
use simple_backend::{app::PerfCounterDataProvider, create_counters};
use simple_backend_model::{PerfHistoryQuery, PerfHistoryQueryResult};
use utoipa_axum::router::OpenApiRouter;

use crate::{
    create_open_api_router,
    utils::{Json, StatusCode},
    S,
};

#[obfuscate_api]
const PATH_GET_PERF_DATA: &str = "/common_api/perf_data";

/// Get performance data
///
/// # Permissions
/// Requires admin_server_maintenance_view_info.
#[utoipa::path(
    get,
    path = PATH_GET_PERF_DATA,
    params(PerfHistoryQuery),
    responses(
        (status = 200, description = "Get was successfull.", body = PerfHistoryQueryResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_perf_data(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Query(_query): Query<PerfHistoryQuery>,
) -> Result<Json<PerfHistoryQueryResult>, StatusCode> {
    COMMON_ADMIN.get_perf_data.incr();
    if api_caller_permissions.admin_server_maintenance_view_info {
        let data = state.perf_counter_data().get_history().await;
        Ok(data.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub fn perf_router(s: S) -> OpenApiRouter {
    create_open_api_router!(s, get_perf_data,)
}

create_counters!(
    CommonAdminCounters,
    COMMON_ADMIN,
    COMMON_ADMIN_PERF_COUNTERS_LIST,
    get_perf_data,
);

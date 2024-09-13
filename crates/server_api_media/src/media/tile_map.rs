use axum::{
    body::Body, extract::{Path, State}, Router
};
use axum_extra::TypedHeader;
use headers::{ContentLength, ContentType};
use model::{MapTileX, MapTileY, MapTileZ};
use simple_backend::{app::GetTileMap, create_counters};
use tracing::error;

use crate::{app::StateBase, utils::StatusCode};

pub const PATH_GET_MAP_TILE: &str = "/media_api/map_tile/:z/:x/:y";

/// Get map tile PNG file.
///
/// Returns a .png even if the URL does not have it.
#[utoipa::path(
    get,
    path = "/media_api/map_tile/{z}/{x}/{y}",
    params(MapTileZ, MapTileX, MapTileY),
    responses(
        (status = 200, description = "Get map tile PNG file.", body = Vec<u8>, content_type = "image/png"),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_map_tile<S: GetTileMap>(
    State(state): State<S>,
    Path(z): Path<MapTileZ>,
    Path(x): Path<MapTileX>,
    Path(y): Path<MapTileY>,
) -> Result<(TypedHeader<ContentType>, TypedHeader<ContentLength>, Body), StatusCode> {
    MEDIA.get_map_tile.incr();

    let y_string = y.y.trim_end_matches(".png");
    let y = y_string
        .parse::<u32>()
        .map_err(|_| StatusCode::NOT_ACCEPTABLE)?;

    let byte_count_and_data_stream = state
        .tile_map()
        .map_tile_byte_count_and_byte_stream(z.z, x.x, y)
        .await
        .map_err(|e| {
            error!("{:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match byte_count_and_data_stream {
        Some((byte_count, data_stream)) =>
            Ok((TypedHeader(ContentType::png()), TypedHeader(ContentLength(byte_count)), Body::from_stream(data_stream))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub fn tile_map_router<S: StateBase + GetTileMap>(s: S) -> Router {
    use axum::routing::get;

    Router::new()
        .route(PATH_GET_MAP_TILE, get(get_map_tile::<S>))
        .with_state(s)
}

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_TILE_MAP_COUNTERS_LIST,
    get_map_tile,
);

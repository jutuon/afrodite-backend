use axum::{middleware, Router};
use server_api::app::GetConfig;
use server_state::S;

use crate::api;

/// Private routes only accessible when WebSocket is connected.
pub struct ConnectedApp {
    state: S,
}

impl ConnectedApp {
    pub fn new(state: S) -> Self {
        Self { state }
    }

    pub fn state(&self) -> S {
        self.state.clone()
    }

    pub fn private_profile_server_router(&self) -> Router {
        let private = Router::new()
            .merge(api::profile::filters_router(self.state.clone()))
            .merge(api::profile::profile_data_router(self.state.clone()))
            .merge(api::profile::location_router(self.state.clone()))
            .merge(api::profile::favorite_router(self.state.clone()))
            .merge(api::profile::iterate_profiles_router(self.state.clone()))
            .merge(api::profile::statistics_router(self.state.clone()))
            .merge(api::profile_admin::admin_statistics_router(
                self.state.clone(),
            ))
            .merge(api::profile_admin::admin_profile_name_allowlist_router(
                self.state.clone(),
            ))
            .merge(api::profile_admin::admin_profile_text_router(
                self.state.clone(),
            ));

        let private = if self.state.config().debug_mode() {
            private.merge(api::profile::benchmark_router(self.state.clone()))
        } else {
            private
        };

        private.route_layer({
            middleware::from_fn_with_state(self.state(), api::utils::authenticate_with_access_token)
        })
    }
}

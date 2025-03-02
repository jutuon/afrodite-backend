#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! HTTP API types and request handlers for all servers.

use utoipa::OpenApi;

use self::utils::SecurityApiAccessTokenDefault;

// Routes
pub mod chat;

pub use server_api::{app, internal_api, utils};
pub use server_common::{data::DataError, result};

// API docs

#[derive(OpenApi)]
#[openapi(
    components(schemas(
        // Chat
        model_chat::chat::PendingMessage,
    )),
    modifiers(&SecurityApiAccessTokenDefault),
)]
pub struct ApiDocChat;

pub use server_api::{db_write, db_write_multiple};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct BackendVersion {
    /// Backend code version.
    pub backend_code_version: String,
    /// Semver version of the backend.
    pub backend_version: String,
    /// Semver version of the protocol used by the backend.
    pub protocol_version: String,
}


#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub enum EventToClient {
    AccountStateChanged,
}
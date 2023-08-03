/*
 * pihka-backend
 *
 * Pihka backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct BackendVersion {
    /// Backend code version.
    #[serde(rename = "backend_code_version")]
    pub backend_code_version: String,
    /// Semver version of the backend.
    #[serde(rename = "backend_version")]
    pub backend_version: String,
    /// Semver version of the protocol used by the backend.
    #[serde(rename = "protocol_version")]
    pub protocol_version: String,
}

impl BackendVersion {
    pub fn new(backend_code_version: String, backend_version: String, protocol_version: String) -> BackendVersion {
        BackendVersion {
            backend_code_version,
            backend_version,
            protocol_version,
        }
    }
}



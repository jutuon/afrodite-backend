/*
 * pihka-backend
 *
 * Pihka backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// EncryptedMessage : Encrypted message container for client.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct EncryptedMessage {
    #[serde(rename = "pgp_message")]
    pub pgp_message: String,
    /// Encryption version
    #[serde(rename = "version")]
    pub version: i64,
}

impl EncryptedMessage {
    /// Encrypted message container for client.
    pub fn new(pgp_message: String, version: i64) -> EncryptedMessage {
        EncryptedMessage {
            pgp_message,
            version,
        }
    }
}


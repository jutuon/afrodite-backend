/*
 * dating-app-backend
 *
 * Dating app backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// PendingMessage : Client uses this type even if it is not directly in API routes
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct PendingMessage {
    #[serde(rename = "id")]
    pub id: Box<models::PendingMessageId>,
    /// Unix time when server received the message.
    #[serde(rename = "unix_time")]
    pub unix_time: Box<models::UnixTime>,
}

impl PendingMessage {
    /// Client uses this type even if it is not directly in API routes
    pub fn new(id: models::PendingMessageId, unix_time: models::UnixTime) -> PendingMessage {
        PendingMessage {
            id: Box::new(id),
            unix_time: Box::new(unix_time),
        }
    }
}


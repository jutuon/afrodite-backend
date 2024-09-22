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

/// ReceivedLikesIteratorSessionId : Session ID type for received likes iterator so that client can detect server restarts and ask user to refresh received likes.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReceivedLikesIteratorSessionId {
    #[serde(rename = "id")]
    pub id: String,
}

impl ReceivedLikesIteratorSessionId {
    /// Session ID type for received likes iterator so that client can detect server restarts and ask user to refresh received likes.
    pub fn new(id: String) -> ReceivedLikesIteratorSessionId {
        ReceivedLikesIteratorSessionId {
            id,
        }
    }
}

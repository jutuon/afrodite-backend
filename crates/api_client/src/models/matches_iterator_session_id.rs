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

/// MatchesIteratorSessionId : Session ID type for matches iterator so that client can detect server restarts and ask user to matches.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct MatchesIteratorSessionId {
    #[serde(rename = "id")]
    pub id: i64,
}

impl MatchesIteratorSessionId {
    /// Session ID type for matches iterator so that client can detect server restarts and ask user to matches.
    pub fn new(id: i64) -> MatchesIteratorSessionId {
        MatchesIteratorSessionId {
            id,
        }
    }
}


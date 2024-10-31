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

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResetMatchesIteratorResult {
    #[serde(rename = "s")]
    pub s: Box<models::MatchesIteratorSessionId>,
}

impl ResetMatchesIteratorResult {
    pub fn new(s: models::MatchesIteratorSessionId) -> ResetMatchesIteratorResult {
        ResetMatchesIteratorResult {
            s: Box::new(s),
        }
    }
}


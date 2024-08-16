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

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct MatchesSyncVersion {
    #[serde(rename = "version")]
    pub version: i64,
}

impl MatchesSyncVersion {
    pub fn new(version: i64) -> MatchesSyncVersion {
        MatchesSyncVersion {
            version,
        }
    }
}


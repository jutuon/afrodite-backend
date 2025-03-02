/*
 * afrodite-backend
 *
 * Dating app backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// ProfileContentVersion : Version UUID for public profile content.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProfileContentVersion {
    #[serde(rename = "v")]
    pub v: String,
}

impl ProfileContentVersion {
    /// Version UUID for public profile content.
    pub fn new(v: String) -> ProfileContentVersion {
        ProfileContentVersion {
            v,
        }
    }
}


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

/// NewsTranslationVersion : News translation version which prevents editing newer version than user has seen.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct NewsTranslationVersion {
    #[serde(rename = "version")]
    pub version: i64,
}

impl NewsTranslationVersion {
    /// News translation version which prevents editing newer version than user has seen.
    pub fn new(version: i64) -> NewsTranslationVersion {
        NewsTranslationVersion {
            version,
        }
    }
}

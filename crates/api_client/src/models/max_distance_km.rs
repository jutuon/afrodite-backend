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

/// MaxDistanceKm : Profile iterator max distance in kilometers.  The value is equal or greater than 1.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaxDistanceKm {
    #[serde(rename = "value")]
    pub value: i64,
}

impl MaxDistanceKm {
    /// Profile iterator max distance in kilometers.  The value is equal or greater than 1.
    pub fn new(value: i64) -> MaxDistanceKm {
        MaxDistanceKm {
            value,
        }
    }
}

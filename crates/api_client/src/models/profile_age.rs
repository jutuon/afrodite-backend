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

/// ProfileAge : Profile age value which is in inclusive range `[18, 99]`.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProfileAge {
    #[serde(rename = "value")]
    pub value: i32,
}

impl ProfileAge {
    /// Profile age value which is in inclusive range `[18, 99]`.
    pub fn new(value: i32) -> ProfileAge {
        ProfileAge {
            value,
        }
    }
}


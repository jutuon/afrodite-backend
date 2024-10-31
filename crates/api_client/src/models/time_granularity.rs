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

/// 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum TimeGranularity {
    #[serde(rename = "Minutes")]
    Minutes,
    #[serde(rename = "Hours")]
    Hours,

}

impl std::fmt::Display for TimeGranularity {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Minutes => write!(f, "Minutes"),
            Self::Hours => write!(f, "Hours"),
        }
    }
}

impl Default for TimeGranularity {
    fn default() -> TimeGranularity {
        Self::Minutes
    }
}


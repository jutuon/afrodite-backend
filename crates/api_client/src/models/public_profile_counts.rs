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
pub struct PublicProfileCounts {
    #[serde(rename = "man")]
    pub man: i64,
    #[serde(rename = "non_binary")]
    pub non_binary: i64,
    #[serde(rename = "woman")]
    pub woman: i64,
}

impl PublicProfileCounts {
    pub fn new(man: i64, non_binary: i64, woman: i64) -> PublicProfileCounts {
        PublicProfileCounts {
            man,
            non_binary,
            woman,
        }
    }
}


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

/// FcmDeviceToken : Firebase Cloud Messaging device token.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct FcmDeviceToken {
    #[serde(rename = "token")]
    pub token: String,
}

impl FcmDeviceToken {
    /// Firebase Cloud Messaging device token.
    pub fn new(token: String) -> FcmDeviceToken {
        FcmDeviceToken {
            token,
        }
    }
}


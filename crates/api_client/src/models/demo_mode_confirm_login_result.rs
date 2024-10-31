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
pub struct DemoModeConfirmLoginResult {
    /// This password is locked.
    #[serde(rename = "locked")]
    pub locked: bool,
    #[serde(rename = "token", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub token: Option<Option<Box<models::DemoModeToken>>>,
}

impl DemoModeConfirmLoginResult {
    pub fn new(locked: bool) -> DemoModeConfirmLoginResult {
        DemoModeConfirmLoginResult {
            locked,
            token: None,
        }
    }
}


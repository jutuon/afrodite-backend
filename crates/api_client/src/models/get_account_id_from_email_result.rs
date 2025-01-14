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
pub struct GetAccountIdFromEmailResult {
    #[serde(rename = "aid", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub aid: Option<Option<Box<models::AccountId>>>,
}

impl GetAccountIdFromEmailResult {
    pub fn new() -> GetAccountIdFromEmailResult {
        GetAccountIdFromEmailResult {
            aid: None,
        }
    }
}

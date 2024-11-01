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
pub struct AccountContent {
    #[serde(rename = "data")]
    pub data: Vec<models::ContentInfoDetailed>,
}

impl AccountContent {
    pub fn new(data: Vec<models::ContentInfoDetailed>) -> AccountContent {
        AccountContent {
            data,
        }
    }
}


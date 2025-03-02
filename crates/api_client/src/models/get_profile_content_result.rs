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

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetProfileContentResult {
    #[serde(rename = "c", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub c: Option<Option<Box<models::ProfileContent>>>,
    #[serde(rename = "v", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub v: Option<Option<Box<models::ProfileContentVersion>>>,
}

impl GetProfileContentResult {
    pub fn new() -> GetProfileContentResult {
        GetProfileContentResult {
            c: None,
            v: None,
        }
    }
}


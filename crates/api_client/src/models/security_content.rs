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
pub struct SecurityContent {
    #[serde(rename = "c", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub c: Option<Option<Box<models::ContentInfoWithFd>>>,
}

impl SecurityContent {
    pub fn new() -> SecurityContent {
        SecurityContent {
            c: None,
        }
    }
}


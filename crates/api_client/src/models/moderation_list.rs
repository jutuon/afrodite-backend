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

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModerationList {
    #[serde(rename = "list")]
    pub list: Vec<models::Moderation>,
}

impl ModerationList {
    pub fn new(list: Vec<models::Moderation>) -> ModerationList {
        ModerationList {
            list,
        }
    }
}


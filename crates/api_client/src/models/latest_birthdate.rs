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
pub struct LatestBirthdate {
    #[serde(rename = "birthdate", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub birthdate: Option<Option<String>>,
}

impl LatestBirthdate {
    pub fn new() -> LatestBirthdate {
        LatestBirthdate {
            birthdate: None,
        }
    }
}


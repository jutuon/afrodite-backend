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
pub struct ReceivedBlocksPage {
    #[serde(rename = "profiles")]
    pub profiles: Vec<models::AccountId>,
    #[serde(rename = "version")]
    pub version: Box<models::ReceivedBlocksSyncVersion>,
}

impl ReceivedBlocksPage {
    pub fn new(profiles: Vec<models::AccountId>, version: models::ReceivedBlocksSyncVersion) -> ReceivedBlocksPage {
        ReceivedBlocksPage {
            profiles,
            version: Box::new(version),
        }
    }
}


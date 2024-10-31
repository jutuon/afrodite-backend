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
pub struct PendingMessageAcknowledgementList {
    #[serde(rename = "ids")]
    pub ids: Vec<models::PendingMessageId>,
}

impl PendingMessageAcknowledgementList {
    pub fn new(ids: Vec<models::PendingMessageId>) -> PendingMessageAcknowledgementList {
        PendingMessageAcknowledgementList {
            ids,
        }
    }
}


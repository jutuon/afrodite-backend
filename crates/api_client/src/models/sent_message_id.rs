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
pub struct SentMessageId {
    #[serde(rename = "c")]
    pub c: Box<models::ClientId>,
    #[serde(rename = "l")]
    pub l: Box<models::ClientLocalId>,
}

impl SentMessageId {
    pub fn new(c: models::ClientId, l: models::ClientLocalId) -> SentMessageId {
        SentMessageId {
            c: Box::new(c),
            l: Box::new(l),
        }
    }
}


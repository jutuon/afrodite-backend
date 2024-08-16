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
pub struct Moderation {
    #[serde(rename = "content")]
    pub content: Box<models::ModerationRequestContent>,
    #[serde(rename = "moderator_id")]
    pub moderator_id: Box<models::AccountId>,
    #[serde(rename = "request_creator_id")]
    pub request_creator_id: Box<models::AccountId>,
    #[serde(rename = "request_id")]
    pub request_id: Box<models::ModerationRequestId>,
}

impl Moderation {
    pub fn new(content: models::ModerationRequestContent, moderator_id: models::AccountId, request_creator_id: models::AccountId, request_id: models::ModerationRequestId) -> Moderation {
        Moderation {
            content: Box::new(content),
            moderator_id: Box::new(moderator_id),
            request_creator_id: Box::new(request_creator_id),
            request_id: Box::new(request_id),
        }
    }
}


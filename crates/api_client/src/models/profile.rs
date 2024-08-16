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

/// Profile : Public profile info
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    #[serde(rename = "age")]
    pub age: i64,
    #[serde(rename = "attributes")]
    pub attributes: Vec<models::ProfileAttributeValue>,
    #[serde(rename = "name")]
    pub name: String,
    /// Profile text support is disabled for now.
    #[serde(rename = "profile_text")]
    pub profile_text: String,
    #[serde(rename = "unlimited_likes", skip_serializing_if = "Option::is_none")]
    pub unlimited_likes: Option<bool>,
}

impl Profile {
    /// Public profile info
    pub fn new(age: i64, attributes: Vec<models::ProfileAttributeValue>, name: String, profile_text: String) -> Profile {
        Profile {
            age,
            attributes,
            name,
            profile_text,
            unlimited_likes: None,
        }
    }
}


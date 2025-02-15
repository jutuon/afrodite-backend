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
pub struct AttributeIdAndHash {
    #[serde(rename = "h")]
    pub h: Box<models::ProfileAttributeHash>,
    #[serde(rename = "id")]
    pub id: i32,
}

impl AttributeIdAndHash {
    pub fn new(h: models::ProfileAttributeHash, id: i32) -> AttributeIdAndHash {
        AttributeIdAndHash {
            h: Box::new(h),
            id,
        }
    }
}


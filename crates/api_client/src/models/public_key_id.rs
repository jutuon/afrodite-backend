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
pub struct PublicKeyId {
    #[serde(rename = "id")]
    pub id: i64,
}

impl PublicKeyId {
    pub fn new(id: i64) -> PublicKeyId {
        PublicKeyId {
            id,
        }
    }
}


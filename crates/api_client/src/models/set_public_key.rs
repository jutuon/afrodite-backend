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
pub struct SetPublicKey {
    #[serde(rename = "data")]
    pub data: Box<models::PublicKeyData>,
    #[serde(rename = "version")]
    pub version: Box<models::PublicKeyVersion>,
}

impl SetPublicKey {
    pub fn new(data: models::PublicKeyData, version: models::PublicKeyVersion) -> SetPublicKey {
        SetPublicKey {
            data: Box::new(data),
            version: Box::new(version),
        }
    }
}


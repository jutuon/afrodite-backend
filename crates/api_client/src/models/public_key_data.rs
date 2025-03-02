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

/// PublicKeyData : Data for asymmetric encryption public key. Client defines the format for the public key.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct PublicKeyData {
    #[serde(rename = "data")]
    pub data: String,
}

impl PublicKeyData {
    /// Data for asymmetric encryption public key. Client defines the format for the public key.
    pub fn new(data: String) -> PublicKeyData {
        PublicKeyData {
            data,
        }
    }
}


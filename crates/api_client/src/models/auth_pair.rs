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

/// AuthPair : AccessToken and RefreshToken
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthPair {
    #[serde(rename = "access")]
    pub access: Box<models::AccessToken>,
    #[serde(rename = "refresh")]
    pub refresh: Box<models::RefreshToken>,
}

impl AuthPair {
    /// AccessToken and RefreshToken
    pub fn new(access: models::AccessToken, refresh: models::RefreshToken) -> AuthPair {
        AuthPair {
            access: Box::new(access),
            refresh: Box::new(refresh),
        }
    }
}


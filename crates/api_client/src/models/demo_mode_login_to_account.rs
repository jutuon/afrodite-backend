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
pub struct DemoModeLoginToAccount {
    #[serde(rename = "account_id")]
    pub account_id: Box<models::AccountId>,
    #[serde(rename = "token")]
    pub token: Box<models::DemoModeToken>,
}

impl DemoModeLoginToAccount {
    pub fn new(account_id: models::AccountId, token: models::DemoModeToken) -> DemoModeLoginToAccount {
        DemoModeLoginToAccount {
            account_id: Box::new(account_id),
            token: Box::new(token),
        }
    }
}


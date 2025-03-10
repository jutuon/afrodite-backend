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
pub struct GetAllAdminsResult {
    #[serde(rename = "admins")]
    pub admins: Vec<models::AdminInfo>,
}

impl GetAllAdminsResult {
    pub fn new(admins: Vec<models::AdminInfo>) -> GetAllAdminsResult {
        GetAllAdminsResult {
            admins,
        }
    }
}


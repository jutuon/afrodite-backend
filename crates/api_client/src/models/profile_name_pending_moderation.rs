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
pub struct ProfileNamePendingModeration {
    #[serde(rename = "id")]
    pub id: Box<models::AccountId>,
    #[serde(rename = "name")]
    pub name: String,
}

impl ProfileNamePendingModeration {
    pub fn new(id: models::AccountId, name: String) -> ProfileNamePendingModeration {
        ProfileNamePendingModeration {
            id: Box::new(id),
            name,
        }
    }
}


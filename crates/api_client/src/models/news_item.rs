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
pub struct NewsItem {
    #[serde(rename = "aid_creator", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub aid_creator: Option<Option<Box<models::AccountId>>>,
    #[serde(rename = "aid_editor", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub aid_editor: Option<Option<Box<models::AccountId>>>,
    #[serde(rename = "body")]
    pub body: String,
    #[serde(rename = "creation_time")]
    pub creation_time: Box<models::UnixTime>,
    #[serde(rename = "edit_time", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub edit_time: Option<Option<Box<models::UnixTime>>>,
    #[serde(rename = "title")]
    pub title: String,
}

impl NewsItem {
    pub fn new(body: String, creation_time: models::UnixTime, title: String) -> NewsItem {
        NewsItem {
            aid_creator: None,
            aid_editor: None,
            body,
            creation_time: Box::new(creation_time),
            edit_time: None,
            title,
        }
    }
}

/*
 * dating-app-backend
 *
 * Dating app backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// ProfileContent : Current content in public profile.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProfileContent {
    /// Primary profile image which is shown in grid view.
    #[serde(rename = "c0", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub c0: Option<Option<Box<models::ContentInfo>>>,
    #[serde(rename = "c1", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub c1: Option<Option<Box<models::ContentInfo>>>,
    #[serde(rename = "c2", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub c2: Option<Option<Box<models::ContentInfo>>>,
    #[serde(rename = "c3", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub c3: Option<Option<Box<models::ContentInfo>>>,
    #[serde(rename = "c4", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub c4: Option<Option<Box<models::ContentInfo>>>,
    #[serde(rename = "c5", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub c5: Option<Option<Box<models::ContentInfo>>>,
    #[serde(rename = "grid_crop_size", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub grid_crop_size: Option<Option<f64>>,
    #[serde(rename = "grid_crop_x", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub grid_crop_x: Option<Option<f64>>,
    #[serde(rename = "grid_crop_y", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub grid_crop_y: Option<Option<f64>>,
}

impl ProfileContent {
    /// Current content in public profile.
    pub fn new() -> ProfileContent {
        ProfileContent {
            c0: None,
            c1: None,
            c2: None,
            c3: None,
            c4: None,
            c5: None,
            grid_crop_size: None,
            grid_crop_x: None,
            grid_crop_y: None,
        }
    }
}


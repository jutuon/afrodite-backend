/*
 * pihka-backend
 *
 * Pihka backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

/// SetProfileContent : Update normal or pending profile content



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SetProfileContent {
    #[serde(rename = "content_id_0")]
    pub content_id_0: Box<crate::models::ContentId>,
    #[serde(rename = "content_id_1", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub content_id_1: Option<Option<Box<crate::models::ContentId>>>,
    #[serde(rename = "content_id_2", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub content_id_2: Option<Option<Box<crate::models::ContentId>>>,
    #[serde(rename = "content_id_3", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub content_id_3: Option<Option<Box<crate::models::ContentId>>>,
    #[serde(rename = "content_id_4", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub content_id_4: Option<Option<Box<crate::models::ContentId>>>,
    #[serde(rename = "content_id_5", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub content_id_5: Option<Option<Box<crate::models::ContentId>>>,
    #[serde(rename = "grid_crop_size", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub grid_crop_size: Option<Option<f64>>,
    #[serde(rename = "grid_crop_x", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub grid_crop_x: Option<Option<f64>>,
    #[serde(rename = "grid_crop_y", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub grid_crop_y: Option<Option<f64>>,
}

impl SetProfileContent {
    /// Update normal or pending profile content
    pub fn new(content_id_0: crate::models::ContentId) -> SetProfileContent {
        SetProfileContent {
            content_id_0: Box::new(content_id_0),
            content_id_1: None,
            content_id_2: None,
            content_id_3: None,
            content_id_4: None,
            content_id_5: None,
            grid_crop_size: None,
            grid_crop_x: None,
            grid_crop_y: None,
        }
    }
}



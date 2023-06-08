/*
 * pihka-backend
 *
 * Pihka backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct NormalImages {
    #[serde(rename = "data")]
    pub data: Vec<crate::models::ContentId>,
}

impl NormalImages {
    pub fn new(data: Vec<crate::models::ContentId>) -> NormalImages {
        NormalImages {
            data,
        }
    }
}



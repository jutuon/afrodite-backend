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
pub struct SentLikesPage {
    #[serde(rename = "profiles")]
    pub profiles: Vec<crate::models::AccountId>,
    #[serde(rename = "version")]
    pub version: Box<crate::models::SentLikesSyncVersion>,
}

impl SentLikesPage {
    pub fn new(profiles: Vec<crate::models::AccountId>, version: crate::models::SentLikesSyncVersion) -> SentLikesPage {
        SentLikesPage {
            profiles,
            version: Box::new(version),
        }
    }
}



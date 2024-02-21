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
pub struct MatchesPage {
    #[serde(rename = "profiles")]
    pub profiles: Vec<crate::models::AccountId>,
    #[serde(rename = "version")]
    pub version: Box<crate::models::MatchesSyncVersion>,
}

impl MatchesPage {
    pub fn new(profiles: Vec<crate::models::AccountId>, version: crate::models::MatchesSyncVersion) -> MatchesPage {
        MatchesPage {
            profiles,
            version: Box::new(version),
        }
    }
}



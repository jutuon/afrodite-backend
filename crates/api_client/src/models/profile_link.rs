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
pub struct ProfileLink {
    #[serde(rename = "id")]
    pub id: Box<crate::models::AccountId>,
    #[serde(rename = "version")]
    pub version: Box<crate::models::ProfileVersion>,
}

impl ProfileLink {
    pub fn new(id: crate::models::AccountId, version: crate::models::ProfileVersion) -> ProfileLink {
        ProfileLink {
            id: Box::new(id),
            version: Box::new(version),
        }
    }
}



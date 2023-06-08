/*
 * pihka-backend
 *
 * Pihka backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

/// Profile : Prfile for HTTP GET



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Profile {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "profile_text")]
    pub profile_text: String,
    #[serde(rename = "version")]
    pub version: Box<crate::models::ProfileVersion>,
}

impl Profile {
    /// Prfile for HTTP GET
    pub fn new(name: String, profile_text: String, version: crate::models::ProfileVersion) -> Profile {
        Profile {
            name,
            profile_text,
            version: Box::new(version),
        }
    }
}



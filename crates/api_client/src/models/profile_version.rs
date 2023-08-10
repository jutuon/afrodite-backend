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
pub struct ProfileVersion {
    #[serde(rename = "version_uuid")]
    pub version_uuid: uuid::Uuid,
}

impl ProfileVersion {
    pub fn new(version_uuid: uuid::Uuid) -> ProfileVersion {
        ProfileVersion {
            version_uuid,
        }
    }
}


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
pub struct SentLikesSyncVersion {
    #[serde(rename = "version")]
    pub version: i64,
}

impl SentLikesSyncVersion {
    pub fn new(version: i64) -> SentLikesSyncVersion {
        SentLikesSyncVersion {
            version,
        }
    }
}



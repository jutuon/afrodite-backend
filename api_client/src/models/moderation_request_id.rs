/*
 * pihka-backend
 *
 * Pihka backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * Contact: 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ModerationRequestId {
    #[serde(rename = "request_row_id")]
    pub request_row_id: i64,
}

impl ModerationRequestId {
    pub fn new(request_row_id: i64) -> ModerationRequestId {
        ModerationRequestId {
            request_row_id,
        }
    }
}



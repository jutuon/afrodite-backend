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
pub struct ModerationQueueTypeParam {
    #[serde(rename = "queue")]
    pub queue: crate::models::ModerationQueueType,
}

impl ModerationQueueTypeParam {
    pub fn new(queue: crate::models::ModerationQueueType) -> ModerationQueueTypeParam {
        ModerationQueueTypeParam {
            queue,
        }
    }
}


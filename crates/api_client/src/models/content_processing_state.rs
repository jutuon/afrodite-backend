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
pub struct ContentProcessingState {
    #[serde(rename = "content_id", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub content_id: Option<Option<Box<crate::models::ContentId>>>,
    #[serde(rename = "state")]
    pub state: crate::models::ContentProcessingStateType,
    /// Current position in processing queue.  If ProcessingContentId is added to empty queue, then this will be 1.
    #[serde(rename = "wait_queue_position", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub wait_queue_position: Option<Option<i64>>,
}

impl ContentProcessingState {
    pub fn new(state: crate::models::ContentProcessingStateType) -> ContentProcessingState {
        ContentProcessingState {
            content_id: None,
            state,
            wait_queue_position: None,
        }
    }
}



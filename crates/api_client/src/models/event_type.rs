/*
 * pihka-backend
 *
 * Pihka backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

/// EventType : Identifier for event.

/// Identifier for event.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum EventType {
    #[serde(rename = "AccountStateChanged")]
    AccountStateChanged,
    #[serde(rename = "AccountCapabilitiesChanged")]
    AccountCapabilitiesChanged,
    #[serde(rename = "NewMessageReceived")]
    NewMessageReceived,
    #[serde(rename = "LikesChanged")]
    LikesChanged,
    #[serde(rename = "ReceivedBlocksChanged")]
    ReceivedBlocksChanged,
    #[serde(rename = "LatestViewedMessageChanged")]
    LatestViewedMessageChanged,
    #[serde(rename = "ContentProcessingStateChanged")]
    ContentProcessingStateChanged,

}

impl ToString for EventType {
    fn to_string(&self) -> String {
        match self {
            Self::AccountStateChanged => String::from("AccountStateChanged"),
            Self::AccountCapabilitiesChanged => String::from("AccountCapabilitiesChanged"),
            Self::NewMessageReceived => String::from("NewMessageReceived"),
            Self::LikesChanged => String::from("LikesChanged"),
            Self::ReceivedBlocksChanged => String::from("ReceivedBlocksChanged"),
            Self::LatestViewedMessageChanged => String::from("LatestViewedMessageChanged"),
            Self::ContentProcessingStateChanged => String::from("ContentProcessingStateChanged"),
        }
    }
}

impl Default for EventType {
    fn default() -> EventType {
        Self::AccountStateChanged
    }
}





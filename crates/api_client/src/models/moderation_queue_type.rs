/*
 * dating-app-backend
 *
 * Dating app backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// ModerationQueueType : Subset of NextQueueNumberType containing only moderation queue types.
/// Subset of NextQueueNumberType containing only moderation queue types.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum ModerationQueueType {
    #[serde(rename = "MediaModeration")]
    MediaModeration,
    #[serde(rename = "InitialMediaModeration")]
    InitialMediaModeration,

}

impl std::fmt::Display for ModerationQueueType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::MediaModeration => write!(f, "MediaModeration"),
            Self::InitialMediaModeration => write!(f, "InitialMediaModeration"),
        }
    }
}

impl Default for ModerationQueueType {
    fn default() -> ModerationQueueType {
        Self::MediaModeration
    }
}


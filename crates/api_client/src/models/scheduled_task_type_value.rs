/*
 * afrodite-backend
 *
 * Dating app backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScheduledTaskTypeValue {
    #[serde(rename = "scheduled_task_type")]
    pub scheduled_task_type: models::ScheduledTaskType,
}

impl ScheduledTaskTypeValue {
    pub fn new(scheduled_task_type: models::ScheduledTaskType) -> ScheduledTaskTypeValue {
        ScheduledTaskTypeValue {
            scheduled_task_type,
        }
    }
}


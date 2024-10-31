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

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetProfileStatisticsHistoryResult {
    #[serde(rename = "values")]
    pub values: Vec<models::ProfileStatisticsHistoryValue>,
}

impl GetProfileStatisticsHistoryResult {
    pub fn new(values: Vec<models::ProfileStatisticsHistoryValue>) -> GetProfileStatisticsHistoryResult {
        GetProfileStatisticsHistoryResult {
            values,
        }
    }
}


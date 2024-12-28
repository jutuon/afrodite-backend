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
pub struct PerfMetricQueryResult {
    #[serde(rename = "metrics")]
    pub metrics: Vec<models::PerfMetricValues>,
}

impl PerfMetricQueryResult {
    pub fn new(metrics: Vec<models::PerfMetricValues>) -> PerfMetricQueryResult {
        PerfMetricQueryResult {
            metrics,
        }
    }
}


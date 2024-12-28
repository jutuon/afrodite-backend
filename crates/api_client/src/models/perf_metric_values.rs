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
pub struct PerfMetricValues {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "values")]
    pub values: Vec<models::PerfMetricValueArea>,
}

impl PerfMetricValues {
    pub fn new(name: String, values: Vec<models::PerfMetricValueArea>) -> PerfMetricValues {
        PerfMetricValues {
            name,
            values,
        }
    }
}

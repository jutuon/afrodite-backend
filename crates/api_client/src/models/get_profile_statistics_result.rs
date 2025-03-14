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
pub struct GetProfileStatisticsResult {
    #[serde(rename = "age_counts")]
    pub age_counts: Box<models::ProfileAgeCounts>,
    #[serde(rename = "generation_time")]
    pub generation_time: Box<models::UnixTime>,
}

impl GetProfileStatisticsResult {
    pub fn new(age_counts: models::ProfileAgeCounts, generation_time: models::UnixTime) -> GetProfileStatisticsResult {
        GetProfileStatisticsResult {
            age_counts: Box::new(age_counts),
            generation_time: Box::new(generation_time),
        }
    }
}


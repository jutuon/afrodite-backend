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
pub struct PerfHistoryValue {
    #[serde(rename = "counter_name")]
    pub counter_name: String,
    #[serde(rename = "values")]
    pub values: Vec<crate::models::PerfValueArea>,
}

impl PerfHistoryValue {
    pub fn new(counter_name: String, values: Vec<crate::models::PerfValueArea>) -> PerfHistoryValue {
        PerfHistoryValue {
            counter_name,
            values,
        }
    }
}


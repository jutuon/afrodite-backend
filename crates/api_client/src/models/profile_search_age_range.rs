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
pub struct ProfileSearchAgeRange {
    /// Max value for this field is 99.
    #[serde(rename = "max")]
    pub max: i32,
    /// Min value for this field is 18.
    #[serde(rename = "min")]
    pub min: i32,
}

impl ProfileSearchAgeRange {
    pub fn new(max: i32, min: i32) -> ProfileSearchAgeRange {
        ProfileSearchAgeRange {
            max,
            min,
        }
    }
}


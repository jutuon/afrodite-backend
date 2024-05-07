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
pub struct Translation {
    /// Attribute name or attribute value key.
    #[serde(rename = "key")]
    pub key: String,
    /// Translated text.
    #[serde(rename = "value")]
    pub value: String,
}

impl Translation {
    pub fn new(key: String, value: String) -> Translation {
        Translation {
            key,
            value,
        }
    }
}


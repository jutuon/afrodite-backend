/*
 * pihka-backend
 *
 * Pihka backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

/// AccessToken : This is just a random string.



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct AccessToken {
    /// API token which server generates.
    #[serde(rename = "access_token")]
    pub access_token: String,
}

impl AccessToken {
    /// This is just a random string.
    pub fn new(access_token: String) -> AccessToken {
        AccessToken {
            access_token,
        }
    }
}


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
pub struct AccountId {
    #[serde(rename = "account_id")]
    pub account_id: uuid::Uuid,
}

impl AccountId {
    pub fn new(account_id: uuid::Uuid) -> AccountId {
        AccountId {
            account_id,
        }
    }
}



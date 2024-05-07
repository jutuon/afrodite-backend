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
pub struct ProfileAttributes {
    #[serde(rename = "attribute_order")]
    pub attribute_order: crate::models::AttributeOrderMode,
    /// List of attributes.  Attributes are sorted by Attribute ID and ID can be used to index this list.
    #[serde(rename = "attributes")]
    pub attributes: Vec<crate::models::Attribute>,
}

impl ProfileAttributes {
    pub fn new(attribute_order: crate::models::AttributeOrderMode, attributes: Vec<crate::models::Attribute>) -> ProfileAttributes {
        ProfileAttributes {
            attribute_order,
            attributes,
        }
    }
}


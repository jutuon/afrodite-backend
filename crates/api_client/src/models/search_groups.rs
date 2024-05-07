/*
 * pihka-backend
 *
 * Pihka backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

/// SearchGroups : My gender and what gender I'm searching for.  Fileds should be read \"I'm x and I'm searching for y\".



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SearchGroups {
    #[serde(rename = "man_for_man", skip_serializing_if = "Option::is_none")]
    pub man_for_man: Option<bool>,
    #[serde(rename = "man_for_non_binary", skip_serializing_if = "Option::is_none")]
    pub man_for_non_binary: Option<bool>,
    #[serde(rename = "man_for_woman", skip_serializing_if = "Option::is_none")]
    pub man_for_woman: Option<bool>,
    #[serde(rename = "non_binary_for_man", skip_serializing_if = "Option::is_none")]
    pub non_binary_for_man: Option<bool>,
    #[serde(rename = "non_binary_for_non_binary", skip_serializing_if = "Option::is_none")]
    pub non_binary_for_non_binary: Option<bool>,
    #[serde(rename = "non_binary_for_woman", skip_serializing_if = "Option::is_none")]
    pub non_binary_for_woman: Option<bool>,
    #[serde(rename = "woman_for_man", skip_serializing_if = "Option::is_none")]
    pub woman_for_man: Option<bool>,
    #[serde(rename = "woman_for_non_binary", skip_serializing_if = "Option::is_none")]
    pub woman_for_non_binary: Option<bool>,
    #[serde(rename = "woman_for_woman", skip_serializing_if = "Option::is_none")]
    pub woman_for_woman: Option<bool>,
}

impl SearchGroups {
    /// My gender and what gender I'm searching for.  Fileds should be read \"I'm x and I'm searching for y\".
    pub fn new() -> SearchGroups {
        SearchGroups {
            man_for_man: None,
            man_for_non_binary: None,
            man_for_woman: None,
            non_binary_for_man: None,
            non_binary_for_non_binary: None,
            non_binary_for_woman: None,
            woman_for_man: None,
            woman_for_non_binary: None,
            woman_for_woman: None,
        }
    }
}


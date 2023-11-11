/*
 * pihka-backend
 *
 * Pihka backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

/// MapTileZ : Z coordinate (or zoom number) of slippy map tile.



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct MapTileZ {
    #[serde(rename = "z")]
    pub z: i32,
}

impl MapTileZ {
    /// Z coordinate (or zoom number) of slippy map tile.
    pub fn new(z: i32) -> MapTileZ {
        MapTileZ {
            z,
        }
    }
}


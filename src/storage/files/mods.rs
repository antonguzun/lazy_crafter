use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mod {
    #[serde(rename = "adds_tags")]
    pub adds_tags: Vec<Value>,
    pub domain: String,
    #[serde(rename = "generation_type")]
    pub generation_type: String,
    #[serde(rename = "generation_weights")]
    pub generation_weights: Vec<Value>,
    #[serde(rename = "grants_effects")]
    pub grants_effects: Vec<Value>,
    pub groups: Vec<String>,
    #[serde(rename = "implicit_tags")]
    pub implicit_tags: Vec<String>,
    #[serde(rename = "is_essence_only")]
    pub is_essence_only: bool,
    pub name: String,
    #[serde(rename = "required_level")]
    pub required_level: u64,
    #[serde(rename = "spawn_weights")]
    pub spawn_weights: Vec<SpawnWeight>,
    pub stats: Vec<Stat>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnWeight {
    pub tag: String,
    pub weight: u32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stat {
    pub id: String,
    // pub max: i64,
    // pub min: i64,
}

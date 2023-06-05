use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpawnWeight {
    pub tag: String,
    pub weight: u32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stat {
    pub id: String,
    pub max: Option<f64>,
    pub min: Option<f64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Requirements {
    pub level: u64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemBaseRich {
    pub name: String,
    pub item_class: String,
    pub tags: Vec<String>,
    pub domain: String,
    pub release_state: String,
    pub requirements: Option<Requirements>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mod {
    pub domain: String,
    pub generation_type: String,
    pub is_essence_only: bool,
    pub name: String,
    pub required_level: u64,
    pub spawn_weights: Vec<SpawnWeight>,
    pub stats: Vec<Stat>,
    pub groups: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Hash, Eq)]
pub struct StatTranslation {
    pub English: Vec<LanguageInstance>,
    pub ids: Vec<String>,
    pub hidden: Option<bool>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct LanguageInstance {
    pub condition: Vec<Condition>,
    pub format: Vec<String>,
    pub index_handlers: Vec<Vec<String>>,
    pub string: String,
}
impl Hash for LanguageInstance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.format.hash(state);
        self.index_handlers.hash(state);
        self.string.hash(state);
    }
}
impl PartialEq for LanguageInstance {
    fn eq(&self, other: &Self) -> bool {
        self.format == other.format
            && self.index_handlers == other.index_handlers
            && self.string == other.string
    }
}
impl Eq for LanguageInstance {}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub negated: Option<bool>,
}

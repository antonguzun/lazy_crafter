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
    /// Human-readable representation shipped inline with the mod.
    /// Present in the RePoE-fork exports (both poe1 and poe2), absent in the
    /// legacy RePoE poe1 dump (which ships representations in a separate file).
    #[serde(default)]
    pub text: Option<String>,
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

/// Where the loader should take a mod's human-readable representation from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepresentationSource {
    /// Read each mod's representation from its inline `text` field
    /// (RePoE-fork exports, including the poe2 export).
    #[serde(rename = "inline_text")]
    InlineText,
    /// Read representations from a separate `mods_representation_pob.json`
    /// keyed by mod id (legacy RePoE poe1 dump).
    #[serde(rename = "pob_file")]
    PobFile,
}

impl Default for RepresentationSource {
    fn default() -> Self {
        // Backwards compatible: a data directory without a manifest is assumed
        // to be the legacy poe1 dump that ships a separate representation file.
        RepresentationSource::PobFile
    }
}

/// Optional `manifest.json` placed next to the data files. It tells the loader
/// which game the dataset belongs to and how to parse it, so the same binary
/// can consume either a poe1 or a poe2 dataset by just swapping the `data`
/// directory. Every field is optional and falls back to the legacy poe1
/// behaviour, so old data directories keep working without a manifest.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Manifest {
    /// Free-form game id, e.g. "poe1" or "poe2". Used only for diagnostics and
    /// to pick sensible UI defaults; parsing does not depend on it.
    #[serde(default)]
    pub game: Option<String>,
    /// How to resolve mod representations.
    #[serde(default)]
    pub representation: RepresentationSource,
    /// Where the dataset was produced from (diagnostics only).
    #[serde(default)]
    pub source: Option<String>,
}

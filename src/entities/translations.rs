use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatTranslation {
    pub English: Vec<LanguageInstance>,
    pub ids: Vec<String>,
    pub hidden: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageInstance {
    pub condition: Vec<Condition>,
    pub format: Vec<String>,
    pub index_handlers: Vec<Vec<String>>,
    pub string: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub negated: Option<bool>,
}

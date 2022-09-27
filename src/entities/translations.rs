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

impl LanguageInstance {
    pub fn get_representation_string(&self) -> String {
        let repr = self.string.clone();
        for (i, f) in self.format.iter().enumerate() {
            if f != "ignore" {
                let mut to_replace = "{0}";
                if i == 0 {to_replace = "{0}";}
                if i == 1 {to_replace = "{1}";}                
                repr.replace(to_replace, f);
            }
        }
        repr
    }
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub negated: Option<bool>,
}

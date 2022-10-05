use crate::storage::files::mods::Stat;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatTranslation {
    pub English: Vec<LanguageInstance>,
    pub ids: Vec<String>,
    pub hidden: Option<bool>,
}
impl StatTranslation {
    pub fn get_eng_representation_string(&self, stat: &Stat) -> std::string::String {
        for i in self.English.iter() {
            let mut repr = i.string.clone();
            let stat_max = stat.max.unwrap();
            let stat_min = stat.min.unwrap();

            let mut cond_passed = true;
            for c in i.condition.iter() {
                if c.negated == Some(true) {
                    return repr;
                }
                match c.min {
                    Some(min) => {
                        if stat_min < min {
                            cond_passed = false;
                        }
                    }
                    None => (),
                }
                match c.max {
                    Some(max) => {
                        if stat_max > max {
                            cond_passed = false;
                        }
                    }
                    None => (),
                }
            }
            if cond_passed {
                let to_str = match stat_max == stat_min {
                    true => format!("{}", stat_max),
                    false => format!("({}-{})", stat_min, stat_max),
                };
                repr = repr.replace("{0}", &to_str);
                return repr;
            }
        }
        unreachable!("No english representation found for stat {:?}", stat);
    }
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

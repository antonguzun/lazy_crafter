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
        let stat_max = stat.max.unwrap();
        let stat_min = stat.min.unwrap();
        let mut translation_id_position = 100;
        for i in 0..self.ids.len() {
            if self.ids[i] == stat.id {
                translation_id_position = i;
                break;
            }
        }
        if translation_id_position == 100 {
            panic!("Could not find translation for stat with id {}", stat.id);
        }

        for i in self.English.iter() {
            let mut repr = i.string.clone();
            let mut cond_passed = true;
            
            let condition = i.condition[translation_id_position].clone();
            let format = i.format[translation_id_position].clone();

            if condition.negated == Some(true) {
                return repr;
            }
            if format == "ignore" {
                continue;
            }
            match condition.min {
                Some(min) => {
                    if stat_min < min {
                        cond_passed = false;
                    }
                }
                None => (),
            }
            match condition.max {
                Some(max) => {
                    if stat_max > max {
                        cond_passed = false;
                    }
                }
                None => (),
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
        println!("No english representation found for stat {:?}", stat);
        "".to_string()
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

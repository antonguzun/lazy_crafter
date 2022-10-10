use std::collections::HashMap;

use crate::storage::files::mods::Stat;
use serde::{Deserialize, Serialize};

use super::mods::Mod;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Hash, Eq)]
pub struct LanguageInstance {
    pub condition: Vec<Condition>,
    pub format: Vec<String>,
    pub index_handlers: Vec<Vec<String>>,
    pub string: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Hash, Eq)]
pub struct Condition {
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub negated: Option<bool>,
}

// [
//     "divide_by_six",
//     "per_minute_to_per_second_2dp",
//     "per_minute_to_per_second_2dp_if_required",
//     "divide_by_three",
//     "tree_expansion_jewel_passive",
//     "divide_by_four",
//     "per_minute_to_per_second_0dp",
//     "divide_by_fifty",
//     "60%_of_value",
//     "divide_by_twelve",
//     "divide_by_ten_0dp",
//     "double",
//     "negate_and_double",
//     "30%_of_value",
//     "divide_by_one_hundred_2dp_if_required",
//     "mod_value_to_item_class",
//     "canonical_stat",
//     "divide_by_ten_1dp",
//     "metamorphosis_reward_description",
//     "per_minute_to_per_second",
//     "passive_hash",
//     "milliseconds_to_seconds_0dp",
//     "divide_by_one_thousand",
//     "display_indexable_support",
//     "multiplicative_damage_modifier",
//     "divide_by_twenty_then_double_0dp",
//     "old_leech_permyriad",
//     "affliction_reward_type",
//     "divide_by_fifteen_0dp",
//     "old_leech_percent",
//     "milliseconds_to_seconds",
//     "milliseconds_to_seconds_2dp_if_required",
//     "divide_by_one_hundred_2dp",
//     "per_minute_to_per_second_1dp",
//     "divide_by_two_0dp",
//     "divide_by_one_hundred",
//     "divide_by_five",
//     "milliseconds_to_seconds_1dp",
//     "milliseconds_to_seconds_2dp",
//     "divide_by_one_hundred_and_negate",
//     "deciseconds_to_seconds",
//     "negate",
//     "times_one_point_five",
//     "times_twenty",
//     "multiply_by_four",
//     "divide_by_ten_1dp_if_required"
// ]

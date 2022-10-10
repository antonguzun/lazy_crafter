use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct StatTranslation {
    pub English: Vec<LanguageInstance>,
    pub ids: Vec<String>,
    pub hidden: Option<bool>,
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

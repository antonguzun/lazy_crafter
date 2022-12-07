pub fn handle_stat_value(index_handler: &str, value: i64) -> String {
    match index_handler {
        "per_minute_to_per_second" => return per_minute_to_per_second(value),
        "divide_by_three" => return divide_by_three(value),
        "divide_by_four" => return divide_by_four(value),
        "divide_by_five" => return divide_by_five(value),
        "divide_by_six" => return divide_by_six(value),
        "divide_by_twelve" => return divide_by_twelve(value),
        "divide_by_fifty" => return divide_by_fifty(value),
        "divide_by_one_hundred" => return divide_by_one_hundred(value),
        "divide_by_one_thousand" => return divide_by_one_thousand(value),
        "60%_of_value" => return sixty_percent_of_value(value),
        "30%_of_value" => return thirty_percent_of_value(value),
        "double" => return double(value),
        _ => return value.to_string(),
    }
}

fn per_minute_to_per_second(value: i64) -> String {
    (value as f64 / 60.0).round().to_string()
}

fn divide_by_three(value: i64) -> String {
    (value as f64 / 3.0).round().to_string()
}

fn divide_by_four(value: i64) -> String {
    (value as f64 / 4.0).round().to_string()
}

fn divide_by_five(value: i64) -> String {
    (value as f64 / 5.0).round().to_string()
}

fn divide_by_six(value: i64) -> String {
    (value as f64 / 6.0).round().to_string()
}

fn divide_by_twelve(value: i64) -> String {
    (value as f64 / 12.0).round().to_string()
}

fn divide_by_fifty(value: i64) -> String {
    (value as f64 / 50.0).round().to_string()
}
fn divide_by_one_hundred(value: i64) -> String {
    (value as f64 / 100.0).round().to_string()
}

fn divide_by_one_thousand(value: i64) -> String {
    (value as f64 / 1000.0).round().to_string()
}

fn double(value: i64) -> String {
    (value * 2).to_string()
}

fn sixty_percent_of_value(value: i64) -> String {
    (value as f64 * 0.6).round().to_string()
}

fn thirty_percent_of_value(value: i64) -> String {
    (value as f64 * 0.3).round().to_string()
}

/* TODO!
[
    "per_minute_to_per_second_2dp",
    "per_minute_to_per_second_2dp_if_required",
    "tree_expansion_jewel_passive",
    "per_minute_to_per_second_0dp",
    "divide_by_ten_0dp",
    "negate_and_double",
    "divide_by_one_hundred_2dp_if_required",
    "mod_value_to_item_class",
    "canonical_stat",
    "divide_by_ten_1dp",
    "metamorphosis_reward_description",
    "passive_hash",
    "milliseconds_to_seconds_0dp",
    "display_indexable_support",
    "multiplicative_damage_modifier",
    "divide_by_twenty_then_double_0dp",
    "old_leech_permyriad",
    "affliction_reward_type",
    "divide_by_fifteen_0dp",
    "old_leech_percent",
    "milliseconds_to_seconds",
    "milliseconds_to_seconds_2dp_if_required",
    "divide_by_one_hundred_2dp",
    "per_minute_to_per_second_1dp",
    "divide_by_two_0dp",
    "milliseconds_to_seconds_1dp",
    "milliseconds_to_seconds_2dp",
    "divide_by_one_hundred_and_negate",
    "deciseconds_to_seconds",
    "negate",
    "times_one_point_five",
    "times_twenty",
    "multiply_by_four",
    "divide_by_ten_1dp_if_required"
]
*/

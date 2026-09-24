use crate::entities::craft_repo::{CraftRepo, ModsMatchMode, RollRequirements};
use crate::usecases::item_parser::ParsedItem;
use log::debug;
use regex::Regex;
use std::collections::{HashMap, HashSet};
pub struct ModMatcher {
    pub accepted_modset_by_mod_id: HashMap<String, HashSet<String>>,
}

impl ModMatcher {
    pub fn new(
        selected_mods: HashSet<String>,
        item_base_name: &str,
        repo: &impl CraftRepo,
    ) -> Result<ModMatcher, String> {
        let mut accepted_modset_by_mod_id = HashMap::new();

        for m_id in selected_mods.into_iter() {
            let subset = repo.get_subset_of_mods(&m_id, item_base_name)?;
            debug!("Got subset: {:?}", &subset);
            accepted_modset_by_mod_id.insert(m_id, subset);
        }
        Ok(ModMatcher {
            accepted_modset_by_mod_id,
        })
    }
}

pub fn check_matching(
    matcher: ModMatcher,
    crafted_mod_ids: HashSet<String>,
    mode: ModsMatchMode,
) -> bool {
    // no selected mods: nothing to wait for, any craft is a match
    if matcher.accepted_modset_by_mod_id.is_empty() {
        return true;
    }
    for (_, accepted_set) in matcher.accepted_modset_by_mod_id {
        let mut matched = false;
        debug!("Looking for: {:?}", &accepted_set);
        for crafted_mod_id in &crafted_mod_ids {
            if accepted_set.contains(crafted_mod_id) {
                debug!("matched {}", crafted_mod_id);
                matched = true;
            } else {
                debug!("match failed {}", crafted_mod_id);
            }
        }
        match mode {
            ModsMatchMode::All => {
                if !matched {
                    return false;
                }
            }
            ModsMatchMode::Any => {
                if matched {
                    return true;
                }
            }
        }
    }
    matches!(mode, ModsMatchMode::All)
}

/// Check both the selected tiers and the values shown in advanced item text.
pub fn check_matching_with_rolls(
    matcher: ModMatcher,
    item: &ParsedItem,
    mode: ModsMatchMode,
    requirements: RollRequirements,
) -> bool {
    let crafted_ids: HashSet<String> = item.mods.iter().cloned().collect();
    if requirements.min_percent == 0 || matcher.accepted_modset_by_mod_id.is_empty() {
        return check_matching(matcher, crafted_ids, mode);
    }

    if mode == ModsMatchMode::Any && requirements.ignore_roll_at_or_count > 0 {
        let matching_count = crafted_ids
            .iter()
            .filter(|id| {
                matcher
                    .accepted_modset_by_mod_id
                    .values()
                    .any(|set| set.contains(*id))
            })
            .count();
        if matching_count >= requirements.ignore_roll_at_or_count {
            return true;
        }
    }

    let roll_re = roll_regex();
    let passing_ids = item
        .mods
        .iter()
        .zip(&item.raw_mods)
        .filter(|(_, text)| meets_roll_threshold(text, requirements.min_percent, &roll_re))
        .map(|(id, _)| id.clone())
        .collect();
    check_matching(matcher, passing_ids, mode)
}

fn roll_regex() -> Regex {
    Regex::new(r"([+-]?\d+(?:\.\d+)?)\(([+-]?\d+(?:\.\d+)?)-([+-]?\d+(?:\.\d+)?)\)")
        .expect("valid advanced roll expression")
}

fn meets_roll_threshold(text: &str, min_percent: u8, re: &Regex) -> bool {
    // Fixed stats have no range and are already perfect. The item parser
    // resolves each mod against its full range representation before this.
    re.captures_iter(text).all(|cap| {
        let mut value: f64 = cap[1].parse().unwrap();
        let start: f64 = cap[2].parse().unwrap();
        let end: f64 = cap[3].parse().unwrap();
        // A factored-out sign, e.g. -60(75-50), applies to the entire range.
        // Preserve endpoint order, which can be descending in the clipboard.
        if value < 0.0 && start >= 0.0 && end >= 0.0 {
            value = -value;
        }
        if start == end {
            return (value - start).abs() < 1e-9;
        }
        let position = (value - start) / (end - start);
        position >= -1e-9
            && position <= 1.0 + 1e-9
            && position * 100.0 + 1e-9 >= f64::from(min_percent)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn matcher_with(sets: Vec<(&str, Vec<&str>)>) -> ModMatcher {
        let mut accepted_modset_by_mod_id = HashMap::new();
        for (mod_id, set) in sets {
            accepted_modset_by_mod_id.insert(
                mod_id.to_string(),
                set.into_iter().map(String::from).collect(),
            );
        }
        ModMatcher {
            accepted_modset_by_mod_id,
        }
    }

    fn crafted(ids: Vec<&str>) -> HashSet<String> {
        ids.into_iter().map(String::from).collect()
    }

    #[test]
    fn all_mode_requires_every_selected_mod() {
        let m = || matcher_with(vec![("a", vec!["a1", "a2"]), ("b", vec!["b1"])]);
        assert!(check_matching(
            m(),
            crafted(vec!["a1", "b1"]),
            ModsMatchMode::All
        ));
        assert!(!check_matching(
            m(),
            crafted(vec!["a2"]),
            ModsMatchMode::All
        ));
    }

    #[test]
    fn any_mode_requires_at_least_one_selected_mod() {
        let m = || matcher_with(vec![("a", vec!["a1", "a2"]), ("b", vec!["b1"])]);
        assert!(check_matching(
            m(),
            crafted(vec!["a2", "x"]),
            ModsMatchMode::Any
        ));
        assert!(!check_matching(
            m(),
            crafted(vec!["x", "y"]),
            ModsMatchMode::Any
        ));
    }

    #[test]
    fn empty_selection_matches_in_both_modes() {
        assert!(check_matching(
            matcher_with(vec![]),
            crafted(vec!["x"]),
            ModsMatchMode::All
        ));
        assert!(check_matching(
            matcher_with(vec![]),
            crafted(vec!["x"]),
            ModsMatchMode::Any
        ));
    }

    fn rolled_item(mods: &[(&str, &str)]) -> ParsedItem {
        ParsedItem {
            item_class: String::new(),
            item_base_name: String::new(),
            item_name: String::new(),
            mods: mods.iter().map(|(id, _)| id.to_string()).collect(),
            raw_mods: mods.iter().map(|(_, text)| text.to_string()).collect(),
        }
    }

    #[test]
    fn roll_boundaries_decimals_hybrids_and_fixed_stats() {
        let re = roll_regex();
        for (text, threshold, expected) in [
            ("10(10-20)% increased Armour", 0, true),
            ("10(10-20)% increased Armour", 1, false),
            ("15(10-20)% increased Armour", 50, true),
            ("15(10-20)% increased Armour", 51, false),
            ("20(10-20)% increased Armour", 100, true),
            ("0.26(0.2-0.4)% Leeched as Life", 30, true),
            ("0.26(0.2-0.4)% Leeched as Life", 31, false),
            ("Adds 10(5-10) to 15(15-25) Damage", 50, false),
            ("+20(10-20) Armour; +15(10-20) Evasion", 50, true),
            ("+20(10-20) Armour; +15(10-20) Evasion", 51, false),
            ("+1 to Level of Skills", 100, true),
            ("5(5-5)% increased Damage", 100, true),
            ("-60(75-50) to Mana Cost", 60, true),
            ("-60(75-50) to Mana Cost", 61, false),
            ("-7(-10--5) to Cost", 60, true),
        ] {
            assert_eq!(
                meets_roll_threshold(text, threshold, &re),
                expected,
                "{text}, {threshold}%"
            );
        }
    }

    #[test]
    fn roll_filter_obeys_and_or_and_checks_better_tiers() {
        let m = || matcher_with(vec![("a", vec!["a", "a_better"]), ("b", vec!["b"])]);
        let item = rolled_item(&[("a_better", "10(10-20)"), ("b", "20(10-20)")]);
        let requirements = RollRequirements {
            min_percent: 80,
            ignore_roll_at_or_count: 0,
        };
        assert!(check_matching_with_rolls(
            m(),
            &item,
            ModsMatchMode::Any,
            requirements
        ));
        assert!(!check_matching_with_rolls(
            m(),
            &item,
            ModsMatchMode::All,
            requirements
        ));
        let low = rolled_item(&[("a_better", "10(10-20)")]);
        assert!(!check_matching_with_rolls(
            m(),
            &low,
            ModsMatchMode::Any,
            requirements
        ));
        assert!(check_matching_with_rolls(
            m(),
            &low,
            ModsMatchMode::Any,
            RollRequirements::default()
        ));
    }

    #[test]
    fn or_exception_requires_n_distinct_qualifying_mods() {
        let m = || {
            matcher_with(vec![
                ("a", vec!["a", "a_better"]),
                ("a_better", vec!["a_better"]),
                ("b", vec!["b"]),
            ])
        };
        let requirements = RollRequirements {
            min_percent: 100,
            ignore_roll_at_or_count: 2,
        };
        let one = rolled_item(&[("a_better", "10(10-20)"), ("unselected", "20(10-20)")]);
        assert!(!check_matching_with_rolls(
            m(),
            &one,
            ModsMatchMode::Any,
            requirements
        ));
        let two = rolled_item(&[("a_better", "10(10-20)"), ("b", "10(10-20)")]);
        assert!(check_matching_with_rolls(
            m(),
            &two,
            ModsMatchMode::Any,
            requirements
        ));
        assert!(!check_matching_with_rolls(
            m(),
            &two,
            ModsMatchMode::All,
            requirements
        ));
        assert!(!check_matching_with_rolls(
            m(),
            &two,
            ModsMatchMode::Any,
            RollRequirements {
                ignore_roll_at_or_count: 3,
                ..requirements
            }
        ));
        let wrong_tier = rolled_item(&[("a_better", "10(10-20)"), ("b_worse", "20(10-20)")]);
        assert!(!check_matching_with_rolls(
            m(),
            &wrong_tier,
            ModsMatchMode::Any,
            requirements
        ));
        let duplicate = rolled_item(&[("a_better", "10(10-20)"), ("a_better", "10(10-20)")]);
        assert!(!check_matching_with_rolls(
            m(),
            &duplicate,
            ModsMatchMode::Any,
            requirements
        ));
    }
}

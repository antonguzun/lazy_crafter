use crate::entities::craft_repo::{CraftRepo, ModsMatchMode};
use log::{debug, info};
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
            accepted_modset_by_mod_id
                .insert(m_id, subset);
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
        assert!(check_matching(m(), crafted(vec!["a1", "b1"]), ModsMatchMode::All));
        assert!(!check_matching(m(), crafted(vec!["a2"]), ModsMatchMode::All));
    }

    #[test]
    fn any_mode_requires_at_least_one_selected_mod() {
        let m = || matcher_with(vec![("a", vec!["a1", "a2"]), ("b", vec!["b1"])]);
        assert!(check_matching(m(), crafted(vec!["a2", "x"]), ModsMatchMode::Any));
        assert!(!check_matching(m(), crafted(vec!["x", "y"]), ModsMatchMode::Any));
    }

    #[test]
    fn empty_selection_matches_in_both_modes() {
        assert!(check_matching(matcher_with(vec![]), crafted(vec!["x"]), ModsMatchMode::All));
        assert!(check_matching(matcher_with(vec![]), crafted(vec!["x"]), ModsMatchMode::Any));
    }
}

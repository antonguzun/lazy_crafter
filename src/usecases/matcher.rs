use crate::entities::craft_repo::CraftRepo;

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
            accepted_modset_by_mod_id
                .insert(m_id, subset)
                .ok_or("matcher internal error")?;
        }
        Ok(ModMatcher {
            accepted_modset_by_mod_id,
        })
    }
}

pub fn check_matching(matcher: ModMatcher, crafted_mod_ids: HashSet<String>) -> bool {
    for (_, accepted_set) in matcher.accepted_modset_by_mod_id {
        let mut matched = false;
        for crafted_mod_id in &crafted_mod_ids {
            if accepted_set.contains(crafted_mod_id) {
                matched = true;
            }
        }
        if !matched {
            return false;
        }
    }
    true
}

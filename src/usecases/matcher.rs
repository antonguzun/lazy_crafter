use crate::entities::craft_repo::CraftRepo;
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

pub fn check_matching(matcher: ModMatcher, crafted_mod_ids: HashSet<String>) -> bool {
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
        if !matched {
            return false;
        }
    }
    true
}

use std::collections::HashMap;

use crate::entities::craft_repo::{CraftRepo, ItemBase, ModItem, ModsQuery};

pub fn find_mods(repo: &impl CraftRepo, query: &ModsQuery) -> Vec<ModItem> {
    repo.find_mods(query)
}

pub fn get_item_classes(repo: &impl CraftRepo) -> Vec<String> {
    repo.get_item_classes()
}

pub fn get_item_bases(repo: &impl CraftRepo, item_class: &str) -> Vec<ItemBase> {
    repo.get_item_bases(item_class)
}

pub fn get_item_class_by_item_name(repo: &impl CraftRepo) -> HashMap<String, String> {
    repo.get_item_class_by_item_name()
}

pub fn get_weight_of_target_and_better_mods(
    repo: &impl CraftRepo,
    query: &ModsQuery,
    target_mod_key: String,
) -> u32 {
    repo.get_weight_of_target_and_better_mods(query, target_mod_key)
}

pub fn get_affected_weight_of_target_mod(repo: &impl CraftRepo, query: &ModsQuery) -> u32 {
    repo.get_affected_weight_of_target_mod(query)
}

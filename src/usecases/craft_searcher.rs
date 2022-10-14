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

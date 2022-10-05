use crate::entities::craft_repo::{CraftRepo, ModItem, ModsQuery};

pub fn find_mods(repo: &impl CraftRepo, query: &ModsQuery) -> std::vec::Vec<ModItem> {
    repo.find_mods(query)
}

pub fn get_item_classes(repo: &impl CraftRepo) -> std::vec::Vec<String> {
    repo.get_item_classes()
}

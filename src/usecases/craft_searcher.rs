use crate::entities::craft_repo::{CraftRepo, ModItem, ModsQuery};

pub fn find_mods<'a, 'b>(repo: &'a impl CraftRepo, query: &'b ModsQuery) -> std::vec::Vec<&'a ModItem> {
    repo.find_mods(query)
}

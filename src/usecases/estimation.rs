use crate::entities::craft_repo::{CraftRepo, Estimation, ItemBase, ModItem, ModsQuery};

pub fn calculate_estimation_for_craft(
    repo: &impl CraftRepo,
    query: &ModsQuery,
) -> Result<Estimation, String> {
    if query.selected_mods.is_empty() {
        return Err("No mods selected".to_string());
    }
    Err("not implemented".to_string())
}

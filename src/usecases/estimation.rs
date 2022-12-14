use crate::entities::craft_repo::{CraftRepo, Estimation, ItemBase, ModItem, ModsQuery};

pub fn calculate_estimation_for_craft(
    repo: &impl CraftRepo,
    query: &ModsQuery,
) -> Result<Estimation, String> {
    Err("not implemented".to_string())
}

use crate::entities::craft_repo::{CraftRepo, Estimation, ItemBase, ModItem, ModsQuery};



fn chaos_variants_ratio(prefix_count: usize, suffix_count: usize) -> f64 {
    match (prefix_count, suffix_count) {
        (1, 3) => 0.2814,
        (2, 2) => 0.2836,
        (2, 3) => 0.1725,
        (3, 1) => 0.1016,
        (3, 2) => 0.0775,
        (3, 3) => 0.0833,
        _ => 0.0,
    }
}


pub fn calculate_estimation_for_craft(
    repo: &impl CraftRepo,
    query: &ModsQuery,
) -> Result<Estimation, String> {
    if query.selected_mods.is_empty() {
        return Err("no mods selected".to_string());
    }

    let prefix_count = query.selected_mods.iter().filter(|m| m.generation_type == "prefix").count();
    let suffix_count = query.selected_mods.iter().filter(|m| m.generation_type == "suffix").count();
    
    if prefix_count > 3 || suffix_count > 3 {
        return Err("too many affixes selected".to_string());
    }

    let mut variant_with_ratios = vec![];
    for pc in prefix_count..3 {
        for sc in suffix_count..3 {
            let ratio = chaos_variants_ratio(pc, sc);
            if ratio == 0.0 {
                continue;
            }
            variant_with_ratios.push((pc, sc, ratio));
        }
    }

    // for each variant calculate probability with permutations between each mod type
    // multiply probabilities for each variant ratio
    // sum all variants probabilities

    Err("not implemented".to_string())


}

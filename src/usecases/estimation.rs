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

fn probability_for_variant(
    prefix_count: usize,
    suffix_count: usize,
    selected_mods: &Vec<ModItem>,
    available_mods: &Vec<ModItem>,
) -> f64 {
    let selected_prefixes = selected_mods
        .iter()
        .filter(|m| m.generation_type == "prefix")
        .collect::<Vec<&ModItem>>();
    // let fake_prefixes = prefix_count - selected_prefixes.len();
    let cases_probability: Vec<f64> = vec![0.0];

    // let prefix_summarized_weight = available_mods
    //     .iter()
    //     .filter(|m| m.generation_type == "prefix")
    //     .map(|m| m.weight)
    //     .sum::<u32>();

    // // let prefix_mod_key_with_weight = vec![];

    // let prefix_permutations: Vec<Option<ModItem>> = vec![];

    // for i in 0..prefix_count {
    //     for j in 0..prefix_count {}
    // }

    cases_probability.iter().sum()
}

pub fn calculate_estimation_for_craft(
    repo: &impl CraftRepo,
    query: &ModsQuery,
) -> Result<Estimation, String> {
    let selected_mods = query.selected_mods.clone();
    if selected_mods.is_empty() {
        return Err("no mods selected".to_string());
    }

    let required_prefix_count = selected_mods
        .iter()
        .filter(|m| m.generation_type == "prefix")
        .count();
    let required_suffix_count = selected_mods
        .iter()
        .filter(|m| m.generation_type == "suffix")
        .count();

    if required_prefix_count > 3 || required_suffix_count > 3 {
        return Err("too many affixes selected".to_string());
    }

    let mut variant_with_ratios = vec![];
    for pc in required_prefix_count..3 {
        for sc in required_suffix_count..3 {
            let ratio = chaos_variants_ratio(pc, sc);
            if ratio == 0.0 {
                continue;
            }
            variant_with_ratios.push((pc, sc, ratio));
        }
    }

    let available_mods_query = ModsQuery {
        string_query: query.string_query.clone(),
        item_base: query.item_base.clone(),
        item_level: query.item_level,
        selected_mods: vec![],
    };
    let available_mods = repo.find_mods(&available_mods_query);

    let sum: f64 = variant_with_ratios
        .iter()
        .map(|(pc, sc, ratio)| {
            probability_for_variant(*pc, *sc, &selected_mods, &available_mods) * ratio
        })
        .sum();
    Ok(Estimation { probability: sum })
}

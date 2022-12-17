use log::debug;

use crate::entities::craft_repo::{CraftRepo, Estimation, ItemBase, ModItem, ModsQuery};
use crate::usecases::craft_searcher::get_weight_of_target_and_better_mods;
use itertools::Itertools;

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
    repo: &impl CraftRepo,
    prefix_count: usize,
    suffix_count: usize,
    selected_mods: &Vec<(ModItem, u32)>,
    available_mods: &Vec<ModItem>,
) -> f64 {
    let selected_prefixes = selected_mods
        .iter()
        .filter(|(m, _w)| m.generation_type == "prefix")
        .map(|(m, _w)| m)
        .collect::<Vec<&ModItem>>();

    let mut cases_probability: Vec<f64> = vec![0.0];

    let target_prefixes = selected_prefixes
        .iter()
        .map(|m| m.mod_key.clone())
        .collect::<Vec<String>>();
    let mut prefixes_mod_keys = target_prefixes.clone();
    for _i in 1..(prefix_count - selected_prefixes.len() + 1) {
        prefixes_mod_keys.push(String::from("FAKE"));
    }
    debug!("prefixes_mod_keys: {:?}", prefixes_mod_keys);
    let weight = available_mods
        .iter()
        .filter(|m| m.generation_type == "prefix")
        .map(|m| m.weight)
        .sum::<u32>();

    for permutation in prefixes_mod_keys.iter().permutations(prefix_count).unique() {
        let mut local_weight = weight;
        let mut local_p = vec![1.0];
        for mod_key in permutation.iter() {
            let w;
            if *mod_key == "FAKE" {
                w = (local_weight as f64 * 0.1) as u32;
            } else {
                w = selected_mods
                    .iter()
                    .find(|(m, _w)| m.mod_key == **mod_key)
                    .unwrap()
                    .1;
                local_p.push(w as f64 / local_weight as f64);
            }
            local_weight -= w;
        }
        cases_probability.push(local_p.iter().product());
        debug!("permutation: {:?}", permutation);
    }
    let res_sum = cases_probability.iter().sum();

    debug!("p{}s{}: prob={}", prefix_count, suffix_count, res_sum);
    res_sum
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
    debug!(
        "required_prefix_count: {}; required_suffix_count: {}",
        required_prefix_count, required_suffix_count
    );

    let mut variant_with_ratios = vec![];
    for pc in required_prefix_count..3 + 1 {
        for sc in required_suffix_count..3 + 1 {
            let ratio = chaos_variants_ratio(pc, sc);
            if ratio == 0.0 {
                continue;
            }
            variant_with_ratios.push((pc, sc, ratio));
        }
    }
    debug!("variant_with_ratios: {:?}", variant_with_ratios);

    let available_mods_query = ModsQuery {
        string_query: "".to_string(),
        item_base: query.item_base.clone(),
        item_level: query.item_level,
        selected_mods: vec![],
    };
    let available_mods = repo.find_mods(&available_mods_query);

    let target_mods_with_weights = selected_mods
        .iter()
        .map(|m| {
            (
                m.clone(),
                get_weight_of_target_and_better_mods(repo, query, m.mod_key.clone()),
            )
        })
        .collect();

    debug!("target_mods_with_weights: {:?}", target_mods_with_weights);

    let sum: f64 = variant_with_ratios
        .iter()
        .map(|(pc, sc, ratio)| {
            probability_for_variant(repo, *pc, *sc, &target_mods_with_weights, &available_mods)
                * ratio
        })
        .sum();
    Ok(Estimation { probability: sum })
}

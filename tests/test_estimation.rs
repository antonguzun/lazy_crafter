use lazy_crafter::entities::craft_repo::{CraftRepo, Estimation, ItemBase, ModItem, ModsQuery};
use lazy_crafter::storage::files::local_db::FileRepo;
use lazy_crafter::usecases::estimation::calculate_estimation_for_craft;

use rstest::{fixture, rstest};

#[fixture]
fn repo() -> impl CraftRepo {
    FileRepo::new().unwrap()
}

#[rstest]
#[case(ModsQuery {
    string_query: "".to_string(),
    item_base: "Carnal Boots".to_string(),
    item_level: 100,
    selected_mods: vec![
            ModItem { required_level: 24, weight: 1000, generation_type: "prefix".to_string(), representation: "(40-49) to maximum Life".to_string(), mod_key: "IncreasedLife4".to_string() }, 
            ModItem { required_level: 30, weight: 1000, generation_type: "prefix".to_string(), representation: "20% increased Movement Speed".to_string(), mod_key: "MovementVelocity3".to_string() }
        ],
}, Estimation { probability: 0.02665 })]
fn test_estimation(repo: impl CraftRepo, #[case] query: ModsQuery, #[case] expected: Estimation) {
    let estimation = calculate_estimation_for_craft(&repo, &query);
    assert_eq!(estimation, Ok(expected));
}

#[rstest]
#[case(ModsQuery {
    string_query: "".to_string(),
    item_base: "Abyssus".to_string(),
    item_level: 100,
    selected_mods: vec![],
}, "No mods selected".to_string())]
fn test_estimation_negative(
    repo: impl CraftRepo,
    #[case] query: ModsQuery,
    #[case] expected: String,
) {
    let estimation = calculate_estimation_for_craft(&repo, &query);
    assert_eq!(estimation, Err(expected));
}

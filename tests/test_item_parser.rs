use lazy_crafter::entities::craft_repo::CraftRepo;
use lazy_crafter::storage::files::local_db::FileRepo;
use lazy_crafter::usecases::craft_searcher::{parse_raw_item, ParsedItem};

use rstest::{fixture, rstest};

#[fixture]
fn repo() -> impl CraftRepo {
    FileRepo::new().unwrap()
}

#[rstest]
#[case("Item Class: Bows
Rarity: Magic
Imperial Bow of Restoration
--------
Bow
Physical Damage: 29-117
Critical Strike Chance: 5.00%
Attacks per Second: 1.45
--------
Requirements:
Level: 66
Dex: 212
--------
Sockets: G G G-G G B
--------
Item Level: 81
--------
24% increased Elemental Damage with Attack Skills (implicit)
--------
Gain 3 Life per Enemy Hit by Attacks", ParsedItem {
    item_class: "Bow".to_string(),
    item_base_name: "Imperial Bow".to_string(),
    item_name: "Imperial Bow of Restoration".to_string(),
    mods: vec!["LifeGainPerTargetLocal2".to_string()],
    raw_mods: vec!["Gain 3 Life per Enemy Hit by Attacks".to_string()],
})]
#[case("Item Class: Bows
Rarity: Magic
Freezing Imperial Bow of the Drake
--------
Bow
Physical Damage: 29-117
Elemental Damage: 44-84 (augmented)
Critical Strike Chance: 5.00%
Attacks per Second: 1.45
--------
Requirements:
Level: 66
Dex: 212
--------
Sockets: G G G-G G B
--------
Item Level: 81
--------
24% increased Elemental Damage with Attack Skills (implicit)
--------
Adds 44 to 84 Cold Damage
+22% to Fire Resistance", ParsedItem {
    item_class: "Bow".to_string(),
    item_base_name: "Imperial Bow".to_string(),
    item_name: "Freezing Imperial Bow of the Drake".to_string(),
    mods: vec![
        "LocalAddedColdDamageTwoHand5".to_string(),
        "FireResist3".to_string(),
    ],
    raw_mods: vec![
        "Adds 44 to 84 Cold Damage".to_string(),
        "+22% to Fire Resistance".to_string(),
    ],
})]
#[case("Item Class: Bows
Rarity: Magic
Long Bow of Success
--------
Bow
Physical Damage: 8-33
Critical Strike Chance: 6.00%
Attacks per Second: 1.30
--------
Requirements:
Level: 9
Dex: 38
--------
Sockets: B-G
--------
Item Level: 13
--------
Gain 4 Life per Enemy Killed", ParsedItem {
    item_class: "Bow".to_string(),
    item_base_name: "Long Bow".to_string(),
    item_name: "Long Bow of Success".to_string(),
    mods: vec![
        "LifeGainedFromEnemyDeath1".to_string(),
    ],
    raw_mods: vec![
        "Gain 4 Life per Enemy Killed".to_string(),
    ],
})]
fn test_parse_raw_item(repo: impl CraftRepo, #[case] input: &str, #[case] expected: ParsedItem) {
    assert_eq!(parse_raw_item(&repo, &input), Ok(expected));
}

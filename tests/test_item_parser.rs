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
#[case("Item Class: Shields
Rarity: Normal
Copper Tower Shield
--------
Chance to Block: 24%
Armour: 164
--------
Requirements:
Level: 24
Str: 62
--------
Sockets: R-R
--------
Item Level: 26
--------
+39 to maximum Life (implicit)", ParsedItem {
    item_class: "Shield".to_string(),
    item_base_name: "Copper Tower Shield".to_string(),
    item_name: "Copper Tower Shield".to_string(),
    mods: vec![],
    raw_mods: vec![],
})]
#[case("Item Class: Boots
Rarity: Magic
Magpie's Ringmail Boots
--------
Armour: 29
Energy Shield: 7
--------
Requirements:
Level: 16
Str: 15
Int: 15
--------
Sockets: B
--------
Item Level: 26
--------
9% increased Rarity of Items found", ParsedItem {
    item_class: "Boots".to_string(),
    item_base_name: "Ringmail Boots".to_string(),
    item_name: "Magpie's Ringmail Boots".to_string(),
    mods: vec!["ItemFoundRarityIncrease1".to_string()],
    raw_mods: vec!["9% increased Rarity of Items found".to_string()],
})]
#[case("Item Class: Thrusting One Hand Swords
Rarity: Magic
Antique Rapier of the Penguin
--------
One Handed Sword
Physical Damage: 12-46
Critical Strike Chance: 6.50%
Attacks per Second: 1.30
Weapon Range: 14
--------
Requirements:
Level: 26
Dex: 89
--------
Sockets: G-G-G
--------
Item Level: 27
--------
+25% to Global Critical Strike Multiplier (implicit)
--------
+18% to Cold Resistance", ParsedItem { 
    item_class: "Thrusting One Hand Sword".to_string(),
    item_base_name: "Antique Rapier".to_string(),
    item_name: "Antique Rapier of the Penguin".to_string(),
    mods: vec!["ColdResist3".to_string()],
    raw_mods: vec!["+18% to Cold Resistance".to_string()]
})]
fn test_parse_raw_item(repo: impl CraftRepo, #[case] input: &str, #[case] expected: ParsedItem) {
    assert_eq!(parse_raw_item(&repo, &input), Ok(expected));
}

#[rstest]
#[case("", "No item class matches in string".to_string())]
#[case("Item Class: BlaBla
Rarity: Magic
Antique Rapier of the Penguin
--------
One Handed Sword", "Item class not found in db: BlaBla".to_string())]
#[case("Item Class: Thrusting One Hand Swords
Antique Rapier of the Penguin
--------
+18% to Magic Resistance", "Found wrong count of mods".to_string())]
#[case("Item Class: Thrusting One Hand Swords
Antique Penguin of the Rapier
--------
+18% to Magic Resistance", "No item base found".to_string())]
fn test_parse_raw_item_negative(
    repo: impl CraftRepo,
    #[case] input: &str,
    #[case] expected: String,
) {
    assert_eq!(parse_raw_item(&repo, &input), Err(expected));
}

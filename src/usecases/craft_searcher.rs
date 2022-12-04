use std::collections::HashMap;

use crate::entities::craft_repo::{CraftRepo, ItemBase, ModItem, ModsQuery};

pub fn find_mods(repo: &impl CraftRepo, query: &ModsQuery) -> Vec<ModItem> {
    repo.find_mods(query)
}

pub fn get_item_classes(repo: &impl CraftRepo) -> Vec<String> {
    repo.get_item_classes()
}

pub fn get_item_bases(repo: &impl CraftRepo, item_class: &str) -> Vec<ItemBase> {
    repo.get_item_bases(item_class)
}

pub fn get_item_class_by_item_name(repo: &impl CraftRepo) -> HashMap<String, String> {
    repo.get_item_class_by_item_name()
}

#[derive(Debug, PartialEq)]
pub struct ParsedItem {
    item_class: String,
    item_base_name: String,
    item_name: String,
    mods: Vec<String>,
    raw_mods: Vec<String>,
}

pub fn parse_raw_item(craft_repo: &impl CraftRepo, raw_item: &str) -> Result<ParsedItem, String> {
    let item_class = raw_item
        .split("\n")
        .find_map(|row| match craft_repo.item_class_if_exists(row.trim()) {
            true => Some(row.trim().to_string()),
            false => None,
        })
        .ok_or("No item class found".to_string())?;

    let (item_base_name, item_name) = raw_item
        .split("\n")
        .find_map(
            |row| match craft_repo.string_to_item_base(&item_class, row.trim()) {
                Ok(base_name) => Some((base_name, row.trim().to_string())),
                Err(_) => None,
            },
        )
        .ok_or("No item base found".to_string())?;

    let last_part = match raw_item.split("--------").last() {
        Some(last_part) => last_part,
        None => return Err("No mods found".to_string()),
    };

    let mut mods = vec![];
    let mut raw_mods = vec![];

    last_part.split("\n").for_each(|row| {
        match craft_repo.string_to_mod(&item_class, &item_base_name, row.trim()) {
            Ok(mod_name) => {
                mods.push(mod_name);
                raw_mods.push(row.trim().to_string());
            }
            Err(_) => {}
        }
    });

    if last_part.trim().split("\n").count() != mods.len() {
        return Err("Found wrong count of mods".to_string());
    }

    Ok(ParsedItem {
        item_class,
        item_base_name,
        item_name,
        mods,
        raw_mods,
    })

    // Err("Can not parse item".to_string())
}

#[test]
fn test_parse_raw_item1() {
    let str = "Item Class: Bows
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
Gain 3 Life per Enemy Hit by Attacks"
        .to_string();
    let parsed_item = ParsedItem {
        item_class: "Bow".to_string(),
        item_base_name: "Imperial Bow".to_string(),
        item_name: "Imperial Bow of Restoration".to_string(),
        mods: vec!["LifeGainPerTargetLocal2".to_string()],
        raw_mods: vec!["Gain 3 Life per Enemy Hit by Attacks".to_string()],
    };
    use crate::storage::files::local_db::FileRepo;
    let repo = FileRepo::new().unwrap();
    assert_eq!(parse_raw_item(&repo, &str), Ok(parsed_item));
}

#[test]
fn test_parse_raw_item2() {
    let str = "Item Class: Bows
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
+22% to Fire Resistance"
        .to_string();
    let parsed_item = ParsedItem {
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
    };
    use crate::storage::files::local_db::FileRepo;
    let repo = FileRepo::new().unwrap();
    assert_eq!(parse_raw_item(&repo, &str), Ok(parsed_item));
}

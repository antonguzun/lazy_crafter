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
    pub item_class: String,
    pub item_base_name: String,
    pub item_name: String,
    pub mods: Vec<String>,
    pub raw_mods: Vec<String>,
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
}

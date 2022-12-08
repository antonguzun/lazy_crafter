use std::collections::HashMap;

use log::debug;
use regex::Regex;

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
    let re = Regex::new(r"Item Class: (.*)\n").expect("regexp error during item class fetching");
    let raw_item_class = re
        .captures(raw_item)
        .ok_or("No item class matches in string".to_string())?
        .get(1)
        .ok_or("No item class in string found".to_string())?
        .as_str()
        .trim();

    let item_class =
        if craft_repo.item_class_if_exists(raw_item_class[..raw_item_class.len() - 1].trim()) {
            raw_item_class[..raw_item_class.len() - 1].trim()
        } else if craft_repo.item_class_if_exists(raw_item_class.trim()) {
            raw_item_class.trim()
        } else {
            return Err(format!(
                "Item class not found in db: {}",
                &raw_item_class.trim()
            ));
        };

    let (item_base_name, item_name) = raw_item
        .split("\n")
        .find_map(
            |row| match craft_repo.string_to_item_base(item_class, row.trim()) {
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

    let mut need_to_parse_deeper = false;
    if last_part.trim().split("\n").count() != mods.len() {
        match last_part.trim().split("\n").last() {
            Some(value) if !value.contains("implicit") => {
                need_to_parse_deeper = true;
            }
            Some(_) => {}
            None => {}
        }
    }
    let mut added_complex_mods = 0;
    if need_to_parse_deeper {
        let mod_lines: Vec<&str> = last_part.trim().split("\n").collect();

        let mut chunked_mods = vec![];
        for chunk in mod_lines.chunks(2) {
            if chunk.len() == 2 {
                let v = format!("{}\n{}", chunk[0].clone(), chunk[1].clone());
                chunked_mods.push(v);
                let v = format!("{}\n{}", chunk[1].clone(), chunk[0].clone());
                chunked_mods.push(v);
            }
        }
        for chunk in mod_lines[1..].chunks(2) {
            if chunk.len() == 2 {
                let v = format!("{}\n{}", chunk[0].clone(), chunk[1].clone());
                chunked_mods.push(v);
                let v = format!("{}\n{}", chunk[1].clone(), chunk[0].clone());
                chunked_mods.push(v);
            }
        }
        chunked_mods.into_iter().for_each(|row| {
            match craft_repo.string_to_mod(&item_class, &item_base_name, row.trim()) {
                Ok(mod_name) => {
                    mods.push(mod_name);
                    raw_mods.push(row.trim().to_string());
                    added_complex_mods += 1;
                }
                Err(_) => {}
            }
        });
    }

    if last_part.trim().split("\n").count() != mods.len() && added_complex_mods == 0 {
        match last_part.trim().split("\n").last() {
            Some(value) if !value.contains("implicit") => {
                return Err("Found wrong count of mods".to_string());
            }
            Some(_) => {}
            None => {}
        }
    } else if last_part.trim().split("\n").count() != (mods.len() + added_complex_mods) {
        // guess complex mod based on two mods always
        return Err("Found wrong count of mods".to_string());
    }

    Ok(ParsedItem {
        item_class: item_class.to_string(),
        item_base_name,
        item_name,
        mods,
        raw_mods,
    })
}

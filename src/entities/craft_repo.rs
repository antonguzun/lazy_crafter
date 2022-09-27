use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone)]
pub struct ModItem {
    pub item_level: u64,
    pub weight: u32,
    pub representation: String,
    pub mod_key: String,
    // max: i32,
    // min: i32,
}

pub struct ModsQuery {
    pub string_query: String,
    pub item_level: u64,
    pub item_class: String,
}

#[derive(Debug, EnumIter, Copy, Clone)]
pub enum ItemClass {
    Helmet,
    Boots,
}

impl ItemClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            ItemClass::Helmet => "helmet",
            ItemClass::Boots => "boots",
        }
    }
}


pub trait CraftRepo {
    fn find_mods(&self, search: &ModsQuery) -> Vec<&ModItem>;
    fn get_item_classes(&self) -> Vec<String>;
}


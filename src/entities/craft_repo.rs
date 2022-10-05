use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ModItem {
    pub required_level: u64,
    pub weight: u32,
    pub representation: String,
    pub mod_key: String,
}

pub struct ModsQuery {
    pub string_query: String,
    pub item_level: u64,
    pub item_class: String,
    pub selected_mods: Vec<ModItem>,
}

pub trait CraftRepo {
    fn find_mods(&self, search: &ModsQuery) -> Vec<ModItem>;
    fn get_item_classes(&self) -> Vec<String>;
}

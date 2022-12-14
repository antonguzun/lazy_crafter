use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ModItem {
    pub required_level: u64,
    pub weight: u32,
    pub generation_type: String,
    pub representation: String,
    pub mod_key: String,
}

#[derive(Debug, Clone)]
pub struct ItemBase {
    pub required_level: u64,
    pub name: String,
}

pub struct ModsQuery {
    pub string_query: String,
    pub item_level: u64,
    pub item_base: String,
    pub selected_mods: Vec<ModItem>,
}

pub trait CraftRepo {
    fn find_mods(&self, search: &ModsQuery) -> Vec<ModItem>;
    fn get_item_classes(&self) -> Vec<String>;
    fn get_item_bases(&self, item_class: &str) -> Vec<ItemBase>;
    fn get_item_class_by_item_name(&self) -> HashMap<String, String>;
    fn item_class_if_exists(&self, item_class: &str) -> bool;
    fn string_to_item_base(&self, item_class: &str, item_name: &str) -> Result<String, String>;
    fn string_to_mod(
        &self,
        item_class: &str,
        item_name: &str,
        mod_name: &str,
    ) -> Result<String, String>;
}

pub struct Data {
    pub mods_table: Vec<ModItem>,
    pub item_classes: Vec<String>,
    pub item_bases: Vec<ItemBase>,
    pub item_class_by_base_name: HashMap<String, String>,
    pub estimation: Option<Result<Estimation, String>>,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            mods_table: Vec::new(),
            item_classes: Vec::new(),
            item_bases: Vec::new(),
            item_class_by_base_name: HashMap::new(),
            estimation: None,
        }
    }
}
#[derive(Debug, Clone)]
pub struct Message {
    pub text: String,
    pub created_at: i64,
}

#[derive(Debug)]
pub struct UiStates {
    pub filter_string: String,
    pub item_string: String,
    pub item_level: String,
    pub max_autocraft_tries: String,
    pub selected: Vec<ModItem>,
    pub selected_item_class_as_filter: String,
    pub selected_item_base_as_filter: String,
    pub selected_item_level_as_filter: u64,
    pub selected_max_autocraft_tries: u64,
    pub messages: Vec<Message>,
}

impl Default for UiStates {
    fn default() -> Self {
        Self {
            filter_string: "".to_string(),
            item_string: "".to_string(),
            item_level: "100".to_string(),
            max_autocraft_tries: "5".to_string(),

            selected: vec![],
            selected_item_class_as_filter: "Helmet".to_string(),
            selected_item_base_as_filter: "Iron Hat".to_string(),
            selected_item_level_as_filter: 100,
            selected_max_autocraft_tries: 5,
            messages: vec![],
        }
    }
}
#[derive(Debug)]
pub struct Estimation {
    pub probability: f64,
}

#[derive(PartialEq)]
pub enum UiEvents {
    Started,
    ChangeModFilter,
    ChangeItemBase,
    AddToSelectedMods,
    CleanSelectedMods,
    InsertionItemData,
}

#[derive(PartialEq)]
pub enum BackEvents {
    Error(String),
}

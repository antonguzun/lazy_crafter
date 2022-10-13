#[derive(Debug, Clone)]
pub struct ModItem {
    pub required_level: u64,
    pub weight: u32,
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
}

pub struct Data {
    pub mods_table: Vec<ModItem>,
}
impl Default for Data {
    fn default() -> Self {
        Self {
            mods_table: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct UiStates {
    pub filter_string: String,
    pub item_string: String,
    pub item_level: String,
    pub selected: Vec<ModItem>,
    pub selected_item_class_as_filter: String,
    pub selected_item_base_as_filter: String,
    pub selected_item_level_as_filter: u64,
}

impl Default for UiStates {
    fn default() -> Self {
        Self {
            filter_string: "".to_string(),
            item_string: "".to_string(),
            item_level: "100".to_string(),

            selected: vec![],
            selected_item_class_as_filter: "Helmet".to_string(),
            selected_item_base_as_filter: "Iron Hat".to_string(),
            selected_item_level_as_filter: 100,
        }
    }
}
pub enum UiEvents {
    Started,
    ChangeModFilter,
    ChangeItemBase,
    AddToSelectedMods,
    CleanSelectedMods,
}

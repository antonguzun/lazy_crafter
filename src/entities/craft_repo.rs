use std::collections::{HashMap, HashSet};

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

#[derive(Debug, Clone)]
pub struct ModsQuery {
    pub string_query: String,
    pub item_level: u64,
    pub item_base: String,
    pub selected_mods: Vec<ModItem>,
    /// How the already selected mods are combined. Decides whether mods
    /// conflicting with the selection are still offered in the list, see
    /// `find_mods`.
    pub match_mode: ModsMatchMode,
}

#[derive(Debug, Clone)]
pub struct Query {
    pub string_query: String,
    pub item_level: u64,
    pub item_base: String,
    pub selected_mod_keys: Vec<String>,
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
    fn get_weight_of_target_and_better_mods(
        &self,
        query: &ModsQuery,
        target_mod_key: String,
    ) -> u32;
    fn get_affected_weight_of_target_mod(&self, query: &ModsQuery) -> u32;
    fn get_subset_of_mods(&self, mod_id: &str, item_base: &str) -> Result<HashSet<String>, String>;
    fn representation_by_mod_id(&self, mod_id: &str) -> String;
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

/// Which game's dataset the app is working with. Switched at runtime from the
/// UI; the db thread reloads the repository when it changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameVersion {
    Poe1,
    Poe2,
}

impl GameVersion {
    pub fn label(&self) -> &'static str {
        match self {
            GameVersion::Poe1 => "PoE 1",
            GameVersion::Poe2 => "PoE 2",
        }
    }
}

/// How selected mods are combined when checking a crafted item:
/// `All` — every selected mod must be present (AND),
/// `Any` — at least one selected mod is enough (OR).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModsMatchMode {
    All,
    Any,
}

impl ModsMatchMode {
    pub fn label(&self) -> &'static str {
        match self {
            ModsMatchMode::All => "AND",
            ModsMatchMode::Any => "OR",
        }
    }
}

/// A zero threshold preserves tier-only matching. The OR exception counts
/// distinct crafted mods, not selected alternatives that match the same mod.
#[derive(Debug, Clone, Copy, Default)]
pub struct RollRequirements {
    pub min_percent: u8,
    /// Zero disables the exception; it never applies in AND mode.
    pub ignore_roll_at_or_count: usize,
}

#[derive(Debug)]
pub struct UiStates {
    pub filter_string: String,
    pub item_string: String,
    pub item_level: String,
    pub max_autocraft_tries: String,
    pub selected: Vec<ModItem>,
    pub selected_mods_match_mode: ModsMatchMode,
    pub roll_requirements: RollRequirements,
    pub selected_item_class_as_filter: String,
    pub selected_item_base_as_filter: String,
    pub selected_item_level_as_filter: u64,
    pub selected_max_autocraft_tries: u64,
    pub selected_game_version: GameVersion,
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
            selected_mods_match_mode: ModsMatchMode::Any,
            roll_requirements: RollRequirements::default(),
            selected_item_class_as_filter: "Helmet".to_string(),
            selected_item_base_as_filter: "Iron Hat".to_string(),
            selected_item_level_as_filter: 100,
            selected_max_autocraft_tries: 5,
            selected_game_version: GameVersion::Poe2,
            messages: vec![],
        }
    }
}
#[derive(Debug, PartialEq)]
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
    ChangeGameVersion,
}

#[derive(PartialEq)]
pub enum BackEvents {
    Error(String),
}

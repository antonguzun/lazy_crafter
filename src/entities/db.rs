use eframe::epaint::ahash::HashSet;

use crate::entities::{mods::Mod, translations::StatTranslation};
use std::collections::HashMap;


pub struct ModItem {
    pub item_level: u64,
    pub weight: u32,
    // max: i32,
    // min: i32,
}

pub struct LocalDB {
    pub translations: HashMap<String, StatTranslation>,
    pub mods: HashMap<String, Mod>,
    pub item_tags: HashSet<String>,
    pub search_map: HashMap<String, HashMap<String, ModItem>>,
}

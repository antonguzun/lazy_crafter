use crate::entities::craft_repo::{CraftRepo, ModItem, ModsQuery};
use crate::storage::files::{
    mods::{ItemBase, Mod, SpawnWeight, Stat},
    translations::StatTranslation,
};
use eframe::glow::HasContext;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;

fn load_from_json<T>(path: &str) -> Vec<T>
where
    T: Default + serde::de::DeserializeOwned,
{
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    serde_json::from_str(&contents).unwrap()
}

fn json_to_hashmap<T>(path: &str) -> HashMap<String, T>
where
    T: Default + serde::de::DeserializeOwned,
{
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    serde_json::from_str(&contents).unwrap()
}

pub struct LocalDB {
    pub translations: HashMap<String, StatTranslation>,
    pub mods: HashMap<String, Mod>,
    // pub item_tags: HashSet<String>,
    pub base_items_by_name: HashMap<String, ItemBase>,
    pub item_classes: HashSet<String>,
}

pub struct FileRepo {
    db: LocalDB,
}

impl FileRepo {
    pub fn new() -> Result<FileRepo, String> {
        let translations: Vec<StatTranslation> = load_from_json("data/stat_translations.min.json");
        let translations: HashMap<String, StatTranslation> = translations
            .into_iter()
            .map(|t| (t.ids[0].clone(), t))
            .collect();
        let mods: HashMap<String, Mod> = json_to_hashmap("data/mods.min.json");
        let t: HashMap<String, ItemBase> = json_to_hashmap("data/base_items.min.json");
        let base_items_by_name: HashMap<String, ItemBase> =
            t.iter().map(|(k, v)| (v.name.clone(), v.clone())).collect();
        let item_classes = HashSet::from_iter(t.iter().map(|(k, v)| v.item_class.clone()));
        Ok(Self {
            db: LocalDB {
                translations,
                mods,
                // item_tags,
                base_items_by_name,
                item_classes,
            },
        })
    }
}

impl CraftRepo for FileRepo {
    fn find_mods(&self, search: &ModsQuery) -> std::vec::Vec<&ModItem> {
        // let filter = search.string_query.trim().to_lowercase();
        // let filters: Vec<&str> = filter.split(' ').collect();
        // let mut v1: Vec<&Mod> = vec![];
        // let mut v2: Vec<&Mod> = vec![];
        // let sub_db = self.db.search_map.get(search.item_class.as_str()).unwrap();
        // for (k, m) in sub_db.iter() {
        //     let verbose_str = m.representation.to_lowercase();
        //     if verbose_str.contains(&filter) {
        //         v1.push(&m);
        //     } else if filters.iter().all(|f| verbose_str.contains(&*f)) {
        //         v2.push(&m);
        //     }
        // }
        // v1.extend(v2);
        // v1
        vec![]
    }

    fn get_item_classes(&self) -> Vec<String> {
        self.db.item_classes.iter().map(|s| s.clone()).collect()
    }
}

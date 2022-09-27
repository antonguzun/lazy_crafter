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
    pub translations_by_stat_id: HashMap<String, StatTranslation>,
    pub mods: HashMap<String, Mod>,
    // pub item_tags_by_item_class: HashMap<String, HashSet<String>>,
    pub base_items_by_name: HashMap<String, ItemBase>,
    pub item_classes: HashSet<String>,
    pub mod_id_by_tags: HashMap<String, Vec<String>>,
}

pub struct FileRepo {
    db: LocalDB,
}

impl FileRepo {
    pub fn new() -> Result<FileRepo, String> {
        let translations: Vec<StatTranslation> = load_from_json("data/stat_translations.min.json");
        let translations_by_stat_id: HashMap<String, StatTranslation> = translations
            .into_iter()
            .map(|t| (t.ids[0].clone(), t))
            .collect();
        let mods: HashMap<String, Mod> = json_to_hashmap("data/mods.min.json");
        let raw_base_items: HashMap<String, ItemBase> = json_to_hashmap("data/base_items.min.json");
        let base_items_by_name: HashMap<String, ItemBase> = raw_base_items
            .iter()
            .map(|(k, v)| (v.name.clone(), v.clone()))
            .collect();
        let item_classes =
            HashSet::from_iter(raw_base_items.iter().map(|(k, v)| v.item_class.clone()));

        let all_tags: HashSet<String> = HashSet::from_iter(
            raw_base_items
                .values()
                .filter(|b| b.domain == "item" || b.release_state == "released")
                .flat_map(|b| b.tags.clone()),
        );
        let mut mod_id_by_tags: HashMap<String, Vec<String>> = HashMap::new();
        mods.iter().for_each(|(mod_id, m)| {
            m.spawn_weights.iter().for_each(|sw| {
                if all_tags.contains(&sw.tag) && sw.weight > 0 {
                    mod_id_by_tags.insert(sw.tag.to_string(), vec![mod_id.clone()]);
                }
            })
        });
        print!("tags: {:?}", mod_id_by_tags.keys());
        Ok(Self {
            db: LocalDB {
                translations_by_stat_id,
                mods,
                // item_tags_by_item_class,
                base_items_by_name,
                item_classes,
                mod_id_by_tags,
            },
        })
    }
}

impl CraftRepo for FileRepo {
    fn find_mods(&self, search: &ModsQuery) -> std::vec::Vec<&ModItem> {
        let mut tags: std::vec::Vec<String> = vec![];
        for i in self.db.base_items_by_name.values() {
            if i.item_class == search.item_class {
                tags = i.tags.clone();
                break;
            }
        }
        println!("tags for {}: {:?}", search.item_class, tags);
        // let filter = self.db..trim().to_lowercase();
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

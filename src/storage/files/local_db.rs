use crate::entities::craft_repo::{CraftRepo, ModItem, ModsQuery};
use crate::storage::files::{
    mods::{ItemBase, Mod},
    translations::StatTranslation,
};

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
            .map(|(_k, v)| (v.name.clone(), v.clone()))
            .collect();
        let item_classes =
            HashSet::from_iter(raw_base_items.iter().map(|(_k, v)| v.item_class.clone()));

        let all_tags: HashSet<String> = HashSet::from_iter(
            raw_base_items
                .values()
                .filter(|b| b.domain == "item" && b.release_state == "released")
                .flat_map(|b| b.tags.clone()),
        );
        let mut mod_id_by_tags: HashMap<String, Vec<String>> = HashMap::new();
        mods.iter().for_each(|(mod_id, m)| {
            m.spawn_weights.iter().for_each(|sw| {
                if all_tags.contains(&sw.tag) && sw.weight > 0 {
                    match mod_id_by_tags.get_mut(&sw.tag) {
                        Some(v) => v.push(mod_id.clone()),
                        None => {
                            mod_id_by_tags.insert(sw.tag.clone(), vec![mod_id.clone()]);
                        }
                    }
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
    /// find_mods
    ///     includes:
    ///         tags by selected item class
    ///         domain by selected item class
    ///         generation_type: "prefix" or "suffix"
    ///     excludes:
    ///         groups by selected mods
    ///     order by mod_key filtered by contains
    fn find_mods(&self, search: &ModsQuery) -> std::vec::Vec<ModItem> {
        let item = self
            .db
            .base_items_by_name
            .values()
            .find(|i| i.item_class == search.item_class)
            .unwrap();
        println!("tags for {}: {:?}", search.item_class, item.tags);
        let mut mod_ids: HashSet<String> = HashSet::new();
        for t in &item.tags {
            let ms = self.db.mod_id_by_tags.get(t);
            if ms.is_some() {
                mod_ids.extend(ms.unwrap().clone());
            }
        }
        let selected_groups: HashSet<std::string::String> = HashSet::from_iter(
            search
                .selected_mods
                .iter()
                .map(|m| self.db.mods.get(&m.mod_key).unwrap())
                .flat_map(|m| m.groups.clone()),
        );
        let target_gen_types = ["suffix", "prefix"];
        let mut res = vec![];
        for m_id in mod_ids {
            let m = self.db.mods.get(&m_id).unwrap();
            if m.stats.is_empty()
                || m.domain != item.domain
                || !target_gen_types.contains(&m.generation_type.as_str())
                || m.groups.iter().any(|g| selected_groups.contains(g))
            {
                continue;
            }
            let representations =
                m.stats.iter().map(
                    |s| match self.db.translations_by_stat_id.get(&m.stats[0].id) {
                        Some(t) => t.get_eng_representation_string(&m.stats[0]),
                        None => m.stats[0].id.clone(),
                    },
                );
            let mut representation = String::new();
            for r in representations {
                representation.push_str(&r);
                representation.push_str("\n");
            }
            let mod_item = ModItem {
                required_level: m.required_level,
                weight: m
                    .spawn_weights
                    .iter()
                    .filter(|sw| sw.weight > 0 && item.tags.contains(&sw.tag))
                    .next()
                    .unwrap()
                    .weight,
                representation: representation,
                mod_key: m_id.clone(),
            };
            res.push(mod_item);
        }
        res.sort_by(|a, b| a.mod_key.to_lowercase().cmp(&b.mod_key.to_lowercase()));

        let filter = search.string_query.trim().to_lowercase();
        let filters: Vec<&str> = filter.split(' ').collect();
        let mut v1 = vec![];
        let mut v2 = vec![];
        for m in res.iter() {
            let verbose_str = m.representation.to_lowercase();
            if verbose_str.contains(&filter) {
                v1.push(m.clone());
            } else if filters.iter().all(|f| verbose_str.contains(&*f)) {
                v2.push(m.clone());
            }
        }
        v1.extend(v2);
        v1
    }

    fn get_item_classes(&self) -> Vec<String> {
        self.db.item_classes.iter().map(|s| s.clone()).collect()
    }
}

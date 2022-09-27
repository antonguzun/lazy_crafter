use crate::entities::craft_repo::{CraftRepo, ItemClass, ModItem, ModsQuery};
use crate::storage::files::{mods::Mod, translations::StatTranslation};
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

fn json_to_hashmap(path: &str) -> HashMap<String, Mod> {
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    serde_json::from_str(&contents).unwrap()
}

pub struct ItemBase {
    pub name: String,
    pub item_class: String,
    pub tags: Vec<String>,
}

pub struct Mod {
    pub name: String,
    pub item_class: String,
    pub tags: Vec<String>,
    pub stats: Vec<Stat>,
}
pub struct LocalDB {
    pub translations: HashMap<String, StatTranslation>,
    pub mods: HashMap<String, Mod>,
    pub item_tags: HashSet<String>,
    pub search_map: HashMap<String, HashMap<String, ModItem>>,
    // pub item_base_by_name: HashMap<String, ItemBase>,
    // pub item_classes: HashSet<String>,
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
        let item_tags = HashSet::from_iter(["helmet".to_string()]);

        let mut search_map = HashMap::default();

        let mut helmet_mods: HashMap<String, ModItem> = HashMap::default();

        for (k, v) in mods.iter() {
            if v.domain == "item" {
                for sw in v.spawn_weights.iter() {
                    if (sw.tag == "helmet".to_string() || sw.tag == "default".to_string())
                        && sw.weight > 0
                    {
                        v.stats.iter().for_each(|stat| {
                            helmet_mods.insert(
                                stat.id.clone(),
                                ModItem {
                                    item_level: v.required_level,
                                    weight: sw.weight,
                                    representation: match translations.get(k) {
                                        Some(v) => v.English[0].get_representation_string(),
                                        None => stat.id.clone(),
                                    },
                                    mod_key: k.clone(),
                                    // max: sw.max,
                                    // min: sw.min,
                                },
                            );
                        });
                    }
                }
            }
        }
        search_map.insert("helmet".to_string(), helmet_mods);

        Ok(Self {
            db: LocalDB {
                translations,
                mods,
                item_tags,
                search_map,
            },
        })
    }
}

impl CraftRepo for FileRepo {
    fn find_mods(&self, search: &ModsQuery) -> std::vec::Vec<&ModItem> {
        let filter = search.string_query.trim().to_lowercase();
        let filters: Vec<&str> = filter.split(' ').collect();
        let mut v1: Vec<&ModItem> = vec![];
        let mut v2: Vec<&ModItem> = vec![];
        let sub_db = self.db.search_map.get(search.item_class.as_str()).unwrap();
        for (k, m) in sub_db.iter() {
            let verbose_str = m.representation.to_lowercase();
            if verbose_str.contains(&filter) {
                v1.push(&m);
            } else if filters.iter().all(|f| verbose_str.contains(&*f)) {
                v2.push(&m);
            }
        }
        v1.extend(v2);
        v1
    }
}

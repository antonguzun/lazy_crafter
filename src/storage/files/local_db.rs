use crate::entities::{
    db::{LocalDB, ModItem},
    mods::Mod,
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

fn json_to_hashmap(path: &str) -> HashMap<String, Mod> {
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    serde_json::from_str(&contents).unwrap()
}

pub struct FileRepo {}

impl FileRepo {
    pub fn new() -> Result<LocalDB, String> {
        let translations: Vec<StatTranslation> = load_from_json("data/stat_translations.min.json");
        let translations = translations
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

        Ok(LocalDB {
            translations,
            mods,
            item_tags,
            search_map,
        })
    }
}

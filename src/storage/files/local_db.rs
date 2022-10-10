use crate::entities::craft_repo::{CraftRepo, ModItem, ModsQuery};

use crate::storage::files::{
    mods::{ItemBase, Mod, Stat},
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
        let mut translations_by_stat_id: HashMap<String, StatTranslation> = HashMap::new();
        for t in translations {
            for id in &t.ids {
                translations_by_stat_id.insert(id.clone(), t.clone());
            }
        }

        let mods: HashMap<String, Mod> = json_to_hashmap("data/mods.min.json");
        let raw_base_items: HashMap<String, ItemBase> = json_to_hashmap("data/base_items.min.json");
        let base_items_by_name: HashMap<String, ItemBase> = raw_base_items
            .iter()
            .map(|(_k, v)| (v.name.clone(), v.clone()))
            .collect();
        let item_classes = HashSet::from_iter(
            raw_base_items
                .iter()
                .filter(|(_k, v)| v.domain == "item")
                .map(|(_k, v)| v.item_class.clone()),
        );

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

impl FileRepo {
    fn get_stats_representation(&self, t: StatTranslation, stats: Vec<Stat>) -> Result<String, ()> {
        let mut stats_positions_by_id: HashMap<String, usize> = HashMap::default();

        for (pos, t_id) in t.ids.iter().enumerate() {
            for s in &stats {
                let id = t_id.clone();
                if s.id == id {
                    stats_positions_by_id.insert(id, pos);
                }
            }
        }
        for i in t.English.iter() {
            let mut cond_passed = true;
            for s in &stats {
                let stat_position = stats_positions_by_id.get(&s.id).unwrap();
                let stat_max = s.max.unwrap();
                let stat_min = s.min.unwrap();
                let condition = &i.condition[stat_position.clone()];
                if condition.negated == Some(true) {
                    return Ok(i.string.clone());
                }
                match condition.min {
                    Some(min) => {
                        if stat_min < min {
                            cond_passed = false;
                        }
                    }
                    None => (),
                }
                match condition.max {
                    Some(max) => {
                        if stat_max > max {
                            cond_passed = false;
                        }
                    }
                    None => (),
                }
            }
            if cond_passed {
                let mut repr = i.string.clone();
                for s in stats {
                    let stat_position = stats_positions_by_id.get(&s.id).unwrap().clone();
                    let stat_max = s.max.unwrap();
                    let stat_min = s.min.unwrap();

                    let to_str = match stat_max == stat_min {
                        true => format!("{}", stat_max),
                        false => format!("({}-{})", stat_min, stat_max),
                    };
                    let v = [
                        '{',
                        std::char::from_digit(stat_position.try_into().unwrap(), 10).unwrap(),
                        '}',
                    ];
                    let from = String::from_iter(v);
                    repr = repr.replace(&from, &to_str);
                }
                return Ok(repr);
            }
        }
        println!("No english representation found for stats {:?}", stats);
        Err(())
    }

    fn get_mods_representation(&self, m: &Mod) -> Result<std::string::String, ()> {
        type Group = Vec<Stat>;
        let mut kk: HashMap<StatTranslation, Group> = HashMap::default();
        for s in m.stats.iter() {
            let t = match self.db.translations_by_stat_id.get(&s.id) {
                Some(t) => t.clone(),
                None => return Err(()),
            };
            let g = kk.get(&t);
            if g.is_some() {
                let mut gg = g.unwrap().clone();
                gg.push(s.clone());
                kk.insert(t, gg);
            } else {
                kk.insert(t, vec![s.clone()]);
            }
        }
        let skip_repr = String::from("");
        let mut reprs = Vec::new();
        for (t, g) in kk {
            let r = match self.get_stats_representation(t, g) {
                Ok(s) => s,
                Ok(skip_repr) => {
                    continue;
                }
                Err(_) => return Err(()),
            };
            if &r == "" {
                continue;
            }
            reprs.push(r.to_string());
        }
        reprs.sort();
        Ok(reprs.join("\n").to_string())
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
            let mod_item = ModItem {
                required_level: m.required_level,
                weight: m
                    .spawn_weights
                    .iter()
                    .filter(|sw| sw.weight > 0 && item.tags.contains(&sw.tag))
                    .next()
                    .unwrap()
                    .weight,
                representation: self
                    .get_mods_representation(m)
                    .unwrap_or_else(|_| format!("representation_err: {}", &m_id)),
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

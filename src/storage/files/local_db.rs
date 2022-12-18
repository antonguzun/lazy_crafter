use crate::entities::craft_repo::{CraftRepo, ItemBase, ModItem, ModsQuery};
use crate::storage::files::representation::handle_stat_value;
use crate::storage::files::schemas::{ItemBaseRich, Mod, Stat, StatTranslation};
use log::{debug, error};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::iter::zip;

const LOG_TARGET: &str = "file_db";

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
    pub base_items_by_name: HashMap<String, ItemBaseRich>,
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
        let raw_base_items: HashMap<String, ItemBaseRich> =
            json_to_hashmap("data/base_items.min.json");
        let base_items_by_name: HashMap<String, ItemBaseRich> = raw_base_items
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
        debug!(target: LOG_TARGET, "tags: {:?}", mod_id_by_tags.keys());
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
        for i in t.English.iter().rev() {
            // reverse important else representation calculation is wrong
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
                    let index_handler: String = match i.index_handlers[stat_position.clone()].get(0)
                    {
                        Some(ih) => ih.to_string(),
                        None => String::from("pass"),
                    };

                    let to_str = match stat_max == stat_min {
                        true => format!("{}", handle_stat_value(&index_handler, stat_max)),
                        false => format!(
                            "({}-{})",
                            handle_stat_value(&index_handler, stat_min),
                            handle_stat_value(&index_handler, stat_max)
                        ),
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
        error!(
            target: LOG_TARGET,
            "No english representation found for stats {:?}", stats
        );
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
                Ok(s) if s == skip_repr => {
                    continue;
                }
                Ok(s) => s,
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

    fn create_mod_items(
        &self,
        mod_ids: &HashSet<String>,
        item: &ItemBaseRich,
        selected_groups: HashSet<std::string::String>,
        max_item_level: u64,
    ) -> Vec<ModItem> {
        let target_gen_types = ["suffix", "prefix"];
        let mut res = vec![];
        for m_id in mod_ids {
            let m = self.db.mods.get(m_id).unwrap();

            if m.required_level > max_item_level
                || m.stats.is_empty()
                || m.domain != item.domain
                || !target_gen_types.contains(&m.generation_type.as_str())
                || m.groups.iter().any(|g| selected_groups.contains(g))
            {
                continue;
            }
            let mod_item = ModItem {
                required_level: m.required_level,
                generation_type: m.generation_type.clone(),
                weight: m
                    .spawn_weights
                    .iter()
                    .filter(|sw| sw.weight > 0 && item.tags.contains(&sw.tag))
                    .next()
                    .unwrap()
                    .weight,
                representation: self
                    .get_mods_representation(m)
                    .unwrap_or_else(|_| format!("representation_err: {}", m_id)),
                mod_key: m_id.clone(),
            };
            res.push(mod_item);
        }
        res
    }

    fn get_weight_of_target_and_better_mods(
        &self,
        mod_ids: &HashSet<String>,
        item: &ItemBaseRich,
        target_mod_key: &str,
        max_item_level: u64,
    ) -> u32 {
        let target_gen_types = ["suffix", "prefix"];
        let mut res = vec![];
        let target_mod = self.db.mods.get(target_mod_key).unwrap();
        for m_id in mod_ids {
            let m = self.db.mods.get(m_id).unwrap();
            if m.type_field != target_mod.type_field
                || m.required_level > max_item_level
                || m.stats.is_empty()
                || m.domain != item.domain
                || m.domain != target_mod.domain
                || !target_gen_types.contains(&m.generation_type.as_str())
            {
                continue;
            }

            let mut filter_by_stats = false;
            for (ts, ms) in target_mod.stats.iter().zip(m.stats.iter()) {
                if ts.id != ms.id || ts.min >= ms.min || ts.max >= ms.max {
                    filter_by_stats = true;
                    break;
                }
            }
            if filter_by_stats {
                continue;
            }
            let weight = m // todo! move to trait
                .spawn_weights
                .iter()
                .find_map(|sw| {
                    if sw.weight > 0 && item.tags.contains(&sw.tag) {
                        Some(sw.weight)
                    } else {
                        None
                    }
                })
                .unwrap();
            res.push(weight)
        }
        res.iter().sum()
    }

    fn get_affected_weight_of_target_mod(
        &self,
        mod_ids: &HashSet<String>,
        item: &ItemBaseRich,
        selected_groups: HashSet<std::string::String>,
        max_item_level: u64,
        affixes_types: Vec<String>,
    ) -> u32 {
        let target_gen_types = affixes_types
            .iter()
            .map(|v| v.as_str())
            .collect::<Vec<&str>>();
        let mut res = vec![];
        for m_id in mod_ids {
            let m = self.db.mods.get(m_id).unwrap();
            if m.required_level > max_item_level
                || m.stats.is_empty()
                || m.domain != item.domain
                || !target_gen_types.contains(&m.generation_type.as_str())
                || !m.groups.iter().any(|g| selected_groups.contains(g))
            {
                continue;
            }
            let weight = m // todo! move to trait
                .spawn_weights
                .iter()
                .find_map(|sw| {
                    if sw.weight > 0 && item.tags.contains(&sw.tag) {
                        Some(sw.weight)
                    } else {
                        None
                    }
                })
                .unwrap();
            res.push(weight)
        }
        res.iter().sum()
    }
}

fn filter_mods_by_text(mods: &mut Vec<ModItem>, query_string: String) -> Vec<ModItem> {
    let filter = query_string.trim().to_lowercase();
    let filters: Vec<&str> = filter.split(' ').collect();
    let (mut v1, mut v2) = (vec![], vec![]);
    for m in mods.iter() {
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
            .find(|i| i.name == search.item_base)
            .unwrap();
        debug!(
            target: LOG_TARGET,
            "tags for {}: {:?}", search.item_base, item.tags
        );
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

        let mut res = self.create_mod_items(&mod_ids, item, selected_groups, search.item_level);
        res.sort_by(|a, b| a.mod_key.to_lowercase().cmp(&b.mod_key.to_lowercase()));
        filter_mods_by_text(&mut res, search.string_query.clone())
    }

    fn get_item_classes(&self) -> Vec<String> {
        let mut r: Vec<String> = self.db.item_classes.iter().map(|s| s.clone()).collect();
        r.sort();
        r
    }

    fn get_item_bases(&self, item_class: &str) -> Vec<ItemBase> {
        let mut r: Vec<ItemBase> = self
            .db
            .base_items_by_name
            .iter()
            .filter(|(_, bi)| bi.domain == "item" && bi.item_class == item_class.to_string())
            .map(|(s, bi)| ItemBase {
                name: s.to_string(),
                required_level: match bi.requirements {
                    Some(ref r) => r.level,
                    None => 100,
                },
            })
            .collect();
        r.sort_by(|a, b| a.name.cmp(&b.name));
        r
    }

    fn get_item_class_by_item_name(&self) -> HashMap<String, String> {
        HashMap::from_iter(
            self.db
                .base_items_by_name
                .iter()
                .filter(|(_, bi)| bi.domain == "item")
                .map(|(s, bi)| (s.clone(), bi.item_class.clone())),
        )
    }

    fn item_class_if_exists(&self, item_class: &str) -> bool {
        self.db.item_classes.contains(item_class)
    }

    fn string_to_item_base(&self, item_class: &str, item_name: &str) -> Result<String, String> {
        self.get_item_bases(item_class)
            .into_iter()
            .filter(|i| item_name.contains(&i.name))
            .map(|i| i.name)
            .next()
            .ok_or(format!("{} not found in {}", item_name, item_class))
    }

    // parse raw mod string to mod key
    // provided raw mod string and each available mod for item_base to common template
    // added filtration by stat min/max to avoid wrong matches
    fn string_to_mod(
        &self,
        item_class: &str,
        item_name: &str,
        mod_name: &str,
    ) -> Result<String, String> {
        let query = ModsQuery {
            item_base: item_name.to_string(),
            item_level: 100,
            string_query: "".to_string(),
            selected_mods: vec![],
        };
        let mods = self.find_mods(&query);

        use regex::Regex;
        let values: Vec<i32> = Regex::new(r"\d+")
            .unwrap()
            .find_iter(&mod_name.replace("\r", "").replace("Gramts", "Gain"))
            .map(|m| m.as_str().parse::<i32>().unwrap())
            .collect();
        // add "\+?" to pattern because our mods representation doesn't contain it (look stat_translations.json field "format")
        let mod_template = Regex::new(r"\+?\d+").unwrap().replace_all(mod_name, "#");

        let res = mods
            .into_iter()
            // .filter(|m| m.representation.starts_with("Reflect")) // debug, REMOVE!
            .find_map(|m| {
                // add (\d+-\d+\) because its how we represent range values
                // but don't care about "\+?" because we already skipped "format" in representation
                let re = Regex::new(r"\(\d+-\d+\)|\d+")
                    .unwrap()
                    .replace_all(&m.representation, "#");
                match re == mod_template {
                    true => {
                        let source_mod = self.db.mods.get(&m.mod_key).unwrap();
                        let source_mod_stats_count: usize = source_mod
                            .stats
                            .iter()
                            .filter(|s| s.id != "dummy_stat_display_nothing")
                            .count();
                        if values.len() != source_mod_stats_count {
                            return None;
                        }
                        let mut values_correct = true;
                        zip(&values, &source_mod.stats).for_each(|(v, s)| {
                            match s.min {
                                Some(min) => {
                                    if i64::from(*v) < min {
                                        values_correct = false;
                                    }
                                }
                                None => {}
                            }
                            match s.max {
                                Some(max) => {
                                    if max < i64::from(*v) {
                                        values_correct = false;
                                    }
                                }
                                None => {}
                            }
                        });
                        if values_correct {
                            return Some(m);
                        }
                        let mut values_correct = true;
                        let values: Vec<i32> = values.clone().into_iter().rev().collect();
                        zip(values, &source_mod.stats).for_each(|(v, s)| {
                            match s.min {
                                Some(min) => {
                                    if i64::from(v) < min {
                                        values_correct = false;
                                    }
                                }
                                None => {}
                            }
                            match s.max {
                                Some(max) => {
                                    if max < i64::from(v) {
                                        values_correct = false;
                                    }
                                }
                                None => {}
                            }
                        });
                        if !values_correct {
                            return None;
                        }
                        Some(m)
                    }
                    false => None,
                }
            })
            .ok_or("Can't find mod".to_string())?;
        Ok(res.mod_key)
    }

    fn get_weight_of_target_and_better_mods(
        &self,
        query: &ModsQuery,
        target_mod_key: String,
    ) -> u32 {
        let item = self
            .db
            .base_items_by_name
            .values()
            .find(|i| i.name == query.item_base)
            .unwrap();
        debug!(
            target: LOG_TARGET,
            "tags for {}: {:?}", query.item_base, item.tags
        );
        let mut mod_ids: HashSet<String> = HashSet::new();
        for t in &item.tags {
            let ms = self.db.mod_id_by_tags.get(t);
            if ms.is_some() {
                mod_ids.extend(ms.unwrap().clone());
            }
        }

        self.get_weight_of_target_and_better_mods(&mod_ids, item, &target_mod_key, query.item_level)
    }

    fn get_affected_weight_of_target_mod(&self, query: &ModsQuery) -> u32 {
        let item = self
            .db
            .base_items_by_name
            .values()
            .find(|i| i.name == query.item_base)
            .unwrap();
        debug!(
            target: LOG_TARGET,
            "tags for {}: {:?}", query.item_base, item.tags
        );
        let mut mod_ids: HashSet<String> = HashSet::new();
        for t in &item.tags {
            let ms = self.db.mod_id_by_tags.get(t);
            if ms.is_some() {
                mod_ids.extend(ms.unwrap().clone());
            }
        }
        let selected_groups: HashSet<std::string::String> = HashSet::from_iter(
            query
                .selected_mods
                .iter()
                .map(|m| self.db.mods.get(&m.mod_key).unwrap())
                .flat_map(|m| m.groups.clone()),
        );
        let affixes_types = query
            .selected_mods
            .iter()
            .map(|m| m.generation_type.clone())
            .collect();
        self.get_affected_weight_of_target_mod(
            &mod_ids,
            item,
            selected_groups,
            query.item_level,
            affixes_types,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};

    #[fixture]
    fn repo() -> FileRepo {
        FileRepo::new().unwrap()
    }

    #[rstest]
    #[case("TwoHandChanceToFreeze2".to_string(), "25% chance to Freeze".to_string())]
    #[case("AttackerTakesDamage2".to_string(), "Reflects (5-10) Physical Damage to Melee Attackers".to_string())]
    #[case("LocalIncreasedPhysicalDamagePercent1".to_string(), "(40-49)% increased Physical Damage".to_string())]
    #[case("LifeRegeneration7".to_string(), "Regenerate (48-64) Life per second".to_string())]
    #[case("GainLifeOnBlock6_".to_string(), "(86-100) Life gained when you Block".to_string())]
    fn test_repr(repo: FileRepo, #[case] mod_id: String, #[case] expected: String) {
        let mod_item = repo.db.mods.get(&mod_id).unwrap();
        let repr = repo.get_mods_representation(mod_item).unwrap();
        assert_eq!(repr, expected);
    }
}

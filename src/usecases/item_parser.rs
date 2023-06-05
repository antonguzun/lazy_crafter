use log::{debug, warn};
use regex::Regex;

use crate::entities::craft_repo::CraftRepo;

#[derive(Debug, PartialEq)]
pub struct ParsedItem {
    pub item_class: String,
    pub item_base_name: String,
    pub item_name: String,
    pub mods: Vec<String>,
    pub raw_mods: Vec<String>,
}

fn fetch_item_class<'a>(craft_repo: &impl CraftRepo, raw_item: &'a str) -> Result<&'a str, String> {
    let re = Regex::new(r"Item Class: (.*)\n").expect("regexp error during item class fetching");
    let raw_item_class = re
        .captures(raw_item)
        .ok_or("No item class matches in string".to_string())?
        .get(1)
        .ok_or("No item class in string found".to_string())?
        .as_str()
        .trim();

    if craft_repo.item_class_if_exists(raw_item_class[..raw_item_class.len() - 1].trim()) {
        return Ok(raw_item_class[..raw_item_class.len() - 1].trim());
    } else if craft_repo.item_class_if_exists(raw_item_class.trim()) {
        return Ok(raw_item_class.trim());
    } else {
        return Err(format!(
            "Item class not found in db: {}",
            &raw_item_class.trim()
        ));
    }
}

#[derive(Debug, Clone)]
struct ItemDTO<'a> {
    item_class: &'a str,
    item_base_name: String,
    item_name: String,
    last_part: &'a str, // contains text with mods
}
fn fetch_item_base<'a>(
    craft_repo: &impl CraftRepo,
    raw_item: &'a str,
    item_class: &'a str,
) -> Result<ItemDTO<'a>, String> {
    let (item_base_name, item_name) = raw_item
        .split("\n")
        .find_map(
            |row| match craft_repo.string_to_item_base(&item_class, row.trim()) {
                Ok(base_name) => Some((base_name, row.trim().to_string())),
                Err(_) => None,
            },
        )
        .ok_or("No item base found".to_string())?;

    let last_part = match raw_item.split("--------").last() {
        Some(last_part) => last_part,
        None => return Err("No mods found".to_string()),
    };
    Ok(ItemDTO {
        item_class,
        item_base_name,
        item_name,
        last_part,
    })
}

struct ModsDTO {
    mods: Vec<String>,
    raw_mods: Vec<String>,
}
#[deprecated(since = "0.5.0", note = "please use `fetch_mods` instead")]
fn fetch_mods_old(craft_repo: &impl CraftRepo, item_dto: ItemDTO) -> Result<ModsDTO, String> {
    // The idea of usage item text without extra (alt) information wasn't good. That why function is deprecated.

    let mut mods = vec![];
    let mut raw_mods = vec![];

    debug!("start parsing mods in {}", &item_dto.last_part);
    item_dto.last_part.split("\n").for_each(|row| {
        match craft_repo.string_to_mod_old(&item_dto.item_class, &item_dto.item_base_name, row.trim()) {
            Ok(mod_name) => {
                mods.push(mod_name);
                raw_mods.push(row.trim().to_string());
            }
            Err(_) => {}
        }
    });

    let mut need_to_parse_deeper = false;
    if item_dto.last_part.trim().split("\n").count() != mods.len() {
        match item_dto.last_part.trim().split("\n").last() {
            Some(value) if !value.contains("implicit") => {
                need_to_parse_deeper = true;
            }
            Some(_) => {}
            None => {}
        }
    }
    let mut added_complex_mods = 0;
    if need_to_parse_deeper {
        debug!("start parsing mods deeper");

        let mod_lines: Vec<&str> = item_dto.last_part.clone().trim().split("\n").collect();

        let mut chunked_mods = vec![];
        for chunk in mod_lines.chunks(2) {
            if chunk.len() == 2 {
                let v = format!("{}\n{}", chunk[0].clone(), chunk[1].clone()).replace("\r", "");
                chunked_mods.push(v);
                let v = format!("{}\n{}", chunk[1].clone(), chunk[0].clone()).replace("\r", "");
                chunked_mods.push(v);
            }
        }
        for chunk in mod_lines[1..].chunks(2) {
            if chunk.len() == 2 {
                let v = format!("{}\n{}", chunk[0].clone(), chunk[1].clone()).replace("\r", "");
                chunked_mods.push(v);
                let v = format!("{}\n{}", chunk[1].clone(), chunk[0].clone()).replace("\r", "");
                chunked_mods.push(v);
            }
        }

        debug!("start parsing mods in chunks {:?}", &chunked_mods);

        chunked_mods.into_iter().for_each(|row| {
            match craft_repo.string_to_mod_old(
                &item_dto.item_class,
                &item_dto.item_base_name,
                row.trim(),
            ) {
                Ok(mod_name) => {
                    mods.push(mod_name);
                    raw_mods.push(row.trim().to_string());
                    added_complex_mods += 1;
                }
                Err(_) => {}
            }
        });

        if added_complex_mods == 0 {
            warn!("found no complex mods for item");
        }
    }

    if item_dto.last_part.trim().split("\n").count() != mods.len() && added_complex_mods == 0 {
        match item_dto.last_part.trim().split("\n").last() {
            Some(value) if !value.contains("implicit") => {
                debug!("found wrong count of mods {:?}", mods);
                return Err("Found wrong count of mods".to_string());
            }
            Some(_) => {}
            None => {}
        }
    } else if item_dto.last_part.trim().split("\n").count() != (mods.len() + added_complex_mods) {
        // guess complex mod based on two mods always
        debug!("found wrong count of mods with complex {:?}", mods);
        return Err("Found wrong count of mods".to_string());
    };
    Ok(ModsDTO { mods, raw_mods })
}

#[derive(Debug, PartialEq)]
enum ModGenerationTypeEnum {
    Prefix,
    Suffix,
    Other,
}

fn string_to_mod_gen_type(value: &str) -> ModGenerationTypeEnum {
    if value == "Prefix" {
        return ModGenerationTypeEnum::Prefix;
    } else if value == "Suffix" {
        return ModGenerationTypeEnum::Suffix;
    } else {
        return ModGenerationTypeEnum::Other;
    }
}

#[derive(Debug, PartialEq)]
struct RawModDTO {
    generation_type: ModGenerationTypeEnum,
    mod_decription: Option<String>,
    mod_id: String,
    mod_name: Option<String>,
    mod_text: Vec<String>,
    tags: Vec<String>,
    tier: Option<u32>,
}

impl RawModDTO {
    fn new(
        meta_info: ModMetaInfo,
        mod_id: String,
        mod_text: Vec<String>,
        mod_decription: Option<String>,
    ) -> Self {
        Self {
            generation_type: meta_info.generation_type,
            tier: meta_info.tier,
            tags: meta_info.tags,
            mod_name: meta_info.mod_name,
            mod_id,
            mod_text,
            mod_decription,
        }
    }
}
struct ModMetaInfo {
    generation_type: ModGenerationTypeEnum,
    tier: Option<u32>,
    tags: Vec<String>,
    mod_name: Option<String>,
}
fn create_meta_mods_regexp_patter() -> Result<Regex, String> {
    let meta_mod_line_re =
        Regex::new(r"\{\s+(\w+)\s+Modifier\s+(.*?)\s+\(Tier:\s+(\d+)\)\s+—\s+(.*?)(?:\s+\}|$)")
            .expect("regexp error during item class fetching");
    Ok(meta_mod_line_re)
}

fn fetch_mods(craft_repo: &impl CraftRepo, item_dto: ItemDTO) -> Result<Vec<RawModDTO>, String> {
    let mut mods: Vec<RawModDTO> = vec![];
    debug!("start parsing mods in {}", &item_dto.last_part);

    let meta_mod_line_re = create_meta_mods_regexp_patter()?;

    let mut mod_meta: Option<ModMetaInfo> = None;
    let mut descr = None;
    let mut mod_text = vec![]; // may me multiline

    for row in item_dto.last_part.split("\n") {
        let trimmed_row = row.trim();
        let cap_curr = meta_mod_line_re.captures(&trimmed_row);
        match cap_curr {
            // row contains meta info for mod
            Some(c) => {
                // row contains meta info for mod
                if let Some(last_mod_meta) = mod_meta {
                    // let's close prev cap and continue new one
                    let mod_id  = craft_repo.string_to_mod(
                        &item_dto.item_class,
                        &item_dto.item_base_name,
                        &mod_text.join("\n"),
                    )?;
                    let mod_to_save = RawModDTO::new(last_mod_meta, mod_id, mod_text, descr);
                    mods.push(mod_to_save);
                };

                let curr_mod_meta = ModMetaInfo {
                    generation_type: string_to_mod_gen_type(&c[1]),
                    tier: c[3].parse::<u32>().ok(),
                    tags: c[4]
                        .split(",")
                        .into_iter()
                        .map(|v| v.trim().to_owned())
                        .collect(),
                    mod_name: Some(c[2].to_owned()),
                };

                mod_meta = Some(curr_mod_meta);
                descr = None;
                mod_text = vec![];
            }
            None => {
                if trimmed_row.starts_with("(") & trimmed_row.ends_with(")") {
                    // row contains desctiption
                    descr = Some(trimmed_row.to_owned());
                } else {
                    // row contains mod info
                    mod_text.push(trimmed_row.to_owned());
                }
            }
        };
    }
    // close prev cap
    if let Some(last_mod_meta) = mod_meta {
        let mod_id  = craft_repo.string_to_mod(
            &item_dto.item_class,
            &item_dto.item_base_name,
            &mod_text.join("\n"),
        )?;
        let mod_to_save = RawModDTO::new(last_mod_meta, mod_id, mod_text, descr);
        mods.push(mod_to_save);
    };

    Ok(mods)
}

pub fn parse_raw_item(craft_repo: &impl CraftRepo, raw_item: &str) -> Result<ParsedItem, String> {
    let item_class = fetch_item_class(craft_repo, raw_item)?;

    let item_dto = fetch_item_base(craft_repo, raw_item, item_class)?;

    let mods_dto = fetch_mods(craft_repo, item_dto.clone())?;

    Ok(ParsedItem {
        item_class: item_class.to_string(),
        item_base_name: item_dto.item_base_name,
        item_name: item_dto.item_name,
        mods: mods_dto.iter().map(|m| m.mod_id.to_owned()).collect(),
        raw_mods: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::files::local_db::FileRepo;
    use rstest::{fixture, rstest};

    #[fixture]
    fn repo() -> impl CraftRepo {
        FileRepo::new().unwrap()
    }
    #[rstest]
    #[case("{ Prefix Modifier \"Remora\'s\" (Tier: 1) — Life, Physical, Attack }".to_string(), vec!["Prefix", "\"Remora\'s\"", "1", "Life, Physical, Attack"])]
    #[case("{ Suffix Modifier \"of the Seal\" (Tier: 7) — Elemental, Cold, Resistance }".to_string(), vec!["Suffix", "\"of the Seal\"", "7", "Elemental, Cold, Resistance" ])]
    fn test_mata_mod_patten(#[case] row: String, #[case] expected: Vec<&str>) {
        let re = create_meta_mods_regexp_patter().unwrap();
        assert_eq!(re.is_match(&row), true);
        let cap = re.captures(&row).unwrap();
        assert_eq!(&cap[1], expected[0]);
        assert_eq!(&cap[2], expected[1]);
        assert_eq!(&cap[3], expected[2]);
        assert_eq!(&cap[4], expected[3]);
    }
    #[rstest]
    #[case("0.26(0.2-0.4)% of Physical Attack Damage Leeched as Life".to_string())]
    #[case("(Leeched Life is recovered over time. Multiple Leeches can occur simultaneously, up to a maximum rate)".to_string())]
    fn test_mata_mod_patten_failed(#[case] row: String) {
        let re = create_meta_mods_regexp_patter().unwrap();
        assert_eq!(re.is_match(&row), false);
        let cap = re.captures(&row);
        assert_eq!(cap.is_none(), true)
    }

    #[rstest]
    fn test_fetching_mods(repo: impl CraftRepo) {
        let item_dto = ItemDTO{
            item_class: "Gloves",
            item_base_name: "Gripped Gloves".to_owned(),
            item_name: "Remora's Gripped Gloves of the Seal".to_owned(),
            last_part: "{ Prefix Modifier \"Remora\'s\" (Tier: 1) — Life, Physical, Attack }
0.26(0.2-0.4)% of Physical Attack Damage Leeched as Life
(Leeched Life is recovered over time. Multiple Leeches can occur simultaneously, up to a maximum rate)
{ Suffix Modifier \"of the Seal\" (Tier: 7) — Elemental, Cold, Resistance }
+12(12-17)% to Cold Resistance",
        };
        let mods = fetch_mods(&repo, item_dto).unwrap();
        assert_eq!(mods.len(), 2);
        let expected_mod1 = RawModDTO {
            generation_type: ModGenerationTypeEnum::Prefix,
            tier: Some(1),
            tags: vec!["Life".to_owned(),"Physical".to_owned(),"Attack".to_owned()],
            mod_id: "LifeLeechPermyriad1".to_owned(),
            mod_name: Some("\"Remora\'s\"".to_owned()),
            mod_text: vec!["0.26(0.2-0.4)% of Physical Attack Damage Leeched as Life".to_owned()],
            mod_decription: Some("(Leeched Life is recovered over time. Multiple Leeches can occur simultaneously, up to a maximum rate)".to_owned()),
        };
        assert_eq!(mods[0], expected_mod1);

        let expected_mod2 = RawModDTO {
            generation_type: ModGenerationTypeEnum::Suffix,
            tier: Some(7),
            tags: vec![
                "Elemental".to_owned(),
                "Cold".to_owned(),
                "Resistance".to_owned(),
            ],
            mod_id: "ColdResist2".to_owned(),
            mod_name: Some("\"of the Seal\"".to_owned()),
            mod_text: vec!["+12(12-17)% to Cold Resistance".to_owned()],
            mod_decription: None,
        };

        assert_eq!(mods[1], expected_mod2);
    }
}

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

    // the mods block is not always the last "--------" section: flags like
    // "Fractured Item" / "Corrupted" are appended after it. Prefer the last
    // section that contains explicit mod meta lines; fall back to the legacy
    // behaviour for copies without advanced descriptions.
    let explicit_meta_re = Regex::new(r"(Prefix|Suffix)\s+Modifier")
        .expect("regexp error during mods section lookup");
    let last_part = match raw_item
        .split("--------")
        .filter(|part| explicit_meta_re.is_match(part))
        .last()
        .or_else(|| raw_item.split("--------").last())
    {
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
    // `(?:\w+\s+)*?` skips qualifiers before the generation type, e.g.
    // "{ Fractured Suffix Modifier ... }"
    let meta_mod_line_re = Regex::new(
        r"\{\s+(?:\w+\s+)*?(\w+)\s+Modifier\s+(.*?)\s+\(Tier:\s+(\d+)\)\s+(—\s+(.*?)(?:\s+\})|$)?",
    )
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
                    let mod_id = craft_repo.string_to_mod(
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
                    tags: c
                        .get(5)
                        .map_or("", |m| m.as_str())
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
                } else if !trimmed_row.is_empty() {
                    // row contains mod info; blank rows (the copied text ends
                    // with a newline) would otherwise become empty parts and
                    // leave a dangling separator in the joined representation
                    mod_text.push(trimmed_row.to_owned());
                }
            }
        };
    }
    // close prev cap
    if let Some(last_mod_meta) = mod_meta {
        let mod_id = craft_repo.string_to_mod(
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
        raw_mods: mods_dto.iter().map(|m| m.mod_text.join("; ")).collect(),
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
    #[case("{ Suffix Modifier \"of the Seal\" (Tier: 7) }".to_string(), vec!["Suffix", "\"of the Seal\"", "7", ""])]
    #[case("{ Fractured Suffix Modifier \"of Flexure\" (Tier: 1) — Evasion }".to_string(), vec!["Suffix", "\"of Flexure\"", "1", "Evasion"])]
    fn test_meta_mod_patten(#[case] row: String, #[case] expected: Vec<&str>) {
        let re = create_meta_mods_regexp_patter().unwrap();
        assert_eq!(re.is_match(&row), true);
        let cap = re.captures(&row).unwrap();
        // assert_eq!(cap.len(), 6);
        assert_eq!(&cap[1], expected[0]);
        assert_eq!(&cap[2], expected[1]);
        assert_eq!(&cap[3], expected[2]);
        assert_eq!(&cap.get(5).map_or("", |m| m.as_str()), &expected[3]);
    }
    #[rstest]
    #[case("0.26(0.2-0.4)% of Physical Attack Damage Leeched as Life".to_string())]
    #[case("(Leeched Life is recovered over time. Multiple Leeches can occur simultaneously, up to a maximum rate)".to_string())]
    fn test_meta_mod_patten_failed(#[case] row: String) {
        let re = create_meta_mods_regexp_patter().unwrap();
        // assert_eq!(re.is_match(&row), false);
        let cap: Option<regex::Captures> = re.captures(&row);
        assert_eq!(cap.is_none(), true)
    }

    #[test]
    fn test_parse_fractured_poe2_item() {
        use crate::entities::craft_repo::GameVersion;
        use crate::storage::files::local_db::data_dir_for;
        // mods must come from the section before the trailing "Fractured Item"
        // flag, and the fractured mod's meta line must be recognized
        if !std::path::Path::new(&data_dir_for(GameVersion::Poe2)).exists() {
            return;
        }
        let repo = FileRepo::new_for_version(GameVersion::Poe2).unwrap();
        let raw = "Item Class: Boots\nRarity: Rare\nBramble Stride\nDaggerfoot Shoes\n--------\nEvasion Rating: 140\nEnergy Shield: 43\n--------\nRequires: Level 80, 59 Dex, 59 Int\n--------\nSockets: S S \n--------\nItem Level: 82\n--------\n{ Fractured Suffix Modifier \"of Flexure\" (Tier: 1) — Evasion }\nGain Deflection Rating equal to 23(21-23)% of Evasion Rating\n{ Suffix Modifier \"of Bameth\" (Tier: 1) — Chaos, Resistance }\n+27(24-27)% to Chaos Resistance\n--------\nFractured Item\n";
        let parsed = parse_raw_item(&repo, raw).unwrap();
        assert_eq!(parsed.mods.len(), 2);
        assert!(parsed.mods.contains(&"EvasionGrantsDeflection5".to_string()));
        assert!(parsed.mods.contains(&"ChaosResist6".to_string()));
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

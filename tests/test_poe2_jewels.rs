use lazy_crafter::entities::craft_repo::{CraftRepo, GameVersion, ModsMatchMode, ModsQuery};
use lazy_crafter::storage::files::local_db::FileRepo;
use lazy_crafter::usecases::item_parser::parse_raw_item;
use std::collections::HashSet;

const JEWELS: [&str; 8] = [
    "Ruby",
    "Emerald",
    "Sapphire",
    "Diamond",
    "Time-Lost Ruby",
    "Time-Lost Emerald",
    "Time-Lost Sapphire",
    "Time-Lost Diamond",
];

#[test]
fn poe2_jewels_are_selectable_with_separate_mod_pools() {
    let repo = FileRepo::new_for_version(GameVersion::Poe2).unwrap();
    assert!(repo.get_item_classes().contains(&"Jewel".to_string()));
    let bases = repo.get_item_bases("Jewel");
    assert_eq!(
        bases
            .iter()
            .map(|b| b.name.as_str())
            .collect::<HashSet<_>>(),
        JEWELS.into_iter().collect()
    );
    assert!(bases.iter().all(|b| b.required_level == 20));
    let classes = repo.get_item_class_by_item_name();
    assert!(!classes.contains_key("Timeless Jewel"));

    for base in JEWELS {
        assert_eq!(classes.get(base).map(String::as_str), Some("Jewel"));
        let query = ModsQuery {
            item_base: base.to_string(),
            item_level: 1,
            string_query: String::new(),
            selected_mods: vec![],
            match_mode: ModsMatchMode::All,
        };
        let mods = repo.find_mods(&query);
        assert!(mods.iter().any(|m| m.generation_type == "prefix"), "{base}");
        assert!(mods.iter().any(|m| m.generation_type == "suffix"), "{base}");
        assert!(
            mods.iter().all(|m| !m.representation.is_empty()
                && !m.representation.contains("representation_err")
                && !m.representation.contains('[')),
            "{base}"
        );
        let keys: HashSet<_> = mods.iter().map(|m| m.mod_key.as_str()).collect();
        for modifier in &mods {
            assert_eq!(
                repo.representation_by_mod_id(&modifier.mod_key),
                modifier.representation
            );
        }
        let time_lost = base.starts_with("Time-Lost");
        assert_eq!(
            keys.contains("JewelRadiusSmallNodeEffect"),
            time_lost,
            "{base}"
        );
        for (colour, stat) in [
            ("Ruby", "Armour"),
            ("Emerald", "Accuracy"),
            ("Sapphire", "EnergyShield"),
        ] {
            let expected = base.ends_with(colour) || base.ends_with("Diamond");
            let prefix = if time_lost { "JewelRadius" } else { "Jewel" };
            assert_eq!(
                keys.contains(format!("{prefix}{stat}").as_str()),
                expected,
                "{base}: {stat}"
            );
            let other_prefix = if time_lost { "Jewel" } else { "JewelRadius" };
            assert!(
                !keys.contains(format!("{other_prefix}{stat}").as_str()),
                "{base}"
            );
        }
        let selected = mods[0].clone();
        let filtered = repo.find_mods(&ModsQuery {
            selected_mods: vec![selected.clone()],
            ..query
        });
        assert!(!filtered.iter().any(|m| m.mod_key == selected.mod_key));
    }
}

#[test]
fn poe2_jewels_parse_magic_and_rare_clipboard_items() {
    let repo = FileRepo::new_for_version(GameVersion::Poe2).unwrap();
    for base in JEWELS {
        let (mod_key, mod_type, affix, text) = if base.starts_with("Time-Lost") {
            (
                "JewelRadiusSmallNodeEffect",
                "Suffix",
                "of Potency",
                "20(15-25)% increased Effect of Small Passive Skills in Radius",
            )
        } else if base == "Emerald" {
            (
                "JewelAccuracy",
                "Prefix",
                "Accurate",
                "7(5-10)% increased Accuracy Rating",
            )
        } else if base == "Sapphire" {
            (
                "JewelEnergyShield",
                "Prefix",
                "Shimmering",
                "15(10-20)% increased maximum Energy Shield",
            )
        } else {
            (
                "JewelArmour",
                "Prefix",
                "Armoured",
                "15(10-20)% increased Armour",
            )
        };
        for rarity in ["Magic", "Rare"] {
            let name = if rarity == "Rare" {
                format!("Test Stone\n{base}")
            } else if mod_type == "Suffix" {
                format!("{base} {affix}")
            } else {
                format!("{affix} {base}")
            };
            let radius = if base.starts_with("Time-Lost") {
                "Radius: Small\n--------\n"
            } else {
                ""
            };
            let raw = format!("Item Class: Jewels\nRarity: {rarity}\n{name}\n--------\n{radius}Item Level: 80\n--------\n{{ {mod_type} Modifier \"{affix}\" (Tier: 1) }}\n{text}\n");
            let parsed =
                parse_raw_item(&repo, &raw).unwrap_or_else(|e| panic!("{base} ({rarity}): {e}"));
            assert_eq!(parsed.item_class, "Jewel");
            assert_eq!(parsed.item_base_name, base);
            assert_eq!(parsed.mods, vec![mod_key.to_string()]);
        }
    }
}

#[test]
fn time_lost_notable_grants_parse_from_clipboard() {
    let repo = FileRepo::new_for_version(GameVersion::Poe2).unwrap();
    for base in ["Time-Lost Sapphire", "Time-Lost Diamond"] {
        let raw = format!(
            "Item Class: Jewels\nRarity: Rare\nTest Stone\n{base}\n--------\n\
             Radius: Small\n--------\nItem Level: 80\n--------\n\
             {{ Prefix Modifier \"Serene\" (Tier: 1) — Defences, Energy Shield }}\n\
             Notable Passive Skills in Radius also grant 6(5-7)% faster start of Energy Shield Recharge\n\
             {{ Suffix Modifier \"of Potency\" (Tier: 1) }}\n\
             20(15-25)% increased Effect of Small Passive Skills in Radius\n\
             {{ Suffix Modifier \"of Marshalling\" (Tier: 1) }}\n\
             Notable Passive Skills in Radius also grant Minions have 8(5-10)% increased Critical Hit Chance\n"
        );
        // The Windows clipboard uses CRLF; the parser must still match each mod.
        let parsed = parse_raw_item(&repo, &raw.replace('\n', "\r\n")).unwrap();
        assert_eq!(parsed.item_base_name, base);
        assert_eq!(
            parsed.mods,
            vec![
                "JewelRadiusEnergyShieldDelay",
                "JewelRadiusSmallNodeEffect",
                "JewelRadiusMinionCriticalChance",
            ]
        );
        assert!(parsed.raw_mods[0].starts_with("Notable Passive Skills in Radius also grant "));
    }
}

#[test]
fn time_lost_small_passive_grants_parse_from_clipboard() {
    let repo = FileRepo::new_for_version(GameVersion::Poe2).unwrap();
    for base in ["Time-Lost Sapphire", "Time-Lost Diamond"] {
        let raw = format!(
            "Item Class: Jewels\nRarity: Rare\nTest Stone\n{base}\n--------\n\
             Radius: Small\n--------\nItem Level: 80\n--------\n\
             {{ Prefix Modifier \"Serene\" (Tier: 1) — Defences, Energy Shield }}\n\
             Notable Passive Skills in Radius also grant 6(5-7)% faster start of Energy Shield Recharge\n\
             {{ Suffix Modifier \"of Barriers\" (Tier: 1) }}\n\
             Small Passive Skills in Radius also grant Gain additional Stun Threshold equal to 2(1-2)% of maximum Energy Shield\n"
        );
        let parsed = parse_raw_item(&repo, &raw.replace('\n', "\r\n")).unwrap();
        assert_eq!(parsed.item_base_name, base);
        assert_eq!(
            parsed.mods,
            vec![
                "JewelRadiusEnergyShieldDelay",
                "JewelRadiusStunThresholdfromEnergyShield"
            ]
        );
        assert!(parsed.raw_mods[1].starts_with("Small Passive Skills in Radius also grant "));
    }
}

#[test]
fn time_lost_radius_upgrades_parse_with_unscalable_annotation() {
    let repo = FileRepo::new_for_version(GameVersion::Poe2).unwrap();
    for base in JEWELS
        .into_iter()
        .filter(|base| base.starts_with("Time-Lost"))
    {
        for (radius, affix, expected) in [
            ("Medium", "Greater", "JewelRadiusMediumSize"),
            ("Large", "Grand", "JewelRadiusLargeSize"),
        ] {
            for annotation in ["", " — Unscalable Value"] {
                let effect = format!("Upgrades Radius to {radius}{annotation}");
                let raw = format!(
                    "Item Class: Jewels\nRarity: Rare\nTest Stone\n{base}\n--------\n\
                     Radius: {radius}\n--------\nItem Level: 80\n--------\n\
                     {{ Prefix Modifier \"{affix}\" (Tier: 1) }}\n{effect}\n\
                     {{ Suffix Modifier \"of Potency\" (Tier: 1) }}\n\
                     20(15-25)% increased Effect of Small Passive Skills in Radius\n"
                );
                let parsed = parse_raw_item(&repo, &raw.replace('\n', "\r\n")).unwrap();
                assert_eq!(parsed.item_base_name, base);
                assert_eq!(parsed.mods, vec![expected, "JewelRadiusSmallNodeEffect"]);
                assert_eq!(parsed.raw_mods[0], effect);
            }
        }
    }
    for invalid in [
        "Upgrades Radius to Huge — Unscalable Value",
        "Upgrades Radius to Medium — Unknown Annotation",
        "Upgrades Radius to Medium — Unscalable Value extra text",
    ] {
        assert!(repo
            .string_to_mod("Jewel", "Time-Lost Diamond", invalid)
            .is_err());
    }
    assert!(repo
        .string_to_mod(
            "Jewel",
            "Diamond",
            "Upgrades Radius to Medium — Unscalable Value"
        )
        .is_err());
}

#[test]
fn notable_grant_matching_is_limited_to_time_lost_bases_and_exact_effects() {
    let repo = FileRepo::new_for_version(GameVersion::Poe2).unwrap();
    for (base, effect, expected) in [
        (
            "Time-Lost Ruby",
            "3(2-3)% increased Armour",
            "JewelRadiusArmour",
        ),
        (
            "Time-Lost Emerald",
            "2(1-2)% increased Accuracy Rating",
            "JewelRadiusAccuracy",
        ),
    ] {
        for passive in ["Notable", "Small"] {
            let text = format!("{passive} Passive Skills in Radius also grant {effect}");
            assert_eq!(repo.string_to_mod("Jewel", base, &text).unwrap(), expected);
        }
    }
    for passive in ["Notable", "Small"] {
        let text = format!("{passive} Passive Skills in Radius also grant 12(10-15)% faster start of Energy Shield Recharge");
        assert!(repo.string_to_mod("Jewel", "Sapphire", &text).is_err());
        assert!(repo
            .string_to_mod("Jewel", "Time-Lost Sapphire", &text)
            .is_err());
    }
}

#[test]
fn poe1_does_not_expose_poe2_jewels() {
    let repo = FileRepo::new_for_version(GameVersion::Poe1).unwrap();
    let classes = repo.get_item_class_by_item_name();
    for base in JEWELS {
        assert!(!classes.contains_key(base));
    }
    assert!(!repo.get_item_bases("Helmet").is_empty());
}


use lazy_crafter::entities::craft_repo::CraftRepo;
use lazy_crafter::storage::files::local_db::FileRepo;
use lazy_crafter::usecases::item_parser::{parse_raw_item, ParsedItem};

use rstest::{fixture, rstest};


#[fixture]
fn repo() -> impl CraftRepo {
    FileRepo::new().unwrap()
}

#[rstest]
#[case("Item Class: Gloves
Rarity: Magic
Remora's Gripped Gloves of the Seal
--------
Evasion Rating: 226
--------
Requirements:
Level: 70
Dex: 95
--------
Sockets: G-G 
--------
Item Level: 87
--------
{ Implicit Modifier — Damage, Attack }
15(14-18)% increased Projectile Attack Damage (implicit)
--------
{ Prefix Modifier \"Remora\'s\" (Tier: 1) — Life, Physical, Attack }
0.26(0.2-0.4)% of Physical Attack Damage Leeched as Life
(Leeched Life is recovered over time. Multiple Leeches can occur simultaneously, up to a maximum rate)
{ Suffix Modifier \"of the Seal\" (Tier: 7) — Elemental, Cold, Resistance }
+12(12-17)% to Cold Resistance", ParsedItem {
    item_class: "".to_string(),
    item_base_name: "".to_string(),
    item_name: "".to_string(),
    mods: vec!["".to_string()],
    raw_mods: vec!["".to_string()],
})]
#[case("Item Class: Body Armours
Rarity: Rare
Foe Coat
Battle Lamellar
--------
Armour: 377 (augmented)
Evasion Rating: 360 (augmented)
--------
Requirements:
Level: 54
Str: 79
Dex: 79
--------
Sockets: G-G-G-R 
--------
Item Level: 57
--------
{ Prefix Modifier \"Flexible\" (Tier: 6) — Defences, Armour, Evasion }
+45(28-48) to Armour
+28(28-48) to Evasion Rating
{ Suffix Modifier \"of the Newt\" (Tier: 11) — Life }
Regenerate 1.2(1-2) Life per second
{ Suffix Modifier \"of the Penguin\" (Tier: 6) — Elemental, Cold, Resistance }
+21(18-23)% to Cold Resistance
{ Suffix Modifier \"of Eviction\" (Tier: 4) — Chaos, Resistance }
+16(16-20)% to Chaos Resistance
", ParsedItem {
    item_class: "".to_string(),
    item_base_name: "".to_string(),
    item_name: "".to_string(),
    mods: vec!["".to_string()],
    raw_mods: vec!["".to_string()],
})]
#[case("Item Class: Boots
Rarity: Rare
Havoc Stride
Fugitive Boots
--------
Evasion Rating: 164 (augmented)
Energy Shield: 48 (augmented)
--------
Requirements:
Level: 70
Dex: 56
Int: 76
--------
Sockets: B-B-B-B 
--------
Item Level: 74
--------
{ Implicit Modifier — Chaos, Resistance }
+15(13-17)% to Chaos Resistance (implicit)
--------
{ Prefix Modifier \"Hale\" (Tier: 9) — Life }
+3(3-9) to maximum Life
{ Prefix Modifier \"Wasp's\" (Tier: 3) — Defences, Evasion, Energy Shield }
32(27-32)% increased Evasion and Energy Shield
12(12-13)% increased Stun and Block Recovery
{ Prefix Modifier \"Chalybeous\" (Tier: 4) — Mana }
+59(55-59) to maximum Mana
{ Suffix Modifier \"of Expulsion\" (Tier: 3) — Chaos, Resistance }
+23(21-25)% to Chaos Resistance
{ Suffix Modifier \"of the Salamander\" (Tier: 7) — Elemental, Fire, Resistance }
+12(12-17)% to Fire Resistance
", ParsedItem {
    item_class: "".to_string(),
    item_base_name: "".to_string(),
    item_name: "".to_string(),
    mods: vec!["".to_string()],
    raw_mods: vec!["".to_string()],
})]
fn test_parse_raw_item2(
    repo: impl CraftRepo,
    #[case] input: &str,
    #[case] expected: ParsedItem,
) {
    assert_eq!(parse_raw_item(&repo, &input), Ok(expected));
}

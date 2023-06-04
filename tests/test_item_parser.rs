use lazy_crafter::entities::craft_repo::CraftRepo;
use lazy_crafter::storage::files::local_db::FileRepo;
use lazy_crafter::usecases::item_parser::{parse_raw_item, ParsedItem};

use rstest::{fixture, rstest};

#[fixture]
fn repo() -> impl CraftRepo {
    FileRepo::new().unwrap()
}

#[rstest]
#[case("Item Class: Bows
Rarity: Magic
Imperial Bow of Restoration
--------
Bow
Physical Damage: 29-117
Critical Strike Chance: 5.00%
Attacks per Second: 1.45
--------
Requirements:
Level: 66
Dex: 212
--------
Sockets: G G G-G G B
--------
Item Level: 81
--------
24% increased Elemental Damage with Attack Skills (implicit)
--------
Gain 3 Life per Enemy Hit by Attacks", ParsedItem {
    item_class: "Bow".to_string(),
    item_base_name: "Imperial Bow".to_string(),
    item_name: "Imperial Bow of Restoration".to_string(),
    mods: vec!["LifeGainPerTargetLocal2".to_string()],
    raw_mods: vec!["Gain 3 Life per Enemy Hit by Attacks".to_string()],
})]
#[case("Item Class: Bows
Rarity: Magic
Freezing Imperial Bow of the Drake
--------
Bow
Physical Damage: 29-117
Elemental Damage: 44-84 (augmented)
Critical Strike Chance: 5.00%
Attacks per Second: 1.45
--------
Requirements:
Level: 66
Dex: 212
--------
Sockets: G G G-G G B
--------
Item Level: 81
--------
24% increased Elemental Damage with Attack Skills (implicit)
--------
Adds 44 to 84 Cold Damage
+22% to Fire Resistance", ParsedItem {
    item_class: "Bow".to_string(),
    item_base_name: "Imperial Bow".to_string(),
    item_name: "Freezing Imperial Bow of the Drake".to_string(),
    mods: vec![
        "LocalAddedColdDamageTwoHand5".to_string(),
        "FireResist3".to_string(),
    ],
    raw_mods: vec![
        "Adds 44 to 84 Cold Damage".to_string(),
        "+22% to Fire Resistance".to_string(),
    ],
})]
#[case("Item Class: Bows
Rarity: Magic
Long Bow of Success
--------
Bow
Physical Damage: 8-33
Critical Strike Chance: 6.00%
Attacks per Second: 1.30
--------
Requirements:
Level: 9
Dex: 38
--------
Sockets: B-G
--------
Item Level: 13
--------
Gain 4 Life per Enemy Killed", ParsedItem {
    item_class: "Bow".to_string(),
    item_base_name: "Long Bow".to_string(),
    item_name: "Long Bow of Success".to_string(),
    mods: vec![
        "LifeGainedFromEnemyDeath1".to_string(),
    ],
    raw_mods: vec![
        "Gain 4 Life per Enemy Killed".to_string(),
    ],
})]
#[case("Item Class: Shields
Rarity: Normal
Copper Tower Shield
--------
Chance to Block: 24%
Armour: 164
--------
Requirements:
Level: 24
Str: 62
--------
Sockets: R-R
--------
Item Level: 26
--------
+39 to maximum Life (implicit)", ParsedItem {
    item_class: "Shield".to_string(),
    item_base_name: "Copper Tower Shield".to_string(),
    item_name: "Copper Tower Shield".to_string(),
    mods: vec![],
    raw_mods: vec![],
})]
#[case("Item Class: Boots
Rarity: Magic
Magpie's Ringmail Boots
--------
Armour: 29
Energy Shield: 7
--------
Requirements:
Level: 16
Str: 15
Int: 15
--------
Sockets: B
--------
Item Level: 26
--------
9% increased Rarity of Items found", ParsedItem {
    item_class: "Boots".to_string(),
    item_base_name: "Ringmail Boots".to_string(),
    item_name: "Magpie's Ringmail Boots".to_string(),
    mods: vec!["ItemFoundRarityIncrease1".to_string()],
    raw_mods: vec!["9% increased Rarity of Items found".to_string()],
})]
#[case("Item Class: Thrusting One Hand Swords
Rarity: Magic
Antique Rapier of the Penguin
--------
One Handed Sword
Physical Damage: 12-46
Critical Strike Chance: 6.50%
Attacks per Second: 1.30
Weapon Range: 14
--------
Requirements:
Level: 26
Dex: 89
--------
Sockets: G-G-G
--------
Item Level: 27
--------
+25% to Global Critical Strike Multiplier (implicit)
--------
+18% to Cold Resistance", ParsedItem { 
    item_class: "Thrusting One Hand Sword".to_string(),
    item_base_name: "Antique Rapier".to_string(),
    item_name: "Antique Rapier of the Penguin".to_string(),
    mods: vec!["ColdResist3".to_string()],
    raw_mods: vec!["+18% to Cold Resistance".to_string()]
})]
#[case("Item Class: Shields
Rarity: Magic
Spiny Copper Tower Shield of the Prism
--------
Chance to Block: 24%
Armour: 164
--------
Requirements:
Level: 24
Str: 62
--------
Sockets: R-R
--------
Item Level: 26
--------
+39 to maximum Life (implicit)
--------
+8% to all Elemental Resistances
Reflects 10 Physical Damage to Melee Attackers
", ParsedItem {
    item_class: "Shield".to_string(),
    item_base_name: "Copper Tower Shield".to_string(),
    item_name: "Spiny Copper Tower Shield of the Prism".to_string(),
    mods: vec!["AllResistances2".to_string(), "AttackerTakesDamage2".to_string()],
    raw_mods: vec!["+8% to all Elemental Resistances".to_string(), "Reflects 10 Physical Damage to Melee Attackers".to_string()],
})]
#[case("Item Class: Thrusting One Hand Swords
Rarity: Magic
Heavy Antique Rapier of Light
--------
One Handed Sword
Physical Damage: 17-67 (augmented)
Critical Strike Chance: 6.50%
Attacks per Second: 1.30
Weapon Range: 14
--------
Requirements:
Level: 26
Dex: 89
--------
Sockets: G-G-G
--------
Item Level: 27
--------
+25% to Global Critical Strike Multiplier (implicit)
--------
45% increased Physical Damage
14% increased Global Accuracy Rating
10% increased Light Radius", ParsedItem { 
    item_class: "Thrusting One Hand Sword".to_string(),
    item_base_name: "Antique Rapier".to_string(),
    item_name: "Heavy Antique Rapier of Light".to_string(),
    mods: vec!["LocalIncreasedPhysicalDamagePercent1".to_string(), "LocalLightRadiusAndAccuracyNew2".to_string()],
    raw_mods: vec!["45% increased Physical Damage".to_string(), "14% increased Global Accuracy Rating\n10% increased Light Radius".to_string()],
})]
#[case("Item Class: Bows
Rarity: Magic
Long Bow of Shining
--------
Bow
Physical Damage: 8-33
Critical Strike Chance: 6.00%
Attacks per Second: 1.30
--------
Requirements:
Level: 9
Dex: 38
--------
Sockets: B-G
--------
Item Level: 13
--------
9% increased Global Accuracy Rating
5% increased Light Radius", ParsedItem {
    item_class: "Bow".to_string(),
    item_base_name: "Long Bow".to_string(),
    item_name: "Long Bow of Shining".to_string(),
    mods: vec!["LocalLightRadiusAndAccuracyNew1_".to_string()],
    raw_mods: vec!["9% increased Global Accuracy Rating\n5% increased Light Radius".to_string()],
})]
#[case("Item Class: Heist Tools
Rarity: Rare
Rapture Instrument
Skeleton Keyring
--------
This item can be equipped by:
Vinderi, the Dismantler
Isla, the Engineer
--------
Requirements:
Level 4 in Trap Disarmament
--------
Item Level: 83
--------
11% increased Trap Disarmament speed (implicit)
--------
38% increased Damage
16% increased Job speed
10% chance to not generate Alert Level on opening a Chest using Trap Disarmament
--------
Can only be equipped to Heist members.
", ParsedItem {
    item_class: "".to_string(),
    item_base_name: "".to_string(),
    item_name: "".to_string(),
    mods: vec!["".to_string()],
    raw_mods: vec!["".to_string()],
})]
#[case("Item Class: Heist Brooches
Rarity: Rare
Doom Choker
Foliate Brooch
--------
Any Heist member can equip this item.
--------
Requirements:
Level 5 in Any Job
--------
Item Level: 83
--------
26% increased Rarity of Items dropped in Heists (implicit)
--------
Heist Chests have a 13% chance to Duplicate contained Basic Currency
Heist Chests have a 9% chance to Duplicate contained Legion Splinters
11% increased Job speed
--------
Can only be equipped to Heist members.
", ParsedItem {
    item_class: "".to_string(),
    item_base_name: "".to_string(),
    item_name: "".to_string(),
    mods: vec!["".to_string()],
    raw_mods: vec!["".to_string()],
})]
#[case("Item Class: Heist Cloaks
Rarity: Rare
Empyrean Refuge
Whisper-woven Cloak
--------
Any Heist member can equip this item.
--------
Requirements:
Level 5 in Any Job
--------
Item Level: 83
--------
5% reduced raising of Alert Level (implicit)
--------
18% increased Damage while area is not in Lockdown
Players deal 18% increased Damage while area is not in Lockdown
12% reduced raising of Alert Level from opening Chests
10% increased Job Experience gain
--------
Can only be equipped to Heist members.
", ParsedItem {
    item_class: "".to_string(),
    item_base_name: "".to_string(),
    item_name: "".to_string(),
    mods: vec!["".to_string()],
    raw_mods: vec!["".to_string()],
})]
#[case("Item Class: Heist Gear
Rarity: Rare
Blight Implement
Obsidian Sharpening Stone
--------
Any Heist member can equip this item.
--------
Requirements:
Level 5 in Any Job
--------
Item Level: 83
--------
27% increased Melee Damage (implicit)
--------
1 to 60 added Lightning Damage
Players and their Minions have 1 to 60 added Lightning Damage
The Ring's Cut reduced by 15%
19% increased Job speed
--------
Can only be equipped to Heist members.
", ParsedItem {
    item_class: "".to_string(),
    item_base_name: "".to_string(),
    item_name: "".to_string(),
    mods: vec!["".to_string()],
    raw_mods: vec!["".to_string()],
})]
#[case("Item Class: Gloves
Rarity: Magic
Polished Gripped Gloves of the Starfish
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
15% increased Projectile Attack Damage (implicit)
--------
Adds 4 to 7 Physical Damage to Attacks
Regenerate 23.9 Life per second
", ParsedItem {
    item_class: "".to_string(),
    item_base_name: "".to_string(),
    item_name: "".to_string(),
    mods: vec!["".to_string()],
    raw_mods: vec!["".to_string()],
})]
#[case("Item Class: Gloves
Rarity: Magic
Moth's Gripped Gloves of Abjuration
--------
Evasion Rating: 267 (augmented)
--------
Requirements:
Level: 70
Dex: 95
--------
Sockets: G-G 
--------
Item Level: 87
--------
15% increased Projectile Attack Damage (implicit)
--------
+12% chance to Suppress Spell Damage
18% increased Evasion Rating
9% increased Stun and Block Recovery
", ParsedItem {
    item_class: "".to_string(),
    item_base_name: "".to_string(),
    item_name: "".to_string(),
    mods: vec!["".to_string()],
    raw_mods: vec!["".to_string()],
})]
#[case("Item Class: Boots
Rarity: Magic
Stallion's Shackled Boots of the Newt
--------
Evasion Rating: 63
Energy Shield: 13
--------
Requirements:
Level: 34
Dex: 34
Int: 34
--------
Sockets: B B 
--------
Item Level: 35
--------
Regenerate 1.5 Life per second
20% increased Movement Speed
", ParsedItem {
    item_class: "".to_string(),
    item_base_name: "".to_string(),
    item_name: "".to_string(),
    mods: vec!["".to_string()],
    raw_mods: vec!["".to_string()],
})]
fn test_parse_raw_item(repo: impl CraftRepo, #[case] input: &str, #[case] expected: ParsedItem) {
    assert_eq!(parse_raw_item(&repo, &input), Ok(expected));
}

// #[rstest]
// #[case(
//     "Item Class: Bows
// Rarity: Magic
// Smouldering Long Bow of Rejuvenation
// --------
// Bow
// Physical Damage: 8-33
// Elemental Damage: 16-33 (augmented)
// Critical Strike Chance: 6.00%
// Attacks per Second: 1.30
// --------
// Requirements:
// Level: 9
// Dex: 38
// --------
// Sockets: B-G
// --------
// Item Level: 13
// --------
// Adds 16 to 33 Fire Damage
// Gramts 2 Life per Enemy Hit",
//     "FIX"
// )]
// fn test_parse_raw_item_debug(
//     repo: impl CraftRepo,
//     #[case] input: &str,
//     #[case] expected: ParsedItem,
// ) {
//     assert_eq!(parse_raw_item(&repo, &input), Ok(expected));
// }

#[rstest]
#[case("", "No item class matches in string".to_string())]
#[case("Item Class: BlaBla
Rarity: Magic
Antique Rapier of the Penguin
--------
One Handed Sword", "Item class not found in db: BlaBla".to_string())]
#[case("Item Class: Thrusting One Hand Swords
Antique Rapier of the Penguin
--------
+18% to Magic Resistance", "Found wrong count of mods".to_string())]
#[case("Item Class: Thrusting One Hand Swords
Antique Penguin of the Rapier
--------
+18% to Magic Resistance", "No item base found".to_string())]
#[case("Item Class: Gloves
Rarity: Rare
Apocalypse Grip
Stealth Gloves
--------
Evasion Rating: 256
--------
Requirements:
Level: 62
Dex: 97
--------
Sockets: B-B G
--------
Item Level: 83
--------
+15% chance to Suppress Spell Damage
Adds 7 to 13 Cold Damage to Attacks
+39 to maximum Life
8% increased Rarity of Items found
+16% to Fire Resistance
+10% to Lightning Resistance
--------
Corrupted", "Found wrong count of mods".to_string())]
fn test_parse_raw_item_negative(
    repo: impl CraftRepo,
    #[case] input: &str,
    #[case] expected: String,
) {
    assert_eq!(parse_raw_item(&repo, &input), Err(expected));
}

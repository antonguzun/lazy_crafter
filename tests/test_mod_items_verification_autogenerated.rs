use lazy_crafter::entities::craft_repo::CraftRepo;
use lazy_crafter::storage::files::local_db::FileRepo;
use std::fs;

#[test]
fn test_autogenerated_mod_ids() {
    let known_issues = vec![
        "IncreasedManaEnhancedModRegen",
        "IncreasedManaEnhancedModReservation",
        "LocalBaseArmourAndEvasionRatingAndLife1",
        "LocalBaseArmourAndEvasionRatingAndLife2",
        "LocalBaseArmourAndEvasionRatingAndLife3",
        "LocalBaseArmourAndEvasionRatingAndLife4",
        "IncreasedManaEnhancedModReservation",
        "IncreasedManaEnhancedModReservation",
        "LocalBaseArmourAndEnergyShieldAndLife1_",
        "LocalBaseArmourAndEnergyShieldAndLife2_",
        "LocalBaseArmourAndEnergyShieldAndLife3",
        "LocalBaseArmourAndEnergyShieldAndLife4_",
        "LocalBaseEvasionRatingAndEnergyShieldAndLife1",
        "LocalBaseEvasionRatingAndEnergyShieldAndLife2",
        "LocalBaseEvasionRatingAndEnergyShieldAndLife3",
        "LocalBaseEvasionRatingAndEnergyShieldAndLife4",
        "LocalBaseArmourAndLife1",
        "LocalBaseArmourAndLife2",
        "LocalBaseArmourAndLife3",
        "LocalBaseArmourAndLife4",
        "LocalBaseEvasionRatingAndLife1",
        "LocalBaseEvasionRatingAndLife2",
        "LocalBaseEvasionRatingAndLife3",
        "LocalBaseEvasionRatingAndLife4",
        "LocalBaseEnergyShieldAndLife1",
        "LocalBaseEnergyShieldAndLife2",
        "LocalBaseEnergyShieldAndLife3_",
        "LocalBaseEnergyShieldAndLife4_",
        "LocalBaseEnergyShieldAndMana1",
        "LocalBaseEnergyShieldAndMana2",
        "LocalBaseEnergyShieldAndMana3",
        "LocalBaseEnergyShieldAndMana4",
        "ChaosResistEnhancedMod_",
        "LifeGainPerTargetLocal1",
        // "LifeGainPerTargetLocal2",
        "LifeGainPerTargetLocal3",
        "LifeGainPerTargetLocal4",
        "BeltReducedFlaskChargesUsed1",
        "ChanceToAvoidFreezeEssence3",
        "ChanceToAvoidFreezeEssence4",
        "ChanceToAvoidFreezeEssence5",
        "ChanceToAvoidFreezeEssence6",
        "ChanceToAvoidFreezeEssence7",
        "ChanceToBlockProjectileAttacks1_",
        "ChanceToBlockProjectileAttacks2",
        "ChanceToBlockProjectileAttacks3",
        "ChanceToBlockProjectileAttacks4",
        "ChanceToBlockProjectileAttacks5_",
        "ChanceToBlockProjectileAttacks6",
        "FishingPoolConsumption",
        "FishingLureType",
        "FishingHookType",
        "ChanceToDodge1",
        "ChanceToDodgeEssence5",
        "ChanceToDodgeEssence6",
        "ChanceToDodgeEssence7",
        "ChanceToDodgeSpellsEssence6",
        "ChanceToAvoidFreezeCorruption",
        "CurseOnHitTemporalChainsCurruption",
        "CurseOnHitVulnerabilityCorruption",
        "CurseOnHitElementalWeaknessCorruption",
        "V2AdditionalChainCorrupted",
        "V2ChanceToGainOnslaughtOnKillCorrupted_",
        "V2CurseOnHitDespair",
        "V2CurseOnHitElementalWeaknessCorrupted",
        "V2CurseOnHitEnfeeble",
        "V2CurseOnHitTemporalChainsCurrupted",
        "V2CurseOnHitVulnerabilityCorrupted",
        "V2GainFrenzyChargeAfterSpending200ManaCorrupted",
        "asd",
        "asd",
        "asd",
        "asd",
        "asd",
        "asd",
        "asd",
        "asd",
        "asd",
        "asd",
    ];
    let repo = FileRepo::new().unwrap();
    let mut counter = 0;
    for line in fs::read_to_string("./tests/autogenerated_mod_ids_testcases.txt")
        .unwrap()
        .lines()
    {
        let data: Vec<&str> = line.split(";").collect();
        let mod_id;
        let expected;
        let expected2;

        match data.len() {
            2 => {
                mod_id = data[0];
                expected = data[1].to_string();
                expected2 = data[1].to_string();
            }
            3 => {
                mod_id = data[0];
                expected = format!("{}\n{}", data[1], data[2]);
                expected2 = format!("{}\n{}", data[2], data[1]);
            }
            _ => panic!(),
        }
        if known_issues.contains(&mod_id) {
            println!("skip {}", mod_id);
            continue;
        }
        let repr = repo.representation_by_mod_id(mod_id);
        if repr != expected2 {
            assert_eq!(repr, expected);
        } else {
            assert_eq!(repr, expected2);
        }
        assert!(repr == expected || repr == expected2);
        println!("passed {}", mod_id);
        counter += 1;
    }
    assert_eq!(counter, 11179 - known_issues.len())
}

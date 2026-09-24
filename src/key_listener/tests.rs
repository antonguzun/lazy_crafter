use super::*;
use crate::entities::craft_repo::{GameVersion, ModsMatchMode, ModsQuery};
use rdev::Button;

const MATCH: &str = "Item Class: Jewels\nRarity: Magic\nArmoured Ruby\n--------\nItem Level: 80\n--------\n{ Prefix Modifier \"Armoured\" (Tier: 1) }\n15(10-20)% increased Armour\n";
const NO_MATCH: &str = "Item Class: Jewels\nRarity: Magic\nBlasting Ruby\n--------\nItem Level: 80\n--------\n{ Prefix Modifier \"Blasting\" (Tier: 1) }\n5(4-6)% increased Area of Effect\n";

fn fixture(max_tries: u64) -> (FileRepo, Arc<Mutex<UiStates>>) {
    let repo = FileRepo::new_for_version(GameVersion::Poe2).unwrap();
    let armour = repo
        .find_mods(&ModsQuery {
            item_base: "Ruby".to_string(),
            item_level: 80,
            string_query: String::new(),
            selected_mods: vec![],
            match_mode: ModsMatchMode::All,
        })
        .into_iter()
        .find(|m| m.mod_key == "JewelArmour")
        .unwrap();
    let state = UiStates {
        selected: vec![armour],
        selected_max_autocraft_tries: max_tries,
        selected_mods_match_mode: ModsMatchMode::All,
        ..UiStates::default()
    };
    (repo, Arc::new(Mutex::new(state)))
}

fn assert_released(events: &[EventType]) {
    assert!(events.ends_with(&[
        EventType::KeyRelease(Key::KeyC),
        EventType::KeyRelease(Key::ControlLeft),
        EventType::KeyRelease(Key::Alt),
        EventType::KeyRelease(Key::ShiftLeft),
    ]));
}

fn clicks(events: &[EventType]) -> usize {
    events
        .iter()
        .filter(|e| **e == EventType::ButtonPress(Button::Left))
        .count()
}

#[test]
fn temporary_open_and_read_errors_recover_without_extra_rolls() {
    let (repo, state) = fixture(2);
    let mut reads = [
        Ok(NO_MATCH.to_string()),
        Err("Open clipboard: OS error 5".to_string()),
        Err("Read clipboard: OS error 5".to_string()),
        Ok(MATCH.to_string()),
    ]
    .into_iter();
    let mut events = vec![];
    let mut delays = vec![];
    let result = run_craft_with_io(
        &repo,
        state,
        |e| events.push(*e),
        || reads.next().expect("no extra reads after matching"),
        |d| delays.push(d),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(reads.next().is_none());
    assert_eq!(clicks(&events), 1);
    assert_eq!(delays, [300, 40, 80].map(Duration::from_millis));
    assert_released(&events);
}

#[test]
fn persistent_clipboard_failure_stops_and_allows_another_craft() {
    let (repo, state) = fixture(2);
    // Both failure sites used to panic. Check a failure after one currency click.
    for failure in ["Open clipboard: OS error 5", "Read clipboard: OS error 5"] {
        let mut reads = 0;
        let mut events = vec![];
        let result = run_craft_with_io(
            &repo,
            Arc::clone(&state),
            |e| events.push(*e),
            || {
                reads += 1;
                if reads == 1 {
                    Ok(NO_MATCH.to_string())
                } else {
                    Err(failure.to_string())
                }
            },
            |_| {},
        );
        let error = result.unwrap_err();
        assert!(error.contains("after 6 attempts"));
        assert!(error.contains(failure));
        assert_eq!(reads, 7);
        assert_eq!(
            clicks(&events),
            1,
            "no more rolls while clipboard is unavailable"
        );
        assert_released(&events);

        // The same state/repository can be used again after returning the error.
        events.clear();
        assert!(run_craft_with_io(
            &repo,
            Arc::clone(&state),
            |e| events.push(*e),
            || Ok(MATCH.to_string()),
            |_| {}
        )
        .is_ok());
        assert_eq!(clicks(&events), 0);
        assert_released(&events);
    }
}

#[test]
fn parse_error_releases_keys_without_rolling() {
    let (repo, state) = fixture(2);
    let mut events = vec![];
    let result = run_craft_with_io(
        &repo,
        state,
        |e| events.push(*e),
        || Ok("not an item".to_string()),
        |_| {},
    );
    assert!(result.unwrap_err().contains("Could not parse craft"));
    assert_eq!(clicks(&events), 0);
    assert_released(&events);
}

#[test]
fn unchanged_clipboard_still_stops_without_repeated_rolls() {
    let (repo, state) = fixture(2);
    let mut events = vec![];
    let mut reads = 0;
    let result = run_craft_with_io(
        &repo,
        state,
        |e| events.push(*e),
        || {
            reads += 1;
            Ok(NO_MATCH.to_string())
        },
        |_| {},
    );
    assert!(result.unwrap_err().contains("clipboard did not change"));
    assert_eq!(reads, 6);
    assert_eq!(clicks(&events), 1);
    assert_released(&events);
}

#[test]
fn exhausting_roll_limit_releases_keys() {
    let (repo, state) = fixture(1);
    let mut events = vec![];
    assert!(run_craft_with_io(
        &repo,
        state,
        |e| events.push(*e),
        || Ok(NO_MATCH.to_string()),
        |_| {}
    )
    .is_ok());
    assert_eq!(clicks(&events), 1);
    assert_released(&events);
}

#[test]
fn unexpected_unwind_also_releases_keys() {
    let (repo, state) = fixture(2);
    let mut events = vec![];
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run_craft_with_io(
            &repo,
            state,
            |e| events.push(*e),
            || panic!("simulated unexpected failure"),
            |_| {},
        )
    }));
    assert!(result.is_err());
    assert_eq!(clicks(&events), 0);
    assert_released(&events);
}

#[test]
fn autocraft_keeps_rolling_until_the_percent_threshold_is_met() {
    let (repo, state) = fixture(3);
    state.lock().unwrap().roll_requirements.min_percent = 80;
    let mut reads = [MATCH.to_string(), MATCH.replace("15(10-20)", "18(10-20)")].into_iter();
    let mut events = vec![];
    run_craft_with_io(
        &repo,
        state,
        |e| events.push(*e),
        || Ok(reads.next().expect("stop at exactly 80%")),
        |_| {},
    )
    .unwrap();
    assert!(reads.next().is_none());
    assert_eq!(clicks(&events), 1);
    assert_released(&events);
}

#[test]
fn autocraft_or_count_exception_accepts_bottom_rolls_only_when_enabled() {
    for (mode, count, expected_clicks) in [
        (ModsMatchMode::Any, 2, 0),
        (ModsMatchMode::Any, 3, 1),
        (ModsMatchMode::Any, 0, 1),
        (ModsMatchMode::All, 2, 1),
    ] {
        let (repo, state) = fixture(3);
        let area = repo
            .find_mods(&ModsQuery {
                item_base: "Ruby".to_string(),
                item_level: 80,
                string_query: String::new(),
                selected_mods: vec![],
                match_mode: mode,
            })
            .into_iter()
            .find(|m| m.mod_key == "JewelAreaofEffect")
            .expect("area jewel modifier");
        {
            let mut state = state.lock().unwrap();
            state.selected.push(area);
            state.selected_mods_match_mode = mode;
            state.roll_requirements.min_percent = 100;
            state.roll_requirements.ignore_roll_at_or_count = count;
        }
        let low = format!(
            "{}{{ Prefix Modifier \"Blasting\" (Tier: 1) }}\n4(4-6)% increased Area of Effect\n",
            MATCH
                .replace("Rarity: Magic", "Rarity: Rare")
                .replace("15(10-20)", "10(10-20)")
        );
        let high = low
            .replace("10(10-20)", "20(10-20)")
            .replace("4(4-6)", "6(4-6)");
        let mut reads = [low, high].into_iter();
        let mut events = vec![];
        run_craft_with_io(
            &repo,
            state,
            |e| events.push(*e),
            || Ok(reads.next().expect("stop on a matching item")),
            |_| {},
        )
        .unwrap();
        assert_eq!(clicks(&events), expected_clicks, "{mode:?}, N={count}");
        assert_released(&events);
    }
}

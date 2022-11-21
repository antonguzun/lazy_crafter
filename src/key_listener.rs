use chrono::{DateTime, Utc};
use rdev::{listen, simulate, Button, EventType, Key};
use std::collections::HashSet;
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, SystemTime};

fn hash_event_type(event_type: EventType) -> String {
    format!("{:?}", &event_type)
}

fn create_target_hash_set() -> HashSet<String> {
    let mut target_events = HashSet::new();
    // target_events.insert(hash_event_type(EventType::KeyPress(Key::ControlLeft)));
    target_events.insert(hash_event_type(EventType::KeyRelease(Key::ControlLeft)));
    // target_events.insert(hash_event_type(EventType::KeyPress(Key::ShiftLeft)));
    target_events.insert(hash_event_type(EventType::KeyRelease(Key::Alt)));
    // target_events.insert(hash_event_type(EventType::KeyPress(Key::KeyD)));
    target_events.insert(hash_event_type(EventType::KeyRelease(Key::KeyC)));
    target_events
}

fn send(event_type: &EventType) {
    let delay = Duration::from_millis(20);
    match simulate(event_type) {
        Ok(()) => (),
        Err(_) => {
            println!("We could not send {:?}", event_type);
        }
    }
    thread::sleep(delay);
}
#[cfg(target_os = "windows")]
fn run_craft() {
    use clipboard_win::{formats, Clipboard, Getter, Setter};

    println!("run crafting");
    send(&EventType::KeyPress(Key::ShiftLeft));

    for i in 0..3 {
        send(&EventType::KeyPress(Key::ControlLeft));
        send(&EventType::KeyPress(Key::KeyC));
        send(&EventType::KeyRelease(Key::ControlLeft));
        send(&EventType::KeyRelease(Key::KeyC));
        let _clip = Clipboard::new_attempts(10).expect("Open clipboard");
        println!("##### try {} #####", i);

        let mut output = String::new();
        formats::Unicode
            .read_clipboard(&mut output)
            .expect("Read sample");

        // do magic
        println!("copied {}", output);

        output.clear();
        send(&EventType::ButtonPress(Button::Left));
        send(&EventType::ButtonRelease(Button::Left));
    }
    send(&EventType::KeyRelease(Key::ShiftLeft));
}

#[cfg(target_os = "linux")]
fn run_craft() {}

pub fn run_listener_in_background() {
    let (schan, rchan) = channel();
    thread::spawn(move || {
        listen(move |event| {
            schan
                .send(event)
                .unwrap_or_else(|e| println!("Could not send event {:?}", e));
        })
        .expect("Could not listen");
    });
    thread::spawn(move || {
        let keypress_bandwidth = Duration::from_millis(1000);
        let mut events = Vec::new();
        let target_events = create_target_hash_set();
        // println!("target_events {:?}", target_events);
        let mut last_combo = SystemTime::now() - Duration::from_secs(500);
        for event in rchan.iter() {
            events.push(event);
            events.retain(|e| e.time > SystemTime::now() - keypress_bandwidth);
            let current_events =
                HashSet::from_iter(events.iter().map(|e| hash_event_type(e.event_type)));
            if target_events.is_subset(&current_events)
                && last_combo < SystemTime::now() - keypress_bandwidth
            {
                let t: DateTime<Utc> = last_combo.clone().into();
                println!("You pressed combo! prev combo at {}", t.to_rfc3339());
                last_combo = SystemTime::now();
                events.clear();
                run_craft();
                let delay = Duration::from_millis(100);
                thread::sleep(delay);
            }
        }
    });
}

fn find_mods(data: String) -> Vec<String> {
    vec![]
}

#[test]
fn test_find_mods1() {
    let str = "Item Class: Bows
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
Gain 3 Life per Enemy Hit by Attacks"
        .to_string();
    let mods = vec!["LifeGainPerTargetLocal2".to_string()];
    assert_eq!(find_mods(str), mods);
}

#[test]
fn test_find_mods2() {
    let str = "Item Class: Bows
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
+22% to Fire Resistance"
        .to_string();
    let mods = vec![
        "LocalAddedColdDamageTwoHand5".to_string(),
        "FireResist3".to_string(),
    ];
    assert_eq!(find_mods(str), mods);
}

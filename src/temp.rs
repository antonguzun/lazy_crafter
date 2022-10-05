use chrono::{DateTime, Utc};
use eframe::egui::epaint::ahash::HashMap;
use lazy_crafter::entities::craft_repo::{CraftRepo, ModItem, ModsQuery};
use rdev::{listen, simulate, EventType, Key};
use std::collections::HashSet;
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, SystemTime};
extern crate x11_clipboard;
use egui::Sense;
use egui_extras::{Size, TableBuilder};
use lazy_crafter::storage::files::local_db::FileRepo;
use x11_clipboard::Clipboard;

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
    // Let ths OS catchup (at least MacOS)
    thread::sleep(delay);
}

fn run_craft() {
    println!("run crafting");
    let clipboard = Clipboard::new().unwrap();

    send(&EventType::KeyPress(Key::ControlLeft));
    send(&EventType::KeyPress(Key::KeyC));
    send(&EventType::KeyRelease(Key::ControlLeft));
    send(&EventType::KeyRelease(Key::KeyC));

    println!("try copy");
    let val = clipboard
        .load(
            clipboard.setter.atoms.clipboard,
            clipboard.setter.atoms.string,
            clipboard.setter.atoms.property,
            Duration::from_millis(300),
        )
        .unwrap();
    let val = String::from_utf8(val).unwrap();

    println!("{}", val);
}

fn main() {
    let db = FileRepo::new().unwrap();
    println!("db initialized");
    println!(
        "{:?}",
        db.translations.get("+1_max_charged_attack_stages").unwrap()
    );

    let (schan, rchan) = channel();
    let _listener = thread::spawn(move || {
        listen(move |event| {
            schan
                .send(event)
                .unwrap_or_else(|e| println!("Could not send event {:?}", e));
        })
        .expect("Could not listen");
    });

    let keypress_bandwidth = Duration::from_millis(1000);
    let mut events = Vec::new();
    let target_events = create_target_hash_set();
    println!("target_events {:?}", target_events);
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
}

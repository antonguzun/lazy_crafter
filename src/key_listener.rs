use crate::entities::craft_repo::{BackEvents, CraftRepo, UiStates};
use crate::storage::files::local_db::FileRepo;
use crate::usecases::matcher::{check_matching, ModMatcher};
use chrono::{DateTime, Utc};
use log::{debug, info};
use rdev::{listen, simulate, EventType, Key};
use std::collections::HashSet;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
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
    target_events.insert(hash_event_type(EventType::KeyRelease(Key::ShiftLeft)));
    // target_events.insert(hash_event_type(EventType::KeyPress(Key::KeyD)));
    target_events.insert(hash_event_type(EventType::KeyRelease(Key::KeyE)));
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
fn run_craft(craft_repo: &impl CraftRepo, ui_states: Arc<Mutex<UiStates>>) -> Result<(), String> {
    use crate::usecases::item_parser;
    use clipboard_win::{formats, Clipboard, Getter, Setter};
    use rdev::Button;

    println!("run crafting");

    let selected_mods = ui_states.lock().unwrap().selected.clone();
    let selected_mod_keys: HashSet<String> =
        HashSet::from_iter(selected_mods.iter().map(|m| m.mod_key.clone()));
    let max_tries = ui_states
        .lock()
        .unwrap()
        .selected_max_autocraft_tries
        .clone();
    send(&EventType::KeyPress(Key::ShiftLeft));
    send(&EventType::KeyPress(Key::Alt));

    let mut prev_output = String::new();
    let mut down_counter = max_tries;

    let mut no_changes_in_clipboard_counter: u32 = 0;
    while down_counter > 0 {
        send(&EventType::KeyPress(Key::ControlLeft));
        send(&EventType::KeyPress(Key::KeyC));
        send(&EventType::KeyRelease(Key::ControlLeft));
        send(&EventType::KeyRelease(Key::KeyC));
        let _clip = Clipboard::new_attempts(10).expect("Open clipboard");
        println!("##### try {} #####", down_counter);

        let mut output = String::new();
        formats::Unicode
            .read_clipboard(&mut output)
            .expect("Read sample");
        println!("copied {}", output);
        if no_changes_in_clipboard_counter == 5 {
            break;
        }
        if output == prev_output {
            info!("No change in clipboard, skipping");
            no_changes_in_clipboard_counter = no_changes_in_clipboard_counter.saturating_add(1);
            let delay = Duration::from_millis(40);
            thread::sleep(delay);
            continue;
        } else {
            no_changes_in_clipboard_counter = 0;
        }
        prev_output = output.clone();
        let parsed_craft = match item_parser::parse_raw_item(craft_repo, &output) {
            Ok(parsed_craft) => parsed_craft,
            Err(e) => {
                let err_message = format!("Could not parse craft: {}", e);
                info!("{}", err_message);
                send(&EventType::KeyRelease(Key::ShiftLeft));
                send(&EventType::KeyRelease(Key::Alt));
                output.clear();
                return Err(err_message);
            }
        };
        println!("parsed {:#?}", &parsed_craft);
        let crafted_mod_keys: HashSet<String> = HashSet::from_iter(parsed_craft.mods);
        // FIXME! create mathcer only once!
        let matcher = ModMatcher::new(selected_mod_keys, &parsed_craft.item_base_name, craft_repo)?;

        if check_matching(matcher, crafted_mod_keys) {
            info!("Crafted all target mods successfully");
            send(&EventType::KeyRelease(Key::ShiftLeft));
            send(&EventType::KeyRelease(Key::Alt));
            output.clear();
            break;
        }

        output.clear();

        send(&EventType::ButtonPress(Button::Left));
        send(&EventType::ButtonRelease(Button::Left));
        down_counter -= 1;
        info!("Mod changed");
    }
    info!("All attempts were exhausted");
    send(&EventType::KeyRelease(Key::ShiftLeft));
    send(&EventType::KeyRelease(Key::Alt));
    Ok(())
}

#[cfg(target_os = "linux")]
fn run_craft(_repo: &impl CraftRepo, _ui_states: Arc<Mutex<UiStates>>) -> Result<(), String> {
    Err(String::from("Auto crafting is not supported on linux yet"))
}

pub fn run_listener_in_background(sender: Sender<BackEvents>, ui_states: Arc<Mutex<UiStates>>) {
    // !TODO use common instance between threads
    let craft_repo: FileRepo;
    match FileRepo::new() {
        Ok(repo) => {
            craft_repo = repo;
        }
        Err(e) => {
            sender
                .send(BackEvents::Error(format!(
                    "Autocrafter process initialization error. {}",
                    e
                )))
                .expect("Could not send crafting error event");

            return ();
        }
    }

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
                match run_craft(&craft_repo, Arc::clone(&ui_states)) {
                    Ok(_) => {}
                    Err(e) => {
                        sender
                            .send(BackEvents::Error(e))
                            .expect("Could not send crafting error event");
                    }
                    Err(_) => {}
                }
                let delay = Duration::from_millis(100);
                thread::sleep(delay);
            }
        }
    });
}

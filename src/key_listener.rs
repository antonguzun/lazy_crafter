use crate::entities::craft_repo::{BackEvents, CraftRepo, UiStates};
use crate::storage::files::local_db::FileRepo;
use crate::usecases::matcher::{check_matching, ModMatcher};
use chrono::{DateTime, Utc};
use log::{debug, error, info};
use rdev::{listen, simulate, EventType, Key};
use std::collections::HashSet;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, SystemTime};

fn send(event_type: &EventType) {
    let delay = Duration::from_millis(20);
    match simulate(event_type) {
        Ok(()) => (),
        Err(_) => {
            error!("We could not send {:?}", event_type);
        }
    }
    thread::sleep(delay);
}

/// Crafting holds Shift+Alt down via simulated input. If the app is closed
/// mid-craft the release events are never sent and the keys stay logically
/// pressed system-wide (breaking Ctrl+C and other shortcuts). Called on app
/// shutdown to unstick them.
pub fn release_all_modifiers() {
    for key in [Key::ShiftLeft, Key::Alt, Key::ControlLeft] {
        send(&EventType::KeyRelease(key));
    }
}

#[cfg(target_os = "windows")]
fn run_craft(craft_repo: &impl CraftRepo, ui_states: Arc<Mutex<UiStates>>) -> Result<(), String> {
    use crate::usecases::item_parser;
    use clipboard_win::{formats, Clipboard, Getter, Setter};
    use rdev::Button;

    info!("run crafting");

    let selected_mods = ui_states.lock().unwrap().selected.clone();
    let match_mode = ui_states.lock().unwrap().selected_mods_match_mode;
    let selected_mod_keys: HashSet<String> =
        HashSet::from_iter(selected_mods.iter().map(|m| m.mod_key.clone()));
    let max_tries = ui_states
        .lock()
        .unwrap()
        .selected_max_autocraft_tries
        .clone();
    send(&EventType::KeyPress(Key::ShiftLeft));
    send(&EventType::KeyPress(Key::Alt));
    // give the game a moment to register the modifiers before the first copy
    thread::sleep(Duration::from_millis(300));

    let mut prev_output = String::new();
    let mut down_counter = max_tries;
    // what the item under the cursor is, logged once and again if it changes
    let mut logged_item = String::new();

    let mut no_changes_in_clipboard_counter: u32 = 0;
    while down_counter > 0 {
        send(&EventType::KeyPress(Key::ControlLeft));
        send(&EventType::KeyPress(Key::KeyC));
        send(&EventType::KeyRelease(Key::ControlLeft));
        send(&EventType::KeyRelease(Key::KeyC));
        let _clip = Clipboard::new_attempts(10).expect("Open clipboard");

        let mut output = String::new();
        formats::Unicode
            .read_clipboard(&mut output)
            .expect("Read sample");
        debug!("try {}: copied {}", down_counter, output);
        if output == prev_output {
            no_changes_in_clipboard_counter = no_changes_in_clipboard_counter.saturating_add(1);
            if no_changes_in_clipboard_counter >= 5 {
                send(&EventType::KeyRelease(Key::ShiftLeft));
                send(&EventType::KeyRelease(Key::Alt));
                output.clear();
                return Err(String::from(
                    "Crafting stopped: clipboard did not change after 5 copy attempts. \
                     Is the game window focused and the cursor over the item?",
                ));
            }
            // usually the game UI just hasn't refreshed the item yet, so back
            // off progressively: 80, 160, 320, 640 ms before each retry
            let delay = Duration::from_millis(40u64 << no_changes_in_clipboard_counter.min(4));
            info!("No change in clipboard, retrying in {:?}", delay);
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
        debug!("parsed {:#?}", &parsed_craft);
        let item = format!(
            "item: {} | class: {} | base: {}",
            parsed_craft.item_name, parsed_craft.item_class, parsed_craft.item_base_name
        );
        if item != logged_item {
            info!("{}", item);
            logged_item = item;
        }
        info!(
            "try {}: [{}]",
            down_counter,
            parsed_craft.raw_mods.join(" | ")
        );
        let crafted_mod_keys: HashSet<String> = HashSet::from_iter(parsed_craft.mods);
        // FIXME! create mathcer only once!
        let matcher = match ModMatcher::new(selected_mod_keys.clone(), &parsed_craft.item_base_name, craft_repo){
            Ok(m) => m,
            Err(e) => {
                error!("stop crafting: {}", e);
                send(&EventType::KeyRelease(Key::ShiftLeft));
                send(&EventType::KeyRelease(Key::Alt));
                output.clear();
                break;
            },
        };
        

        if check_matching(matcher, crafted_mod_keys, match_mode) {
            info!("-> matched ({} mode), stopping", match_mode.label());
            send(&EventType::KeyRelease(Key::ShiftLeft));
            send(&EventType::KeyRelease(Key::Alt));
            output.clear();
            break;
        }

        output.clear();

        send(&EventType::ButtonPress(Button::Left));
        send(&EventType::ButtonRelease(Button::Left));
        down_counter -= 1;
        info!("-> no match, rolling again");
    }
    if down_counter == 0 {
        info!("All attempts were exhausted");
    }
    send(&EventType::KeyRelease(Key::ShiftLeft));
    send(&EventType::KeyRelease(Key::Alt));
    Ok(())
}

#[cfg(target_os = "linux")]
fn run_craft(_repo: &impl CraftRepo, _ui_states: Arc<Mutex<UiStates>>) -> Result<(), String> {
    Err(String::from("Auto crafting is not supported on linux yet"))
}

pub fn run_listener_in_background(
    sender: Sender<BackEvents>,
    ui_states: Arc<Mutex<UiStates>>,
    craft_repo: Arc<RwLock<FileRepo>>,
) {
    let (schan, rchan) = channel();
    thread::spawn(move || {
        listen(move |event| {
            schan
                .send(event)
                .unwrap_or_else(|e| error!("Could not send event {:?}", e));
        })
        .expect("Could not listen");
    });
    thread::spawn(move || {
        let debounce = Duration::from_millis(1000);
        let mut ctrl_held = false;
        let mut last_combo = SystemTime::now() - Duration::from_secs(500);
        for event in rchan.iter() {
            match event.event_type {
                EventType::KeyPress(Key::ControlLeft) => ctrl_held = true,
                EventType::KeyRelease(Key::ControlLeft) => ctrl_held = false,
                EventType::KeyPress(Key::KeyN)
                    if ctrl_held && last_combo < SystemTime::now() - debounce =>
                {
                    let t: DateTime<Utc> = last_combo.into();
                    info!("You pressed combo! prev combo at {}", t.to_rfc3339());
                    last_combo = SystemTime::now();
                    if let Err(e) = run_craft(&*craft_repo.read().unwrap(), Arc::clone(&ui_states))
                    {
                        sender
                            .send(BackEvents::Error(e))
                            .expect("Could not send crafting error event");
                    }
                    // run_craft simulates Ctrl+C and mouse input; the global hook
                    // queued those events while this thread was busy — drop them
                    // so our own input can't re-arm the combo
                    while rchan.try_recv().is_ok() {}
                    ctrl_held = false;
                }
                _ => {}
            }
        }
    });
}

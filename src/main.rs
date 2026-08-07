// use anyhow::Result;
use lazy_crafter::entities::craft_repo::Message;
use lazy_crafter::entities::craft_repo::{BackEvents, Data, ModsQuery, UiEvents, UiStates};
use log::{debug, error, info};
extern crate x11_clipboard;

use lazy_crafter::key_listener;
use lazy_crafter::storage::files::local_db::FileRepo;
use lazy_crafter::ui::ui_app;
use lazy_crafter::usecases::craft_searcher;
use lazy_crafter::usecases::estimation;
use lazy_crafter::utils::sync_ext::MutexLockSExt;
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::thread;

fn handle_event(
    ui_states: &Arc<Mutex<UiStates>>,
    data: &Arc<Mutex<Data>>,
    event: UiEvents,
    craft_repo: &FileRepo,
) -> Result<(), String> {
    if event == UiEvents::Started || event == UiEvents::ChangeGameVersion {
        let item_classes = craft_searcher::get_item_classes(craft_repo);
        let item_class_by_base_name = craft_searcher::get_item_class_by_item_name(craft_repo);
        let data = &mut data.lock_s()?;
        data.item_classes = item_classes;
        data.item_class_by_base_name = item_class_by_base_name;
        debug!(target: "db thread", "Loaded item classes by stat event");
    }
    let ui_state = ui_states.lock_s()?;
    info!(target: "db thread", "Got event, ui_state is {:?}", ui_state);

    let item_class = &ui_state.selected_item_class_as_filter;
    let item_bases = craft_searcher::get_item_bases(craft_repo, &item_class);

    let query = ModsQuery {
        string_query: ui_state.filter_string.clone(),
        item_base: ui_state.selected_item_base_as_filter.clone(),
        item_level: ui_state.selected_item_level_as_filter,
        selected_mods: ui_state.selected.clone(),
        match_mode: ui_state.selected_mods_match_mode,
    };
    drop(ui_state);
    let mod_items = craft_searcher::find_mods(craft_repo, &query);
    let estimation = estimation::calculate_estimation_for_craft(craft_repo, &query);
    let data = &mut data.lock_s()?;
    data.item_bases = item_bases;
    data.estimation = Some(estimation);
    data.mods_table = mod_items;
    debug!(target: "db thread", "Loaded item bases and filtered mods");
    Ok(())
}

/// The default UI selection ("Helmet" / "Iron Hat") is poe1-specific. When a
/// different dataset (e.g. poe2) is loaded those names may not exist, so fall
/// back to the first available class and base to avoid an empty initial view.
fn ensure_valid_default_selection(craft_repo: &FileRepo, ui_states: &Arc<Mutex<UiStates>>) {
    let classes = craft_searcher::get_item_classes(craft_repo);
    if classes.is_empty() {
        return;
    }
    let mut state = ui_states.lock().unwrap();
    let class_ok = classes
        .iter()
        .any(|c| c == &state.selected_item_class_as_filter);
    let base_ok = class_ok
        && craft_searcher::get_item_bases(craft_repo, &state.selected_item_class_as_filter)
            .iter()
            .any(|b| b.name == state.selected_item_base_as_filter);
    if base_ok {
        return;
    }
    let class = classes[0].clone();
    let base = craft_searcher::get_item_bases(craft_repo, &class)
        .into_iter()
        .next()
        .map(|b| b.name);
    state.selected_item_class_as_filter = class;
    if let Some(b) = base {
        state.selected_item_base_as_filter = b;
    }
    info!(
        target: "db thread",
        "adjusted default selection to {} / {}",
        state.selected_item_class_as_filter, state.selected_item_base_as_filter
    );
}

/// Load the dataset for the version currently selected in the UI. Returns None
/// (with an error message pushed to the UI) when the data directory is broken,
/// so the app still starts and shows the error.
fn init_craft_repo(ui_states: &Arc<Mutex<UiStates>>) -> Option<Arc<RwLock<FileRepo>>> {
    let version = ui_states.lock().unwrap().selected_game_version;
    match FileRepo::new_for_version(version) {
        Ok(repo) => Some(Arc::new(RwLock::new(repo))),
        Err(e) => {
            error!(target: "db thread", "Database initialization error! {}", e);
            ui_states.lock().unwrap().messages.push(Message {
                text: format!("Database initialization error! {}", e),
                created_at: chrono::Local::now().timestamp(),
            });
            None
        }
    }
}

fn run_db_in_background(
    receiver: mpsc::Receiver<UiEvents>,
    ui_states: Arc<Mutex<UiStates>>,
    data: Arc<Mutex<Data>>,
    craft_repo: Arc<RwLock<FileRepo>>,
) {
    ensure_valid_default_selection(&craft_repo.read().unwrap(), &ui_states);

    thread::spawn(move || {
        let mut current_version = ui_states.lock().unwrap().selected_game_version;
        for event in &receiver {
            if event == UiEvents::ChangeGameVersion {
                let version = ui_states.lock().unwrap().selected_game_version;
                if version != current_version {
                    match FileRepo::new_for_version(version) {
                        Ok(new_repo) => {
                            *craft_repo.write().unwrap() = new_repo;
                            current_version = version;
                            ui_states.lock().unwrap().selected.clear();
                            ensure_valid_default_selection(&craft_repo.read().unwrap(), &ui_states);
                            info!(target: "db thread", "switched dataset to {}", version.label());
                        }
                        Err(e) => {
                            error!(target: "db thread", "failed to load {} dataset: {}", version.label(), e);
                            // revert the UI toggle: the old dataset stays active
                            let ui_states = &mut ui_states.lock().unwrap();
                            ui_states.selected_game_version = current_version;
                            ui_states.messages.push(Message {
                                text: format!("Failed to load {} dataset: {}", version.label(), e),
                                created_at: chrono::Local::now().timestamp(),
                            });
                            continue;
                        }
                    }
                }
            }
            match handle_event(&ui_states, &data, event, &craft_repo.read().unwrap()) {
                Ok(_) => (),
                Err(e) => {
                    error!(target: "db thread", "{}", e);
                    return;
                }
            };
        }
    });
    info!("db started");
}

fn main() {
    // ui works in main tread
    // db loader works in another thread and wait events from main tread
    lazy_crafter::logging::init();
    info!("Start app");
    let (ui_tx, ui_rx): (mpsc::Sender<UiEvents>, mpsc::Receiver<UiEvents>) = mpsc::channel();
    let (back_tx, back_rx): (mpsc::Sender<BackEvents>, mpsc::Receiver<BackEvents>) =
        mpsc::channel();

    let data = Arc::new(Mutex::new(Data::default()));
    let ui_states = Arc::new(Mutex::new(UiStates::default()));

    if let Some(craft_repo) = init_craft_repo(&ui_states) {
        run_db_in_background(
            ui_rx,
            Arc::clone(&ui_states),
            Arc::clone(&data),
            Arc::clone(&craft_repo),
        );
        key_listener::run_listener_in_background(back_tx, Arc::clone(&ui_states), craft_repo);
    }
    info!("start ui");
    ui_app::run_ui_in_main_thread(ui_tx, back_rx, ui_states, data);
    // the window may be closed mid-craft while Shift+Alt are held down
    key_listener::release_all_modifiers();
    info!("released modifiers on exit");
}

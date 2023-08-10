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
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

fn handle_event(
    ui_states: &Arc<Mutex<UiStates>>,
    data: &Arc<Mutex<Data>>,
    event: UiEvents,
    craft_repo: &FileRepo,
) -> Result<(), String> {
    if event == UiEvents::Started {
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

fn run_db_in_background(
    receiver: mpsc::Receiver<UiEvents>,
    ui_states: Arc<Mutex<UiStates>>,
    data: Arc<Mutex<Data>>,
) {
    let craft_repo: FileRepo;
    match FileRepo::new() {
        Ok(repo) => {
            craft_repo = repo;
        }
        Err(e) => {
            error!(target: "db thread", "Database initialization error! {}", e);
            let ui_states = &mut ui_states.lock().unwrap();
            let message = Message {
                text: format!("Database initialization error! {}", e),
                created_at: chrono::Local::now().timestamp(),
            };

            ui_states.messages.push(message);
            return ();
        }
    }

    thread::spawn(move || loop {
        for event in &receiver {
            match handle_event(&ui_states, &data, event, &craft_repo) {
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
    env_logger::init();
    info!("Start app");
    let (ui_tx, ui_rx): (mpsc::Sender<UiEvents>, mpsc::Receiver<UiEvents>) = mpsc::channel();
    let (back_tx, back_rx): (mpsc::Sender<BackEvents>, mpsc::Receiver<BackEvents>) =
        mpsc::channel();

    let data = Arc::new(Mutex::new(Data::default()));
    let ui_states = Arc::new(Mutex::new(UiStates::default()));

    run_db_in_background(ui_rx, Arc::clone(&ui_states), Arc::clone(&data));
    key_listener::run_listener_in_background(back_tx, Arc::clone(&ui_states));
    info!("start ui");
    ui_app::run_ui_in_main_thread(ui_tx, back_rx, ui_states, data);
}

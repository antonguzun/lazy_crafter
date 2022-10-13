use lazy_crafter::entities::craft_repo::{Data, ModsQuery, UiEvents, UiStates};
extern crate x11_clipboard;

use lazy_crafter::storage::files::local_db::FileRepo;
use lazy_crafter::ui::ui_app;
use lazy_crafter::usecases::craft_searcher;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

fn run_db_in_background(
    receiver: mpsc::Receiver<UiEvents>,
    ui_states: Arc<Mutex<UiStates>>,
    data: Arc<Mutex<Data>>,
) {
    let ui_states_clone = ui_states.clone();
    let data_clone = data.clone();
    let craft_repo = FileRepo::new().unwrap();

    thread::spawn(move || loop {
        for _received in &receiver {
            let d = ui_states_clone.lock().unwrap();
            println!("Got event, ui_state is {:?}", d);

            let query = ModsQuery {
                string_query: d.filter_string.clone(),
                item_base: d.selected_item_base_as_filter.clone(),
                item_level: d.selected_item_level_as_filter,
                selected_mods: d.selected.clone(),
            };

            let mod_items = craft_searcher::find_mods(&craft_repo, &query);
            let mods_table = &mut data_clone.lock().unwrap().mods_table;
            mods_table.clear();
            mods_table.extend(mod_items);
        }
    });
}

fn main() {
    // ui works in main tread
    // db loader works in another thread and wait events from main tread

    let (tx, rx): (mpsc::Sender<UiEvents>, mpsc::Receiver<UiEvents>) = mpsc::channel();

    let data = Arc::new(Mutex::new(Data::default()));
    let ui_states = Arc::new(Mutex::new(UiStates::default()));

    run_db_in_background(rx, ui_states.clone(), data.clone());
    ui_app::run_ui_in_main_tread(tx, ui_states, data);
}

use lazy_crafter::entities::craft_repo::{Data, ModsQuery, UiEvents, UiStates};
extern crate x11_clipboard;

use lazy_crafter::storage::files::local_db::FileRepo;
use lazy_crafter::ui::EguiApp;
use lazy_crafter::usecases::craft_searcher;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

fn main() {
    let (sender, rx): (mpsc::Sender<UiEvents>, mpsc::Receiver<UiEvents>) = mpsc::channel();

    let data = Arc::new(Mutex::new(Data {
        mods_table: Vec::new(),
    }));
    let ui_states = Arc::new(Mutex::new(UiStates {
        filter_string: "".to_string(),
        selected: vec![],
        selected_item_tag_as_filter: "Helmet".to_string(),
        selected_item_level_as_filter: 100,
    }));
    let ui_states_clone = ui_states.clone();
    let data_clone = data.clone();
    let craft_repo = FileRepo::new().unwrap();

    thread::spawn(move || loop {
        for _received in &rx {
            let d = ui_states_clone.lock().unwrap();
            println!("Got event, ui_state is {:?}", d);

            let query = ModsQuery {
                string_query: d.filter_string.clone(),
                item_class: d.selected_item_tag_as_filter.clone(),
                item_level: d.selected_item_level_as_filter,
                selected_mods: d.selected.clone(),
            };

            let mod_items = craft_searcher::find_mods(&craft_repo, &query);
            let mods_table = &mut data_clone.lock().unwrap().mods_table;
            mods_table.clear();
            mods_table.extend(mod_items);
        }
    });
    sender.send(UiEvents::Started).unwrap();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Lazy Crafter",
        native_options,
        Box::new(|cc| Box::new(EguiApp::new(cc, ui_states, data, sender))),
    );
}

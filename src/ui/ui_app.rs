use crate::entities::craft_repo::{Data, UiEvents, UiStates};

use crate::input_schemas::parse_item_level;
use crate::storage::files::local_db::FileRepo;
use crate::ui::{buttons, comboboxes, inputs, tables};
use crate::usecases::craft_searcher;
use eframe::egui;

use std::sync::{mpsc, Arc, Mutex};

const APP_NAME: &str = "Lazy Crafter";

pub fn run_ui_in_main_tread(
    sender: mpsc::Sender<UiEvents>,
    ui_states: Arc<Mutex<UiStates>>,
    data: Arc<Mutex<Data>>,
) {
    sender.send(UiEvents::Started).unwrap();
    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(egui::Vec2 {
        x: 1300.0,
        y: 600.0,
    });
    eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(|cc| Box::new(EguiApp::new(cc, ui_states, data, sender))),
    );
}

pub struct EguiApp {
    ui_states: Arc<Mutex<UiStates>>,
    craft_repo: FileRepo,
    data: Arc<Mutex<Data>>,
    event_tx: mpsc::Sender<UiEvents>,
}

impl EguiApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        ui_states: Arc<Mutex<UiStates>>,
        data: Arc<Mutex<Data>>,
        event_tx: mpsc::Sender<UiEvents>,
    ) -> Self {
        Self {
            ui_states,
            craft_repo: FileRepo::new().unwrap(),
            data,
            event_tx,
        }
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let item_classes = craft_searcher::get_item_classes(&self.craft_repo);
        let item_class = &self
            .ui_states
            .lock()
            .unwrap()
            .selected_item_class_as_filter
            .clone();
        let item_bases = craft_searcher::get_item_bases(&self.craft_repo, &item_class);

        egui::SidePanel::left("input_panel").show(ctx, |ui| {
            ui.heading("Input");
            ui.set_min_width(200.0);

            comboboxes::show_combobox_with_classes(
                ui,
                item_classes,
                &self.ui_states,
                &self.event_tx,
            );
            comboboxes::show_combobox_with_bases(ui, item_bases, &self.ui_states, &self.event_tx);
            // show_level_input(ui, item_bases, &self.ui_states, &self.event_tx);
            // show_item_input(ui, item_bases, &self.ui_states, &self.event_tx);

            ui.label("item lvl");
            if ui
                .text_edit_singleline(&mut self.ui_states.lock().unwrap().item_level)
                .changed()
            {
                let state = &mut self.ui_states.lock().unwrap();
                match parse_item_level(&state.item_level) {
                    Ok(level) => {
                        // let state = &mut self.ui_states.lock().unwrap();
                        state.selected_item_level_as_filter = level as u64;
                        self.event_tx.send(UiEvents::ChangeModFilter).unwrap();
                    }
                    Err(_) => (),
                }
            };

            ui.label("or paste item");
            if ui
                .code_editor(&mut self.ui_states.lock().unwrap().item_string)
                .changed()
            {
                // self.event_tx.send(UiEvents::ChangeModFilter).unwrap();
            };
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Mods");
            ui.set_min_width(450.0);
            ui.horizontal(|ui| {
                let filter_string = &mut self.ui_states.lock().unwrap().filter_string;
                ui.label("filter: ");
                inputs::show_mods_filter_input(ui, filter_string, &self.event_tx);
            });
            let mod_items = self.data.lock().unwrap().mods_table.clone();
            let selected_mods = &mut self.ui_states.lock().unwrap().selected;
            tables::show_table_of_filtered_mods(ui, mod_items, selected_mods, &self.event_tx);
        });

        egui::SidePanel::right("selected_mods_panel").show(ctx, |ui| {
            ui.heading("Selected");
            ui.set_min_width(450.0);
            let _selected_mods = self.ui_states.lock().unwrap().selected.clone();
            tables::show_table_of_selected(ui, self.ui_states.lock().unwrap().selected.clone());
            let selected_mods = &mut self.ui_states.lock().unwrap().selected;
            buttons::show_cleaning_selected_mods_button(ui, selected_mods, &self.event_tx);
        });
    }
}

// Rarity: Magic
// Crafted Item
// Iron Hat
// --------
// Quality: +20% (augmented)
// Armour: 10
// --------
// Requirements:
// Str: 9
// --------
// Item Level: 83
// --------
// +17 to maximum Life
// 19% increased Rarity of Items found
// --------

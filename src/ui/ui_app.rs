use crate::entities::craft_repo::{Data, UiEvents, UiStates};

use crate::input_schemas::parse_item_level;
use crate::ui::{buttons, comboboxes, inputs, tables};
use eframe::egui;
use std::sync::{mpsc, Arc, Mutex};

const APP_NAME: &str = "Lazy Crafter";

pub fn run_ui_in_main_thread(
    sender: mpsc::Sender<UiEvents>,
    ui_states: Arc<Mutex<UiStates>>,
    data: Arc<Mutex<Data>>,
) {
    sender.send(UiEvents::Started).unwrap();
    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(egui::Vec2 {
        x: 1100.0,
        y: 600.0,
    });
    eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(|cc| Box::new(EguiApp::new(cc, ui_states, data, sender))),
    );
}

struct EguiApp {
    ui_states: Arc<Mutex<UiStates>>,
    data: Arc<Mutex<Data>>,
    event_tx: mpsc::Sender<UiEvents>,
}

impl EguiApp {
    fn new(
        _cc: &eframe::CreationContext<'_>,
        ui_states: Arc<Mutex<UiStates>>,
        data: Arc<Mutex<Data>>,
        event_tx: mpsc::Sender<UiEvents>,
    ) -> Self {
        Self {
            ui_states,
            data,
            event_tx,
        }
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("input_panel").show(ctx, |ui| {
            ui.heading("Input");
            ui.set_min_width(200.0);

            let item_classes = self.data.lock().unwrap().item_classes.clone();
            comboboxes::show_combobox_with_classes(
                ui,
                item_classes,
                &self.ui_states,
                &self.event_tx,
            );
            let item_bases = self.data.lock().unwrap().item_bases.clone();
            comboboxes::show_combobox_with_bases(ui, item_bases, &self.ui_states, &self.event_tx);
            // show_level_input(ui, item_bases, &self.ui_states, &self.event_tx);
            ui.horizontal(|ui| {
                ui.set_max_width(150.0);
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
                ui.label("item lvl");
            });

            ui.label("or paste item");
            inputs::show_item_input(
                ui,
                &self.data.lock().unwrap().item_class_by_base_name,
                &self.ui_states,
                &self.event_tx,
            );
        });
        egui::SidePanel::right("selected_mods_panel").show(ctx, |ui| {
            ui.heading("Selected");
            let selected_mods = self.ui_states.lock().unwrap().selected.clone();
            let total_suff_weight: u32 = self
                .data
                .lock()
                .unwrap()
                .mods_table
                .iter()
                .filter(|m| m.generation_type == String::from("prefix"))
                .map(|m| m.weight)
                .sum();
            let total_suff_weight: f64 = total_suff_weight.try_into().unwrap();
            let total_pref_weight: u32 = self
                .data
                .lock()
                .unwrap()
                .mods_table
                .iter()
                .filter(|m| m.generation_type == String::from("suffix"))
                .map(|m| m.weight)
                .sum();
            let total_pref_weight: f64 = total_pref_weight.try_into().unwrap();
            let estimate: f64 = selected_mods
                .iter()
                .map(|m| match m.generation_type.as_str() {
                    "prefix" => m.weight as f64 / total_pref_weight,
                    "suffix" => m.weight as f64 / total_suff_weight,
                    _ => {
                        panic!("lke")
                    }
                })
                .product();
            // let estimate: f64 = 0.0;
            ui.label(format!("estimate ~{}%", estimate * 100.0));

            tables::show_table_of_selected(ui, selected_mods);
            let selected_mods = &mut self.ui_states.lock().unwrap().selected;
            buttons::show_cleaning_selected_mods_button(ui, selected_mods, &self.event_tx);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Mods");
            ui.horizontal(|ui| {
                let filter_string = &mut self.ui_states.lock().unwrap().filter_string;
                ui.label("filter: ");
                inputs::show_mods_filter_input(ui, filter_string, &self.event_tx);
            });

            let mod_items = self.data.lock().unwrap().mods_table.clone();
            let selected_mods = &mut self.ui_states.lock().unwrap().selected;
            tables::show_table_of_filtered_mods(ui, mod_items, selected_mods, &self.event_tx);
        });
    }
}

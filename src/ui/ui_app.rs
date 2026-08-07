use crate::entities::craft_repo::{
    BackEvents, Data, GameVersion, Message, ModsMatchMode, UiEvents, UiStates,
};

use crate::input_schemas::{parse_item_level, parse_max_tries};
use crate::ui::{buttons, comboboxes, errors, inputs, tables};
// use anyhow::Result;
use chrono;
use eframe::egui;
use egui::{Visuals, Style};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

const APP_NAME: &str = "Lazy Crafter";

pub fn run_ui_in_main_thread(
    sender: mpsc::Sender<UiEvents>,
    receiver: mpsc::Receiver<BackEvents>,
    ui_states: Arc<Mutex<UiStates>>,
    data: Arc<Mutex<Data>>,
) {
    sender.send(UiEvents::Started);
    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(egui::Vec2 {
        x: 1100.0,
        y: 600.0,
    });

    let ui_states_clone = Arc::clone(&ui_states);
    thread::spawn(move || {
        for event in receiver.iter() {
            match event {
                BackEvents::Error(err) => {
                    ui_states_clone.lock().unwrap().messages.push(Message {
                        text: err.to_string(),
                        created_at: chrono::Local::now().timestamp(),
                    });
                }
                _ => (),
            };
        }
    });

    eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(|cc| {
            let style: Style = Style {
                visuals: Visuals::dark(),
                ..Style::default()
            };
            cc.egui_ctx.set_style(style);
            Box::new(
            EguiApp::new(cc, ui_states, data, sender))}
        ),
    );
}

struct EguiApp {
    ui_states: Arc<Mutex<UiStates>>,
    data: Arc<Mutex<Data>>,
    event_tx: mpsc::Sender<UiEvents>,
    combobox_filter_query: String,
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "fontin".to_owned(),
        egui::FontData::from_static(include_bytes!("../../assets/Fontin-Regular.ttf")),
    );

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "fontin".to_owned());
    ctx.set_fonts(fonts);
}

impl EguiApp {
    fn new(
        cc: &eframe::CreationContext<'_>,
        ui_states: Arc<Mutex<UiStates>>,
        data: Arc<Mutex<Data>>,
        event_tx: mpsc::Sender<UiEvents>,
    ) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        Self {
            ui_states,
            data,
            event_tx,
            combobox_filter_query: String::new(),
        }
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("input_panel").show(ctx, |ui| {
            ui.heading("Input");
            ui.set_min_width(200.0);

            ui.horizontal(|ui| {
                ui.label("game:");
                let changed = {
                    let state = &mut self.ui_states.lock().unwrap();
                    let poe1 = ui.selectable_value(
                        &mut state.selected_game_version,
                        GameVersion::Poe1,
                        GameVersion::Poe1.label(),
                    );
                    let poe2 = ui.selectable_value(
                        &mut state.selected_game_version,
                        GameVersion::Poe2,
                        GameVersion::Poe2.label(),
                    );
                    let changed = poe1.changed() || poe2.changed();
                    if changed {
                        // selected mods belong to the previous dataset
                        state.selected.clear();
                    }
                    changed
                };
                if changed {
                    self.event_tx.send(UiEvents::ChangeGameVersion).unwrap();
                }
            });
            ui.separator();

            let item_classes = self.data.lock().unwrap().item_classes.clone();
            comboboxes::show_combobox_with_classes(
                ui,
                item_classes,
                &self.ui_states,
                &self.event_tx,
                &mut self.combobox_filter_query,
            );
            let item_bases = self.data.lock().unwrap().item_bases.clone();
            comboboxes::show_combobox_with_bases(
                ui,
                item_bases,
                &self.ui_states,
                &self.event_tx,
                &mut self.combobox_filter_query,
            );
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
            ui.label("Max autocraft tries:");
            ui.horizontal(|ui| {
                ui.set_max_width(150.0);
                if ui
                    .text_edit_singleline(&mut self.ui_states.lock().unwrap().max_autocraft_tries)
                    .changed()
                {
                    let state = &mut self.ui_states.lock().unwrap();
                    match parse_max_tries(&state.max_autocraft_tries) {
                        Ok(max) => {
                            // let state = &mut self.ui_states.lock().unwrap();
                            state.selected_max_autocraft_tries = max as u64;
                        }
                        Err(_) => (),
                    }
                };
            });

            ui.heading("Selected");
            ui.horizontal(|ui| {
                ui.label("match:");
                // the mode decides which mods stay offered in the table (in OR
                // mode conflicting alternatives do), so refilter on change
                let changed = {
                    let mode = &mut self.ui_states.lock().unwrap().selected_mods_match_mode;
                    let all = ui
                        .selectable_value(mode, ModsMatchMode::All, ModsMatchMode::All.label())
                        .on_hover_text("stop when the item has ALL selected mods");
                    let any = ui
                        .selectable_value(mode, ModsMatchMode::Any, ModsMatchMode::Any.label())
                        .on_hover_text("stop when the item has ANY of the selected mods");
                    all.changed() || any.changed()
                };
                if changed {
                    self.event_tx.send(UiEvents::ChangeModFilter).unwrap();
                }
            });
            let selected_mods = self.ui_states.lock().unwrap().selected.clone();

            // estimations removed from ui while it not ready
            // let total_pref_weight: u32 = self
            //     .data
            //     .lock()
            //     .unwrap()
            //     .mods_table
            //     .iter()
            //     .filter(|m| m.generation_type == String::from("prefix"))
            //     .map(|m| m.weight)
            //     .sum();
            // let total_suff_weight: u32 = self
            //     .data
            //     .lock()
            //     .unwrap()
            //     .mods_table
            //     .iter()
            //     .filter(|m| m.generation_type == String::from("suffix"))
            //     .map(|m| m.weight)
            //     .sum();
            // let total_suff_weight: f64 = total_suff_weight.try_into().unwrap();
            // let total_pref_weight: f64 = total_pref_weight.try_into().unwrap();
            // let estimation = &self.data.lock().unwrap().estimation;
            // ui.label(format!("total pref weight: {}", total_pref_weight));
            // ui.label(format!("total suff weight: {}", total_suff_weight));
            // match estimation {
            //     Some(r) => match r {
            //         Ok(est) => {
            //             ui.label(format!("estimate ~ {}%", est.probability * 100.0));
            //         }
            //         Err(err) => {
            //             ui.label(format!("Error during estimate: {}", err));
            //         }
            //     },
            //     None => (),
            // }

            tables::show_table_of_selected(ui, selected_mods);

            buttons::show_cleaning_selected_mods_button(
                ui,
                &mut self.ui_states.lock().unwrap().selected,
                &self.event_tx,
            );

            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                errors::show_errors(ui, &mut self.ui_states.lock().unwrap().messages);
            });
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
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
        ()
    }
}

use crate::entities::craft_repo::{Data, ModItem, UiEvents, UiStates};

extern crate x11_clipboard;
use crate::input_schemas::parse_item_level;
use crate::storage::files::local_db::FileRepo;
use crate::usecases::craft_searcher;
use eframe::egui;
use egui::Sense;
use egui_extras::{Size, TableBuilder};
use std::sync::{mpsc, Arc, Mutex};

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

fn calculate_row_height(row: &ModItem, one_row_height: f32) -> f32 {
    let cnt_of_rows = &row.representation.chars().filter(|&c| c == '\n').count();
    let cnt_of_rows = u16::try_from(cnt_of_rows.clone()).ok().unwrap_or(10);
    (cnt_of_rows + 1) as f32 * one_row_height
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
        // println!("bases: {:?}", item_bases);

        egui::SidePanel::left("input_panel").show(ctx, |ui| {
            ui.heading("Input");
            ui.set_min_width(200.0);

            egui::ComboBox::from_label("item class")
                .selected_text(format!(
                    "{:?}",
                    &mut self.ui_states.lock().unwrap().selected_item_class_as_filter
                ))
                .show_ui(ui, |ui| {
                    item_classes.iter().for_each(|i| {
                        if ui
                            .selectable_value(
                                &mut self.ui_states.lock().unwrap().selected_item_class_as_filter,
                                i.to_string(),
                                i.to_string(),
                            )
                            .changed()
                        {
                            let selected = &mut self.ui_states.lock().unwrap().selected;
                            selected.clear();
                            self.event_tx.send(UiEvents::AddToSelectedMods).unwrap();
                        };
                    });
                });
            egui::ComboBox::from_label("item base")
                .selected_text(format!(
                    "{:?}",
                    &mut self.ui_states.lock().unwrap().selected_item_base_as_filter
                ))
                .show_ui(ui, |ui| {
                    item_bases.iter().for_each(|i| {
                        if ui
                            .selectable_value(
                                &mut self.ui_states.lock().unwrap().selected_item_base_as_filter,
                                i.name.to_string(),
                                format!("{} {}", i.name.to_string(), i.required_level.to_string()),
                            )
                            .changed()
                        {
                            let state = &mut self.ui_states.lock().unwrap();
                            state.selected.clear();
                            state.selected_item_level_as_filter = i.required_level;
                            state.item_level = i.required_level.to_string();
                            self.event_tx.send(UiEvents::AddToSelectedMods).unwrap();
                        };
                    });
                });

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
        egui::SidePanel::right("selected_mods_panel").show(ctx, |ui| {
            ui.heading("Selected");
            ui.set_min_width(450.0);
            let selected_table = TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Size::initial(70.0).at_least(70.0))
                .column(Size::remainder().at_least(300.0))
                .resizable(false);

            selected_table
                .header(30.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("weight");
                    });
                    header.col(|ui| {
                        ui.heading("modification");
                    });
                })
                .body(|mut body| {
                    let selected = self.ui_states.lock().unwrap().selected.clone();
                    for row_index in 0..selected.len() {
                        let row_height = calculate_row_height(&selected[row_index], 18.0);
                        body.row(row_height, |mut row| {
                            row.col(|ui| {
                                ui.label((&selected[row_index].weight).to_string());
                            });
                            let label = egui::Label::new(&selected[row_index].representation)
                                .wrap(false)
                                .sense(Sense::click());
                            row.col(|ui| {
                                if ui.add(label).clicked() {
                                    // &self.selected.retain(|x| x == &self.selected[row_index].clone());
                                };
                            });
                        });
                    }
                });
            if ui.button("clean selected").clicked() {
                let selected = &mut self.ui_states.lock().unwrap().selected;
                selected.clear();
                self.event_tx.send(UiEvents::CleanSelectedMods).unwrap();
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Mods");
            ui.set_min_width(450.0);
            ui.horizontal(|ui| {
                ui.label("filter: ");
                if ui
                    .text_edit_singleline(&mut self.ui_states.lock().unwrap().filter_string)
                    .changed()
                {
                    self.event_tx.send(UiEvents::ChangeModFilter).unwrap();
                };
            });

            let mod_items = self.data.lock().unwrap().mods_table.clone();

            let table = TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Size::initial(30.0).at_least(50.0))
                .column(Size::initial(30.0).at_least(50.0))
                .column(Size::remainder().at_least(450.0))
                .resizable(false);

            table
                .header(30.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("#");
                    });
                    header.col(|ui| {
                        ui.heading("weight");
                    });
                    header.col(|ui| {
                        ui.heading("modification");
                    });
                })
                .body(|mut body| {
                    for row_index in 0..mod_items.len() {
                        let row_height = calculate_row_height(&mod_items[row_index], 18.0);
                        body.row(row_height, |mut row| {
                            row.col(|ui| {
                                ui.label((row_index + 1).to_string());
                            });
                            row.col(|ui| {
                                ui.label(&mod_items[row_index].weight.to_string());
                            });
                            let label = egui::Label::new(&mod_items[row_index].representation)
                                .wrap(false)
                                .sense(Sense::click());
                            row.col(|ui| {
                                if ui.add(label).clicked() {
                                    let selected = &mut self.ui_states.lock().unwrap().selected;
                                    selected.push(mod_items[row_index].clone());
                                    println!("selected mods: {:?}", &selected);
                                    self.event_tx.send(UiEvents::AddToSelectedMods).unwrap();
                                };
                            });
                        });
                    }
                });
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

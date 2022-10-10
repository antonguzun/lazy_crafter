use crate::entities::craft_repo::{Data, ModItem, ModsQuery, UiStates};

extern crate x11_clipboard;
use crate::storage::files::local_db::FileRepo;
use crate::usecases::craft_searcher;
use eframe::egui;
use egui::Sense;
use egui_extras::{Size, TableBuilder};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct EguiApp {
    ui_states: Arc<Mutex<UiStates>>,
    craft_repo: FileRepo,
    data: Arc<Mutex<Data>>,
    event_tx: mpsc::Sender<String>,
}

impl EguiApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        ui_states: Arc<Mutex<UiStates>>,
        data: Arc<Mutex<Data>>,
        event_tx: mpsc::Sender<String>,
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

        egui::SidePanel::left("selected_mods_panel").show(ctx, |ui| {
            ui.heading("Selected");
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
                self.event_tx.send("selected changed".to_string());
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Mods");
            ui.horizontal(|ui| {
                ui.label("filter: ");
                if ui
                    .text_edit_singleline(&mut self.ui_states.lock().unwrap().filter_string)
                    .changed()
                {
                    self.event_tx.send("filter changed".to_string());
                };

                let base_combobox = egui::ComboBox::from_label("Select one!")
                    .selected_text(format!(
                        "{:?}",
                        &mut self.ui_states.lock().unwrap().selected_item_tag_as_filter
                    ))
                    .show_ui(ui, |ui| {
                        item_classes.iter().for_each(|i| {
                            if ui
                                .selectable_value(
                                    &mut self.ui_states.lock().unwrap().selected_item_tag_as_filter,
                                    i.to_string(),
                                    i.to_string(),
                                )
                                .changed()
                            {
                                self.event_tx.send("selected changed".to_string());
                            };
                        });
                    });
            });

            let mod_items = self.data.lock().unwrap().mods_table.clone();

            let table = TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Size::initial(30.0).at_least(50.0))
                .column(Size::initial(30.0).at_least(50.0))
                .column(Size::remainder().at_least(300.0))
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
                                    self.event_tx.send("selected changed".to_string());
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

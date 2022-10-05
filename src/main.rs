use lazy_crafter::entities::craft_repo::{ModItem, ModsQuery};

extern crate x11_clipboard;
use eframe::egui;
use egui::Sense;
use egui_extras::{Size, TableBuilder};
use lazy_crafter::storage::files::local_db::FileRepo;
use lazy_crafter::usecases::craft_searcher;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Lazy Crafter",
        native_options,
        Box::new(|cc| Box::new(EguiApp::new(cc))),
    );
}

struct EguiApp {
    name: String,
    selected: Vec<ModItem>,
    selected_item_tag_as_filter: String,
    selected_item_level_as_filter: u64,
    craft_repo: FileRepo,
}

impl EguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            name: "".to_string(),
            selected: vec![],
            selected_item_tag_as_filter: "Helmet".to_string(),
            selected_item_level_as_filter: 100,
            craft_repo: FileRepo::new().unwrap(),
        }
    }
}

fn calculate_row_height(row: &ModItem, one_row_height: f32) -> f32 {
    let cnt_of_rows = &row.representation.chars().filter(|&c| c == '\n').count();
    let cnt_of_rows = u16::try_from(cnt_of_rows.clone()).ok().unwrap_or(10);
    cnt_of_rows as f32 * one_row_height
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let item_classes = craft_searcher::get_item_classes(&self.craft_repo);

        egui::SidePanel::left("selected_mods_panel").show(ctx, |ui| {
            let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
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
                    for row_index in 0..self.selected.len() {
                        let row_height = calculate_row_height(&self.selected[row_index], 18.0);
                        body.row(row_height, |mut row| {
                            row.col(|ui| {
                                ui.label((&self.selected[row_index].weight).to_string());
                            });
                            let label = egui::Label::new(&self.selected[row_index].representation)
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
                self.selected.clear();
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Mods");
            ui.horizontal(|ui| {
                ui.label("filter: ");
                ui.text_edit_singleline(&mut self.name);

                egui::ComboBox::from_label("Select one!")
                    .selected_text(format!("{:?}", self.selected_item_tag_as_filter))
                    .show_ui(ui, |ui| {
                        item_classes.iter().for_each(|i| {
                            ui.selectable_value(
                                &mut self.selected_item_tag_as_filter,
                                i.to_string(),
                                i.to_string(),
                            );
                        });
                    });
            });

            let query = ModsQuery {
                string_query: self.name.clone(),
                item_class: self.selected_item_tag_as_filter.clone(),
                item_level: self.selected_item_level_as_filter,
                selected_mods: self.selected.clone(),
            };
            let mod_items = craft_searcher::find_mods(&self.craft_repo, &query);

            let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
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
                                    self.selected.push(mod_items[row_index].clone());
                                    println!("selected mods: {:?}", &self.selected);
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

use crate::entities::craft_repo::{ModItem, UiEvents};
use crate::ui::common::calculate_row_height;
use egui::{Sense, Ui};
use egui_extras::{Size, TableBuilder};
use std::sync::mpsc;

pub fn show_table_of_filtered_mods(
    ui: &mut Ui,
    rows: Vec<ModItem>,
    selected: &mut Vec<ModItem>,
    events_sender: &mpsc::Sender<UiEvents>,
) {
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
            for row_index in 0..rows.len() {
                let row_height = calculate_row_height(&rows[row_index], 18.0);
                body.row(row_height, |mut row| {
                    row.col(|ui| {
                        ui.label((row_index + 1).to_string());
                    });
                    row.col(|ui| {
                        ui.label(&rows[row_index].weight.to_string());
                    });
                    let label = egui::Label::new(&rows[row_index].representation)
                        .wrap(false)
                        .sense(Sense::click());
                    row.col(|ui| {
                        if ui.add(label).clicked() {
                            selected.push(rows[row_index].clone());
                            println!("selected mods: {:?}", &selected);
                            events_sender.send(UiEvents::AddToSelectedMods).unwrap();
                        };
                    });
                });
            }
        });
}

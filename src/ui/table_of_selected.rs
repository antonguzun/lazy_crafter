use crate::entities::craft_repo::ModItem;
use crate::ui::common::calculate_row_height;
use egui::{Sense, Ui};
use egui_extras::{Size, TableBuilder};

pub fn show_table_of_selected(ui: &mut Ui, rows: Vec<ModItem>) {
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
            for row_index in 0..rows.len() {
                let row_height = calculate_row_height(&rows[row_index], 18.0);
                body.row(row_height, |mut row| {
                    row.col(|ui| {
                        ui.label((&rows[row_index].weight).to_string());
                    });
                    let label = egui::Label::new(&rows[row_index].representation)
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
}

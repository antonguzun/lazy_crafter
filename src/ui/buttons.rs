use crate::entities::craft_repo::{ModItem, UiEvents};
use egui::{Sense, Ui};
use egui_extras::{Size, TableBuilder};
use std::sync::mpsc;

pub fn show_cleaning_selected_mods_button(
    ui: &mut Ui,
    selected_mods: &mut Vec<ModItem>,
    events_sender: &mpsc::Sender<UiEvents>,
) {
    if ui.button("clean selected").clicked() {
        selected_mods.clear();
        events_sender.send(UiEvents::CleanSelectedMods).unwrap();
    }
}

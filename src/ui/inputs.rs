use crate::entities::craft_repo::{ModItem, UiEvents};
use egui::{Sense, Ui};
use egui_extras::{Size, TableBuilder};
use std::sync::mpsc;

pub fn show_mods_filter_input(
    ui: &mut Ui,
    filter_string: &mut String,
    events_sender: &mpsc::Sender<UiEvents>,
) {
    if ui.text_edit_singleline(filter_string).changed() {
        events_sender.send(UiEvents::ChangeModFilter).unwrap();
    };
}

// show_level_input(ui, item_bases, &self.ui_states, &self.event_tx);
// show_item_input(ui, item_bases, &self.ui_states, &self.event_tx);

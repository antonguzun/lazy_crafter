use crate::entities::craft_repo::{ItemBase, UiEvents, UiStates};
use egui::Ui;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

pub fn show_combobox_with_bases(
    ui: &mut Ui,
    item_bases: Vec<ItemBase>,
    ui_states: &Arc<Mutex<UiStates>>,
    events_sender: &mpsc::Sender<UiEvents>,
) {
    egui::ComboBox::from_label("item base")
        .selected_text(format!(
            "{:?}",
            &mut ui_states.lock().unwrap().selected_item_base_as_filter
        ))
        .show_ui(ui, |ui| {
            item_bases.iter().for_each(|i| {
                if ui
                    .selectable_value(
                        &mut ui_states.lock().unwrap().selected_item_base_as_filter,
                        i.name.to_string(),
                        format!("{} {}", i.name.to_string(), i.required_level.to_string()),
                    )
                    .changed()
                {
                    let state = &mut ui_states.lock().unwrap();
                    state.selected.clear();
                    state.selected_item_level_as_filter = i.required_level;
                    state.item_level = i.required_level.to_string();
                    events_sender.send(UiEvents::AddToSelectedMods).unwrap();
                };
            });
        });
}

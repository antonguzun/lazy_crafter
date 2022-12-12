use crate::entities::craft_repo::{ItemBase, UiEvents, UiStates};
use egui::{Color32, Event, Key, RichText, Ui};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

pub fn show_combobox_with_bases(
    ui: &mut Ui,
    item_bases: Vec<ItemBase>,
    ui_states: &Arc<Mutex<UiStates>>,
    events_sender: &mpsc::Sender<UiEvents>,
    filter_query: &mut String,
) {
    let filter = filter_query.clone();
    egui::ComboBox::from_label("item base")
        .selected_text(format!(
            "{}",
            &mut ui_states.lock().unwrap().selected_item_base_as_filter
        ))
        .show_ui(ui, |ui| {
            if &filter.len() > &0 {
                ui.label(RichText::new(&filter).color(Color32::LIGHT_RED));
            }
            item_bases
                .iter()
                .filter(|v| v.name.to_lowercase().contains(filter.as_str()))
                .for_each(|i| {
                    let choices = ui.selectable_value(
                        &mut ui_states.lock().unwrap().selected_item_base_as_filter,
                        i.name.to_string(),
                        format!("{} {}", i.name.to_string(), i.required_level.to_string()),
                    );
                    if choices.changed() {
                        let state = &mut ui_states.lock().unwrap();
                        state.selected.clear();
                        state.selected_item_level_as_filter = i.required_level;
                        state.item_level = i.required_level.to_string();
                        events_sender.send(UiEvents::AddToSelectedMods).unwrap();
                        filter_query.clear();
                    };
                    if choices.clicked_elsewhere() {
                        filter_query.clear();
                    }
                });
            handle_events(ui, filter_query);
        });
}

pub fn show_combobox_with_classes(
    ui: &mut Ui,
    item_classes: Vec<String>,
    ui_states: &Arc<Mutex<UiStates>>,
    events_sender: &mpsc::Sender<UiEvents>,
    filter_query: &mut String,
) {
    let filter = filter_query.clone();
    egui::ComboBox::from_label("item class")
        .selected_text(format!(
            "{}",
            &mut ui_states.lock().unwrap().selected_item_class_as_filter
        ))
        .show_ui(ui, |ui| {
            if &filter.len() > &0 {
                ui.label(RichText::new(&filter).color(Color32::LIGHT_RED));
            }
            item_classes
                .iter()
                .filter(|v| v.to_lowercase().contains(filter.as_str()))
                .for_each(|i| {
                    let choices = ui.selectable_value(
                        &mut ui_states.lock().unwrap().selected_item_class_as_filter,
                        i.to_string(),
                        i.to_string(),
                    );
                    if choices.changed() {
                        let selected_mods = &mut ui_states.lock().unwrap().selected;
                        selected_mods.clear();
                        filter_query.clear();
                        events_sender.send(UiEvents::AddToSelectedMods).unwrap();
                    };
                    if choices.clicked_elsewhere() {
                        filter_query.clear();
                    };
                });
            handle_events(ui, filter_query);
        });
}

fn handle_events(ui: &mut Ui, text: &mut String) {
    let events = ui.input().events.clone();
    for event in events {
        match event {
            Event::Text(text_to_insert) => text.push_str(text_to_insert.to_lowercase().as_str()),
            Event::Key {
                key: Key::Backspace,
                pressed: true,
                modifiers: _,
            } => {
                text.pop();
            }
            Event::Key {
                key: Key::Escape | Key::Delete,
                pressed: true,
                modifiers: _,
            } => {
                text.clear();
            }
            _ => {}
        }
    }
}

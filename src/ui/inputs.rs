use crate::entities::craft_repo::{UiEvents, UiStates};
use egui::{Color32, RichText, Ui};
use log::{debug, error};
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, sync::mpsc};

pub fn show_mods_filter_input(
    ui: &mut Ui,
    filter_string: &mut String,
    events_sender: &mpsc::Sender<UiEvents>,
) {
    if ui.text_edit_singleline(filter_string).changed() {
        events_sender.send(UiEvents::ChangeModFilter).unwrap();
    };
}

/*
Rarity: Magic
Crafted Item
Iron Hat
--------
Quality: +20% (augmented)
Armour: 10
--------
Requirements:
Str: 9
--------
Item Level: 83
--------
+17 to maximum Life
19% increased Rarity of Items found
--------
*/
fn parse_item_name_from_string(string: String) -> Vec<String> {
    let r: Vec<&str> = string.split('\n').collect();
    r.iter().map(|s| s.trim().to_string()).collect()
}

fn parse_item_level_from_string(string: &str) -> Option<u64> {
    use regex::Regex;
    let re = Regex::new(r"Item Level: (\d{1,3})").unwrap();
    for cap in re.captures_iter(string) {
        let raw_level = &cap[1];
        debug!(target: "item level parser", "try {}", raw_level);
        match raw_level.parse::<u64>() {
            Ok(num) => {
                debug!(target: "item level parser", "success {}", num);
                return Some(num);
            }
            _ => {
                debug!(target: "item level parser", "failure {}", raw_level);
                continue;
            }
        }
    }
    None
}

pub fn show_item_input(
    ui: &mut Ui,
    classes_by_name: &HashMap<String, String>,
    ui_states: &Arc<Mutex<UiStates>>,
    events_sender: &mpsc::Sender<UiEvents>,
) {
    let str = ui_states.lock().unwrap().item_string.clone();
    let states = &mut ui_states.lock().unwrap();

    if ui.code_editor(&mut states.item_string).lost_focus() {
        match parse_item_level_from_string(&str) {
            Some(n) => {
                states.item_level = n.to_string();
                states.selected_item_level_as_filter = n;
            }
            None => {}
        }
        for item_name in parse_item_name_from_string(str) {
            match classes_by_name.get(&item_name) {
                Some(class_name) => {
                    states.selected_item_class_as_filter = class_name.clone();
                    states.selected_item_base_as_filter = item_name.to_string();
                    events_sender.send(UiEvents::InsertionItemData).unwrap();
                    return ();
                }
                None => {}
            }
        }
    };
}

// show_level_input(ui, item_bases, &self.ui_states, &self.event_tx);

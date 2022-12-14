use crate::entities::craft_repo::Message;
use egui::{Color32, RichText, Sense, Ui};
use log::debug;

const LOG_TARGET: &str = "ui";

pub fn show_errors(ui: &mut Ui, messages: &mut Vec<Message>) {
    if messages.len() > 0 {
        if ui.button("clean errors").clicked() {
            messages.clear();
        }
    }

    let texts = messages
        .iter()
        .map(|m| m.text.clone())
        .collect::<Vec<String>>();
    for (i, message) in texts.iter().enumerate() {
        let label = egui::Label::new(RichText::new(message).color(Color32::LIGHT_RED))
            .wrap(false)
            .sense(Sense::click());
        if ui.add(label).clicked() {
            messages.remove(i);
            debug!(target: LOG_TARGET, "removed error");
        };
    }
}

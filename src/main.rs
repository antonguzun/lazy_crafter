use chrono::{DateTime, Utc};
use eframe::egui::epaint::ahash::HashMap;
use lazy_crafter::entities::db::LocalDB;
use rdev::{listen, simulate, EventType, Key};
use std::collections::HashSet;
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, SystemTime};
extern crate x11_clipboard;
use eframe::egui;
use egui::Sense;
use egui_extras::{Size, TableBuilder};
use lazy_crafter::storage::files::local_db::FileRepo;
use x11_clipboard::Clipboard;

fn hash_event_type(event_type: EventType) -> String {
    format!("{:?}", &event_type)
}

fn create_target_hash_set() -> HashSet<String> {
    let mut target_events = HashSet::new();
    // target_events.insert(hash_event_type(EventType::KeyPress(Key::ControlLeft)));
    target_events.insert(hash_event_type(EventType::KeyRelease(Key::ControlLeft)));
    // target_events.insert(hash_event_type(EventType::KeyPress(Key::ShiftLeft)));
    target_events.insert(hash_event_type(EventType::KeyRelease(Key::Alt)));
    // target_events.insert(hash_event_type(EventType::KeyPress(Key::KeyD)));
    target_events.insert(hash_event_type(EventType::KeyRelease(Key::KeyC)));
    target_events
}

fn send(event_type: &EventType) {
    let delay = Duration::from_millis(20);
    match simulate(event_type) {
        Ok(()) => (),
        Err(_) => {
            println!("We could not send {:?}", event_type);
        }
    }
    // Let ths OS catchup (at least MacOS)
    thread::sleep(delay);
}

fn run_craft() {
    println!("run crafting");
    let clipboard = Clipboard::new().unwrap();

    send(&EventType::KeyPress(Key::ControlLeft));
    send(&EventType::KeyPress(Key::KeyC));
    send(&EventType::KeyRelease(Key::ControlLeft));
    send(&EventType::KeyRelease(Key::KeyC));

    println!("try copy");
    let val = clipboard
        .load(
            clipboard.setter.atoms.clipboard,
            clipboard.setter.atoms.string,
            clipboard.setter.atoms.property,
            Duration::from_millis(300),
        )
        .unwrap();
    let val = String::from_utf8(val).unwrap();

    println!("{}", val);
}

// fn main() {
//     let db = FileRepo::new().unwrap();
//     println!("db initialized");
//     println!(
//         "{:?}",
//         db.translations.get("+1_max_charged_attack_stages").unwrap()
//     );

//     let (schan, rchan) = channel();
//     let _listener = thread::spawn(move || {
//         listen(move |event| {
//             schan
//                 .send(event)
//                 .unwrap_or_else(|e| println!("Could not send event {:?}", e));
//         })
//         .expect("Could not listen");
//     });

//     let keypress_bandwidth = Duration::from_millis(1000);
//     let mut events = Vec::new();
//     let target_events = create_target_hash_set();
//     println!("target_events {:?}", target_events);
//     let mut last_combo = SystemTime::now() - Duration::from_secs(500);

//     for event in rchan.iter() {
//         events.push(event);
//         events.retain(|e| e.time > SystemTime::now() - keypress_bandwidth);
//         let current_events =
//             HashSet::from_iter(events.iter().map(|e| hash_event_type(e.event_type)));
//         if target_events.is_subset(&current_events)
//             && last_combo < SystemTime::now() - keypress_bandwidth
//         {
//             let t: DateTime<Utc> = last_combo.clone().into();
//             println!("You pressed combo! prev combo at {}", t.to_rfc3339());
//             last_combo = SystemTime::now();
//             events.clear();
//             run_craft();
//             let delay = Duration::from_millis(100);
//             thread::sleep(delay);
//         }
//     }
// }

struct SubDb {
    item_level: u64,
    weight: u32,
    // max: i32,
    // min: i32,
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    let db = FileRepo::new().unwrap();
    eframe::run_native(
        "Lazy Crafter",
        native_options,
        Box::new(|cc| Box::new(MyEguiApp::new(cc, db))),
    );
}

struct MyEguiApp {
    name: String,
    selected: Vec<String>,
    selected_item_tag_as_filter: String,
    selected_item_level_as_filter: u32,
    db: LocalDB,
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>, db: LocalDB) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self {
            name: "".to_string(),
            selected: vec![],
            selected_item_tag_as_filter: "helmet".to_string(),
            selected_item_level_as_filter: 100,
            db,
        }
    }
}

fn get_list(filter: &str, db: &LocalDB, item_type: &str) -> std::vec::Vec<String> {
    let filter = filter.trim().to_lowercase();
    let filters: Vec<&str> = filter.split(' ').collect();
    let mut v1: Vec<String> = vec![];
    let mut v2: Vec<String> = vec![];
    for (k, m) in db.search_map.get(item_type).unwrap().iter() {
        let verbose_str = match db.translations.get(k) {
            Some(t) => t.English[0].get_representation_string(),
            None => k.to_string(),
        };
        if verbose_str.to_lowercase().contains(&filter) {
            v1.push(verbose_str);
        } else if filters.iter().all(|f| verbose_str.to_lowercase().contains(&*f)) {
            v2.push(verbose_str);
        }

    }
    // for (i, st) in db.translations.iter() {
    //     let val = st.English[0].get_representation_string();
    //     if val.to_lowercase().contains(&filter) {
    //         v1.push(val);
    //     } else if filters.iter().all(|f| val.to_lowercase().contains(&*f)) {
    //         v2.push(val);
    //     }
    // }
    v1.extend(v2);
    v1
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::left("selected_mods_panel").show(ctx, |ui| {
            let text_height2 = egui::TextStyle::Body.resolve(ui.style()).size;
            ui.heading("Selected");
            let mut selected_table = TableBuilder::new(ui)
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
                    body.rows(text_height2, self.selected.len(), |row_index, mut row| {
                        row.col(|ui| {
                            ui.label((100).to_string());
                        });
                        let label = egui::Label::new(&self.selected[row_index])
                            .wrap(false)
                            .sense(Sense::click());
                        row.col(|ui| {
                            if ui.add(label).clicked() {
                                // &self.selected.retain(|x| x == &self.selected[row_index].clone());
                            };
                        });
                    });
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
                // ui.text_edit_singleline(&mut self.selected_item_level_as_filter);

                egui::ComboBox::from_label( "Select one!").selected_text(format!("{:?}", self.selected_item_tag_as_filter))
                    .show_ui(ui, |ui| {
                        self.db.item_tags.iter().for_each(|tag| {
                            ui.selectable_value(
                                &mut self.selected_item_tag_as_filter,
                                tag.clone(),
                                tag.clone(),
                            );
                        });
                    });
            });

            let table_data = get_list(&self.name, &self.db, &self.selected_item_tag_as_filter);

            let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
            let mut table = TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Size::initial(30.0).at_least(50.0))
                .column(Size::remainder().at_least(300.0))
                .resizable(false);

            table
                .header(30.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("#");
                    });
                    header.col(|ui| {
                        ui.heading("modification");
                    });
                })
                .body(|mut body| {
                    body.rows(text_height, table_data.len(), |row_index, mut row| {
                        row.col(|ui| {
                            ui.label((row_index + 1).to_string());
                        });
                        let label = egui::Label::new(&table_data[row_index])
                            .wrap(false)
                            .sense(Sense::click());
                        row.col(|ui| {
                            if ui.add(label).clicked() {
                                self.selected.push(table_data[row_index].clone());
                            };
                        });
                    });
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

use crate::entities::{db::LocalDB, translations::StatTranslation};
use std::fs::File;
use std::io::Read;

fn load_from_json<T>(path: &str) -> Vec<T>
where
    T: Default + serde::de::DeserializeOwned,
{
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let t: Vec<T> = serde_json::from_str(&contents).unwrap();
    t
}

pub struct FileRepo {}

impl FileRepo {
    pub fn new() -> Result<LocalDB, String> {
        let translations: Vec<StatTranslation> = load_from_json("data/stat_translations.min.json");
        let translations = translations
            .into_iter()
            .map(|t| (t.ids[0].clone(), t))
            .collect();
        Ok(LocalDB { translations })
    }
}

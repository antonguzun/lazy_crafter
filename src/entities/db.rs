use crate::entities::translations::StatTranslation;
use std::collections::HashMap;

pub struct LocalDB {
    pub translations: HashMap<String, StatTranslation>,
}

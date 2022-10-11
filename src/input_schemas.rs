pub fn parse_item_level(raw: &String) -> Result<u32, String> {
    let mut raw = raw.trim();
    match raw.trim().parse::<u32>() {
        Ok(level) => match level {
            1..=100 => Ok(level),
            _ => Err("Cannot parse item level".to_string()),
        },
        _ => Err("Cannot parse item level".to_string()),
    }
}

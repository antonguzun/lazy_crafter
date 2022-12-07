pub fn parse_item_level(raw: &String) -> Result<u32, String> {
    let raw = raw.trim();
    match raw.trim().parse::<u32>() {
        Ok(level) => match level {
            1..=100 => Ok(level),
            _ => Err("Cannot parse item level".to_string()),
        },
        _ => Err("Cannot parse item level".to_string()),
    }
}


pub fn parse_max_tries(raw: &String) -> Result<u32, String> {
    let raw = raw.trim();
    match raw.trim().parse::<u32>() {
        Ok(max) => match max {
            1..=1000 => Ok(max),
            _ => Err("Cannot parse max_tries".to_string()),
        },
        _ => Err("Cannot parse max_tries".to_string()),
    }
}

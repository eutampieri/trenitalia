/// This function returns the normalized typing distance between two strings
pub fn match_strings(first: &str, second: &str) -> f64 {
    if first.to_lowercase() == second.to_lowercase() {
        1.0
    } else {
        strsim::normalized_damerau_levenshtein(&first.to_lowercase(), &second.to_lowercase())
    }
}

macro_rules! current_timestamp_ms {
    () => {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    };
}

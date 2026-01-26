pub fn trim_quotes(value: &str) -> String {
    if value.starts_with("\"") && value.ends_with("\"") {
        return value[1..value.len() - 1].to_string();
    } else if value.starts_with("\'") && value.ends_with("\'") {
        return value[1..value.len() - 1].to_string();
    }
    return value.to_string();
}

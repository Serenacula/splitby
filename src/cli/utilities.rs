use crate::types::Delimiter;

pub fn trim_quotes(value: &str) -> String {
    if value.starts_with("\"") && value.ends_with("\"") {
        return value[1..value.len() - 1].to_string();
    } else if value.starts_with("\'") && value.ends_with("\'") {
        return value[1..value.len() - 1].to_string();
    }
    return value.to_string();
}

pub fn parse_delimiter_token(value: &str) -> Delimiter {
    let trimmed = trim_quotes(value);
    if trimmed.len() > 1 && trimmed.starts_with('/') && trimmed.ends_with('/') {
        return Delimiter::Regex(trimmed[1..trimmed.len() - 1].to_string());
    }
    Delimiter::Literal(trimmed)
}

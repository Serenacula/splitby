use regex::Regex as SimpleRegex;

pub fn parse_hex(hex_str: &str) -> Option<Vec<u8>> {
    if !hex_str.starts_with("0x") && !hex_str.starts_with("0X") {
        return None;
    }

    let hex_digits = &hex_str[2..];
    if hex_digits.is_empty() {
        return None;
    }

    if hex_digits.len() % 2 != 0 {
        return None; // Odd number of hex digits
    }

    let mut bytes = Vec::with_capacity(hex_digits.len() / 2);
    for chunk in hex_digits.as_bytes().chunks(2) {
        let hex_pair = std::str::from_utf8(chunk).ok()?;
        match u8::from_str_radix(hex_pair, 16) {
            Ok(byte_value) => bytes.push(byte_value),
            Err(_) => return None,
        }
    }

    Some(bytes)
}

pub fn parse_selection_token(
    token: &str,
    selection_regex: &SimpleRegex,
) -> Result<(i32, i32), String> {
    let trimmed = token.trim();
    let captures = selection_regex
        .captures(trimmed)
        .ok_or_else(|| format!("invalid selection: '{token}'"))?;
    let start_match = captures
        .name("start")
        .ok_or_else(|| format!("invalid selection: '{token}'"))?;
    let end_token = captures
        .name("end")
        .map(|value| value.as_str())
        .unwrap_or_else(|| start_match.as_str());

    let start_lowered = start_match.as_str().to_ascii_lowercase();
    let start = match start_lowered.as_str() {
        "start" | "first" => Ok(1),
        "end" | "last" => Ok(-1),
        _ => start_lowered
            .parse::<i32>()
            .map_err(|_| format!("invalid selection: '{token}'")),
    }?;

    let end_lowered = end_token.to_ascii_lowercase();
    let end = match end_lowered.as_str() {
        "start" | "first" => Ok(1),
        "end" | "last" => Ok(-1),
        _ => end_lowered
            .parse::<i32>()
            .map_err(|_| format!("invalid selection: '{token}'")),
    }?;

    Ok((start, end))
}

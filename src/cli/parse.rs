use std::path::PathBuf;

use regex::Regex as SimpleRegex;

use crate::cli::help_version::*;
use crate::cli::types::*;
use crate::cli::utilities::*;
use crate::types::InputMode;
use crate::types::SelectionMode;
use crate::types::{Align, JoinMode};

pub enum ParseResult {
    FlagParsed,
    FlagNotParsed,
    Finished,
}

pub fn parse_flags(
    arg: &str,
    consuming: &mut Consuming,
    raw_instructions: &mut CLIArguments,
) -> Result<ParseResult, String> {
    if consuming.input {
        raw_instructions.input = Some(PathBuf::from(arg));
        consuming.input = false;
        return Ok(ParseResult::FlagParsed);
    }
    if consuming.output {
        raw_instructions.output = Some(PathBuf::from(arg));
        consuming.output = false;
        return Ok(ParseResult::FlagParsed);
    }
    if consuming.delim {
        raw_instructions.delimiter = Some(arg.to_string());
        consuming.delim = false;
        return Ok(ParseResult::FlagParsed);
    }
    if consuming.join {
        raw_instructions.join = Some(arg.as_bytes().to_vec());
        consuming.join = false;
        return Ok(ParseResult::FlagParsed);
    }
    if consuming.placeholder {
        raw_instructions.placeholder = Some(arg.as_bytes().to_vec());
        consuming.placeholder = false;
        return Ok(ParseResult::FlagParsed);
    }
    if consuming.align {
        // Our valid align possibilities:
        // - Normal align flags -> set the align
        // - anything else -> assume we're not consuming and set to default
        if let Some(align_result) = parse_align(&arg) {
            match align_result {
                Align::Left => raw_instructions.align = Align::Left,
                Align::Right => raw_instructions.align = Align::Right,
                Align::Squash => raw_instructions.align = Align::Squash,
                Align::None => raw_instructions.align = Align::None,
            }
            consuming.align = false;
            return Ok(ParseResult::FlagParsed);
        }
        // No valid align flag detected, assume default and continue
        raw_instructions.align = Align::Left;
        consuming.align = false;
    }
    // Handle consuming flags
    if arg.starts_with("--input") && arg != "--input" {
        if !arg.starts_with("--input=") {
            return Err(format!("invalid input flag: '{arg}'"));
        }
        let value = arg.split("=").nth(1);
        if let Some(value) = value {
            raw_instructions.input = Some(PathBuf::from(trim_quotes(value)));
        } else {
            return Err(format!("empty input value"));
        }
        return Ok(ParseResult::FlagParsed);
    }
    if arg.starts_with("--output") && arg != "--output" {
        if !arg.starts_with("--output=") {
            return Err(format!("invalid output flag: '{arg}'"));
        }
        let value = arg.split("=").nth(1);
        if let Some(value) = value {
            raw_instructions.output = Some(PathBuf::from(trim_quotes(value)));
        } else {
            return Err(format!("empty output value"));
        }
        return Ok(ParseResult::FlagParsed);
    }
    if arg.starts_with("--delimiter") && arg != "--delimiter" {
        if !arg.starts_with("--delimiter=") {
            return Err(format!("invalid delimiter flag: '{arg}'"));
        }
        let value = arg.split("=").nth(1);
        if let Some(value) = value {
            raw_instructions.delimiter = Some(trim_quotes(value));
        } else {
            raw_instructions.delimiter = Some("".to_string());
        }
        return Ok(ParseResult::FlagParsed);
    }
    if arg.starts_with("--join") && arg != "--join" {
        if !arg.starts_with("--join=") {
            return Err(format!("invalid join flag: '{arg}'"));
        }
        let value = arg.split("=").nth(1);
        if let Some(value) = value {
            raw_instructions.join = Some(trim_quotes(value).as_bytes().to_vec());
        } else {
            raw_instructions.join = Some("".as_bytes().to_vec());
        }
        return Ok(ParseResult::FlagParsed);
    }
    if arg.starts_with("--placeholder") && arg != "--placeholder" {
        if !arg.starts_with("--placeholder=") {
            return Err(format!("invalid placeholder flag: '{arg}'"));
        }
        let value = arg.split("=").nth(1);
        if let Some(value) = value {
            raw_instructions.placeholder = Some(trim_quotes(value).as_bytes().to_vec());
        } else {
            raw_instructions.placeholder = Some("".as_bytes().to_vec());
        }
        return Ok(ParseResult::FlagParsed);
    }
    if arg.starts_with("--align") && arg != "--align" {
        if !arg.starts_with("--align=") {
            return Err(format!("invalid align flag: '{arg}'"));
        }
        let value = arg.split("=").nth(1);
        if let Some(value) = value {
            raw_instructions.align = parse_align(&trim_quotes(value)).unwrap_or(Align::Left);
        } else {
            return Err(format!("empty align value"));
        }
        return Ok(ParseResult::FlagParsed);
    }

    if arg.starts_with("-d") && arg != "-d" {
        // Support -d, -d',' and -d","
        let delim_value = &arg[2..]; // characters after -d
        raw_instructions.delimiter = Some(trim_quotes(delim_value));
        return Ok(ParseResult::FlagParsed);
    }

    // Handle non-consuming flags
    match arg {
        "-v" | "--version" => {
            print_version();
            return Ok(ParseResult::Finished);
        }
        "-h" | "--help" => {
            print_help();
            return Ok(ParseResult::Finished);
        }
        "--per-line" => {
            raw_instructions.input_mode = InputMode::PerLine;
            return Ok(ParseResult::FlagParsed);
        }
        "--whole-string" | "-w" => {
            raw_instructions.input_mode = InputMode::WholeString;
            return Ok(ParseResult::FlagParsed);
        }
        "--zero-terminated" | "-z" => {
            raw_instructions.input_mode = InputMode::ZeroTerminated;
            return Ok(ParseResult::FlagParsed);
        }
        "--bytes" | "-b" => {
            raw_instructions.selection_mode = SelectionMode::Bytes;
            return Ok(ParseResult::FlagParsed);
        }
        "--characters" | "-c" => {
            raw_instructions.selection_mode = SelectionMode::Chars;
            return Ok(ParseResult::FlagParsed);
        }
        "--fields" | "-f" => {
            raw_instructions.selection_mode = SelectionMode::Fields;
            return Ok(ParseResult::FlagParsed);
        }
        "--input" | "-i" => {
            consuming.input = true;
            return Ok(ParseResult::FlagParsed);
        }
        "--output" | "-o" => {
            consuming.output = true;
            return Ok(ParseResult::FlagParsed);
        }
        "--delimiter" | "-d" => {
            consuming.delim = true;
            return Ok(ParseResult::FlagParsed);
        }
        "--join" | "-j" => {
            consuming.join = true;
            return Ok(ParseResult::FlagParsed);
        }
        "--placeholder" | "-p" => {
            consuming.placeholder = true;
            return Ok(ParseResult::FlagParsed);
        }
        "--align" | "-a" => {
            consuming.align = true;
            return Ok(ParseResult::FlagParsed);
        }
        "--skip-empty" | "-e" => {
            raw_instructions.skip_empty = true;
            return Ok(ParseResult::FlagParsed);
        }
        "--no-skip-empty" | "-E" => {
            raw_instructions.skip_empty = false;
            return Ok(ParseResult::FlagParsed);
        }
        "--count" => {
            raw_instructions.count = true;
            return Ok(ParseResult::FlagParsed);
        }
        "--invert" => {
            raw_instructions.invert = true;
            return Ok(ParseResult::FlagParsed);
        }
        "--strict" => {
            raw_instructions.strict_bounds = true;
            raw_instructions.strict_range_order = true;
            raw_instructions.strict_return = true;
            raw_instructions.strict_utf8 = true;
            return Ok(ParseResult::FlagParsed);
        }
        "--no-strict" => {
            raw_instructions.strict_bounds = false;
            raw_instructions.strict_range_order = false;
            raw_instructions.strict_return = false;
            raw_instructions.strict_utf8 = false;
            return Ok(ParseResult::FlagParsed);
        }
        "--strict-bounds" => {
            raw_instructions.strict_bounds = true;
            return Ok(ParseResult::FlagParsed);
        }
        "--no-strict-bounds" => {
            raw_instructions.strict_bounds = false;
            return Ok(ParseResult::FlagParsed);
        }
        "--strict-return" => {
            raw_instructions.strict_return = true;
            return Ok(ParseResult::FlagParsed);
        }
        "--no-strict-return" => {
            raw_instructions.strict_return = false;
            return Ok(ParseResult::FlagParsed);
        }
        "--strict-range-order" => {
            raw_instructions.strict_range_order = true;
            return Ok(ParseResult::FlagParsed);
        }
        "--no-strict-range-order" => {
            raw_instructions.strict_range_order = false;
            return Ok(ParseResult::FlagParsed);
        }
        "--strict-utf8" => {
            raw_instructions.strict_utf8 = true;
            return Ok(ParseResult::FlagParsed);
        }
        "--no-strict-utf8" => {
            raw_instructions.strict_utf8 = false;
            return Ok(ParseResult::FlagParsed);
        }
        _ => return Ok(ParseResult::FlagNotParsed),
    }
}

pub fn parse_align(arg: &str) -> Option<Align> {
    match arg.to_lowercase().as_str() {
        "left" => Some(Align::Left),
        "right" => Some(Align::Right),
        "squash" => Some(Align::Squash),
        "none" => Some(Align::None),
        _ => None,
    }
}

pub fn parse_join(arg: &[u8]) -> Option<JoinMode> {
    match arg {
        b"@auto" => Some(JoinMode::Auto),
        b"@after-previous" => Some(JoinMode::AfterPrevious),
        b"@before-next" => Some(JoinMode::BeforeNext),
        b"@first" => Some(JoinMode::First),
        b"@last" => Some(JoinMode::Last),
        b"@space" => Some(JoinMode::Space),
        b"@none" => Some(JoinMode::None),
        // Regular string join or hex
        _ => {
            // Try parsing as hex first
            match parse_hex(&arg) {
                Some(hex_bytes) => Some(JoinMode::String(hex_bytes)),
                None => Some(JoinMode::String(arg.to_vec())),
            }
        }
    }
}

pub fn parse_placeholder(arg: &[u8]) -> Option<Vec<u8>> {
    match parse_hex(&arg) {
        Some(hex_bytes) => Some(hex_bytes),
        None => Some(arg.to_vec()),
    }
}

pub fn parse_hex(hex_str: &[u8]) -> Option<Vec<u8>> {
    if !hex_str.starts_with(b"0x") && !hex_str.starts_with(b"0X") {
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
    for chunk in hex_digits.chunks(2) {
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

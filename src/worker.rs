use std::sync::Arc;

use crate::types::*;
use fancy_regex::Regex as FancyRegex;
use regex::Regex as SimpleRegex;

pub fn process_bytes(instructions: &Arc<Instructions>, record: Record) -> Result<Vec<u8>, String> {
    Err(format!("bytes not implemented"))
}

pub fn process_chars(instructions: &Arc<Instructions>, record: Record) -> Result<Vec<u8>, String> {
    Err(format!("chars not implemented"))
}

pub fn process_simple_regex(
    instructions: &Arc<Instructions>,
    engine: &SimpleRegex,
    record: Record,
) -> Result<Vec<u8>, String> {
    Ok(record.bytes)
}

pub fn process_fancy_regex(
    instructions: &Arc<Instructions>,
    engine: &FancyRegex,
    record: Record,
) -> Result<Vec<u8>, String> {
    Err(format!("fancy regex not implemented"))
}

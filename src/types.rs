use fancy_regex::Regex as FancyRegex;
use regex::Regex as SimpleRegex;
use std::path::PathBuf;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum InputMode {
    PerLine,
    WholeString,
    ZeroTerminated,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SelectionMode {
    Fields,
    Bytes,
    Chars,
}

pub enum RegexEngine {
    Simple(SimpleRegex),
    Fancy(FancyRegex),
}

pub struct Instructions {
    // Input
    pub input_mode: InputMode,
    pub input: Option<PathBuf>,
    // Processing
    pub selection_mode: SelectionMode,
    pub selections: Vec<(i32, i32)>,
    pub invert: bool,
    pub skip_empty: bool,
    // Failure Modes
    pub strict_return: bool,
    pub strict_bounds: bool,
    pub strict_range_order: bool,
    // Output
    pub output: Option<PathBuf>,
    pub count: bool,
    pub join: String,
    pub replace_range_delimiter: Option<String>, // might get rid of this not sure yet

    pub regex_engine: Option<RegexEngine>,
}

pub struct Record {
    pub index: usize,
    pub bytes: Vec<u8>,
}

pub enum RecordResult {
    Ok { index: usize, bytes: Vec<u8> },
    Err { index: usize, error: String },
}

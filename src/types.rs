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
    pub input_mode: InputMode,
    pub input: Option<PathBuf>,
    pub selection_mode: SelectionMode,
    pub selections: Vec<(i32, i32)>,
    pub invert: bool,
    pub skip_empty: bool,
    pub placeholder: Option<Vec<u8>>,
    pub strict_return: bool,
    pub strict_bounds: bool,
    pub strict_range_order: bool,
    pub strict_utf8: bool,
    pub output: Option<PathBuf>,
    pub count: bool,
    pub join: Option<Vec<u8>>,
    pub regex_engine: Option<RegexEngine>,
}

pub struct Record {
    pub index: usize,
    pub bytes: Vec<u8>,
    pub has_terminator: bool,
}

pub struct OutputRecord {
    pub bytes: Vec<u8>,
    pub has_terminator: bool,
}

pub enum ResultChunk {
    Ok {
        start_index: usize,
        outputs: Vec<OutputRecord>,
    },
    Err {
        index: usize,
        error: String,
    },
}

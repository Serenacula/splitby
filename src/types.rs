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

#[derive(Clone, PartialEq, Eq)]
pub enum JoinMode {
    String(Vec<u8>), // Regular string join
    Auto,            // @auto: existing logic
    AfterPrevious,   // @after-previous: use delimiter after previous field
    BeforeNext,      // @before-next: use delimiter before next field
    First,           // @first: use first delimiter in record
    Last,            // @last: use last delimiter in record
    Space,           // @space: use space character
    None,            // @none: no join (equivalent to "")
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Align {
    Left,
    Right,
    Squash,
    None,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Delimiter {
    Literal(String),
    Regex(String),
}

#[derive(Clone)]
pub enum RegexEngine {
    Simple(SimpleRegex),
    Fancy(FancyRegex),
}

pub struct Config {
    pub input: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub input_mode: InputMode,
    pub selection_mode: SelectionMode,
    pub selections: Vec<(i32, i32)>,
    pub invert: bool,
    pub terminator: Option<Vec<u8>>,
    pub skip_empty_fields: bool,
    pub skip_empty_lines: bool,
    pub skip_undelimited: bool,
    pub regex_engine: Option<RegexEngine>,
    pub join: Option<JoinMode>,
    pub placeholder: Option<Vec<u8>>,
    pub align: Align,
    pub count: bool,
    pub strict_return: bool,
    pub strict_bounds: bool,
    pub strict_range_order: bool,
    pub strict_utf8: bool,
}

pub struct Record {
    pub index: usize,
    pub bytes: Vec<u8>,
    pub has_terminator: bool,
    pub field_widths: Option<Vec<usize>>,
    pub join_widths: Option<Vec<usize>>,
}

pub struct OutputRecord {
    pub bytes: Vec<u8>,
    pub has_terminator: bool,
}

pub struct AppError {
    pub message: String,
    pub exit_code: i32,
}

impl AppError {
    pub fn io(message: String) -> Self    { Self { message, exit_code: 2 } }
    pub fn other(message: String) -> Self { Self { message, exit_code: 1 } }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<String> for AppError {
    fn from(message: String) -> Self { Self::other(message) }
}

pub enum ResultChunk {
    Ok {
        start_index: usize,
        input_count: usize,
        outputs: Vec<OutputRecord>,
    },
    Err {
        index: usize,
        error: String,
    },
}

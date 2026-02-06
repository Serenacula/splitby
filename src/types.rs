use fancy_regex::Regex as FancyRegex;
use regex::bytes::Regex as BytesRegex;
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

pub struct InputInstructions {
    pub regex_engine: Option<RegexEngine>,
    pub align: Align,
    pub ansi_strip_regex: Option<BytesRegex>,
    pub join: Option<JoinMode>,
    pub input_mode: InputMode,
    pub input: Option<PathBuf>,
    pub selections: Vec<(i32, i32)>,
    pub skip_empty: bool,
    pub invert: bool,
    pub placeholder: Option<Vec<u8>>,
    pub strict_bounds: bool,
    pub strict_range_order: bool,
    pub strict_utf8: bool,
}

pub struct TransformInstructions {
    pub input_mode: InputMode,
    pub selection_mode: SelectionMode,
    pub selections: Vec<(i32, i32)>,
    pub invert: bool,
    pub skip_empty: bool,
    pub placeholder: Option<Vec<u8>>,
    pub strict_return: bool,
    pub strict_bounds: bool,
    pub strict_range_order: bool,
    pub strict_utf8: bool,
    pub count: bool,
    pub join: Option<JoinMode>,
    pub regex_engine: Option<RegexEngine>,
    pub align: Align,
    pub ansi_strip_regex: Option<BytesRegex>,
}

pub struct OutputInstructions {
    pub count: bool,
    pub strict_return: bool,
    pub strict_bounds: bool,
    pub input_mode: InputMode,
    pub selections: Vec<(i32, i32)>,
    pub output: Option<PathBuf>,
}

pub struct Instructions {
    pub input_instructions: InputInstructions,
    pub transform_instructions: TransformInstructions,
    pub output_instructions: OutputInstructions,
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

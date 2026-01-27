use std::path::PathBuf;

use crate::types::*;

pub struct Consuming {
    pub input: bool,
    pub output: bool,
    pub delim: bool,
    pub join: bool,
    pub placeholder: bool,
    pub align: bool,
}

pub struct CLIArguments {
    pub output: Option<PathBuf>,
    pub input: Option<PathBuf>,
    pub join: Option<Vec<u8>>, // This is a string because we want to do validation AFTER parsing
    pub align: Align,
    pub delimiter: Option<Delimiter>,
    pub placeholder: Option<Vec<u8>>,
    pub input_mode: InputMode,
    pub selection_mode: SelectionMode,
    pub count: bool,
    pub invert: bool,
    pub skip_empty: bool,
    pub strict_utf8: bool,
    pub strict_return: bool,
    pub strict_bounds: bool,
    pub strict_range_order: bool,
    pub selections: Vec<(i32, i32)>,
}

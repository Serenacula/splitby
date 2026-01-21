mod parse;
mod validation;

use clap::Parser;
use fancy_regex::Regex as FancyRegex;
use regex::Regex as SimpleRegex;
use std::path::PathBuf;

use crate::types::*;
use parse::{parse_hex, parse_selection_token};
use validation::{validate_align, validate_join_mode};

#[derive(Parser)]
#[command(
    name = "splitby",
    version = "v1.0.0",
    about = "Split text by a regex delimiter, select parts of the result.",
    disable_help_subcommand = true
)]
pub struct Options {
    #[arg(
        short = 'i',
        long = "input",
        value_name = "FILE",
        require_equals = true
    )]
    pub input: Option<PathBuf>,

    #[arg(
        short = 'o',
        long = "output",
        value_name = "FILE",
        require_equals = true
    )]
    pub output: Option<PathBuf>,

    #[arg(long = "per-line")]
    pub per_line: bool,

    #[arg(short = 'w', long = "whole-string")]
    pub whole_string: bool,

    #[arg(short = 'z', long = "zero-terminated")]
    pub zero_terminated: bool,

    #[arg(short = 'e', long = "skip-empty")]
    pub skip_empty: bool,

    #[arg(short = 'E', long = "no-skip-empty")]
    pub no_skip_empty: bool,

    #[arg(long = "invert")]
    pub invert: bool,

    #[arg(long = "count")]
    pub count: bool,

    #[arg(long = "strict")]
    pub strict: bool,

    #[arg(long = "no-strict")]
    pub no_strict: bool,

    #[arg(long = "strict-bounds")]
    pub strict_bounds: bool,

    #[arg(long = "no-strict-bounds")]
    pub no_strict_bounds: bool,

    #[arg(long = "strict-return")]
    pub strict_return: bool,

    #[arg(long = "no-strict-return")]
    pub no_strict_return: bool,

    #[arg(long = "strict-range-order")]
    pub strict_range_order: bool,

    #[arg(long = "no-strict-range-order")]
    pub no_strict_range_order: bool,

    #[arg(long = "strict-utf8")]
    pub strict_utf8: bool,

    #[arg(long = "no-strict-utf8")]
    pub no_strict_utf8: bool,

    #[arg(
        short = 'j',
        long = "join",
        num_args = 1,
        value_name = "STRING|HEX",
        require_equals = true,
        allow_hyphen_values = true
    )]
    pub join: Option<String>,

    #[arg(
        long = "placeholder",
        num_args = 1,
        value_name = "STRING|HEX",
        require_equals = true,
        allow_hyphen_values = true,
        action = clap::ArgAction::Append,
    )]
    pub placeholder: Vec<String>,

    #[arg(short = 'f', long = "fields")]
    pub fields: bool,

    #[arg(short = 'b', long = "bytes")]
    pub bytes: bool,

    #[arg(short = 'c', long = "characters")]
    pub chars: bool,

    #[arg(short = 'd', long = "delimiter", value_name = "REGEX")]
    pub delimiter: Option<String>,

    #[arg(long = "align")]
    pub align: bool,

    #[arg(value_name = "SELECTION", num_args = 0.., allow_hyphen_values = true)]
    pub selection_list: Vec<String>,
}

pub struct ParsedConfig {
    pub instructions: Instructions,
    pub reader_instructions: ReaderInstructions,
}

pub fn parse_options(options: Options) -> Result<ParsedConfig, String> {
    // Sorting out our last-flag-wins, since clap doesn't do this automatically
    let mut input_mode: InputMode = InputMode::PerLine;
    let mut selection_mode: SelectionMode = SelectionMode::Fields;
    let mut skip_empty = false;
    let mut strict_return = false;
    let mut strict_bounds = false;
    let mut strict_range_order = true;
    let mut strict_utf8 = false;
    for arg in std::env::args_os() {
        match arg.to_string_lossy().as_ref() {
            "--per-line" => input_mode = InputMode::PerLine,
            "-w" | "--whole-string" => input_mode = InputMode::WholeString,
            "-z" | "--zero-terminated" => input_mode = InputMode::ZeroTerminated,

            "-b" | "--bytes" => selection_mode = SelectionMode::Bytes,
            "-f" | "--fields" => selection_mode = SelectionMode::Fields,
            "-c" | "--characters" => selection_mode = SelectionMode::Chars,

            "-e" | "--skip-empty" => skip_empty = true,
            "-E" | "--no-skip-empty" => skip_empty = false,

            "--strict-return" => strict_return = true,
            "--no-strict-return" => strict_return = false,

            "--strict-bounds" => strict_bounds = true,
            "--no-strict-bounds" => strict_bounds = false,

            "--strict-range-order" => strict_range_order = true,
            "--no-strict-range-order" => strict_range_order = false,

            "--strict-utf8" => strict_utf8 = true,
            "--no-strict-utf8" => strict_utf8 = false,

            "--strict" => {
                strict_return = true;
                strict_bounds = true;
                strict_range_order = true;
                strict_utf8 = true;
            }
            "--no-strict" => {
                strict_return = false;
                strict_bounds = false;
                strict_range_order = false;
                strict_utf8 = false
            }

            _ => {}
        }
    }

    const SELECTION_TOKEN_PATTERN: &str =
        r"(?i)^(?P<start>start|first|end|last|-?\d+)(?:-(?P<end>start|first|end|last|-?\d+))?$";
    let selection_regex = SimpleRegex::new(SELECTION_TOKEN_PATTERN)
        .map_err(|error| format!("internal error: failed to compile selection regex: {error}"))?;

    let mut delimiter: Option<String> = options.delimiter;
    let mut selections: Vec<(i32, i32)> = Vec::new();
    for (index, string_raw) in options.selection_list.iter().enumerate() {
        let parts: Vec<&str> = string_raw.split(",").map(|part| part.trim()).collect();

        if index == 0 && delimiter.is_none() {
            let has_number = parts.iter().any(|part| {
                !part.is_empty() && parse_selection_token(part, &selection_regex).is_ok()
            });

            // if not a selection
            if string_raw.trim() == "," || !has_number {
                delimiter = Some(string_raw.clone());
                continue;
            }
        }

        for part in &parts {
            let trimmed_part = part.trim();
            if trimmed_part.is_empty() {
                continue;
            }

            let token = trimmed_part.to_string();
            match parse_selection_token(&token, &selection_regex) {
                Ok(range) => selections.push(range),
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
            }
        }
    }

    if selection_mode == SelectionMode::Fields && delimiter.is_none() {
        eprintln!(
            "delimiter required: you can provide one with the -d <REGEX> flag or as the first argument"
        );
        std::process::exit(2);
    }

    let regex_engine: Option<RegexEngine> = match selection_mode {
        SelectionMode::Bytes | SelectionMode::Chars => None,
        SelectionMode::Fields => {
            let delimiter: String = delimiter.unwrap_or_else(|| {
                eprintln!("delimiter is required in fields mode (use -d or --delimiter)");
                std::process::exit(2)
            });

            if delimiter.is_empty() {
                eprintln!("empty string is not a valid delimiter");
                std::process::exit(2)
            }

            let simple_regex = SimpleRegex::new(&delimiter);

            match simple_regex {
                Ok(regex) => Some(RegexEngine::Simple(regex)),
                Err(_) => {
                    let fancy_regex = FancyRegex::new(&delimiter)
                        .map_err(|error| format!("failed to compile regex: {error}"))?;
                    Some(RegexEngine::Fancy(fancy_regex))
                }
            }
        }
    };

    let placeholder_value: Option<Vec<u8>> =
        if let Some(placeholder_str) = options.placeholder.last() {
            match parse_hex(placeholder_str) {
                Some(hex_bytes) => Some(hex_bytes),
                None => Some(placeholder_str.as_bytes().to_vec()),
            }
        } else {
            None
        };

    let join = match options.join {
        Some(join_str) => {
            validate_join_mode(&join_str, selection_mode)?;

            // Check if it's a special flag
            match join_str.as_str() {
                "@auto" => Some(JoinMode::Auto),
                "@after-previous" => Some(JoinMode::AfterPrevious),
                "@before-next" => Some(JoinMode::BeforeNext),
                "@first" => Some(JoinMode::First),
                "@last" => Some(JoinMode::Last),
                "@space" => Some(JoinMode::Space),
                "@none" => Some(JoinMode::None),
                // Regular string join or hex
                _ => {
                    // Try parsing as hex first
                    match parse_hex(&join_str) {
                        Some(hex_bytes) => Some(JoinMode::String(hex_bytes)),
                        None => Some(JoinMode::String(join_str.as_bytes().to_vec())),
                    }
                }
            }
        }
        None => None,
    };

    // Validate --align flag
    validate_align(options.align, input_mode, selection_mode)?;

    // Clone regex_engine for ReaderInstructions
    let reader_regex_engine = regex_engine.clone();

    let reader_instructions = ReaderInstructions {
        regex_engine: reader_regex_engine,
        align: options.align,
        input_mode: input_mode,
        input: options.input.clone(),
        selections: selections.clone(),
        skip_empty: skip_empty,
        invert: options.invert,
        placeholder: placeholder_value.clone(),
        strict_bounds: strict_bounds,
        strict_range_order: strict_range_order,
        strict_utf8: strict_utf8,
    };

    let instructions = Instructions {
        input_mode: input_mode,
        input: options.input,
        selection_mode: selection_mode,
        selections: selections,
        invert: options.invert,
        skip_empty: skip_empty,
        placeholder: placeholder_value,
        strict_return: strict_return,
        strict_bounds: strict_bounds,
        strict_range_order: strict_range_order,
        strict_utf8: strict_utf8,
        output: options.output,
        count: options.count,
        join: join,
        regex_engine: regex_engine,
        align: options.align,
    };

    Ok(ParsedConfig {
        instructions,
        reader_instructions,
    })
}

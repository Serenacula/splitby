mod types;
use crate::processing::get_results::get_results;
use crate::processing::process_records::process_records;
use crate::processing::read_input::read_input;
use crate::types::*;
use clap::Parser;
use crossbeam::channel;
use fancy_regex::Regex as FancyRegex;
use regex::Regex as SimpleRegex;
use std::{cmp::max, path::PathBuf, sync::Arc};

mod processing {
    pub mod get_results;
    pub mod process_records;
    pub mod read_input;
}

#[derive(Parser)]
#[command(
    name = "splitby",
    version,
    about = "Split text by a regex delimiter (flags only; no processing yet).",
    disable_help_subcommand = true
)]
struct Options {
    #[arg(short = 'd', long = "delimiter", value_name = "REGEX")]
    delimiter: Option<String>,

    #[arg(long = "per-line")]
    per_line: bool,

    #[arg(short = 'w', long = "whole-string")]
    whole_string: bool,

    #[arg(short = 'z', long = "zero-terminated")]
    zero_terminated: bool,

    #[arg(short = 'j', long = "join", value_name = "STRING")]
    join: Option<String>,

    #[arg(short = 'e', long = "skip-empty")]
    skip_empty: bool,

    #[arg(short = 'E', long = "no-skip-empty")]
    no_skip_empty: bool,

    #[arg(long = "invert")]
    invert: bool,

    #[arg(long = "strict")]
    strict: bool,

    #[arg(long = "no-strict")]
    no_strict: bool,

    #[arg(long = "strict-bounds")]
    strict_bounds: bool,

    #[arg(long = "no-strict-bounds")]
    no_strict_bounds: bool,

    #[arg(long = "strict-return")]
    strict_return: bool,

    #[arg(long = "no-strict-return")]
    no_strict_return: bool,

    #[arg(long = "strict-range-order")]
    strict_range_order: bool,

    #[arg(long = "no-strict-range-order")]
    no_strict_range_order: bool,

    #[arg(long = "strict-utf8")]
    strict_utf8: bool,

    #[arg(long = "no-strict-ut8")]
    no_strict_utf8: bool,

    #[arg(short = 'i', long = "input", value_name = "FILE")]
    input: Option<PathBuf>,

    #[arg(short = 'o', long = "output", value_name = "FILE")]
    output: Option<PathBuf>,

    #[arg(long = "count")]
    count: bool,

    #[arg(
        long = "placeholder",
        value_name = "STRING|HEX",
        num_args = 1,
        allow_hyphen_values = true,
        action = clap::ArgAction::Append,
    )]
    placeholder: Vec<String>,

    #[arg(long = "trim-newline")]
    trim_newline: bool,

    #[arg(
        short = 'f',
        long = "fields",
        value_name = "SELECTION",
        num_args = 0..=1,
        allow_hyphen_values = true,
    )]
    field_list: Vec<String>,

    #[arg(short = 'b',
        long = "bytes",
        value_name = "SELECTION",
        num_args = 0..=1,
        allow_hyphen_values = true,
    )]
    byte_list: Vec<String>,

    #[arg(short = 'c',
        long = "characters",
        value_name = "SELECTION",
        num_args = 0..=1,
        allow_hyphen_values = true,
    )]
    char_list: Vec<String>,

    #[arg(value_name = "SELECTION", num_args = 0.., allow_hyphen_values = true)]
    selection_list: Vec<String>,
}

fn main() {
    use std::time::Instant;

    let profile_enabled = std::env::var("SPLITBY_PROFILE").is_ok();
    let profile_start_time = Instant::now();
    let profile_log = |label: &str| {
        if profile_enabled {
            eprintln!("profile:{label}: {:?}", profile_start_time.elapsed());
        }
    };

    let options = Options::parse();

    profile_log("parsed_options");

    // Sorting out our last-flag-wins, since clap doesn't do this automatically
    let mut input_mode: InputMode = InputMode::PerLine;
    let mut skip_empty = false;
    let mut strict_return = false;
    let mut strict_bounds = false;
    let mut strict_range_order = true;
    let mut strict_utf8 = false;
    let mut field_mode = false;
    let mut byte_mode = false;
    let mut char_mode = false;
    for arg in std::env::args_os() {
        match arg.to_string_lossy().as_ref() {
            "--per-line" => input_mode = InputMode::PerLine,
            "-w" | "--whole-string" => input_mode = InputMode::WholeString,
            "-z" | "--zero-terminated" => input_mode = InputMode::ZeroTerminated,

            "-b" | "--bytes" => byte_mode = true,
            "-f" | "--fields" => field_mode = true,
            "-c" | "--characters" => char_mode = true,

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

    profile_log("selection_start");

    let uses_fields = field_mode || !options.field_list.is_empty();
    let uses_bytes = byte_mode || !options.byte_list.is_empty();
    let uses_chars = char_mode || !options.char_list.is_empty();

    if (uses_fields as u8 + uses_bytes as u8 + uses_chars as u8) > 1 {
        eprintln!("cannot combine --fields, --bytes and --characters");
        std::process::exit(2);
    }
    let selection_mode = if uses_bytes {
        SelectionMode::Bytes
    } else if uses_chars {
        SelectionMode::Chars
    } else {
        SelectionMode::Fields
    };

    let mut selection_strings: Vec<String> = Vec::new();
    match selection_mode {
        SelectionMode::Fields => selection_strings.extend(options.field_list.iter().cloned()),
        SelectionMode::Bytes => selection_strings.extend(options.byte_list.iter().cloned()),
        SelectionMode::Chars => selection_strings.extend(options.char_list.iter().cloned()),
    }
    selection_strings.extend(options.selection_list.iter().cloned());

    fn parse_selection(string_raw: &str) -> Result<(i32, i32), String> {
        fn parse_number(string: &str) -> Result<i32, String> {
            let lowered = string.to_ascii_lowercase();
            match lowered.as_str() {
                "start" | "first" => Ok(1),
                "end" | "last" => Ok(-1),
                _ => lowered
                    .parse::<i32>()
                    .map_err(|_| format!("range has invalid number: {string}")),
            }
        }

        let string = string_raw.trim();

        if let Ok(value) = parse_number(string) {
            return Ok((value, value));
        }

        let split_index: usize;
        if string.starts_with('-') {
            let split_index_search = string.strip_prefix('-').unwrap().find('-');
            if split_index_search.is_none() {
                return Err(format!("invalid selection: {string}"));
            }
            split_index = split_index_search.unwrap() + 1
        } else {
            let split_index_search = string.find('-');
            if split_index_search.is_none() {
                return Err(format!("invalid selection: {string}"));
            }
            split_index = split_index_search.unwrap()
        }

        let (first_split, second_split) = string.split_at(split_index);

        let no_hyphen = &second_split[1..];

        let start = parse_number(first_split);
        let end = parse_number(no_hyphen); // Strip the range hyphen
        if start.is_err() || end.is_err() {
            return Err(format!("invalid range '{string}'"));
        }

        Ok((start.unwrap(), end.unwrap()))
    }

    fn can_parse_as_selection(string: &str) -> bool {
        if string == "," {
            return false; // Just a comma is a delimiter, not a selection
        }
        if string.contains(',') {
            string.split(',').any(|part| {
                let trimmed = part.trim();
                !trimmed.is_empty() && parse_selection(trimmed).is_ok()
            })
        } else {
            parse_selection(string).is_ok()
        }
    }
    profile_log("selection_regex_start");

    fn is_valid_regex(pattern: &str) -> bool {
        SimpleRegex::new(pattern).is_ok() || FancyRegex::new(pattern).is_ok()
    }

    let mut detected_delimiter: Option<String> = None;
    if selection_mode == SelectionMode::Fields
        && options.delimiter.is_none()
        && !selection_strings.is_empty()
    {
        let first_arg = selection_strings[0].trim();
        if !can_parse_as_selection(first_arg) && is_valid_regex(first_arg) {
            detected_delimiter = Some(first_arg.to_string());
            selection_strings.remove(0);
        }
    }

    profile_log("selection_regex_end");

    if selection_mode == SelectionMode::Fields
        && options.delimiter.is_none()
        && detected_delimiter.is_none()
    {
        eprintln!(
            "delimiter required: you can provide one with the -d <REGEX> flag or as the first argument"
        );
        std::process::exit(2);
    }

    let mut selections: Vec<(i32, i32)> = Vec::new();
    let delimiter_was_set = options.delimiter.is_some();

    for (index, string_raw) in selection_strings.iter().enumerate() {
        let is_first = index == 0;
        let trimmed = string_raw.trim();

        let should_check_ambiguity = is_first && !delimiter_was_set;

        if trimmed.contains(',') {
            let parts: Vec<&str> = trimmed.split(',').collect();

            if should_check_ambiguity {
                if trimmed == "," {
                    continue;
                }

                let has_letter = parts.iter().any(|part| {
                    let trimmed_part = part.trim();
                    !trimmed_part.is_empty()
                        && trimmed_part
                            .chars()
                            .any(|char| char.is_alphabetic() && char != '-')
                });

                if has_letter {
                    continue;
                }
            }

            for part in parts {
                let trimmed_part = part.trim();
                if trimmed_part.is_empty() {
                    continue;
                }

                let (start, end) = match parse_selection(trimmed_part) {
                    Ok(range) => range,
                    Err(_) => {
                        eprintln!("invalid selection: '{trimmed_part}'");
                        std::process::exit(2);
                    }
                };

                selections.push((start, end));
            }
        } else {
            if should_check_ambiguity {
                if trimmed == "," {
                    continue;
                }

                if trimmed
                    .chars()
                    .any(|char| char.is_alphabetic() && char != '-')
                {
                    continue;
                }
            }

            let (start, end) = match parse_selection(trimmed) {
                Ok(range) => range,
                Err(_) => {
                    eprintln!("invalid selection: '{trimmed}'");
                    std::process::exit(2);
                }
            };

            selections.push((start, end));
        }
    }

    profile_log("regex_compile_start");

    let regex_engine: Option<RegexEngine> = match selection_mode {
        SelectionMode::Bytes | SelectionMode::Chars => None,
        SelectionMode::Fields => {
            let delimiter: String = options
                .delimiter
                .clone()
                .or(detected_delimiter)
                .unwrap_or_else(|| {
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
                    let fancy_regex = FancyRegex::new(&delimiter).unwrap_or_else(|error| {
                        eprintln!("failed to compile regex: {error}");
                        std::process::exit(2)
                    });
                    Some(RegexEngine::Fancy(fancy_regex))
                }
            }
        }
    };

    profile_log("regex_compile_end");

    let placeholder_value: Option<Vec<u8>> =
        if let Some(placeholder_str) = options.placeholder.last() {
            if placeholder_str.starts_with("0x") || placeholder_str.starts_with("0X") {
                let hex_str = &placeholder_str[2..];
                match u8::from_str_radix(hex_str, 16) {
                    Ok(byte_value) => Some(vec![byte_value]),
                    Err(_) => {
                        eprintln!("invalid hex value for placeholder: {}", placeholder_str);
                        std::process::exit(2);
                    }
                }
            } else {
                Some(placeholder_str.as_bytes().to_vec())
            }
        } else {
            None
        };

    let instructions = Arc::new(Instructions {
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
        join: options.join,
        trim_newline: options.trim_newline,
        regex_engine: regex_engine,
    });

    let (record_sender, record_receiver) = channel::bounded::<Vec<Record>>(1024);
    let (result_sender, result_receiver) =
        channel::bounded::<processing::get_results::ResultChunk>(1024);

    profile_log("worker_threads_start");

    let reader_instructions = Arc::clone(&instructions);
    let reader_sender = record_sender.clone();
    let reader_profile_enabled = profile_enabled;
    let reader_handle = std::thread::spawn(move || {
        let read_start_time = if reader_profile_enabled {
            Some(Instant::now())
        } else {
            None
        };
        let read_result = read_input(
            &reader_instructions.input_mode,
            &reader_instructions.input,
            reader_sender,
        );
        if let Some(start_time) = read_start_time {
            eprintln!("profile:read_input: {:?}", start_time.elapsed());
        }
        read_result
    });
    drop(record_sender);

    let worker_count = if std::env::var("SPLITBY_SINGLE_CORE").is_ok() {
        1
    } else {
        std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1)
    };

    for worker_index in 0..max(worker_count - 1, 1) {
        let worker_instructions = Arc::clone(&instructions);
        let worker_receiver = record_receiver.clone();
        let worker_sender = result_sender.clone();
        let worker_profile_enabled = profile_enabled;
        std::thread::spawn(move || {
            let worker_start_time = if worker_profile_enabled {
                Some(Instant::now())
            } else {
                None
            };
            let worker_result =
                process_records(worker_instructions, worker_receiver, worker_sender)
                    .map_err(|error| eprintln!("{error}"));
            if let Some(start_time) = worker_start_time {
                eprintln!("profile:worker_{worker_index}: {:?}", start_time.elapsed());
            }
            let _ = worker_result;
        });
    }
    drop(result_sender);

    profile_log("worker_threads_spawned");

    let results_start_time = if profile_enabled {
        Some(Instant::now())
    } else {
        None
    };
    let results_status = get_results(instructions, result_receiver);
    if let Some(start_time) = results_start_time {
        eprintln!("profile:get_results: {:?}", start_time.elapsed());
    }

    // Check if read_input thread encountered an I/O error
    if let Err(error) = reader_handle.join().unwrap() {
        eprintln!("{}", error);
        // Exit with code 2 for I/O errors
        let exit_code = if error.contains("failed to open") || error.contains("failed to create") {
            2
        } else {
            1
        };
        std::process::exit(exit_code);
    }

    if let Err(error) = results_status {
        eprintln!("{}", error);
        // Exit with code 2 for I/O errors, code 1 for other errors
        let exit_code = if error.contains("failed to open") || error.contains("failed to create") {
            2
        } else {
            1
        };
        std::process::exit(exit_code);
    }
}

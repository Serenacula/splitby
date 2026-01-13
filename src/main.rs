mod types;
mod worker;
use crate::types::*;
use crate::worker::*;
use clap::Parser;
use crossbeam::channel;
use fancy_regex::Regex as FancyRegex;
use regex::Regex as SimpleRegex;
use std::{
    cmp::max,
    collections::BTreeMap,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::PathBuf,
    sync::Arc,
};

// CLI Parser: Uses clap to handle the basic setup

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

    // Input mode
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
    let options = Options::parse();

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

    // SELECTIONS

    // First, work out the mode we're in
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

    // Merge all raw selection sources and parse
    let mut selection_strings: Vec<String> = Vec::new();
    match selection_mode {
        SelectionMode::Fields => selection_strings.extend(options.field_list.iter().cloned()),
        SelectionMode::Bytes => selection_strings.extend(options.byte_list.iter().cloned()),
        SelectionMode::Chars => selection_strings.extend(options.char_list.iter().cloned()),
    }
    selection_strings.extend(options.selection_list.iter().cloned());

    // PARSING SELECTIONS - defined early so we can reuse it for auto-detection

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

        // First try to parse the whole selection
        if let Ok(value) = parse_number(string) {
            return Ok((value, value));
        }

        // Okay, this is either a range or something invalid, so we need to find the two parts to it
        // Gonna tear this out an just use regex later, but it's good enough for now
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

    // Helper: check if string can be parsed as selection(s), including comma-separated
    fn can_parse_as_selection(string: &str) -> bool {
        if string == "," {
            return false; // Just a comma is a delimiter, not a selection
        }
        if string.contains(',') {
            // Check if any comma-separated part is a valid selection
            string.split(',').any(|part| {
                let trimmed = part.trim();
                !trimmed.is_empty() && parse_selection(trimmed).is_ok()
            })
        } else {
            parse_selection(string).is_ok()
        }
    }

    // Helper: check if string is a valid regex
    fn is_valid_regex(pattern: &str) -> bool {
        SimpleRegex::new(pattern).is_ok() || FancyRegex::new(pattern).is_ok()
    }

    // Automatic delimiter detection (only if -d flag not set and in fields mode)
    // Priority: selections take precedence. If not a selection and valid regex, use as delimiter
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

    // Check if delimiter is required (after auto-detection)
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

        // For all selections after first -> selections (always parse)
        // For first selection, check ambiguity only if delimiter wasn't set
        let should_check_ambiguity = is_first && !delimiter_was_set;

        // Check if this string contains commas
        if trimmed.contains(',') {
            // Split by commas and check each part
            let parts: Vec<&str> = trimmed.split(',').collect();

            // For the first selection string, check if it's ambiguous
            if should_check_ambiguity {
                // If the whole string is just a comma, it's a delimiter
                if trimmed == "," {
                    continue; // Skip this string, it's a delimiter
                }

                // Check if any part contains letters (not numeric)
                // If so, the whole string is a delimiter
                let has_letter = parts.iter().any(|part| {
                    let trimmed_part = part.trim();
                    !trimmed_part.is_empty()
                        && trimmed_part
                            .chars()
                            .any(|char| char.is_alphabetic() && char != '-')
                });

                if has_letter {
                    continue; // Skip this string, it's a delimiter
                }
            }

            // Parse each comma-separated part as a selection
            for part in parts {
                let trimmed_part = part.trim();
                if trimmed_part.is_empty() {
                    continue; // Skip empty parts (e.g., ",1" or "1,")
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
            // No commas, parse as single selection
            // For first selection, check ambiguity
            if should_check_ambiguity {
                // If it's just a comma, it's a delimiter
                if trimmed == "," {
                    continue; // Skip this string, it's a delimiter
                }

                // If it contains letters (not numeric), it's a delimiter
                if trimmed
                    .chars()
                    .any(|char| char.is_alphabetic() && char != '-')
                {
                    continue; // Skip this string, it's a delimiter
                }
            }

            let (start, end) = match parse_selection(trimmed) {
                Ok(range) => range,
                Err(_) => {
                    // For first selection, if parsing fails and delimiter wasn't set,
                    // it might be a delimiter (but we already checked for letters above)
                    eprintln!("invalid selection: '{trimmed}'");
                    std::process::exit(2);
                }
            };

            selections.push((start, end));
        }
    }

    // We don't want to compile this inside the workers, so it gets done here
    let regex_engine: Option<RegexEngine> = match selection_mode {
        SelectionMode::Bytes | SelectionMode::Chars => None,
        SelectionMode::Fields => {
            // Use -d flag if set, otherwise use detected delimiter
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

            // Compile regex - try simple first, fall back to fancy if needed
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

    // Parse placeholder value (hex for byte mode, string for text modes)
    // Take the last value if multiple are provided (last flag wins)
    let placeholder_value: Option<Vec<u8>> =
        if let Some(placeholder_str) = options.placeholder.last() {
            // Check if it's a hex value (starts with 0x)
            if placeholder_str.starts_with("0x") || placeholder_str.starts_with("0X") {
                // Parse hex value (single byte for byte mode)
                let hex_str = &placeholder_str[2..];
                match u8::from_str_radix(hex_str, 16) {
                    Ok(byte_value) => Some(vec![byte_value]),
                    Err(_) => {
                        eprintln!("invalid hex value for placeholder: {}", placeholder_str);
                        std::process::exit(2);
                    }
                }
            } else {
                // String value (for text modes)
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

    let (record_sender, record_receiver) = channel::bounded::<Record>(1024);
    let (result_sender, result_receiver) = channel::bounded::<RecordResult>(1024);

    fn read_input(
        input_mode: &InputMode,
        input_path: &Option<PathBuf>,
        record_sender: channel::Sender<Record>,
    ) -> Result<(), String> {
        let mut reader: Box<dyn BufRead> = match input_path.as_ref() {
            Some(path) => {
                let file = File::open(path)
                    .map_err(|error| format!("failed to open {}: {error}", path.display()))?;
                Box::new(BufReader::new(file))
            }
            None => {
                let stdin = io::stdin();
                Box::new(stdin.lock())
            }
        };
        let mut index: usize = 0;

        match input_mode {
            InputMode::PerLine => {
                let mut buffer: Vec<u8> = Vec::new();
                loop {
                    let bytes_read = reader
                        .read_until(b'\n', &mut buffer)
                        .map_err(|error| format!("{error}"))?;
                    if bytes_read == 0 {
                        return Ok(()); // EOF
                    }

                    // Check if this is just a trailing newline before removing it
                    // (bytes_read == 1 means we only read the newline character)
                    if bytes_read == 1 && buffer == [b'\n'] {
                        // Peek ahead without consuming to check if we're at EOF
                        let peek = reader.fill_buf().map_err(|error| format!("{error}"))?;
                        if peek.is_empty() {
                            // Trailing newline at EOF - skip it
                            buffer.clear();
                            continue;
                        }
                        // Empty line in the middle - process it normally below
                    }

                    // Remove newline (and carriage return if present)
                    if buffer.last() == Some(&b'\n') {
                        buffer.pop();
                        if buffer.last() == Some(&b'\r') {
                            buffer.pop();
                        }
                    }

                    record_sender
                        .send(Record {
                            index: index,
                            bytes: std::mem::take(&mut buffer),
                        })
                        .map_err(|error| format!("{error}"))?;

                    index += 1;
                }
            }
            InputMode::ZeroTerminated => {
                let mut buffer: Vec<u8> = Vec::new();
                loop {
                    let bytes_read = reader
                        .read_until(b'\0', &mut buffer)
                        .map_err(|error| format!("error while reading: {error}"))?;
                    if bytes_read == 0 {
                        return Ok(());
                    }

                    if buffer.last() == Some(&b'\0') {
                        buffer.pop();
                    }

                    record_sender
                        .send(Record {
                            index: index,
                            bytes: std::mem::take(&mut buffer),
                        })
                        .map_err(|error| format!("{error}"))?;

                    index += 1;
                }
            }
            InputMode::WholeString => {
                let mut buffer: Vec<u8> = Vec::new();
                reader
                    .read_to_end(&mut buffer)
                    .map_err(|error| format!("{error}"))?;

                record_sender
                    .send(Record {
                        index: index,
                        bytes: buffer,
                    })
                    .map_err(|error| format!("{error}"))?;

                Ok(())
            }
        }
    }

    fn process_records(
        instructions: Arc<Instructions>,
        record_receiver: channel::Receiver<Record>,
        result_sender: channel::Sender<RecordResult>,
    ) -> Result<(), String> {
        loop {
            // Get the record
            let record = match record_receiver.recv() {
                Ok(record) => record,
                Err(_) => return Ok(()),
            };

            let record_index = record.index;

            let processed_result: Result<Vec<u8>, String> = match instructions.selection_mode {
                SelectionMode::Bytes => process_bytes(&instructions, record),
                SelectionMode::Chars => process_chars(&instructions, record),
                SelectionMode::Fields => {
                    let engine = instructions
                        .regex_engine
                        .as_ref()
                        .ok_or_else(|| "internal error: missing regex engine".to_string())?;
                    process_fields(&instructions, engine, record)
                }
            };

            match processed_result {
                Ok(bytes) => {
                    if instructions.strict_return && bytes.is_empty() {
                        let _ = result_sender.send(RecordResult::Err {
                            index: record_index,
                            error: "strict return error: empty field".to_string(),
                        });
                        return Ok(());
                    }
                    result_sender
                        .send(RecordResult::Ok {
                            index: record_index,
                            bytes,
                        })
                        .map_err(|error| error.to_string())?;
                }
                Err(error) => {
                    let _ = result_sender.send(RecordResult::Err {
                        index: record_index,
                        error,
                    });
                    return Ok(());
                }
            }
        }
    }

    fn get_results(
        instructions: Arc<Instructions>,
        result_receiver: channel::Receiver<RecordResult>,
    ) -> Result<(), String> {
        // Decide record terminator (what separates records in output)
        let record_terminator: Option<u8> = match instructions.input_mode {
            InputMode::PerLine => Some(b'\n'),
            InputMode::ZeroTerminated => Some(b'\0'),
            InputMode::WholeString => None,
        };

        // Output target (file or stdout)
        let mut writer: Box<dyn Write> = match &instructions.output {
            Some(path) => {
                let file = File::create(path)
                    .map_err(|error| format!("failed to create {}: {}", path.display(), error))?;
                Box::new(io::BufWriter::new(file))
            }
            None => {
                let stdout = io::stdout();
                Box::new(io::BufWriter::new(stdout.lock()))
            }
        };

        let mut next_index: usize = 0;
        let mut pending: BTreeMap<usize, Vec<u8>> = BTreeMap::new();
        let mut max_index_seen: Option<usize> = None;

        while let Ok(result) = result_receiver.recv() {
            match result {
                RecordResult::Err { index, error } => {
                    let index = index + 1;
                    match instructions.input_mode {
                        InputMode::WholeString => return Err(error),
                        InputMode::PerLine => return Err(format!("line {index}: {error}")),
                        InputMode::ZeroTerminated => {
                            return Err(format!("record {index}: {error}"));
                        }
                    }
                }
                RecordResult::Ok { index, bytes } => {
                    pending.insert(index, bytes);
                    max_index_seen = Some(max_index_seen.map_or(index, |max| max.max(index)));
                }
            }

            // Flush anything now in order (but buffer the last one if trim_newline is set)
            while let Some(&pending_index) = pending.keys().next() {
                if pending_index == next_index {
                    let is_last_result =
                        instructions.trim_newline && max_index_seen == Some(pending_index);

                    // If this is the last result and trim_newline is set, don't print it yet
                    // We'll print it after the channel closes
                    if is_last_result {
                        break;
                    }

                    if let Some(bytes) = pending.remove(&next_index) {
                        writer
                            .write_all(&bytes)
                            .map_err(|error| error.to_string())?;

                        if let Some(terminator_byte) = record_terminator {
                            writer
                                .write_all(&[terminator_byte])
                                .map_err(|error| error.to_string())?;
                        }

                        next_index += 1;
                    }
                } else {
                    break;
                }
            }
        }

        // Channel closed: flush remaining results
        // The last result (if trim_newline is set) won't get a terminator
        while let Some(bytes) = pending.remove(&next_index) {
            writer
                .write_all(&bytes)
                .map_err(|error| error.to_string())?;

            // Only add terminator if this is not the last result or trim_newline is false
            let is_last_result = instructions.trim_newline && max_index_seen == Some(next_index);

            if let Some(terminator_byte) = record_terminator {
                if !is_last_result {
                    writer
                        .write_all(&[terminator_byte])
                        .map_err(|error| error.to_string())?;
                }
            }

            next_index += 1;
        }

        // Channel closed: all senders dropped.
        // If anything remains pending, indices were skipped (worker died early, etc.)
        if !pending.is_empty() {
            let first_missing = next_index;
            return Err(format!(
                "result stream ended early: missing record {first_missing}"
            ));
        }

        if next_index == 0 {
            if instructions.count {
                writer.write_all(b"0").map_err(|error| error.to_string())?;
            }
            if instructions.strict_return {
                return Err("strict return check failed: no input received".to_string());
            }
            if instructions.strict_bounds && !instructions.selections.is_empty() {
                let (raw_start, _) = instructions.selections[0];
                return Err(format!(
                    "index ({}) out of bounds, must be between 1 and {}",
                    raw_start, 0
                ));
            }
        }

        writer.flush().map_err(|error| error.to_string())?;
        Ok(())
    }

    let reader_instructions = Arc::clone(&instructions);
    let reader_sender = record_sender.clone();
    let reader_handle = std::thread::spawn(move || {
        read_input(
            &reader_instructions.input_mode,
            &reader_instructions.input,
            reader_sender,
        )
    });
    drop(record_sender);

    // Check for single-core mode via environment variable (useful for macOS testing)
    let worker_count = if std::env::var("SPLITBY_SINGLE_CORE").is_ok() {
        1 // Single-core mode: only 1 worker thread
    } else {
        std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1)
    };

    for _ in 0..max(worker_count - 1, 1) {
        let worker_instructions = Arc::clone(&instructions);
        let worker_receiver = record_receiver.clone();
        let worker_sender = result_sender.clone();
        std::thread::spawn(move || {
            let _ = process_records(worker_instructions, worker_receiver, worker_sender)
                .map_err(|error| eprintln!("{error}"));
        });
    }
    drop(result_sender);

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

    if let Err(error) = get_results(instructions, result_receiver) {
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

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

    #[arg(long = "replace-range-delimiter", value_name = "STRING")]
    replace_range_delimiter: Option<String>,

    #[arg(short = 'e', long = "skip-empty")]
    skip_empty: bool,

    #[arg(short = 'E', long = "no-skip-empty")]
    no_skip_empty: bool,

    #[arg(long = "invert")]
    invert: bool,

    #[arg(short = 's', long = "strict")]
    strict: bool,

    #[arg(short = 'S', long = "no-strict")]
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

    #[arg(long = "placeholder")]
    placeholder: bool,

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

            "-s" | "--strict" => {
                strict_return = true;
                strict_bounds = true;
                strict_range_order = true;
                strict_utf8 = true;
            }
            "-S" | "--no-strict" => {
                strict_return = false;
                strict_bounds = false;
                strict_range_order = false;
                strict_utf8 = false
            }

            _ => {}
        }
    }

    // Set the join string default, if no join is provided
    // let join = match (&options.join, input_mode == InputMode::WholeString) {
    //     (Some(join_string), _) => join_string.clone(),
    //     (None, true) => "\n".to_string(), // whole-string default
    //     (None, false) => " ".to_string(), // per-line default
    // };

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

    if selection_mode == SelectionMode::Fields && options.delimiter.is_none() {
        eprintln!("Delimiter required: you can provide one with the -d <REGEX> flag");
        std::process::exit(2);
    }

    // Merge all raw selection sources and parse
    let mut selection_strings: Vec<String> = Vec::new();
    match selection_mode {
        SelectionMode::Fields => selection_strings.extend(options.field_list.iter().cloned()),
        SelectionMode::Bytes => selection_strings.extend(options.byte_list.iter().cloned()),
        SelectionMode::Chars => selection_strings.extend(options.char_list.iter().cloned()),
    }
    selection_strings.extend(options.selection_list.iter().cloned());

    // PARSING SELECTIONS

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

    let mut selections: Vec<(i32, i32)> = Vec::new();
    for string_raw in selection_strings {
        let (start, end) = match parse_selection(string_raw.as_str()) {
            Ok(range) => range,
            Err(_) => {
                eprintln!("invalid selection: '{string_raw}'");
                std::process::exit(2);
            }
        };

        // if start == 0 || end == 0 {
        //     eprintln!("0 is not a valid selection, selections are 1-based");
        //     std::process::exit(2);
        // }

        selections.push((start, end));
    }

    // We don't want to compile this inside the workers, so it gets done here
    let regex_engine: Option<RegexEngine> = match selection_mode {
        SelectionMode::Bytes | SelectionMode::Chars => None,
        SelectionMode::Fields => {
            let delimiter: String = options.delimiter.unwrap_or_else(|| {
                eprintln!("error: delimiter required in Fields mode");
                std::process::exit(2)
            });
            let simple_regex = SimpleRegex::new(&delimiter);

            match simple_regex {
                Ok(regex) => Some(RegexEngine::Simple(regex)),
                Err(_) => {
                    let fancy_regex = FancyRegex::new(&delimiter).unwrap_or_else(|error| {
                        eprintln!("error: failed to compile regex: {error}");
                        std::process::exit(2)
                    });
                    Some(RegexEngine::Fancy(fancy_regex))
                }
            }
        }
    };

    let instructions = Arc::new(Instructions {
        input_mode: input_mode,
        input: options.input,
        selection_mode: selection_mode,
        selections: selections,
        invert: options.invert,
        skip_empty: skip_empty,
        placeholder: options.placeholder,
        strict_return: strict_return,
        strict_bounds: strict_bounds,
        strict_range_order: strict_range_order,
        strict_utf8: strict_utf8,
        output: options.output,
        count: options.count,
        join: options.join,
        replace_range_delimiter: options.replace_range_delimiter,

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

    // fn split_fields(
    //     record: Record,
    //     selection_mode: SelectionMode,
    //     regex_engine: Option<RegexEngine>,
    // ) -> Result<Vec<u8>, String> {
    //     fn split_by_byte(record: Record) -> Vec<u8> {
    //         return record.text.as_bytes().to_vec();
    //     }

    //     match selection_mode {
    //         SelectionMode::Bytes => Ok(record.text.as_bytes().to_vec()),
    //         SelectionMode::Chars => Ok(record.text.chars()),
    //     }
    // }

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

            // Split the record into fields

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

            // Count and exit if --count

            // Choose our selections and ranges

            match processed_result {
                Ok(bytes) => {
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

        // Output target (stdout for now; file can come next)
        let stdout = io::stdout();
        let mut writer = io::BufWriter::new(stdout.lock());

        let mut next_index: usize = 0;
        let mut pending: BTreeMap<usize, Vec<u8>> = BTreeMap::new();

        while let Ok(result) = result_receiver.recv() {
            match result {
                RecordResult::Err { index, error } => {
                    return Err(format!("line {index}: {error}"));
                }
                RecordResult::Ok { index, bytes } => {
                    pending.insert(index, bytes);
                }
            }

            // Flush anything now in order
            while let Some(bytes) = pending.remove(&next_index) {
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
        }

        // Channel closed: all senders dropped.
        // If anything remains pending, indices were skipped (worker died early, etc.)
        if !pending.is_empty() {
            let first_missing = next_index;
            return Err(format!(
                "result stream ended early; missing record {first_missing}"
            ));
        }

        writer.flush().map_err(|error| error.to_string())?;
        Ok(())
    }

    let reader_instructions = Arc::clone(&instructions);
    let reader_sender = record_sender.clone();
    std::thread::spawn(move || {
        let _ = read_input(
            &reader_instructions.input_mode,
            &reader_instructions.input,
            reader_sender,
        )
        .map_err(|error| eprintln!("{error}"));
    });
    drop(record_sender);

    let worker_count = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1);

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

    if let Err(error) = get_results(instructions, result_receiver) {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}

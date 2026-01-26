mod help_version;
mod parse;
mod types;
mod validation;

use self::parse::*;
use self::types::*;
use self::validation::*;
use crate::types::*;

use fancy_regex::Regex as FancyRegex;
use regex::Regex as SimpleRegex;
use std::env;

/// Parse command line arguments and return Instructions
pub fn get_instructions() -> Result<Option<Instructions>, String> {
    let args: Vec<String> = env::args().skip(1).collect();

    // So the logic here is this:
    // - If previous token was consuming flag, treat arg as input for that flag
    //     - join flag
    //     - placeholder flag
    //     - delimiter flag
    //     - align flag
    //         - if it isn't a specific align flag, assume NO FLAG and keep parsing
    // - If is known flag, assume it's a flag
    // - Check if selection:
    //     - selection regex only works on single item, so we need to break it up first with split_regex
    //     - if first arg isn't selection, continue
    //     - check each item with selection_regex. If a subsequent selection fails, error
    //     - put selections into selection list
    // - If delimiter is not set, assume it is a delimiter
    // - Otherwise error: "Invalid argument: {arg}"

    let mut cliArguments = CLIArguments {
        output: None,
        input: None,
        join: None,
        delimiter: None,
        placeholder: None,
        align: Align::None,
        input_mode: InputMode::PerLine,
        selection_mode: SelectionMode::Fields,
        count: false,
        invert: false,
        skip_empty: false,
        strict_utf8: false,
        strict_return: false,
        strict_bounds: false,
        strict_range_order: true,
        selections: Vec::new(),
    };

    let mut consuming = Consuming {
        input: false,
        output: false,
        delim: false,
        join: false,
        placeholder: false,
        align: false,
    };

    let split_regex = SimpleRegex::new(r"[, ]").unwrap();
    let selection_regex = SimpleRegex::new(
        r"^(?i)(?P<start>start|first|end|last|-?\d+)(?:-(?P<end>start|first|end|last|-?\d+))?$",
    )
    .unwrap();
    for arg in args {
        match parse_flags(&arg, &mut consuming, &mut cliArguments) {
            Ok(ParseResult::FlagParsed) => continue,
            Ok(ParseResult::Finished) => return Ok(None),
            Err(e) => return Err(e),
            _ => {
                // No flag parsed, keep going
            }
        }

        // First, check if the whole arg is a single selection token (e.g., "2" or "1-3")
        if selection_regex.is_match(&arg) {
            let parse = parse_selection_token(&arg, &selection_regex);
            match parse {
                Ok(selection) => {
                    cliArguments.selections.push(selection);
                    continue;
                }
                Err(_) => {
                    // This should never happen, but just in case
                    return Err(format!("invalid selection: {}", arg));
                }
            }
        }

        // If it contains commas or spaces, split and check each part
        if arg.contains(',') || arg.contains(' ') {
            let tokens: Vec<&str> = arg.split(|c| c == ',' || c == ' ').collect();
            let mut first_non_empty: Option<&str> = None;
            for token in &tokens {
                let trimmed = token.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if first_non_empty.is_none() {
                    first_non_empty = Some(trimmed);
                }
            }

            // If first non-empty part is a selection, all parts must be selections
            if let Some(first) = first_non_empty {
                if selection_regex.is_match(first) {
                    for token in &tokens {
                        let trimmed = token.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        if !selection_regex.is_match(trimmed) {
                            return Err(format!("invalid selection: {}", trimmed));
                        }
                        let parse = parse_selection_token(trimmed, &selection_regex);
                        match parse {
                            Ok(selection) => cliArguments.selections.push(selection),
                            Err(error) => return Err(error),
                        }
                    }
                    continue;
                }
            }
        }
        // The only possibility left is a bad flag or implicit delimiter
        // First, make sure it isn't a bad flag
        if arg.starts_with("-") {
            return Err(format!("invalid flag: {}", arg));
        }
        // If it's not a selection or flag and we have no delimiter yet, assume it's an implicit
        if cliArguments.delimiter.is_none() {
            cliArguments.delimiter = Some(arg);
            continue;
        }
        // We already have a delimiter, nothing left for it to be
        return Err(format!("invalid argument: {}", arg));
    }

    // Handle validations
    let join: Option<JoinMode> = match cliArguments.join {
        Some(join) => {
            validate_join_mode(&join, cliArguments.selection_mode).map_err(|e| e.to_string())?;
            parse_join(&join)
        }
        None => None,
    };

    let placeholder: Option<Vec<u8>> = match cliArguments.placeholder {
        Some(placeholder) => parse_placeholder(&placeholder),
        None => None,
    };

    validate_align(
        cliArguments.align,
        cliArguments.input_mode,
        cliArguments.selection_mode,
    )
    .map_err(|e| e.to_string())?;

    let regex_engine: Option<RegexEngine> = match cliArguments.selection_mode {
        SelectionMode::Bytes | SelectionMode::Chars => None,
        SelectionMode::Fields => {
            let delimiter: String = cliArguments.delimiter.unwrap_or_else(|| {
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

    // TODO: Parse arguments and build Instructions
    // - Classify each arg as flag, delimiter, or selection
    // - Handle flag values
    // - Build Instructions struct

    // TODO: Parse arguments and build Instructions
    // - Classify each arg as flag, delimiter, or selection
    // - Handle flag values
    // - Build Instructions struct

    // Placeholder - replace with actual parsing logic

    let input_instructions = InputInstructions {
        regex_engine: regex_engine.clone(),
        align: cliArguments.align,
        input_mode: cliArguments.input_mode,
        input: cliArguments.input,
        selections: cliArguments.selections.clone(),
        skip_empty: cliArguments.skip_empty,
        invert: cliArguments.invert,
        placeholder: placeholder.clone(),
        strict_bounds: cliArguments.strict_bounds,
        strict_range_order: cliArguments.strict_range_order,
        strict_utf8: cliArguments.strict_utf8,
    };

    let transform_instructions = TransformInstructions {
        input_mode: cliArguments.input_mode,
        selection_mode: cliArguments.selection_mode,
        selections: cliArguments.selections.clone(),
        invert: cliArguments.invert,
        skip_empty: cliArguments.skip_empty,
        placeholder: placeholder.clone(),
        strict_return: cliArguments.strict_return,
        strict_bounds: cliArguments.strict_bounds,
        strict_range_order: cliArguments.strict_range_order,
        strict_utf8: cliArguments.strict_utf8,
        count: cliArguments.count,
        join: join,
        regex_engine: regex_engine,
        align: cliArguments.align,
    };

    let output_instructions = OutputInstructions {
        output: cliArguments.output,
        input_mode: cliArguments.input_mode,
        selections: cliArguments.selections.clone(),
        strict_bounds: cliArguments.strict_bounds,
        strict_return: cliArguments.strict_return,
        count: cliArguments.count,
    };

    Ok(Some(Instructions {
        input_instructions,
        transform_instructions,
        output_instructions,
    }))
}

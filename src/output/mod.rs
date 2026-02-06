use crossbeam::channel;
use std::{
    collections::BTreeMap,
    io::{self, Write},
};

use crate::types::*;

pub fn get_results(
    output_instructions: OutputInstructions,
    result_receiver: channel::Receiver<ResultChunk>,
) -> Result<(), String> {
    let record_terminator: Option<u8> = match output_instructions.input_mode {
        InputMode::PerLine => Some(b'\n'),
        InputMode::ZeroTerminated => Some(b'\0'),
        InputMode::WholeString => None,
    };

    let mut writer: Box<dyn Write> = match &output_instructions.output {
        Some(path) => {
            let file = std::fs::File::create(path)
                .map_err(|error| format!("failed to create {}: {}", path.display(), error))?;
            Box::new(io::BufWriter::new(file))
        }
        None => {
            let stdout = io::stdout();
            Box::new(io::BufWriter::new(stdout.lock()))
        }
    };

    let output_flush_threshold = std::env::var("SPLITBY_OUTPUT_FLUSH")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(64 * 1024);
    let mut next_index: usize = 0;
    let mut pending: BTreeMap<usize, Vec<OutputRecord>> = BTreeMap::new();
    let mut output_buffer: Vec<u8> = Vec::with_capacity(output_flush_threshold * 2);

    let flush_output =
        |writer: &mut Box<dyn Write>, output_buffer: &mut Vec<u8>| -> Result<(), String> {
            if output_buffer.is_empty() {
                return Ok(());
            }
            writer
                .write_all(output_buffer)
                .map_err(|error| error.to_string())?;
            output_buffer.clear();
            Ok(())
        };

    while let Ok(result) = result_receiver.recv() {
        match result {
            ResultChunk::Err { index, error } => {
                let index = index + 1;
                match output_instructions.input_mode {
                    InputMode::WholeString => return Err(error),
                    InputMode::PerLine => return Err(format!("line {index}: {error}")),
                    InputMode::ZeroTerminated => {
                        return Err(format!("record {index}: {error}"));
                    }
                }
            }
            ResultChunk::Ok {
                start_index,
                outputs,
            } => {
                pending.insert(start_index, outputs);
            }
        }

        while let Some(&pending_index) = pending.keys().next() {
            if pending_index == next_index {
                if let Some(outputs) = pending.remove(&next_index) {
                    let base_index = next_index;
                    let mut offset = 0usize;

                    while offset < outputs.len() {
                        let output_record = &outputs[offset];
                        output_buffer.extend_from_slice(&output_record.bytes);
                        if let Some(terminator_byte) = record_terminator {
                            if output_record.has_terminator {
                                output_buffer.push(terminator_byte);
                            }
                        }

                        if output_buffer.len() >= output_flush_threshold {
                            flush_output(&mut writer, &mut output_buffer)?;
                        }

                        next_index = base_index + offset + 1;
                        offset += 1;
                    }
                }
            } else {
                break;
            }
        }
    }

    while let Some(outputs) = pending.remove(&next_index) {
        for output_record in outputs {
            output_buffer.extend_from_slice(&output_record.bytes);
            if let Some(terminator_byte) = record_terminator {
                if output_record.has_terminator {
                    output_buffer.push(terminator_byte);
                }
            }

            next_index += 1;
        }
    }

    if !pending.is_empty() {
        let first_missing = next_index;
        return Err(format!(
            "result stream ended early: missing record {first_missing}"
        ));
    }

    if next_index == 0 {
        if output_instructions.count {
            writer.write_all(b"0").map_err(|error| error.to_string())?;
        }
        if output_instructions.strict_return {
            return Err("strict return check failed: no input received".to_string());
        }
        if output_instructions.strict_bounds && !output_instructions.selections.is_empty() {
            let (raw_start, _) = output_instructions.selections[0];
            return Err(format!(
                "index ({}) out of bounds, must be between 1 and {}",
                raw_start, 0
            ));
        }
    }

    // Whole-string mode: ensure output ends with a newline if it has content
    if output_instructions.input_mode == InputMode::WholeString
        && !output_buffer.is_empty()
        && output_buffer.last() != Some(&b'\n')
    {
        output_buffer.push(b'\n');
    }

    flush_output(&mut writer, &mut output_buffer)?;
    writer.flush().map_err(|error| error.to_string())?;
    Ok(())
}

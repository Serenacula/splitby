use std::borrow::Cow;

use crate::processing::process_records::worker_utilities::{
    Field, bytes_to_cow_string, estimate_field_count, estimate_output_size, normalise_selection,
};
use crate::types::*;

pub fn process_fields(
    instructions: &Instructions,
    engine: &RegexEngine,
    record: Record,
) -> Result<Vec<u8>, String> {
    let text: Cow<str> = match bytes_to_cow_string(&record.bytes, instructions.strict_utf8) {
        Ok(string) => string,
        Err(e) => return Err(e),
    };

    let delimiter_len = match engine {
        RegexEngine::Simple(regex) => regex.as_str().len(),
        RegexEngine::Fancy(_) => 1,
    };
    let estimated_field_count = estimate_field_count(record.bytes.len(), delimiter_len);
    let mut fields: Vec<Field> = Vec::with_capacity(estimated_field_count);
    let mut cursor = 0usize;

    match engine {
        RegexEngine::Simple(engine) => {
            for delimiter in engine.find_iter(&text) {
                fields.push(Field {
                    text: text[cursor..delimiter.start()].as_bytes(),
                    delimiter: text[delimiter.start()..delimiter.end()].as_bytes(),
                });
                cursor = delimiter.end();
            }
        }
        RegexEngine::Fancy(engine) => {
            for delimiter_result in engine.find_iter(&text) {
                match delimiter_result {
                    Ok(delimiter) => {
                        fields.push(Field {
                            text: text[cursor..delimiter.start()].as_bytes(),
                            delimiter: text[delimiter.start()..delimiter.end()].as_bytes(),
                        });
                        cursor = delimiter.end();
                    }
                    Err(error) => {
                        return Err(format!("regex matching error: {}", error));
                    }
                }
            }
        }
    }

    // Don't add an empty field at the end for whole-string
    let final_text = text[cursor..text.len()].as_bytes();
    if !final_text.is_empty() || instructions.input_mode != InputMode::WholeString {
        fields.push(Field {
            text: text[cursor..text.len()].as_bytes(),
            delimiter: b"",
        });
    }

    if instructions.skip_empty {
        fields = fields
            .into_iter()
            .filter(|field| !field.text.is_empty())
            .collect();
    }

    if instructions.count {
        let count = fields.len();
        return Ok(count.to_string().into_bytes());
    }

    if fields.is_empty() {
        return Ok(Vec::new());
    }

    if instructions.selections.is_empty() {
        if instructions.invert {
            return Ok(Vec::new());
        }
        let mut result: Vec<u8> = Vec::new();
        for field in fields {
            result.extend_from_slice(field.text);
            result.extend_from_slice(field.delimiter);
        }
        return Ok(result);
    }

    let mut normalised_selections: Vec<(usize, usize)> =
        Vec::with_capacity(instructions.selections.len());
    for &(start, end) in &instructions.selections {
        match normalise_selection(
            start,
            end,
            fields.len(),
            instructions.placeholder.is_some(),
            instructions.strict_bounds,
            instructions.strict_range_order,
        ) {
            Ok(Some(range)) => normalised_selections.push(range),
            Ok(None) => {}
            Err(error) => return Err(error),
        }
    }

    let selections = if !instructions.invert {
        if !normalised_selections.is_empty() {
            normalised_selections
        } else {
            vec![(0, fields.len().saturating_sub(1))]
        }
    } else {
        normalised_selections.sort_by(|(start_a, end_a), (start_b, end_b)| {
            start_a.cmp(start_b).then(end_a.cmp(end_b))
        });
        let mut merged: Vec<(usize, usize)> = Vec::with_capacity(normalised_selections.len());
        for (start, end) in normalised_selections {
            if let Some((_, last_end)) = merged.last_mut() {
                if start <= *last_end {
                    *last_end = (*last_end).max(end);
                    continue;
                }
            }
            merged.push((start, end));
        }

        let mut invert_pointer: usize = 0;
        let mut inverted: Vec<(usize, usize)> = Vec::with_capacity(merged.len());
        for (start, end) in &merged {
            if *start > invert_pointer {
                inverted.push((invert_pointer, start.saturating_sub(1)));
            }
            invert_pointer = end.saturating_add(1);
        }
        if invert_pointer < fields.len() {
            inverted.push((invert_pointer, fields.len().saturating_sub(1)));
        }
        inverted
    };

    let estimated_output_size = estimate_output_size(record.bytes.len(), selections.len());
    let mut output: Vec<u8> = Vec::with_capacity(estimated_output_size);

    for (selection_index, selection) in selections.iter().enumerate() {
        for field_index in selection.0..=selection.1 {
            let has_data = field_index < fields.len()
                || (instructions.placeholder.is_some() && !instructions.invert);

            if !has_data {
                continue;
            }

            if field_index < fields.len() {
                output.extend_from_slice(fields[field_index].text);
            } else if let Some(placeholder) = &instructions.placeholder {
                output.extend_from_slice(placeholder);
            }

            let is_last = selection_index == selections.len() - 1 && field_index == selection.1;
            if !is_last {
                let previous_delimiter = if field_index > 0 {
                    fields[field_index - 1].delimiter
                } else {
                    b""
                };
                let current_delimiter = if field_index < fields.len() {
                    fields[field_index].delimiter
                } else {
                    b""
                };
                if let Some(join) = &instructions.join {
                    output.extend_from_slice(join)
                } else if instructions.input_mode == InputMode::WholeString {
                    output.push(b'\n');
                } else if !previous_delimiter.is_empty() {
                    output.extend_from_slice(previous_delimiter);
                } else if !current_delimiter.is_empty() {
                    output.extend_from_slice(current_delimiter);
                } else {
                    output.push(b' ');
                }
            }
        }
    }

    Ok(output)
}

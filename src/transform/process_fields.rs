use std::borrow::Cow;

use crate::transform::worker_utilities::*;
use crate::types::*;

pub fn process_fields(
    transform_instructions: &TransformInstructions,
    engine: &RegexEngine,
    record: Record,
) -> Result<Vec<u8>, String> {
    let text: Cow<str> =
        match bytes_to_cow_string(&record.bytes, transform_instructions.strict_utf8) {
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
    if !final_text.is_empty() || transform_instructions.input_mode != InputMode::WholeString {
        fields.push(Field {
            text: text[cursor..text.len()].as_bytes(),
            delimiter: b"",
        });
    }

    if transform_instructions.skip_empty {
        fields = fields
            .into_iter()
            .filter(|field| !field.text.is_empty())
            .collect();
    }

    if transform_instructions.count {
        let count = fields.len();
        return Ok(count.to_string().into_bytes());
    }

    if fields.is_empty() {
        return Ok(Vec::new());
    }

    let normalised_selections: Vec<(usize, usize)> = match normalise_selections(
        &transform_instructions.selections,
        fields.len(),
        transform_instructions.placeholder.is_some(),
        transform_instructions.strict_bounds,
        transform_instructions.strict_range_order,
    ) {
        Ok(result) => result,
        Err(error) => return Err(error),
    };

    let selections = if transform_instructions.selections.is_empty() {
        vec![(0, fields.len().saturating_sub(1))]
    } else if !transform_instructions.invert {
        normalised_selections
    } else {
        invert_selections(normalised_selections, fields.len())
    };

    let estimated_output_size = estimate_output_size(record.bytes.len(), selections.len());
    let mut output: Vec<u8> = Vec::with_capacity(estimated_output_size);
    let mut strict_return_passed: bool = false;

    // Find first and last delimiters for @first and @last join modes
    let first_delimiter = fields
        .iter()
        .find(|field| !field.delimiter.is_empty())
        .map(|field| field.delimiter)
        .unwrap_or(b"");
    let last_delimiter = fields
        .iter()
        .rev()
        .find(|field| !field.delimiter.is_empty())
        .map(|field| field.delimiter)
        .unwrap_or(b"");

    // Track field position for alignment
    let mut field_position: usize = 0;

    for (selection_index, selection) in selections.iter().enumerate() {
        for field_index in selection.0..=selection.1 {
            let has_data = field_index < fields.len()
                || (transform_instructions.placeholder.is_some() && !transform_instructions.invert);

            if !has_data {
                continue;
            }

            if field_index < fields.len() {
                if !fields[field_index].text.is_empty() {
                    output.extend_from_slice(fields[field_index].text);
                    strict_return_passed = true;
                }
            } else if let Some(placeholder) = &transform_instructions.placeholder {
                output.extend_from_slice(placeholder);
                strict_return_passed = true;
            }

            let is_last = selection_index == selections.len() - 1 && field_index == selection.1;
            if !is_last {
                let current_delimiter = fields[field_index].delimiter;
                let next_delimiter = if field_index < fields.len() - 1 {
                    fields[field_index + 1].delimiter
                } else {
                    b""
                };

                let join: &[u8] = match &transform_instructions.join {
                    Some(JoinMode::String(join_bytes)) => join_bytes,
                    Some(JoinMode::AfterPrevious) => {
                        if !current_delimiter.is_empty() {
                            current_delimiter
                        } else {
                            b" " // Fallback to space if no delimiter
                        }
                    }
                    Some(JoinMode::BeforeNext) => {
                        if !next_delimiter.is_empty() {
                            next_delimiter
                        } else {
                            b" " // Fallback to space if no delimiter
                        }
                    }
                    Some(JoinMode::First) => {
                        if !first_delimiter.is_empty() {
                            first_delimiter
                        } else {
                            b" " // Fallback to space if no delimiter
                        }
                    }
                    Some(JoinMode::Last) => {
                        if !last_delimiter.is_empty() {
                            last_delimiter
                        } else {
                            b" " // Fallback to space if no delimiter
                        }
                    }
                    Some(JoinMode::Space) => b" ",
                    Some(JoinMode::None) => {
                        b"" // No join - do nothing
                    }
                    None | Some(JoinMode::Auto) => {
                        // Default behavior
                        if !current_delimiter.is_empty() {
                            current_delimiter
                        } else if !next_delimiter.is_empty() {
                            next_delimiter
                        } else if !first_delimiter.is_empty() {
                            first_delimiter
                        } else {
                            b" "
                        }
                    }
                };
                output.extend_from_slice(join);

                // Add alignment padding after delimiter (not after final field)
                // Padding aligns the start of the next field
                if let Some(max_widths) = &record.field_widths {
                    if field_position < max_widths.len() {
                        let max_field_width = max_widths[field_position];
                        // Current field text width
                        let current_field_width = if field_index < fields.len() {
                            fields[field_index].text.len()
                        } else if let Some(placeholder) = &transform_instructions.placeholder {
                            placeholder.len()
                        } else {
                            0
                        };

                        // Padding aligns the start of the next field
                        // After max-width field + delimiter, next field starts at: max_field_width + delimiter_width + 1
                        // After current field + delimiter, next field starts at: current_field_width + delimiter_width + 1
                        // Padding needed = (max_field_width + delimiter_width + 1) - (current_field_width + delimiter_width + 1)
                        //                = max_field_width - current_field_width
                        let padding_needed = max_field_width.saturating_sub(current_field_width);
                        for _ in 0..padding_needed {
                            output.push(b' ');
                        }
                    }
                }

                field_position += 1;
            }
        }
    }

    if transform_instructions.strict_return && !strict_return_passed {
        Err("strict returns error: no valid output".to_string())
    } else {
        Ok(output)
    }
}

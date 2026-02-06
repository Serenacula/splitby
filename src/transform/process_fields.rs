use std::borrow::Cow;

use crate::transform::worker_utilities::*;
use crate::types::*;
use crate::width::display_width;

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
    let ansi_regex = transform_instructions.ansi_strip_regex.as_ref();
    let mut field_position: usize = 0;

    for (selection_index, selection) in selections.iter().enumerate() {
        for field_index in selection.0..=selection.1 {
            // Skip if there's no data in this field
            let has_data = field_index < fields.len()
                || (transform_instructions.placeholder.is_some() && !transform_instructions.invert);

            if !has_data {
                continue;
            }

            // Padding setup for align
            let max_field_width = if let Some(max_widths) = &record.field_widths {
                if field_position < max_widths.len() {
                    max_widths[field_position]
                } else {
                    0
                }
            } else {
                0
            };
            let current_field_width = if field_index < fields.len() {
                display_width(fields[field_index].text, ansi_regex)
            } else if let Some(placeholder) = &transform_instructions.placeholder {
                display_width(placeholder, ansi_regex)
            } else {
                0
            };
            let padding_needed = max_field_width.saturating_sub(current_field_width);

            // Push the padding needed for the field
            let push_padding = |output: &mut Vec<u8>, padding_needed: usize| {
                for _ in 0..padding_needed {
                    output.push(b' ');
                }
            };

            // Push the text of the field
            let push_text = |output: &mut Vec<u8>, strict_return_passed: &mut bool| {
                if field_index < fields.len() {
                    if !fields[field_index].text.is_empty() {
                        output.extend_from_slice(fields[field_index].text);
                        *strict_return_passed = true;
                    }
                } else if let Some(placeholder) = &transform_instructions.placeholder {
                    output.extend_from_slice(placeholder);
                    *strict_return_passed = true;
                }
            };

            // Decide on the join string and push
            let push_join = |output: &mut Vec<u8>| -> usize {
                let join = choose_join_bytes(
                    field_index,
                    selection_index,
                    &selections,
                    &fields,
                    transform_instructions.join.as_ref(),
                    first_delimiter,
                    last_delimiter,
                    transform_instructions.placeholder.is_some(),
                    transform_instructions.invert,
                );
                output.extend_from_slice(join);
                display_width(join, ansi_regex)
            };

            // Add the field text or placeholder
            if transform_instructions.align == Align::Right {
                push_padding(&mut output, padding_needed);
            }
            push_text(&mut output, &mut strict_return_passed);

            let is_last = selection_index == selections.len() - 1 && field_index == selection.1;
            if !is_last {
                if transform_instructions.align == Align::Left {
                    push_padding(&mut output, padding_needed);
                }
                let join_len = push_join(&mut output);
                if let Some(max_join_widths) = &record.join_widths
                    && field_position < max_join_widths.len()
                {
                    let max_join_width = max_join_widths[field_position];
                    if max_join_width > join_len {
                        push_padding(&mut output, max_join_width - join_len);
                    }
                }
                if transform_instructions.align == Align::Squash {
                    push_padding(&mut output, padding_needed);
                }
            };

            field_position += 1;
        }
    }

    if transform_instructions.strict_return && !strict_return_passed {
        Err("strict returns error: no valid output".to_string())
    } else {
        Ok(output)
    }
}

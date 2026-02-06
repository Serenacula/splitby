use std::borrow::Cow;

use crate::transform::transform_utilities::{
    Field, bytes_to_cow_string, choose_join_bytes, invert_selections, normalise_selections,
};
use crate::types::{InputInstructions, InputMode, Record, RegexEngine};
use crate::utilities::display_width;

/// This is used when the --align flag is used, to get the largest field widths for each record.
pub fn get_largest_field_widths(
    records: &[Record],
    input_instructions: &InputInstructions,
) -> Result<(Vec<usize>, Vec<usize>), String> {
    if records.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }

    let engine = input_instructions
        .regex_engine
        .as_ref()
        .ok_or_else(|| "internal error: missing regex engine".to_string())?;

    let mut max_widths: Vec<usize> = Vec::new();
    let mut max_join_widths: Vec<usize> = Vec::new();

    for record in records {
        let text: Cow<str> =
            match bytes_to_cow_string(&record.bytes, input_instructions.strict_utf8) {
                Ok(string) => string,
                Err(e) => return Err(e),
            };

        // Extract fields using regex
        let mut fields: Vec<Field> = Vec::new();
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
        if !final_text.is_empty() || input_instructions.input_mode != InputMode::WholeString {
            fields.push(Field {
                text: final_text,
                delimiter: b"",
            });
        }

        // Apply skip_empty filter
        if input_instructions.skip_empty {
            fields = fields
                .into_iter()
                .filter(|field| !field.text.is_empty())
                .collect();
        }

        if fields.is_empty() {
            continue;
        }

        // Normalize selections
        let normalised_selections: Vec<(usize, usize)> = match normalise_selections(
            &input_instructions.selections,
            fields.len(),
            input_instructions.placeholder.is_some(),
            input_instructions.strict_bounds,
            input_instructions.strict_range_order,
        ) {
            Ok(result) => result,
            Err(_) => continue, // Skip records with invalid selections
        };

        // Apply invert if needed
        let selections = if input_instructions.selections.is_empty() {
            vec![(0, fields.len().saturating_sub(1))]
        } else if !input_instructions.invert {
            normalised_selections
        } else {
            invert_selections(normalised_selections, fields.len())
        };

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

        // Determine which field positions will be output and measure their widths
        let mut position_index = 0;
        for (selection_index, selection) in selections.iter().enumerate() {
            for field_index in selection.0..=selection.1 {
                let field_width = if field_index < fields.len() {
                    display_width(fields[field_index].text)
                } else if let Some(placeholder) = &input_instructions.placeholder
                    && !input_instructions.invert
                {
                    display_width(placeholder)
                } else {
                    continue; // Skip if no placeholder and out of bounds
                };

                // Ensure max_widths vec is large enough
                if position_index >= max_widths.len() {
                    max_widths.resize(position_index + 1, 0);
                }

                // Update max width for this position
                if field_width > max_widths[position_index] {
                    max_widths[position_index] = field_width;
                }

                // Track max join width for the gap after this position (if not last)
                let is_last = selection_index == selections.len() - 1 && field_index == selection.1;
                if !is_last {
                    if position_index >= max_join_widths.len() {
                        max_join_widths.resize(position_index + 1, 0);
                    }
                    let join_bytes = choose_join_bytes(
                        field_index,
                        selection_index,
                        &selections,
                        &fields,
                        input_instructions.join.as_ref(),
                        first_delimiter,
                        last_delimiter,
                        input_instructions.placeholder.is_some(),
                        input_instructions.invert,
                    );
                    let join_width = display_width(&join_bytes);
                    if join_width > max_join_widths[position_index] {
                        max_join_widths[position_index] = join_width;
                    }
                }

                position_index += 1;
            }
        }
    }

    Ok((max_widths, max_join_widths))
}

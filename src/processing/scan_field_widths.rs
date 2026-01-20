use std::borrow::Cow;

use crate::processing::process_records::worker_utilities::{
    bytes_to_cow_string, invert_selections, normalise_selections, Field,
};
use crate::types::{InputMode, Record, RegexEngine};
use crate::ReaderInstructions;

pub fn scan_field_widths(
    records: &[Record],
    reader_instructions: &ReaderInstructions,
) -> Result<Vec<usize>, String> {
    if records.is_empty() {
        return Ok(Vec::new());
    }

    let engine = reader_instructions
        .regex_engine
        .as_ref()
        .ok_or_else(|| "internal error: missing regex engine".to_string())?;

    let mut max_widths: Vec<usize> = Vec::new();

    for record in records {
        let text: Cow<str> = match bytes_to_cow_string(&record.bytes, reader_instructions.strict_utf8) {
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
        if !final_text.is_empty() || reader_instructions.input_mode != InputMode::WholeString {
            fields.push(Field {
                text: final_text,
                delimiter: b"",
            });
        }

        // Apply skip_empty filter
        if reader_instructions.skip_empty {
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
            &reader_instructions.selections,
            fields.len(),
            reader_instructions.placeholder.is_some(),
            reader_instructions.strict_bounds,
            reader_instructions.strict_range_order,
        ) {
            Ok(result) => result,
            Err(_) => continue, // Skip records with invalid selections
        };

        // Apply invert if needed
        let selections = if reader_instructions.selections.is_empty() {
            vec![(0, fields.len().saturating_sub(1))]
        } else if !reader_instructions.invert {
            normalised_selections
        } else {
            invert_selections(normalised_selections, fields.len())
        };

        // Determine which field positions will be output and measure their widths
        let mut position_index = 0;
        for selection in selections {
            for field_index in selection.0..=selection.1 {
                let field_width = if field_index < fields.len() {
                    fields[field_index].text.len()
                } else if let Some(placeholder) = &reader_instructions.placeholder {
                    placeholder.len()
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

                position_index += 1;
            }
        }
    }

    Ok(max_widths)
}

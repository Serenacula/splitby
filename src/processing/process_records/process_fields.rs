use std::borrow::Cow;

use crate::processing::process_records::worker_utilities::{
    Field, bytes_to_cow_string, estimate_field_count, estimate_output_size, invert_selections,
    normalise_selections,
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

    let mut normalised_selections: Vec<(usize, usize)> = match normalise_selections(
        &instructions.selections,
        fields.len(),
        instructions.placeholder.is_some(),
        instructions.strict_bounds,
        instructions.strict_range_order,
    ) {
        Ok(result) => result,
        Err(error) => return Err(error),
    };

    let selections = if instructions.selections.is_empty() {
        vec![(0, fields.len().saturating_sub(1))]
    } else if !instructions.invert {
        normalised_selections
    } else {
        invert_selections(normalised_selections, fields.len())
    };

    let estimated_output_size = estimate_output_size(record.bytes.len(), selections.len());
    let mut output: Vec<u8> = Vec::with_capacity(estimated_output_size);
    let mut strict_return_passed: bool = false;

    for (selection_index, selection) in selections.iter().enumerate() {
        for field_index in selection.0..=selection.1 {
            let has_data = field_index < fields.len()
                || (instructions.placeholder.is_some() && !instructions.invert);

            if !has_data {
                continue;
            }

            if field_index < fields.len() {
                if !fields[field_index].text.is_empty() {
                    output.extend_from_slice(fields[field_index].text);
                    strict_return_passed = true;
                }
            } else if let Some(placeholder) = &instructions.placeholder {
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
                if let Some(join) = &instructions.join {
                    output.extend_from_slice(join)
                } else if !current_delimiter.is_empty() {
                    output.extend_from_slice(current_delimiter);
                } else if !next_delimiter.is_empty() {
                    output.extend_from_slice(next_delimiter);
                } else {
                    output.push(b' ');
                }
            }
        }
    }

    if instructions.strict_return && !strict_return_passed {
        Err("strict returns error: no valid output".to_string())
    } else {
        Ok(output)
    }
}

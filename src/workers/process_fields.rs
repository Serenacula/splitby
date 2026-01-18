use std::borrow::Cow;

use crate::types::*;
use crate::workers::worker_utilities::{
    estimate_field_count, estimate_output_size, invert_selections, parse_selection, resolve_index,
    Field,
};

pub fn process_fields(
    instructions: &Instructions,
    engine: &RegexEngine,
    record: Record,
) -> Result<Vec<u8>, String> {
    let text: Cow<str> = match instructions.strict_utf8 {
        true => Cow::Borrowed(
            std::str::from_utf8(&record.bytes)
                .map_err(|_| "input is not valid UTF-8".to_string())?,
        ),
        false => {
            match std::str::from_utf8(&record.bytes) {
                Ok(valid_str) => Cow::Borrowed(valid_str),
                Err(_) => Cow::Owned(String::from_utf8_lossy(&record.bytes).into_owned()),
            }
        }
    };

    let delimiter_len = match engine {
        RegexEngine::Simple(re) => re.as_str().len(),
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

    fields.push(Field {
        text: text[cursor..text.len()].as_bytes(),
        delimiter: b"",
    });

    if instructions.input_mode == InputMode::WholeString {
        while let Some(last_field) = fields.last() {
            if last_field.text.is_empty() {
                fields.pop();
            } else {
                break;
            }
        }
    }

    if instructions.skip_empty {
        let filtered: Vec<Field> = fields.into_iter().filter(|f| !f.text.is_empty()).collect();
        fields = filtered;
    }

    if instructions.count {
        let count = fields.len();
        return Ok(count.to_string().into_bytes());
    }

    if fields.is_empty() || fields.iter().all(|f| f.text.is_empty()) {
        return Ok(Vec::new());
    }

    let selections_to_process = if instructions.invert {
        invert_selections(
            &instructions.selections,
            fields.len(),
            instructions.strict_bounds,
            instructions.strict_range_order,
        )?
    } else {
        instructions.selections.clone()
    };

    if selections_to_process.is_empty() {
        if instructions.invert {
            return Ok(Vec::new());
        }
        let estimated_output_size = if fields.is_empty() {
            0
        } else {
            let total_field_size: usize = fields.iter().map(|f| f.text.len()).sum();
            let delimiter_overhead = (fields.len().saturating_sub(1)) * 2;
            total_field_size + delimiter_overhead
        };
        let mut output: Vec<u8> = Vec::with_capacity(estimated_output_size);
        for (index, field) in fields.iter().enumerate() {
            if index > 0 {
                match &instructions.join {
                    Some(join) => {
                        output.extend_from_slice(join.as_bytes());
                    }
                    None => {
                        if instructions.input_mode == InputMode::WholeString {
                            output.push(b'\n');
                        } else {
                            output.push(b' ');
                        }
                    }
                }
            }
            output.extend_from_slice(field.text);
        }
        return Ok(output);
    }

    let mut output_selections: Vec<Vec<u8>> = Vec::with_capacity(selections_to_process.len());
    let mut selection_field_indices: Vec<(Option<usize>, Option<usize>)> =
        Vec::with_capacity(selections_to_process.len());

    for &(raw_start, raw_end) in &selections_to_process {
        let (process_start, process_end) = match parse_selection(
            raw_start,
            raw_end,
            fields.len(),
            instructions.strict_bounds,
            instructions.strict_range_order,
        ) {
            Ok(Some(range)) => range,
            Ok(None) => {
                if let Some(ref placeholder) = instructions.placeholder {
                    output_selections.push(placeholder.clone());
                    let estimated_first = if fields.is_empty() {
                        None
                    } else {
                        match resolve_index(raw_start, fields.len()) {
                            Ok(resolved) if (resolved as usize) < fields.len() => {
                                Some(resolved as usize)
                            }
                            _ => {
                                if raw_start > fields.len() as i32 {
                                    Some(fields.len() - 1)
                                } else {
                                    Some(0)
                                }
                            }
                        }
                    };
                    let estimated_last = estimated_first;
                    selection_field_indices.push((estimated_first, estimated_last));
                }
                continue;
            }
            Err(error) => {
                return Err(error);
            }
        };

        let range_size = (process_end - process_start + 1) as usize;
        let avg_field_size = if fields.is_empty() {
            50
        } else {
            let total_size: usize = fields.iter().map(|f| f.text.len()).sum();
            total_size / fields.len().max(1)
        };
        let estimated_selection_size = range_size * avg_field_size;
        let mut selection_output: Vec<u8> = Vec::with_capacity(estimated_selection_size);
        let mut selection_has_output = false;
        let mut previous_index: Option<usize> = None;
        let mut first_field_index: Option<usize> = None;
        let mut last_field_index: Option<usize> = None;

        for index in process_start..=process_end {
            if index < 0 || index as usize >= fields.len() {
                continue;
            }

            selection_has_output = true;
            let field_index = index as usize;

            // Track first and last field indices
            if first_field_index.is_none() {
                first_field_index = Some(field_index);
            }
            last_field_index = Some(field_index);

            if let Some(previous_index) = previous_index {
                match &instructions.join {
                    Some(join) => {
                        selection_output.extend_from_slice(join.as_bytes());
                    }
                    None => {
                        // Keep the closest delimiter: after previous, else before next, else space.
                        let delimiter_after_a = fields[previous_index].delimiter;
                        let delimiter_before_b = if index > 0 {
                            fields[index as usize - 1].delimiter
                        } else {
                            b""
                        };

                        if !delimiter_after_a.is_empty() {
                            selection_output.extend_from_slice(delimiter_after_a);
                        } else if !delimiter_before_b.is_empty() {
                            selection_output.extend_from_slice(delimiter_before_b);
                        } else {
                            selection_output.push(b' ');
                        }
                    }
                }
            }

            selection_output.extend_from_slice(fields[field_index].text);
            previous_index = Some(field_index);
        }

        if !selection_has_output {
            if let Some(ref placeholder) = instructions.placeholder {
                output_selections.push(placeholder.clone());
                selection_field_indices.push((None, None));
            }
        } else if selection_has_output {
            output_selections.push(selection_output);
            selection_field_indices.push((first_field_index, last_field_index));
        }
    }

    let estimated_output_size = estimate_output_size(record.bytes.len(), output_selections.len());
    let mut output: Vec<u8> = Vec::with_capacity(estimated_output_size);
    for (index, selection) in output_selections.iter().enumerate() {
        if index > 0 {
            match &instructions.join {
                Some(join) => {
                    output.extend_from_slice(join.as_bytes());
                }
                None => {
                    let delimiter_to_use: &[u8] =
                        if instructions.input_mode == InputMode::WholeString {
                            b"\n"
                        } else {
                            let previous_selection_indices = selection_field_indices[index - 1];
                            let current_selection_indices = selection_field_indices[index];

                            match (previous_selection_indices, current_selection_indices) {
                                ((_, Some(prev_last)), (Some(curr_first), _)) => {
                                    let delimiter_after_prev = fields[prev_last].delimiter;
                                    let delimiter_before_curr = if curr_first > 0 {
                                        fields[curr_first - 1].delimiter
                                    } else {
                                        b""
                                    };

                                    if !delimiter_after_prev.is_empty() {
                                        delimiter_after_prev
                                    } else if !delimiter_before_curr.is_empty() {
                                        delimiter_before_curr
                                    } else {
                                        b" "
                                    }
                                }
                                _ => {
                                    b" "
                                }
                            }
                        };

                    output.extend_from_slice(delimiter_to_use);
                }
            }
        }
        output.extend_from_slice(selection);
    }

    Ok(output)
}

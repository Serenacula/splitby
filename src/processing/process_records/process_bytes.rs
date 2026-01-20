use crate::processing::process_records::worker_utilities::normalise_selection;
use crate::types::*;

pub fn process_bytes(instructions: &Instructions, record: Record) -> Result<Vec<u8>, String> {
    let bytes = &record.bytes;
    let byte_length = bytes.len();

    if instructions.count {
        return Ok(byte_length.to_string().into_bytes());
    }

    if byte_length == 0 {
        if instructions.strict_return {
            return Err("strict returns error: empty record".to_string());
        }
        if instructions.strict_bounds && !instructions.selections.is_empty() {
            return Err("strict bounds error: empty record".to_string());
        }
        return Ok(Vec::new());
    }

    // Initial normalisation pass
    let mut normalised_selections: Vec<(usize, usize)> =
        Vec::with_capacity(instructions.selections.len());
    for &(start, end) in &instructions.selections {
        match normalise_selection(
            start,
            end,
            byte_length,
            instructions.placeholder.is_some(),
            instructions.strict_bounds,
            instructions.strict_range_order,
        ) {
            Ok(Some(range)) => {
                normalised_selections.push(range);
            }
            Ok(None) => continue,
            Err(error) => {
                return Err(error);
            }
        }
    }

    // Invert if applicable
    let selections = if instructions.selections.is_empty() {
        vec![(0, byte_length.saturating_sub(1))]
    } else if !instructions.invert {
        normalised_selections
    } else {
        // Sort
        normalised_selections.sort_by(|(start_a, end_a), (start_b, end_b)| {
            start_a.cmp(start_b).then(end_a.cmp(end_b))
        });

        // Merge
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

        // Build inverted list
        let mut invert_pointer: usize = 0;
        let mut inverted_selections: Vec<(usize, usize)> = Vec::with_capacity(merged.len());
        for (start, end) in &merged {
            if start > &byte_length {
                inverted_selections.push((invert_pointer, byte_length - 1));
                break;
            } else {
                inverted_selections.push((invert_pointer, start.saturating_sub(1)));
                invert_pointer = end + 1;
            }
        }
        if merged.last().is_some_and(|last| last.1 < byte_length) {
            inverted_selections.push((invert_pointer, byte_length - 1));
        }
        inverted_selections
    };

    // Make our real output
    let mut output: Vec<u8> = Vec::with_capacity(byte_length);
    for selection in selections {
        for i in selection.0..=selection.1 {
            if i < byte_length {
                output.push(bytes[i])
            } else {
                if let Some(placeholder) = &instructions.placeholder {
                    output.extend_from_slice(&placeholder);
                }
            }
        }
    }

    if instructions.strict_return && output.is_empty() {
        Err("strict returns error: no valid output".to_string())
    } else {
        Ok(output)
    }
}

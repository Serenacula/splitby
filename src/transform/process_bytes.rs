use crate::transform::worker_utilities::*;
use crate::types::*;

pub fn process_bytes(
    transform_instructions: &TransformInstructions,
    record: Record,
) -> Result<Vec<u8>, String> {
    let bytes = &record.bytes;
    let byte_length = bytes.len();

    if transform_instructions.count {
        return Ok(byte_length.to_string().into_bytes());
    }

    if byte_length == 0 {
        if transform_instructions.strict_return {
            return Err("strict returns error: empty record".to_string());
        }
        if transform_instructions.strict_bounds && !transform_instructions.selections.is_empty() {
            return Err("strict bounds error: empty record".to_string());
        }
        return Ok(Vec::new());
    }

    // Initial normalisation pass
    let normalised_selections: Vec<(usize, usize)> = match normalise_selections(
        &transform_instructions.selections,
        byte_length,
        transform_instructions.placeholder.is_some(),
        transform_instructions.strict_bounds,
        transform_instructions.strict_range_order,
    ) {
        Ok(result) => result,
        Err(error) => return Err(error),
    };

    // Invert if applicable
    let selections = if transform_instructions.selections.is_empty() {
        vec![(0, byte_length.saturating_sub(1))]
    } else if !transform_instructions.invert {
        normalised_selections
    } else {
        invert_selections(normalised_selections, byte_length)
    };

    // Make our real output
    let mut output: Vec<u8> = Vec::with_capacity(byte_length);
    for selection in selections {
        for i in selection.0..=selection.1 {
            if i < byte_length {
                output.push(bytes[i])
            } else {
                if let Some(placeholder) = &transform_instructions.placeholder {
                    output.extend_from_slice(&placeholder);
                }
            }
        }
    }

    if transform_instructions.strict_return && output.is_empty() {
        Err("strict returns error: no valid output".to_string())
    } else {
        Ok(output)
    }
}

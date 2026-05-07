use crate::transform::transform_utilities::*;
use crate::types::*;

pub fn process_bytes(
    config: &Config,
    record: Record,
) -> Result<Vec<u8>, String> {
    let bytes = &record.bytes;
    let byte_length = bytes.len();

    if config.count {
        return Ok(byte_length.to_string().into_bytes());
    }

    if byte_length == 0 {
        if config.strict_return {
            return Err("strict-return error: empty record".to_string());
        }
        if config.strict_bounds && !config.selections.is_empty() {
            return Err("strict-bounds error: empty record".to_string());
        }
        return Ok(Vec::new());
    }

    // Initial normalisation pass
    let normalised_selections: Vec<(usize, usize)> = match normalise_selections(
        &config.selections,
        byte_length,
        config.placeholder.is_some(),
        config.strict_bounds,
        config.strict_range_order,
    ) {
        Ok(result) => result,
        Err(error) => return Err(error),
    };

    // Invert if applicable
    let selections = if config.selections.is_empty() {
        vec![(0, byte_length.saturating_sub(1))]
    } else if !config.invert {
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
                if let Some(placeholder) = &config.placeholder {
                    output.extend_from_slice(&placeholder);
                }
            }
        }
    }

    if config.strict_return && output.is_empty() {
        Err("strict-return error: no valid output".to_string())
    } else {
        Ok(output)
    }
}

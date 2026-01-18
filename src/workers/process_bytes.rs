use crate::types::*;
use crate::workers::worker_utilities::{estimate_output_size, invert_selections, parse_selection};

pub fn process_bytes(instructions: &Instructions, record: Record) -> Result<Vec<u8>, String> {
    let bytes = &record.bytes;
    let byte_length = bytes.len();

    // Handle --count flag: return byte count instead of processing selections
    if instructions.count {
        return Ok(byte_length.to_string().into_bytes());
    }

    // Handle empty input
    if byte_length == 0 {
        return Ok(Vec::new());
    }

    // Apply invert if enabled
    let selections_to_process = if instructions.invert {
        invert_selections(
            &instructions.selections,
            byte_length,
            instructions.strict_bounds,
            instructions.strict_range_order,
        )?
    } else {
        instructions.selections.clone()
    };

    // If no selections provided, output all bytes (matching bash behavior)
    // BUT: if we inverted and got empty selections, output nothing (all fields were selected)
    if selections_to_process.is_empty() {
        if instructions.invert {
            return Ok(Vec::new()); // Inverted to nothing
        }
        return Ok(bytes.to_vec()); // No selections provided, output all
    }

    // Process the selections
    // We process selections and build output_selections, then join them
    // This allows us to handle placeholders (empty strings for invalid selections)
    // Pre-allocate with known size
    let mut output_selections: Vec<Vec<u8>> = Vec::with_capacity(selections_to_process.len());

    // For each set of selections
    for &(raw_start, raw_end) in &selections_to_process {
        match parse_selection(
            raw_start,
            raw_end,
            byte_length,
            instructions.strict_bounds,
            instructions.strict_range_order,
        ) {
            Ok(Some((process_start, process_end))) => {
                // Extract byte slice for this selection
                let start_usize = process_start as usize;
                let end_usize = process_end as usize;
                let selection_bytes = bytes[start_usize..=end_usize].to_vec();
                output_selections.push(selection_bytes);
            }
            Ok(None) => {
                // Invalid range - add placeholder if provided
                if let Some(ref placeholder) = instructions.placeholder {
                    output_selections.push(placeholder.clone());
                }
            }
            Err(error) => {
                return Err(error);
            }
        }
    }

    // Join all selections with the join string (or default delimiter)
    // Pre-allocate output buffer with estimated size
    let estimated_output_size = estimate_output_size(byte_length, output_selections.len());
    let mut output: Vec<u8> = Vec::with_capacity(estimated_output_size);
    for (index, selection) in output_selections.iter().enumerate() {
        if index > 0 && instructions.join.is_some() {
            if let Some(join) = &instructions.join {
                output.extend_from_slice(join.as_bytes());
            }
            // Add join delimiter between selections
        }
        output.extend_from_slice(selection);
    }

    Ok(output)
}

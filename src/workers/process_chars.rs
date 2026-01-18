use std::borrow::Cow;

use crate::types::*;
use crate::workers::worker_utilities::{estimate_output_size, invert_selections, parse_selection};
use unicode_segmentation::UnicodeSegmentation;

pub fn process_chars(instructions: &Instructions, record: Record) -> Result<Vec<u8>, String> {
    // Convert bytes to UTF-8 string (with strict_utf8 validation)
    // Optimization: Try to borrow when data is already valid UTF-8 to avoid allocation
    let text: Cow<str> = match instructions.strict_utf8 {
        true => Cow::Borrowed(
            std::str::from_utf8(&record.bytes)
                .map_err(|_| "input is not valid UTF-8".to_string())?,
        ),
        false => {
            // Try to borrow first - if data is valid UTF-8, no allocation needed
            match std::str::from_utf8(&record.bytes) {
                Ok(valid_str) => Cow::Borrowed(valid_str),
                Err(_) => Cow::Owned(String::from_utf8_lossy(&record.bytes).into_owned()),
            }
        }
    };

    // Build grapheme cluster list
    let graphemes: Vec<&str> = text.graphemes(true).collect();
    let grapheme_count = graphemes.len();

    // Handle --count flag: return grapheme cluster count instead of processing selections
    if instructions.count {
        return Ok(grapheme_count.to_string().into_bytes());
    }

    // Handle empty input
    if grapheme_count == 0 {
        return Ok(Vec::new());
    }

    // Apply invert if enabled
    let selections_to_process = if instructions.invert {
        invert_selections(
            &instructions.selections,
            grapheme_count,
            instructions.strict_bounds,
            instructions.strict_range_order,
        )?
    } else {
        instructions.selections.clone()
    };

    // If no selections provided, output all graphemes (matching bash behavior)
    // BUT: if we inverted and got empty selections, output nothing (all fields were selected)
    if selections_to_process.is_empty() {
        if instructions.invert {
            return Ok(Vec::new()); // Inverted to nothing
        }
        return Ok(text.as_bytes().to_vec()); // No selections provided, output all
    }

    // Process the selections
    // We process selections and build output_selections, then join them
    // This allows us to handle placeholders (space for invalid selections)
    // Pre-allocate with known size
    let mut output_selections: Vec<Vec<u8>> = Vec::with_capacity(selections_to_process.len());

    // For each set of selections
    for &(raw_start, raw_end) in &selections_to_process {
        match parse_selection(
            raw_start,
            raw_end,
            grapheme_count,
            instructions.strict_bounds,
            instructions.strict_range_order,
        ) {
            Ok(Some((process_start, process_end))) => {
                // Extract grapheme clusters for this selection
                let start_usize = process_start as usize;
                let end_usize = process_end as usize;

                // Collect selected graphemes into a string
                let selected_graphemes: String =
                    graphemes[start_usize..=end_usize].iter().copied().collect();

                output_selections.push(selected_graphemes.into_bytes());
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
    let estimated_output_size = estimate_output_size(text.len(), output_selections.len());
    let mut output: Vec<u8> = Vec::with_capacity(estimated_output_size);
    for (index, selection) in output_selections.iter().enumerate() {
        if index > 0 {
            if let Some(join) = &instructions.join {
                output.extend_from_slice(join.as_bytes());
            }
        }
        output.extend_from_slice(selection);
    }

    Ok(output)
}

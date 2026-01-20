use std::borrow::Cow;

use crate::processing::process_records::worker_utilities::{
    bytes_to_cow_string, invert_selections, normalise_selections,
};
use crate::types::*;
use unicode_segmentation::UnicodeSegmentation;

pub fn process_chars(instructions: &Instructions, record: Record) -> Result<Vec<u8>, String> {
    let text: Cow<str> = match bytes_to_cow_string(&record.bytes, instructions.strict_utf8) {
        Ok(string) => string,
        Err(e) => return Err(e),
    };

    let graphemes: Vec<&str> = text.graphemes(true).collect();
    let grapheme_count = graphemes.len();

    if instructions.count {
        return Ok(grapheme_count.to_string().into_bytes());
    }

    if grapheme_count == 0 {
        if instructions.strict_return {
            return Err("strict return error: empty record".to_string());
        }
        if instructions.strict_bounds && !instructions.selections.is_empty() {
            return Err("strict bounds error: empty record".to_string());
        }
        return Ok(Vec::new());
    }

    // Initial normalisation pass
    let normalised_selections: Vec<(usize, usize)> = match normalise_selections(
        &instructions.selections,
        grapheme_count,
        instructions.placeholder.is_some(),
        instructions.strict_bounds,
        instructions.strict_range_order,
    ) {
        Ok(result) => result,
        Err(error) => return Err(error),
    };

    // Invert if applicable
    let selections = if instructions.selections.is_empty() {
        vec![(0, grapheme_count.saturating_sub(1))]
    } else if !instructions.invert {
        normalised_selections
    } else {
        invert_selections(normalised_selections, grapheme_count)
    };

    // Make our real output
    let mut output: Vec<u8> = Vec::with_capacity(grapheme_count);
    for (index, selection) in selections.iter().enumerate() {
        for i in selection.0..=selection.1 {
            if i < grapheme_count {
                output.extend_from_slice(graphemes[i].as_bytes());
            } else if let Some(placeholder) = &instructions.placeholder {
                output.extend_from_slice(&placeholder);
            }
            if !(index == selections.len() - 1 && i == selection.1) {
                match &instructions.join {
                    Some(JoinMode::String(join_bytes)) => {
                        output.extend_from_slice(join_bytes);
                    }
                    Some(JoinMode::None) => {
                        // No join - do nothing
                    }
                    // Other modes should have errored during parsing, but handle gracefully
                    _ => {
                        // This shouldn't happen due to validation, but handle it
                    }
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

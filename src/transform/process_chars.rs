use std::borrow::Cow;
use unicode_segmentation::UnicodeSegmentation;

use crate::transform::worker_utilities::*;
use crate::types::*;

pub fn process_chars(
    transform_instructions: &TransformInstructions,
    record: Record,
) -> Result<Vec<u8>, String> {
    let text: Cow<str> =
        match bytes_to_cow_string(&record.bytes, transform_instructions.strict_utf8) {
            Ok(string) => string,
            Err(e) => return Err(e),
        };

    let graphemes: Vec<&str> = text.graphemes(true).collect();
    let grapheme_count = graphemes.len();

    if transform_instructions.count {
        return Ok(grapheme_count.to_string().into_bytes());
    }

    if grapheme_count == 0 {
        if transform_instructions.strict_return {
            return Err("strict return error: empty record".to_string());
        }
        if transform_instructions.strict_bounds && !transform_instructions.selections.is_empty() {
            return Err("strict bounds error: empty record".to_string());
        }
        return Ok(Vec::new());
    }

    // Initial normalisation pass
    let normalised_selections: Vec<(usize, usize)> = match normalise_selections(
        &transform_instructions.selections,
        grapheme_count,
        transform_instructions.placeholder.is_some(),
        transform_instructions.strict_bounds,
        transform_instructions.strict_range_order,
    ) {
        Ok(result) => result,
        Err(error) => return Err(error),
    };

    // Invert if applicable
    let selections = if transform_instructions.selections.is_empty() {
        vec![(0, grapheme_count.saturating_sub(1))]
    } else if !transform_instructions.invert {
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
            } else if let Some(placeholder) = &transform_instructions.placeholder {
                output.extend_from_slice(&placeholder);
            }
            if !(index == selections.len() - 1 && i == selection.1) {
                match &transform_instructions.join {
                    Some(JoinMode::String(join_bytes)) => {
                        output.extend_from_slice(join_bytes);
                    }
                    Some(JoinMode::None) | _ => {
                        // do nothing
                    }
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

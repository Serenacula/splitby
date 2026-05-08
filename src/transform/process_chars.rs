use std::borrow::Cow;
use unicode_segmentation::UnicodeSegmentation;

use crate::transform::transform_utilities::*;
use crate::types::*;

pub fn process_chars(
    config: &Config,
    record: Record,
) -> Result<Option<Vec<u8>>, String> {
    let text: Cow<str> =
        match bytes_to_cow_string(&record.bytes, config.strict_utf8) {
            Ok(string) => string,
            Err(e) => return Err(e),
        };

    let graphemes: Vec<&str> = text.graphemes(true).collect();
    let grapheme_count = graphemes.len();

    if config.count {
        return Ok(Some(grapheme_count.to_string().into_bytes()));
    }

    if grapheme_count == 0 {
        if config.strict_return {
            return Err("strict-return error: empty record".to_string());
        }
        if config.strict_bounds && !config.selections.is_empty() {
            return Err("strict-bounds error: empty record".to_string());
        }
        return Ok(Some(Vec::new()));
    }

    let normalised_selections: Vec<(usize, usize)> = match normalise_selections(
        &config.selections,
        grapheme_count,
        config.placeholder.is_some(),
        config.strict_bounds,
        config.strict_range_order,
    ) {
        Ok(result) => result,
        Err(error) => return Err(error),
    };

    let selections = if config.selections.is_empty() {
        vec![(0, grapheme_count.saturating_sub(1))]
    } else if !config.invert {
        normalised_selections
    } else {
        invert_selections(normalised_selections, grapheme_count)
    };

    let mut output: Vec<u8> = Vec::with_capacity(grapheme_count);
    for (index, selection) in selections.iter().enumerate() {
        for i in selection.0..=selection.1 {
            if i < grapheme_count {
                output.extend_from_slice(graphemes[i].as_bytes());
            } else if let Some(placeholder) = &config.placeholder {
                output.extend_from_slice(&placeholder);
            }
            if !(index == selections.len() - 1 && i == selection.1) {
                match &config.join {
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

    if config.strict_return && output.is_empty() {
        Err("strict-return error: no valid output".to_string())
    } else {
        Ok(Some(output))
    }
}

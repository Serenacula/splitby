use std::borrow::Cow;

/// From Bytes to Cow string
pub fn bytes_to_cow_string<'a>(bytes: &'a [u8], strict_utf8: bool) -> Result<Cow<'a, str>, String> {
    match std::str::from_utf8(bytes) {
        Ok(string) => Ok(Cow::Borrowed(string)),
        Err(_) => match strict_utf8 {
            false => Ok(Cow::Owned(String::from_utf8_lossy(bytes).into_owned())),
            true => Err("input is not valid UTF-8".to_string()),
        },
    }
}

/// Rough capacity hint for field buffers.
pub fn estimate_field_count(input_len: usize, delimiter_len: usize) -> usize {
    if input_len == 0 {
        return 1;
    }
    let estimated = input_len / 50.max(delimiter_len + 10);
    estimated.max(1).min(10000)
}

/// Rough capacity hint for output buffers.
pub fn estimate_output_size(input_len: usize, selection_count: usize) -> usize {
    if selection_count == 0 {
        return input_len;
    }
    (input_len * 2 / selection_count.max(1)).max(input_len / 4)
}

pub fn resolve_index(raw_index: i32, len: usize) -> Result<i32, String> {
    if raw_index > 0 {
        Ok(raw_index - 1)
    } else {
        const MAX_SAFE_LEN: usize = i32::MAX as usize;
        if len > MAX_SAFE_LEN {
            return Err(format!(
                "input too large: {} fields exceeds maximum of {} fields. \
                negative indices cannot be resolved for inputs this large",
                len, MAX_SAFE_LEN
            ));
        }
        Ok(len as i32 + raw_index)
    }
}

/// Parse and validate a selection range.
pub fn normalise_selection(
    raw_start: i32,
    raw_end: i32,
    length: usize,
    is_placeholder: bool,
    strict_bounds: bool,
    strict_range_order: bool,
) -> Result<Option<(usize, usize)>, String> {
    if strict_bounds && (raw_start == 0 || raw_end == 0) {
        return Err(format!("selections are 1-based, 0 is an invalid index"));
    }

    let start = resolve_index(raw_start, length)?;
    let end = resolve_index(raw_end, length)?;

    if start > end {
        match strict_range_order {
            true => {
                return Err(format!(
                    "end index ({}) is less than start index ({}) in selection {}-{}",
                    raw_end, raw_start, raw_start, raw_end
                ));
            }
            false => {
                return Ok(None);
            }
        };
    }

    if strict_bounds {
        if length == 0 {
            return Err(format!("strict bounds error: no valid fields to select"));
        }

        let is_single_index = raw_start == raw_end;

        if start < 0 || start >= length as i32 {
            if is_single_index {
                return Err(format!(
                    "strict bounds error: index ({}) out of bounds, must be between 1 and {}",
                    raw_start, length
                ));
            } else {
                return Err(format!(
                    "strict bounds error: start index ({}) out of bounds, must be between 1 and {}",
                    raw_start, length
                ));
            }
        }
        if end < 0 || end >= length as i32 {
            return Err(format!(
                "strict bounds error: end index ({}) out of bounds, must be between 1 and {}",
                raw_end, length
            ));
        }
        Ok(Some((start as usize, end as usize)))
    } else {
        if end < 0 {
            return Ok(None);
        }
        let clamped_start = if is_placeholder {
            start.max(0)
        } else if start >= length as i32 {
            return Ok(None);
        } else {
            start.max(0).min(length.saturating_sub(1) as i32)
        };
        let clamped_end = if is_placeholder {
            end.max(0)
        } else {
            end.max(0).min(length.saturating_sub(1) as i32)
        };
        Ok(Some((clamped_start as usize, clamped_end as usize)))
    }
}

pub fn normalise_selections(
    selections: &Vec<(i32, i32)>,
    length: usize,
    is_placeholder: bool,
    is_strict_bounds: bool,
    is_strict_range_order: bool,
) -> Result<Vec<(usize, usize)>, String> {
    let mut normalised_selections: Vec<(usize, usize)> = Vec::with_capacity(selections.len());
    for &(start, end) in selections {
        match normalise_selection(
            start,
            end,
            length,
            is_placeholder,
            is_strict_bounds,
            is_strict_range_order,
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
    Ok(normalised_selections)
}

/// Invert a list of selection ranges by sorting, merging, and building the complement.
pub fn invert_selections(
    mut normalised_selections: Vec<(usize, usize)>,
    length: usize,
) -> Vec<(usize, usize)> {
    // Sort
    normalised_selections
        .sort_by(|(start_a, end_a), (start_b, end_b)| start_a.cmp(start_b).then(end_a.cmp(end_b)));

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
    let mut inverted: Vec<(usize, usize)> = Vec::with_capacity(merged.len());
    for (start, end) in &merged {
        if *start > invert_pointer {
            inverted.push((invert_pointer, start.saturating_sub(1)));
        }
        invert_pointer = end.saturating_add(1);
    }
    if invert_pointer < length {
        inverted.push((invert_pointer, length.saturating_sub(1)));
    }
    inverted
}

pub struct Field<'a> {
    pub text: &'a [u8],
    pub delimiter: &'a [u8],
}

pub fn get_current_delimiter<'a>(field_index: usize, fields: &'a [Field<'a>]) -> &'a [u8] {
    if field_index < fields.len() {
        fields[field_index].delimiter
    } else {
        b""
    }
}

pub fn get_next_delimiter<'a>(
    field_index: usize,
    selection_index: usize,
    selections: &[(usize, usize)],
    fields: &'a [Field<'a>],
    placeholder_is_some: bool,
    invert: bool,
) -> &'a [u8] {
    let next_field_index = {
        let mut found = None;
        // This shouldn't fire inside the main loop
        let selection = if selection_index < selections.len() {
            selections[selection_index]
        } else {
            return b"";
        };
        for next_index in (field_index + 1)..=selection.1 {
            let has_next_data = next_index < fields.len() || (placeholder_is_some && !invert);
            if has_next_data {
                found = Some(next_index);
                break;
            }
        }
        // If not found in current selection, check subsequent selections
        if found.is_none() {
            for next_selection in selections.iter().skip(selection_index + 1) {
                for next_index in next_selection.0..=next_selection.1 {
                    let has_next_data =
                        next_index < fields.len() || (placeholder_is_some && !invert);
                    if has_next_data {
                        found = Some(next_index);
                        break;
                    }
                }
                if found.is_some() {
                    break;
                }
            }
        }
        found
    };

    // For before-next, use delimiter before the next selected field
    let next_delimiter = if let Some(next_selected_idx) = next_field_index
        && next_selected_idx > 0
        && next_selected_idx <= fields.len()
    {
        fields[next_selected_idx - 1].delimiter
    } else {
        b""
    };
    next_delimiter
}

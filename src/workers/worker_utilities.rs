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
pub fn parse_selection(
    raw_start: i32,
    raw_end: i32,
    len: usize,
    strict_bounds: bool,
    strict_range_order: bool,
) -> Result<Option<(i32, i32)>, String> {
    if strict_bounds && (raw_start == 0 || raw_end == 0) {
        return Err(format!("selections are 1-based, 0 is an invalid index"));
    }

    let start = resolve_index(raw_start, len)?;
    let end = resolve_index(raw_end, len)?;

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

    let (process_start, process_end) = if strict_bounds {
        if len == 0 {
            return Err(format!("strict bounds error: no valid fields to select"));
        }

        let is_single_index = raw_start == raw_end;

        if start < 0 || start >= len as i32 {
            if is_single_index {
                return Err(format!(
                    "strict bounds error: index ({}) out of bounds. must be between 1 and {}",
                    raw_start, len
                ));
            } else {
                return Err(format!(
                    "strict bounds error: start index ({}) out of bounds. must be between 1 and {}",
                    raw_start, len
                ));
            }
        }
        if end < 0 || end >= len as i32 {
            return Err(format!(
                "strict bounds error: end index ({}) out of bounds. must be between 1 and {}",
                raw_end, len
            ));
        }
        (start, end)
    } else {
        let max_index = len as i32 - 1;
        let clamped_start = if start < 0 { 0 } else { start };
        let clamped_end = if end > max_index { max_index } else { end };

        if clamped_start > max_index || clamped_end < 0 {
            return Ok(None);
        }

        (clamped_start, clamped_end)
    };

    Ok(Some((process_start, process_end)))
}

pub struct Field<'a> {
    pub text: &'a [u8],
    pub delimiter: &'a [u8],
}

pub fn invert_selections(
    selections: &[(i32, i32)],
    fields_len: usize,
    strict_bounds: bool,
    strict_range_order: bool,
) -> Result<Vec<(i32, i32)>, String> {
    let mut canonical_ranges: Vec<(i32, i32)> = Vec::with_capacity(selections.len());

    for &(raw_start, raw_end) in selections {
        let start = resolve_index(raw_start, fields_len)?;
        let end = resolve_index(raw_end, fields_len)?;

        if end < start {
            if strict_range_order {
                return Err(format!(
                    "end index ({}) is less than start index ({}) in selection {}-{}",
                    raw_end, raw_start, raw_start, raw_end
                ));
            }
            continue; // Skip silently
        }

        if strict_bounds {
            if fields_len == 0 {
                return Err(format!("strict bounds error: no valid fields to select"));
            }
            if start < 0 || start >= fields_len as i32 {
                return Err(format!(
                    "strict bounds error: start index ({}) out of bounds, must be between 1 and {}",
                    raw_start, fields_len
                ));
            }
            if end < 0 || end >= fields_len as i32 {
                return Err(format!(
                    "strict bounds error: end index ({}) out of bounds, must be between 1 and {}",
                    raw_end, fields_len
                ));
            }
        } else {
            let start = start.max(0).min(fields_len as i32 - 1);
            let end = end.max(0).min(fields_len as i32 - 1);

            if start > end {
                continue;
            }
        }

        canonical_ranges.push((start, end));
    }

    canonical_ranges.sort_by_key(|(start, _)| *start);

    let mut merged: Vec<(i32, i32)> = Vec::with_capacity(canonical_ranges.len());
    for range in canonical_ranges {
        if let Some(last) = merged.last_mut() {
            if range.0 <= last.1 + 1 {
                last.1 = last.1.max(range.1);
                continue;
            }
        }
        merged.push(range);
    }

    let mut inverted: Vec<(i32, i32)> = Vec::with_capacity(merged.len() + 1);
    let mut next_field = 0i32;

    for (sel_start, sel_end) in merged {
        if next_field <= sel_start - 1 {
            inverted.push((next_field, sel_start - 1));
        }
        next_field = sel_end + 1;
    }

    if next_field <= (fields_len as i32 - 1) {
        inverted.push((next_field, fields_len as i32 - 1));
    }

    let inverted_1based: Vec<(i32, i32)> = inverted
        .into_iter()
        .map(|(start, end)| (start + 1, end + 1))
        .collect();

    Ok(inverted_1based)
}

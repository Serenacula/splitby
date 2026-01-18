/// Estimate field count from input size and delimiter length
pub fn estimate_field_count(input_len: usize, delimiter_len: usize) -> usize {
    if input_len == 0 {
        return 1;
    }
    // Estimate: assume average field size of 50 bytes
    // Add buffer by dividing by slightly less to account for variation
    let estimated = input_len / 50.max(delimiter_len + 10);
    // Cap at reasonable maximum to avoid excessive allocation
    estimated.max(1).min(10000)
}

/// Estimate output buffer size from input length and selection count
pub fn estimate_output_size(input_len: usize, selection_count: usize) -> usize {
    if selection_count == 0 {
        return input_len; // Output all
    }
    // Assume we're keeping roughly a portion of data
    // Conservative estimate: at least 1/4 of input, or proportional to selections
    (input_len * 2 / selection_count.max(1)).max(input_len / 4)
}

pub fn resolve_index(raw_index: i32, len: usize) -> Result<i32, String> {
    if raw_index > 0 {
        Ok(raw_index - 1)
    } else {
        // Check if len is too large to safely convert to i32
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
///
/// Returns:
/// - `Ok(Some((start, end)))` if the selection is valid and should be processed
/// - `Ok(None)` if the selection is invalid but should be skipped (caller handles placeholder)
/// - `Err(...)` if there's an error that should be returned
pub fn parse_selection(
    raw_start: i32,
    raw_end: i32,
    len: usize,
    strict_bounds: bool,
    strict_range_order: bool,
) -> Result<Option<(i32, i32)>, String> {
    // Check for zero index in strict_bounds mode
    if strict_bounds && (raw_start == 0 || raw_end == 0) {
        return Err(format!("selections are 1-based, 0 is an invalid index"));
    }

    // Resolve the start and end values
    let start = resolve_index(raw_start, len)?;
    let end = resolve_index(raw_end, len)?;

    // Check strict_range_order FIRST (matches bash version order)
    if start > end {
        match strict_range_order {
            true => {
                return Err(format!(
                    "end index ({}) is less than start index ({}) in selection {}-{}",
                    raw_end, raw_start, raw_start, raw_end
                ));
            }
            false => {
                // Invalid range - caller will handle placeholder if needed
                return Ok(None);
            }
        };
    }

    // Check our fail states (strict_bounds) and determine the range to process
    let (process_start, process_end) = if strict_bounds {
        if len == 0 {
            return Err(format!("strict bounds error: no valid fields to select"));
        }

        // Check if this is a single index (start == end) for better error message
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
        // In strict mode, use original indices (they're guaranteed to be valid)
        (start, end)
    } else {
        // When strict_bounds is false, clamp indices (matching bash version behavior)
        // The bash version does one-sided clamping:
        // - Clamp start: if < 0, set to 0 (but don't clamp if > max)
        // - Clamp end: if > max, set to max (but don't clamp if < 0)
        // Then check if still invalid
        let max_index = len as i32 - 1;
        let clamped_start = if start < 0 { 0 } else { start };
        let clamped_end = if end > max_index { max_index } else { end };

        // Check if the clamped range is still invalid (matching bash version check)
        if clamped_start > max_index || clamped_end < 0 {
            // Selection is completely invalid after clamping - caller will handle placeholder if needed
            return Ok(None);
        }

        // Use clamped indices for processing
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
    // Step 1: Resolve selections to 0-based, filtering invalid ones
    // Pre-allocate with known size (same or smaller than input)
    let mut canonical_ranges: Vec<(i32, i32)> = Vec::with_capacity(selections.len());

    for &(raw_start, raw_end) in selections {
        // Resolve indices
        let start = resolve_index(raw_start, fields_len)?;
        let end = resolve_index(raw_end, fields_len)?;

        // Skip invalid ranges
        if end < start {
            if strict_range_order {
                return Err(format!(
                    "end index ({}) is less than start index ({}) in selection {}-{}",
                    raw_end, raw_start, raw_start, raw_end
                ));
            }
            continue; // Skip silently
        }

        // Handle out-of-bounds (when strict_bounds is false)
        // When strict_bounds is true, errors should have been caught earlier, but handle defensively
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
            // Clamp to valid range
            let start = start.max(0).min(fields_len as i32 - 1);
            let end = end.max(0).min(fields_len as i32 - 1);

            // Skip if range is completely out of bounds
            if start > end {
                continue;
            }
        }

        canonical_ranges.push((start, end));
    }

    // Step 2: Sort by start
    canonical_ranges.sort_by_key(|(start, _)| *start);

    // Step 3: Merge overlapping/adjacent ranges
    // Pre-allocate: merged will be same size or smaller
    let mut merged: Vec<(i32, i32)> = Vec::with_capacity(canonical_ranges.len());
    for range in canonical_ranges {
        if let Some(last) = merged.last_mut() {
            if range.0 <= last.1 + 1 {
                // Overlap or adjacent: merge
                last.1 = last.1.max(range.1);
                continue;
            }
        }
        merged.push(range);
    }

    // Step 4: Compute complement intervals
    // Pre-allocate: worst case is gaps between every selection, so +1
    let mut inverted: Vec<(i32, i32)> = Vec::with_capacity(merged.len() + 1);
    let mut next_field = 0i32;

    for (sel_start, sel_end) in merged {
        // Gap before this selection?
        if next_field <= sel_start - 1 {
            inverted.push((next_field, sel_start - 1));
        }
        next_field = sel_end + 1;
    }

    // Tail-end gap?
    if next_field <= (fields_len as i32 - 1) {
        inverted.push((next_field, fields_len as i32 - 1));
    }

    // Step 5: Convert back to 1-based
    let inverted_1based: Vec<(i32, i32)> = inverted
        .into_iter()
        .map(|(start, end)| (start + 1, end + 1))
        .collect();

    Ok(inverted_1based)
}

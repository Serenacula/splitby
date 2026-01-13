use std::borrow::Cow;

use crate::types::*;
use unicode_segmentation::UnicodeSegmentation;

/// Estimate field count from input size and delimiter length
fn estimate_field_count(input_len: usize, delimiter_len: usize) -> usize {
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
fn estimate_output_size(input_len: usize, selection_count: usize) -> usize {
    if selection_count == 0 {
        return input_len; // Output all
    }
    // Assume we're keeping roughly a portion of data
    // Conservative estimate: at least 1/4 of input, or proportional to selections
    (input_len * 2 / selection_count.max(1)).max(input_len / 4)
}

fn resolve_index(raw_index: i32, len: usize) -> Result<i32, String> {
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
fn parse_selection(
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

struct Field<'a> {
    text: &'a [u8],
    delimiter: &'a [u8],
}

fn invert_selections(
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

pub fn process_fields(
    instructions: &Instructions,
    engine: &RegexEngine,
    record: Record,
) -> Result<Vec<u8>, String> {
    // Sort out normalising the text
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

    // Extract fields from text using the appropriate regex engine
    // Pre-allocate fields vector with estimated capacity
    let delimiter_len = match engine {
        RegexEngine::Simple(re) => re.as_str().len(),
        RegexEngine::Fancy(_) => 1, // Conservative estimate for fancy regex
    };
    let estimated_field_count = estimate_field_count(record.bytes.len(), delimiter_len);
    let mut fields: Vec<Field> = Vec::with_capacity(estimated_field_count);
    let mut cursor = 0usize;

    match engine {
        RegexEngine::Simple(engine) => {
            // Find all the delimiters using simple regex
            for delimiter in engine.find_iter(&text) {
                fields.push(Field {
                    text: text[cursor..delimiter.start()].as_bytes(),
                    delimiter: text[delimiter.start()..delimiter.end()].as_bytes(),
                });
                cursor = delimiter.end();
            }
        }
        RegexEngine::Fancy(engine) => {
            // Find all the delimiters using fancy-regex
            // fancy-regex's find_iter returns an iterator, but each match is a Result<Match, Error>
            for delimiter_result in engine.find_iter(&text) {
                match delimiter_result {
                    Ok(delimiter) => {
                        fields.push(Field {
                            text: text[cursor..delimiter.start()].as_bytes(),
                            delimiter: text[delimiter.start()..delimiter.end()].as_bytes(),
                        });
                        cursor = delimiter.end();
                    }
                    Err(e) => {
                        return Err(format!("regex matching error: {}", e));
                    }
                }
            }
        }
    }

    // Add the final field after the last delimiter
    fields.push(Field {
        text: text[cursor..text.len()].as_bytes(),
        delimiter: b"",
    });

    // In whole-string mode, remove trailing empty fields created by trailing delimiters
    // (matching bash behavior: trailing newlines don't create additional fields)
    if instructions.input_mode == InputMode::WholeString {
        // Remove trailing empty fields
        while let Some(last_field) = fields.last() {
            if last_field.text.is_empty() {
                fields.pop();
            } else {
                break;
            }
        }
    }

    // Filter out empty fields if --skip-empty is enabled
    if instructions.skip_empty {
        // Pre-allocate filtered vector: worst case is no fields filtered (same size)
        let filtered: Vec<Field> = fields.into_iter().filter(|f| !f.text.is_empty()).collect();
        fields = filtered;
    }

    // Handle --count flag: return field count instead of processing selections
    // Count happens after skip_empty filtering, so it respects that flag
    if instructions.count {
        let count = fields.len();
        return Ok(count.to_string().into_bytes());
    }

    // Handle edge case: all fields empty (after filtering if skip_empty is enabled)
    if fields.is_empty() || fields.iter().all(|f| f.text.is_empty()) {
        return Ok(Vec::new());
    }

    // Apply invert if enabled
    let selections_to_process = if instructions.invert {
        invert_selections(
            &instructions.selections,
            fields.len(),
            instructions.strict_bounds,
            instructions.strict_range_order,
        )?
    } else {
        instructions.selections.clone()
    };

    // If no selections provided, output all fields (matching bash behavior)
    // BUT: if we inverted and got empty selections, output nothing (all fields were selected)
    if selections_to_process.is_empty() {
        if instructions.invert {
            return Ok(Vec::new()); // Inverted to nothing
        }
        // No selections provided, output all fields
        // Pre-allocate output buffer: estimate size from fields
        let estimated_output_size = if fields.is_empty() {
            0
        } else {
            // Estimate: sum of field sizes plus delimiters
            let total_field_size: usize = fields.iter().map(|f| f.text.len()).sum();
            let delimiter_overhead = (fields.len().saturating_sub(1)) * 2; // Rough delimiter size estimate
            total_field_size + delimiter_overhead
        };
        let mut output: Vec<u8> = Vec::with_capacity(estimated_output_size);
        for (index, field) in fields.iter().enumerate() {
            if index > 0 {
                // Add delimiter/join between fields
                match &instructions.join {
                    Some(join) => {
                        output.extend_from_slice(join.as_bytes());
                    }
                    None => {
                        // Default: space for per-line mode, newline for whole-string mode
                        // When no selections are provided, bash uses spaces (not original delimiters)
                        if instructions.input_mode == InputMode::WholeString {
                            output.push(b'\n');
                        } else {
                            output.push(b' ');
                        }
                    }
                }
            }
            output.extend_from_slice(field.text);
        }
        return Ok(output);
    }

    // Process the extracted fields
    // We process selections and build output_selections, then join them
    // This allows us to handle placeholders (empty strings for invalid selections)
    // Pre-allocate with known size
    let mut output_selections: Vec<Vec<u8>> = Vec::with_capacity(selections_to_process.len());
    // Track first and last field indices for each selection to determine delimiters between selections
    let mut selection_field_indices: Vec<(Option<usize>, Option<usize>)> =
        Vec::with_capacity(selections_to_process.len());

    // For each set of selections
    for &(raw_start, raw_end) in &selections_to_process {
        let (process_start, process_end) = match parse_selection(
            raw_start,
            raw_end,
            fields.len(),
            instructions.strict_bounds,
            instructions.strict_range_order,
        ) {
            Ok(Some(range)) => range,
            Ok(None) => {
                // Invalid range - add placeholder if provided
                if let Some(ref placeholder) = instructions.placeholder {
                    output_selections.push(placeholder.clone());
                    // Estimate field indices for delimiter handling
                    // Use the requested position to determine appropriate delimiter
                    let estimated_first = if fields.is_empty() {
                        None
                    } else {
                        // Try to determine what field position this would be at
                        match resolve_index(raw_start, fields.len()) {
                            Ok(resolved) if (resolved as usize) < fields.len() => {
                                Some(resolved as usize)
                            }
                            _ => {
                                // Out of bounds - use last field if after, first if before
                                if raw_start > fields.len() as i32 {
                                    Some(fields.len() - 1) // After last field
                                } else {
                                    Some(0) // Before first field (or very negative)
                                }
                            }
                        }
                    };
                    let estimated_last = estimated_first;
                    selection_field_indices.push((estimated_first, estimated_last));
                }
                continue;
            }
            Err(error) => {
                return Err(error);
            }
        };

        // Build output for this selection
        // Pre-allocate: estimate size based on range and average field size
        let range_size = (process_end - process_start + 1) as usize;
        let avg_field_size = if fields.is_empty() {
            50
        } else {
            let total_size: usize = fields.iter().map(|f| f.text.len()).sum();
            total_size / fields.len().max(1)
        };
        let estimated_selection_size = range_size * avg_field_size;
        let mut selection_output: Vec<u8> = Vec::with_capacity(estimated_selection_size);
        let mut selection_has_output = false;
        let mut previous_index: Option<usize> = None;
        let mut first_field_index: Option<usize> = None;
        let mut last_field_index: Option<usize> = None;

        // Within each range
        for index in process_start..=process_end {
            if index < 0 || index as usize >= fields.len() {
                continue;
            }

            selection_has_output = true;
            let field_index = index as usize;

            // Track first and last field indices
            if first_field_index.is_none() {
                first_field_index = Some(field_index);
            }
            last_field_index = Some(field_index);

            // Add delimiter/join between fields (never before the first field)
            if let Some(previous_index) = previous_index {
                match &instructions.join {
                    Some(join) => {
                        // Join override: always use the join string
                        selection_output.extend_from_slice(join.as_bytes());
                    }
                    None => {
                        // Default behavior: preserve delimiters intelligently
                        // For seam between field A (previous_index) → field B (index):
                        // 1. If delimiter after A exists and is non-empty → use it
                        // 2. Else if delimiter before B exists and is non-empty → use it
                        // 3. Else → use " " (single space)
                        let delimiter_after_a = fields[previous_index].delimiter;
                        let delimiter_before_b = if index > 0 {
                            fields[index as usize - 1].delimiter
                        } else {
                            b""
                        };

                        if !delimiter_after_a.is_empty() {
                            selection_output.extend_from_slice(delimiter_after_a);
                        } else if !delimiter_before_b.is_empty() {
                            selection_output.extend_from_slice(delimiter_before_b);
                        } else {
                            selection_output.push(b' ');
                        }
                    }
                }
            }

            selection_output.extend_from_slice(fields[field_index].text);
            previous_index = Some(field_index);
        }

        // If selection produced no output and placeholder is provided, add placeholder
        if !selection_has_output {
            if let Some(ref placeholder) = instructions.placeholder {
                output_selections.push(placeholder.clone());
                selection_field_indices.push((None, None));
            }
        } else if selection_has_output {
            output_selections.push(selection_output);
            selection_field_indices.push((first_field_index, last_field_index));
        }
    }

    // Join all selections with the join string (or default delimiter using priority logic)
    // Pre-allocate output buffer with estimated size
    let estimated_output_size = estimate_output_size(record.bytes.len(), output_selections.len());
    let mut output: Vec<u8> = Vec::with_capacity(estimated_output_size);
    for (index, selection) in output_selections.iter().enumerate() {
        if index > 0 {
            // Add join delimiter between selections
            match &instructions.join {
                Some(join) => {
                    output.extend_from_slice(join.as_bytes());
                }
                None => {
                    // Use delimiter priority logic: afterPrevious, beforeNext, space/newline
                    // For whole-string mode, always use newlines (ignore delimiter priority)
                    // For per-line mode, use delimiter priority logic
                    let delimiter_to_use: &[u8] =
                        if instructions.input_mode == InputMode::WholeString {
                            // Whole-string mode: always use newlines
                            b"\n"
                        } else {
                            // Per-line mode: use delimiter priority logic
                            let previous_selection_indices = selection_field_indices[index - 1];
                            let current_selection_indices = selection_field_indices[index];

                            match (previous_selection_indices, current_selection_indices) {
                                ((_, Some(prev_last)), (Some(curr_first), _)) => {
                                    // Get delimiter after previous selection's last field (afterPrevious)
                                    let delimiter_after_prev = fields[prev_last].delimiter;
                                    // Get delimiter before current selection's first field (beforeNext)
                                    let delimiter_before_curr = if curr_first > 0 {
                                        fields[curr_first - 1].delimiter
                                    } else {
                                        b""
                                    };

                                    // Priority: afterPrevious, beforeNext, space
                                    if !delimiter_after_prev.is_empty() {
                                        delimiter_after_prev
                                    } else if !delimiter_before_curr.is_empty() {
                                        delimiter_before_curr
                                    } else {
                                        b" " // Fallback: space for per-line mode
                                    }
                                }
                                _ => {
                                    b" " // Fallback: space for per-line mode
                                }
                            }
                        };

                    output.extend_from_slice(delimiter_to_use);
                }
            }
        }
        output.extend_from_slice(selection);
    }

    Ok(output)
}

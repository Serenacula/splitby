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
    len: usize,
    is_placeholder: bool,
    strict_bounds: bool,
    strict_range_order: bool,
) -> Result<Option<(usize, usize)>, String> {
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

    if strict_bounds {
        if len == 0 {
            return Err(format!("strict bounds error: no valid fields to select"));
        }

        let is_single_index = raw_start == raw_end;

        if start < 0 || start >= len as i32 {
            if is_single_index {
                return Err(format!(
                    "strict bounds error: index ({}) out of bounds, must be between 1 and {}",
                    raw_start, len
                ));
            } else {
                return Err(format!(
                    "strict bounds error: start index ({}) out of bounds, must be between 1 and {}",
                    raw_start, len
                ));
            }
        }
        if end < 0 || end >= len as i32 {
            return Err(format!(
                "strict bounds error: end index ({}) out of bounds, must be between 1 and {}",
                raw_end, len
            ));
        }
        Ok(Some((start as usize, end as usize)))
    } else {
        let clamped_start = if is_placeholder {
            start.max(0)
        } else {
            start.max(0).min(len.saturating_sub(1) as i32)
        };
        let clamped_end = if is_placeholder {
            end.max(0)
        } else {
            end.max(0).min(len.saturating_sub(1) as i32)
        };
        Ok(Some((clamped_start as usize, clamped_end as usize)))
    }
}

pub struct Field<'a> {
    pub text: &'a [u8],
    pub delimiter: &'a [u8],
}

use crate::types::{InputMode, SelectionMode};

pub fn validate_align(align: bool, input_mode: InputMode, selection_mode: SelectionMode) -> Result<(), String> {
    if !align {
        return Ok(());
    }

    if input_mode != InputMode::PerLine {
        return Err("--align is only supported in per-line mode".to_string());
    }

    if selection_mode != SelectionMode::Fields {
        return Err("--align is only supported in fields mode".to_string());
    }

    Ok(())
}

pub fn validate_join_mode(join_str: &str, selection_mode: SelectionMode) -> Result<(), String> {
    if join_str.starts_with('@') {
        if selection_mode != SelectionMode::Fields {
            return Err(
                "join flags (@auto, @after-previous, etc.) are only supported in fields mode".to_string()
            );
        }
    }

    if !join_str.starts_with('@') && selection_mode == SelectionMode::Bytes {
        return Err("join is not supported in byte mode".to_string());
    }

    Ok(())
}

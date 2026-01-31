use crate::{
    cli::types::Consuming,
    types::{Align, InputMode, SelectionMode},
};

pub fn validate_align(
    align: Align,
    input_mode: InputMode,
    selection_mode: SelectionMode,
) -> Result<(), String> {
    if matches!(align, Align::None) {
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

pub fn validate_join_mode(join_str: &[u8], selection_mode: SelectionMode) -> Result<(), String> {
    if join_str.starts_with(b"@") {
        if selection_mode != SelectionMode::Fields {
            return Err(
                "join flags (@auto, @after-previous, etc.) are only supported in fields mode"
                    .to_string(),
            );
        }
    }

    if !join_str.starts_with(b"@") && selection_mode == SelectionMode::Bytes {
        return Err("join is not supported in byte mode".to_string());
    }

    Ok(())
}

pub fn validate_no_consuming(consuming: Consuming) -> Result<(), String> {
    if consuming.input {
        return Err("input set but no input file given".to_string());
    }
    if consuming.output {
        return Err("output set but no output file given".to_string());
    }
    if consuming.delim {
        return Err("delimiter set but no delimiter given".to_string());
    }
    if consuming.join {
        return Err("join set but no join string given".to_string());
    }
    if consuming.placeholder {
        return Err("placeholder set but no placeholder string given".to_string());
    }

    Ok(())
}

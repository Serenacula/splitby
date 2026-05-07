use std::path::PathBuf;

use crate::cli::parse::parse_align;
use crate::cli::types::CLIArguments;
use crate::cli::utilities::parse_delimiter_token;
use crate::types::{Align, InputMode, SelectionMode};

#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct FileConfig {
    pub delimiter: Option<String>,
    pub join: Option<String>,
    pub align: Option<String>,
    pub placeholder: Option<String>,
    pub skip_empty: Option<bool>,
    pub invert: Option<bool>,
    pub strict: Option<bool>,
    pub strict_bounds: Option<bool>,
    pub strict_return: Option<bool>,
    pub strict_range_order: Option<bool>,
    pub strict_utf8: Option<bool>,
    pub input_mode: Option<String>,
    pub selection_mode: Option<String>,
}

fn config_path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_default()
                .join(".config")
        });
    base.join("splitby").join("config.json")
}

pub fn load_file_config() -> FileConfig {
    let path = config_path();
    let contents = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return FileConfig::default(),
        Err(error) => {
            eprintln!("warning: could not read config file {}: {error}", path.display());
            return FileConfig::default();
        }
    };
    match serde_json::from_str(&contents) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("warning: could not parse config file {}: {error}", path.display());
            FileConfig::default()
        }
    }
}

pub fn apply_file_config(file_config: &FileConfig, cli_arguments: &mut CLIArguments) {
    if let Some(delimiter) = &file_config.delimiter {
        cli_arguments.delimiter = Some(parse_delimiter_token(delimiter));
    }
    if let Some(join) = &file_config.join {
        cli_arguments.join = Some(join.as_bytes().to_vec());
    }
    if let Some(align_str) = &file_config.align {
        match parse_align(align_str, false) {
            Ok(Some(align)) => cli_arguments.align = align,
            Ok(None) => cli_arguments.align = Align::Left,
            Err(error) => eprintln!("warning: config file: {error}"),
        }
    }
    if let Some(placeholder) = &file_config.placeholder {
        cli_arguments.placeholder = Some(placeholder.as_bytes().to_vec());
    }
    if let Some(skip_empty) = file_config.skip_empty {
        cli_arguments.skip_empty = skip_empty;
    }
    if let Some(invert) = file_config.invert {
        cli_arguments.invert = invert;
    }
    if let Some(strict) = file_config.strict {
        cli_arguments.strict_bounds = strict;
        cli_arguments.strict_return = strict;
        cli_arguments.strict_range_order = strict;
        cli_arguments.strict_utf8 = strict;
    }
    if let Some(strict_bounds) = file_config.strict_bounds {
        cli_arguments.strict_bounds = strict_bounds;
    }
    if let Some(strict_return) = file_config.strict_return {
        cli_arguments.strict_return = strict_return;
    }
    if let Some(strict_range_order) = file_config.strict_range_order {
        cli_arguments.strict_range_order = strict_range_order;
    }
    if let Some(strict_utf8) = file_config.strict_utf8 {
        cli_arguments.strict_utf8 = strict_utf8;
    }
    if let Some(input_mode_str) = &file_config.input_mode {
        match input_mode_str.as_str() {
            "per-line" => cli_arguments.input_mode = InputMode::PerLine,
            "whole-string" => cli_arguments.input_mode = InputMode::WholeString,
            "zero-terminated" => cli_arguments.input_mode = InputMode::ZeroTerminated,
            other => eprintln!("warning: config file: invalid input-mode '{other}'"),
        }
    }
    if let Some(selection_mode_str) = &file_config.selection_mode {
        match selection_mode_str.as_str() {
            "fields" => cli_arguments.selection_mode = SelectionMode::Fields,
            "bytes" => cli_arguments.selection_mode = SelectionMode::Bytes,
            "characters" => cli_arguments.selection_mode = SelectionMode::Chars,
            other => eprintln!("warning: config file: invalid selection-mode '{other}'"),
        }
    }
}

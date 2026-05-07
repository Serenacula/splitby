use std::path::PathBuf;

use crate::cli::parse::parse_align;
use crate::cli::types::CLIArguments;
use crate::types::{Align, InputMode, SelectionMode};

const DEFAULT_CONFIG: &str = include_str!("default_config.json");

#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct FileConfig {
    pub align: Option<String>,
    pub skip_empty: Option<bool>,
    pub invert: Option<bool>,
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
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&path, DEFAULT_CONFIG);
            return FileConfig::default();
        }
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
    if let Some(align_str) = &file_config.align {
        match parse_align(align_str, false) {
            Ok(Some(align)) => cli_arguments.align = align,
            Ok(None) => cli_arguments.align = Align::Left,
            Err(error) => eprintln!("warning: config file: {error}"),
        }
    }
    if let Some(skip_empty) = file_config.skip_empty {
        cli_arguments.skip_empty = skip_empty;
    }
    if let Some(invert) = file_config.invert {
        cli_arguments.invert = invert;
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

use regex::bytes::Regex;
use std::sync::OnceLock;
use unicode_width::UnicodeWidthStr;

static ANSI_STRIP_REGEX: OnceLock<Regex> = OnceLock::new();

fn ansi_strip_regex() -> &'static Regex {
    ANSI_STRIP_REGEX.get_or_init(|| {
        Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").expect("ANSI strip regex pattern is valid")
    })
}

/// Returns the display width (terminal columns) of `bytes` when interpreted as UTF-8.
/// Only used when align is active. Uses lossy decoding and strips ANSI CSI sequences before measuring.
pub fn display_width(bytes: &[u8]) -> usize {
    let stripped = ansi_strip_regex().replace_all(bytes, b"");
    String::from_utf8_lossy(stripped.as_ref()).width()
}

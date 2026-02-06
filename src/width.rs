use regex::bytes::Regex;
use unicode_width::UnicodeWidthStr;

/// Returns the display width (terminal columns) of `bytes` when interpreted as UTF-8.
/// Uses lossy decoding (invalid UTF-8 replaced with U+FFFD), matching how fields are processed.
/// If `ansi_strip_regex` is provided, ANSI escape sequences are removed before measuring (for align only).
pub fn display_width(bytes: &[u8], ansi_strip_regex: Option<&Regex>) -> usize {
    match ansi_strip_regex {
        Some(re) => {
            let stripped = re.replace_all(bytes, b"");
            String::from_utf8_lossy(stripped.as_ref()).width()
        }
        None => String::from_utf8_lossy(bytes).width(),
    }
}

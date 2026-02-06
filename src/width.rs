use unicode_width::UnicodeWidthStr;

/// Returns the display width (terminal columns) of `bytes` when interpreted as UTF-8.
/// Uses lossy decoding (invalid UTF-8 replaced with U+FFFD), matching how fields are processed.
pub fn display_width(bytes: &[u8]) -> usize {
    String::from_utf8_lossy(bytes).width()
}

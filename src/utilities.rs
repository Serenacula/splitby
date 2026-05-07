use regex::bytes::Regex;
use std::sync::OnceLock;
use unicode_width::UnicodeWidthStr;

static ANSI_STRIP_REGEX: OnceLock<Regex> = OnceLock::new();

fn ansi_strip_regex() -> &'static Regex {
    ANSI_STRIP_REGEX.get_or_init(|| {
        Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").expect("ANSI strip regex pattern is valid")
    })
}

// Strips ANSI CSI sequences before measuring, so coloured output aligns correctly.
pub fn display_width(bytes: &[u8]) -> usize {
    let stripped = ansi_strip_regex().replace_all(bytes, b"");
    String::from_utf8_lossy(stripped.as_ref()).width()
}

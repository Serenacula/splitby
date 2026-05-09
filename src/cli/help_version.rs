pub fn print_help() {
    println!("Usage: splitby <delimiter> <selections> [options]");
    println!("Options:");
    println!("  -h, --help                      Print help text");
    println!("  -v, --version                   Print version number");
    println!("  --input=<FILE>                  Read from file instead of stdin");
    println!("  --output=<FILE>                 Write to file instead of stdout");
    println!("  -d, --delimiter=<REGEX>         Set the delimiter");
    println!("  -j, --join=<STRING|HEX|KEYWORD> Join selections with a string or keyword");
    println!("  -p, --placeholder=<STRING|HEX>  Use placeholder for out-of-bounds selections");
    println!("  -t, --terminator=<STRING|HEX>   Replace the output record terminator");
    println!("  --per-line                      Process input line by line (default)");
    println!("  -w, --whole-string              Process input as a single string");
    println!("  -z, --zero-terminated           Process input as null-terminated strings");
    println!("  -f, --fields                    Select fields split by delimiter (default)");
    println!("  -b, --bytes                     Select bytes");
    println!("  -c, --characters                Select grapheme clusters");
    println!("  -a, --align[=MODE]              Align columns (left|right|squash|none)");
    println!("  --count                         Count fields instead of selecting");
    println!("  -i, --invert                    Invert the selection");
    println!("  -e, --skip-empty-fields         Skip empty fields when indexing");
    println!("  -E, --no-skip-empty-fields");
    println!("  -l, --skip-empty-lines          Suppress records with empty output");
    println!("  -L, --no-skip-empty-lines");
    println!("  -s, --skip-undelimited          Suppress records with no delimiter (fields mode only)");
    println!("  -S, --no-skip-undelimited");
    println!("  --strict                        Enable all strict features");
    println!("  --no-strict                     Disable all strict features");
    println!("  --strict-bounds                 Error if selection is out of bounds");
    println!("  --no-strict-bounds");
    println!("  --strict-return                 Error if result is empty");
    println!("  --no-strict-return");
    println!("  --strict-range-order            Error if range start exceeds end (default: on)");
    println!("  --no-strict-range-order");
    println!("  --strict-utf8                   Error on invalid UTF-8");
    println!("  --no-strict-utf8");
}

pub fn print_version() {
    println!("splitby {}", env!("CARGO_PKG_VERSION"));
}

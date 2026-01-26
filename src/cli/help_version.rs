pub fn print_help() {
    println!("Usage: splitby [options] <delimiter> <selections>");
    println!("Options:");
    println!("  -h, --help        Print help text");
    println!("  -v, --version     Print version number");
    println!("  -i, --input=<FILE>              Provide an input file");
    println!("  -o, --output=<FILE>             Write output to a file");
    println!("  -d, --delimiter=<REGEX>         Specify the delimiter to use");
    println!(
        "  -j, --join=<STRING|HEX|KEYWORD> Join each selection with string or hex or delimiter"
    );
    println!("  -p, --placeholder=<STRING|HEX>  Inserts placeholder for invalid selections");
    println!("  --per-line                      Processes the input line by line (default)");
    println!(
        "  -w, --whole-string              Processes the input as a single string, rather than each line separately"
    );
    println!("  -z, --zero-terminated           Processes the input as zero-terminated strings");
    println!("  -f, --fields                    Select fields split by delimiter (default)");
    println!("  -b, --bytes                     Select bytes from the input");
    println!("  -c, --characters                Select characters from the input");
    println!("  -a, --align=<MODE>              Align output (left|right|squash|none)");
    println!("  --count                         Return the number of results after splitting");
    println!("  --invert                        Inverts the chosen selection");
    println!("  -e, --skip-empty                Skips empty fields when indexing or counting");
    println!(
        "  -E, --no-skip-empty             Does not skip empty fields when indexing or counting"
    );
    println!("  --strict                        Shorthand for all strict features");
    println!("  --no-strict                     Does not enforce strict features");
    println!("  --strict-bounds                 Emit error if range is out of bounds");
    println!("  --no-strict-bounds              Does not emit error if range is out of bounds");
    println!("  --strict-return                 Emit error if there is no result");
    println!("  --no-strict-return              Does not emit error if there is no result");
    println!(
        "  --strict-range-order            Emit error if start of a range is greater than the end"
    );
    println!(
        "  --no-strict-range-order         Does not emit error if start of a range is greater than the end"
    );
    println!("  --strict-utf8                   Emit error on invalid UTF-8 sequences");
    println!("  --no-strict-utf8                Does not emit error on invalid UTF-8 sequences");
}

pub fn print_version() {
    println!("splitby {}", env!("CARGO_PKG_VERSION"));
}

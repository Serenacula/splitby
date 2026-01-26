pub fn print_help() {
    println!("Usage: splitby [options] <delimiter> <selections>");
    println!("Options:");
    println!("  -h, --help        Print help text");
    println!("  -v, --version     Print version number");
    println!("  -i, --input=<FILE>        Provide an input file");
    println!("  -o, --output=<FILE>       Write output to a file");
    println!("  -d, --delimiter=<REGEX>   Specify the delimiter to use (required for fields mode)");
    println!("  -j, --join=<STRING|HEX>  Join each selection with a given string");
    println!("  --placeholder=<STRING|HEX> Inserts placeholder for invalid selections");
    println!("  -p, --per-line        Processes the input line by line (default)");
    println!(
        "  -w, --whole-string    Processes the input as a single string, rather than each line separately"
    );
    println!("  -z, --zero-terminated Processes the input as zero-terminated strings");
    println!("  -f, --fields          Select fields split by delimiter (default)");
    println!("  -b, --bytes           Select bytes from the input");
    println!("  -c, --characters      Select characters from the input");
    println!("  -a, --align           Align output to a specific width");
    println!("  --count               Return the number of results after splitting");
    println!("  --invert              Inverts the chosen selection");
    println!("  -e, --skip-empty      Skips empty fields when indexing or counting");
}

pub fn print_version() {
    println!("splitby {}", env!("CARGO_PKG_VERSION"));
}

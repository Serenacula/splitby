use assert_cmd::Command;
use std::fmt::Write as _;

fn run_success_test(
    description: &str,
    input_bytes: &[u8],
    arguments: &[&str],
    expected_stdout: &[u8],
) {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("splitby"));
    command.args(arguments);
    command.write_stdin(input_bytes);

    let output = command
        .output()
        .unwrap_or_else(|error| panic!("{description}: failed to run: {error}"));

    if !output.status.success() {
        let stderr_text = String::from_utf8_lossy(&output.stderr);
        let stdout_text = String::from_utf8_lossy(&output.stdout);
        panic!(
            "{description}: expected success, got status {}\nargs: {arguments:?}\nstdout: {stdout_text}\nstderr: {stderr_text}",
            output.status
        );
    }

    assert_eq!(
        output.stdout, expected_stdout,
        "{description}: stdout mismatch\nargs: {arguments:?}"
    );
}

fn run_error_test(description: &str, input_bytes: &[u8], arguments: &[&str]) {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("splitby"));
    command.args(arguments);
    command.write_stdin(input_bytes);

    let output = command
        .output()
        .unwrap_or_else(|error| panic!("{description}: failed to run: {error}"));

    if output.status.success() {
        let stderr_text = String::from_utf8_lossy(&output.stderr);
        let stdout_text = String::from_utf8_lossy(&output.stdout);
        panic!(
            "{description}: expected failure, got success\nargs: {arguments:?}\nstdout: {stdout_text}\nstderr: {stderr_text}"
        );
    }
}

fn bytes_to_hex_string(bytes: &[u8]) -> String {
    let mut hex_string = String::new();
    for (index, byte_value) in bytes.iter().enumerate() {
        if index > 0 {
            hex_string.push(' ');
        }
        write!(&mut hex_string, "{:02x}", byte_value).expect("writing to string should not fail");
    }
    hex_string
}

fn run_hex_output_test(
    description: &str,
    input_bytes: &[u8],
    arguments: &[&str],
    expected_hex: &str,
) {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("splitby"));
    command.args(arguments);
    command.write_stdin(input_bytes);

    let output = command
        .output()
        .unwrap_or_else(|error| panic!("{description}: failed to run: {error}"));

    if !output.status.success() {
        let stderr_text = String::from_utf8_lossy(&output.stderr);
        let stdout_text = String::from_utf8_lossy(&output.stdout);
        panic!(
            "{description}: expected success, got status {}\nargs: {arguments:?}\nstdout: {stdout_text}\nstderr: {stderr_text}",
            output.status
        );
    }

    let hex_output = bytes_to_hex_string(&output.stdout);
    assert_eq!(
        hex_output, expected_hex,
        "{description}: hex output mismatch\nargs: {arguments:?}"
    );
}

#[test]
fn basic_usage_tests() {
    run_success_test(
        "Split by space",
        b"this is a test\n",
        &["-d", "\\s+", "1"],
        b"this\n",
    );
    run_success_test(
        "Split by comma",
        b"apple,banana,plum,cherry\n",
        &["-d", ",", "2"],
        b"banana\n",
    );
    run_success_test(
        "Test equals syntax",
        b"this is a test\n",
        &["-w", "--delimiter= "],
        b"this\nis\na\ntest\n",
    );
    run_success_test(
        "Per-line default extracts index 2 from every row",
        b"u v w\nx y z\n",
        &["-d", " ", "2"],
        b"v\ny\n",
    );
    run_success_test(
        "Test with newline delimiter",
        b"this\nis\na\ntest\n",
        &["--whole-string", "-d", "\\n", "2"],
        b"is",
    );
}

#[test]
fn range_and_selection_tests() {
    run_success_test(
        "Negative number",
        b"this is a test\n",
        &["-d", " ", "-1"],
        b"test\n",
    );
    run_success_test(
        "Negative split by comma",
        b"apple,banana,plum,cherry\n",
        &["-d", ",", "-2"],
        b"plum\n",
    );
    run_success_test(
        "Split by space, empty selection",
        b"this is a test\n",
        &["-d", " "],
        b"this is a test\n",
    );
    run_success_test(
        "Split by space, empty selection whole-string",
        b"this is a test\n",
        &["-w", "-d", " "],
        b"this\nis\na\ntest\n",
    );
    run_success_test(
        "Range selection",
        b"this is a test\n",
        &["-d", " ", "1-2"],
        b"this is\n",
    );
    run_success_test(
        "Negative range selection",
        b"this is a test\n",
        &["-d", " ", "-3--1"],
        b"is a test\n",
    );
    run_success_test(
        "Positive to negative range",
        b"this is a test\n",
        &["-d", " ", "2--1"],
        b"is a test\n",
    );
    run_success_test(
        "Negative to positive range",
        b"this is a test\n",
        &["-d", " ", "-3-4"],
        b"is a test\n",
    );
    run_success_test(
        "Split by space with multiple indexes",
        b"this is a test\n",
        &["-d", " ", "1", "2", "3-4"],
        b"this is a test\n",
    );
    run_success_test(
        "Split by space whole-string",
        b"this is a test\n",
        &["-w", "-d", " ", "1", "2", "3-4"],
        b"this\nis\na test\n",
    );
}

#[test]
fn comma_separated_selection_tests() {
    run_success_test(
        "Comma-separated selections: basic",
        b"apple,banana,cherry,date\n",
        &["-d", ",", "1,2,3"],
        b"apple,banana,cherry\n",
    );
    run_success_test(
        "Comma-separated selections: with ranges",
        b"apple,banana,cherry,date,elderberry\n",
        &["-d", ",", "1-2,4,5"],
        b"apple,banana,date,elderberry\n",
    );
    run_success_test(
        "Comma-separated selections: leading comma",
        b"apple,banana,cherry\n",
        &["-d", ",", ",1"],
        b"apple\n",
    );
    run_success_test(
        "Comma-separated selections: trailing comma",
        b"apple,banana,cherry\n",
        &["-d", ",", "1,"],
        b"apple\n",
    );
    run_success_test(
        "Comma-separated selections: mixed with spaces",
        b"apple,banana,cherry,date\n",
        &["-d", ",", "1,2", "3", "4"],
        b"apple,banana,cherry,date\n",
    );
    run_success_test(
        "Comma-separated selections: multiple comma strings",
        b"apple,banana,cherry,date\n",
        &["-d", ",", "1,2", "3,4"],
        b"apple,banana,cherry,date\n",
    );
    run_success_test(
        "Comma-separated selections: negative indices",
        b"apple,banana,cherry,date\n",
        &["-d", ",", "-3,-1"],
        b"banana,date\n",
    );
    run_success_test(
        "Comma-separated selections: mixed positive and negative",
        b"apple,banana,cherry,date\n",
        &["-d", ",", "1,-2"],
        b"apple,cherry\n",
    );
    run_success_test(
        "Comma-separated selections: with join",
        b"apple,banana,cherry\n",
        &["-d", ",", "-j", "|", "1,2,3"],
        b"apple|banana|cherry\n",
    );
    run_success_test(
        "Comma-separated selections: whole-string mode",
        b"apple,banana\ncherry,date\n",
        &["-w", "-d", ",", "1,2"],
        b"apple\nbanana\ncherry",
    );
    run_success_test(
        "Comma-separated selections: byte mode",
        b"hello\n",
        &["--bytes", "1,3,5"],
        b"hlo\n",
    );
    run_success_test(
        "Comma-separated selections: char mode",
        b"hello\n",
        &["--characters", "1,3,5"],
        b"hlo\n",
    );
    run_success_test(
        "Comma-separated selections: with invert",
        b"a,b,c,d\n",
        &["-d", ",", "--invert", "1,3"],
        b"b,d\n",
    );
    run_success_test(
        "Comma-separated selections: empty parts ignored",
        b"apple,banana,cherry\n",
        &["-d", ",", ",,1,,"],
        b"apple\n",
    );
    run_success_test(
        "Comma-separated selections: with -d flag, comma-only splits to empty (skipped)",
        b"apple,banana,cherry\n",
        &["-d", ",", ",", "1"],
        b"apple\n",
    );
    run_error_test(
        "Comma-separated selections: with -d flag, letter treated as selection (errors)",
        b"apple,banana,cherry\n",
        &["-d", ",", "a", "1"],
    );
    run_error_test(
        "Comma-separated selections: with -d flag, mixed letter-number (letter part errors)",
        b"apple,banana,cherry\n",
        &["-d", ",", "1,a", "1"],
    );
}

#[test]
fn optional_delimiter_tests() {
    run_success_test(
        "Optional delimiter: comma as first argument",
        b"apple,banana,cherry\n",
        &[",", "1"],
        b"apple\n",
    );
    run_success_test(
        "Optional delimiter: comma with multiple selections",
        b"apple,banana,cherry\n",
        &[",", "1", "3"],
        b"apple,cherry\n",
    );
    run_success_test(
        "Optional delimiter: regex pattern as first argument",
        b"this is a test\n",
        &["\\s+", "1", "2"],
        b"this is\n",
    );
    run_error_test(
        "Optional delimiter: -d flag takes priority",
        b"apple,banana,cherry\n",
        &["-d", ",", ".", "1"],
    );
    run_error_test(
        "Optional delimiter: selection takes priority over delimiter",
        b"apple,banana,cherry\n",
        &["1", "2"],
    );
    run_success_test(
        "Optional delimiter: single letter as delimiter",
        b"apple,banana,cherry\n",
        &["a", "2"],
        b"pple,b\n",
    );
}

#[test]
fn edge_case_tests() {
    run_success_test(
        "Single field with out-of-range index",
        b"apple\n",
        &["-d", " ", "2"],
        b"\n",
    );
    run_success_test(
        "Single delimiter at beginning",
        b" apple\n",
        &["-d", " ", "2"],
        b"apple\n",
    );
    run_success_test(
        "Single delimiter at end",
        b"apple \n",
        &["-d", " ", "1"],
        b"apple\n",
    );
    run_success_test(
        "Multiple delimiters with spaces and commas",
        b"apple, orange  banana, pear\n",
        &["-d", "[, ]+", "1-3"],
        b"apple, orange  banana\n",
    );
    run_success_test(
        "Delimiter appears multiple times",
        b"apple,,orange\n",
        &["-d", ",", "3"],
        b"orange\n",
    );
    run_success_test(
        "Delimiter appears multiple times with range",
        b"apple,,orange\n",
        &["-d", ",", "1-3"],
        b"apple,,orange\n",
    );
}

#[test]
fn join_and_trim_tests() {
    run_success_test(
        "Can join selections",
        b"boo hoo foo\n",
        &["-d", " ", "-j", ","],
        b"boo,hoo,foo\n",
    );
    run_success_test(
        "Doesn't join in ranges",
        b"boo hoo foo\n",
        &["-d", " ", "-j", ",", "1", "2-3"],
        b"boo,hoo,foo\n",
    );
    run_success_test(
        "Trim newline: per-line mode",
        b"a\nb\nc\n",
        &["--trim-newline", "-d", " ", "1"],
        b"a\nb\nc",
    );
    run_success_test(
        "Trim newline: single line",
        b"a\n",
        &["--trim-newline", "-d", " ", "1"],
        b"a",
    );
    run_success_test(
        "Trim newline: whole-string mode",
        b"a\nb\n",
        &["--trim-newline", "-w", "-d", "\\n", "1"],
        b"a",
    );
    run_success_test(
        "Trim newline: without flag (has newline)",
        b"a\nb\nc\n",
        &["-d", " ", "1"],
        b"a\nb\nc\n",
    );
}

#[test]
fn count_and_invert_tests() {
    run_success_test(
        "Using --count to count fields",
        b"this is a test\n",
        &["-d", " ", "--count"],
        b"4\n",
    );
    run_success_test(
        "Using --count with newline delimiter",
        b"this\nis\na\ntest\n",
        &["-d", "\\n", "--count"],
        b"1\n1\n1\n1\n",
    );
    run_success_test(
        "Using --count with newline delimiter whole-string",
        b"this\nis\na\ntest\n",
        &["--whole-string", "-d", "\\n", "--count"],
        b"4",
    );
    run_success_test(
        "Using --count with extra newline",
        b"this\nis\na\ntest\n\n",
        &["-d", "\\n", "--count"],
        b"1\n1\n1\n1\n",
    );
    run_success_test(
        "Using --count with extra newline whole-string",
        b"this\nis\na\ntest\n\n",
        &["--whole-string", "-d", "\\n", "--count"],
        b"4",
    );
    run_success_test(
        "Count takes precedence over join",
        b"a b c\n",
        &["-d", " ", "--count", "-j", ","],
        b"3\n",
    );
    run_success_test(
        "Per-line default with count (per row)",
        b"one two\nalpha beta gamma\n",
        &["-d", " ", "--count"],
        b"2\n3\n",
    );
    run_success_test(
        "Invert single index",
        b"a b c d\n",
        &["-d", " ", "--invert", "2"],
        b"a c d\n",
    );
    run_success_test(
        "Invert single index whole-string",
        b"a b c d\n",
        &["-d", " ", "--whole-string", "--invert", "2"],
        b"a\nc d\n",
    );
    run_success_test(
        "Invert range selection",
        b"a b c d\n",
        &["-d", " ", "--invert", "2-3"],
        b"a d\n",
    );
    run_success_test(
        "Invert range with join",
        b"a b c d\n",
        &["-d", " ", "--invert", "-j", ",", "2-3"],
        b"a,d\n",
    );
    run_success_test(
        "Invert whole set (empty result)",
        b"a b\n",
        &["-d", " ", "--invert", "1-2"],
        b"\n",
    );
    run_success_test(
        "Invert whole set with placeholder",
        b"a b\n",
        &["-d", " ", "--invert", "--placeholder=?", "1-2"],
        b"\n",
    );
    run_success_test(
        "Invert with count",
        b"a b c\n",
        &["-d", " ", "--count", "--invert", "2"],
        b"3\n",
    );
}

#[test]
fn strictness_tests() {
    run_success_test(
        "Strict bounds feature",
        b"this is a test\n",
        &["-d", " ", "--strict-bounds", "2-4"],
        b"is a test\n",
    );
    run_error_test(
        "Strict bounds with out-of-range index (0)",
        b"this is a test\n",
        &["-d", " ", "--strict-bounds", "0"],
    );
    run_error_test(
        "Strict bounds with out-of-range index (5)",
        b"this is a test\n",
        &["-d", " ", "--strict-bounds", "5"],
    );
    run_error_test(
        "Empty string with strict bounds",
        b"\n",
        &["-d", " ", "--strict-bounds", "1"],
    );
    run_error_test(
        "Strict return feature",
        b",boo\n",
        &["--strict-return", "-d", ",", "1"],
    );
    run_error_test(
        "Strict return with out-of-range index",
        b"this is a test\n",
        &["--strict-return", "-d", "z", "2"],
    );
    run_error_test(
        "Strict return doesn't allow empty fields",
        b",\n",
        &["--strict-return", "-d", ","],
    );
    run_success_test(
        "Strict return counts",
        b",\n",
        &["--strict-return", "--count", "-d", ","],
        b"2\n",
    );
    run_success_test(
        "Start after end (no strict range order)",
        b"this is a test\n",
        &["--no-strict-range-order", "-d", " ", "2-1"],
        b"\n",
    );
    run_success_test(
        "Start after end negative (no strict range order)",
        b"this is a test\n",
        &["--no-strict-range-order", "-d", " ", "-1--2"],
        b"\n",
    );
    run_success_test(
        "Start after end positive-negative (no strict range order)",
        b"this is a test\n",
        &["--no-strict-range-order", "-d", " ", "4--2"],
        b"\n",
    );
    run_success_test(
        "Start after end negative-positive (no strict range order)",
        b"this is a test\n",
        &["--no-strict-range-order", "-d", " ", "-1-3"],
        b"\n",
    );
    run_error_test(
        "Start after end (strict range)",
        b"this is a test\n",
        &["-d", " ", "2-1"],
    );
    run_error_test(
        "Start after end negative (strict range)",
        b"this is a test\n",
        &["-d", " ", "-1--2"],
    );
    run_error_test(
        "Start after end positive-negative (strict range)",
        b"this is a test\n",
        &["-d", " ", "4--2"],
    );
    run_error_test(
        "Start after end negative-positive (strict range)",
        b"this is a test\n",
        &["-d", " ", "-1-3"],
    );
    run_success_test(
        "Works with correct syntax",
        b"this is a test\n",
        &["-d", " ", "1-2"],
        b"this is\n",
    );
    run_success_test(
        "Works with no range",
        b"this is a test\n",
        &["-w", "-d", " "],
        b"this\nis\na\ntest\n",
    );
}

#[test]
fn skip_empty_tests() {
    run_success_test(
        "Starting empty field",
        b",orange\n",
        &["--skip-empty", "-d", ",", "1"],
        b"orange\n",
    );
    run_success_test(
        "Middle field empty",
        b"apple,,orange\n",
        &["--skip-empty", "-d", ",", "2"],
        b"orange\n",
    );
    run_success_test(
        "Final field empty",
        b"orange,\n",
        &["--skip-empty", "-d", ",", "2"],
        b"\n",
    );
    run_success_test(
        "All fields empty",
        b",\n",
        &["--skip-empty", "-d", ","],
        b"\n",
    );
    run_success_test(
        "Known failure",
        b"a  b   c\n",
        &["-d", " ", "--skip-empty", "1-3"],
        b"a b c\n",
    );
    run_success_test(
        "Skip with strict bounds works",
        b"orange,\n",
        &["--skip-empty", "--strict-bounds", "-d", ",", "1"],
        b"orange\n",
    );
    run_error_test(
        "Skip with strict bounds fails",
        b"orange,\n",
        &["--skip-empty", "--strict-bounds", "-d", ",", "2"],
    );
    run_success_test(
        "Skip with strict return works",
        b"orange,\n",
        &["--skip-empty", "--strict-return", "-d", ",", "1"],
        b"orange\n",
    );
    run_error_test(
        "Skip with strict return fails",
        b",,\n",
        &["--skip-empty", "--strict-return", "-d", ",", "1"],
    );
    run_success_test(
        "Starting empty field with count",
        b",orange\n",
        &["--skip-empty", "-d", ",", "--count"],
        b"1\n",
    );
    run_success_test(
        "Middle field empty with count",
        b"apple,,orange\n",
        &["--skip-empty", "-d", ",", "--count"],
        b"2\n",
    );
    run_success_test(
        "Final field empty with count",
        b"orange,\n",
        &["--skip-empty", "-d", ",", "--count"],
        b"1\n",
    );
    run_success_test(
        "All fields empty with count",
        b",\n",
        &["--skip-empty", "-d", ",", "--count"],
        b"0\n",
    );
}

#[test]
fn invalid_input_tests() {
    run_error_test("Delimiter not provided", b"this is a test\n", &["1"]);
    run_error_test("Delimiter empty", b"this is a test\n", &["-d", "", "1"]);
    run_error_test(
        "Invalid delimiter regex",
        b"this is a test\n",
        &["-d", "[[", "1"],
    );
    run_success_test("Empty input", b"\n", &["-d", "\\s+", "1"], b"\n");
    run_error_test("Empty -i input", b"", &["-i", "", "-d", ","]);
    run_error_test(
        "Invalid index format",
        b"this is a test\n",
        &["-d", "\\s+", "1a"],
    );
    run_error_test(
        "Invalid range format",
        b"this is a test\n",
        &["-d", "\\s+", "1-2a"],
    );
}

#[test]
fn byte_mode_tests() {
    run_success_test(
        "Byte mode: single byte",
        b"hello\n",
        &["--bytes", "1"],
        b"h\n",
    );
    run_success_test(
        "Byte mode: byte range",
        b"hello\n",
        &["--bytes", "1-3"],
        b"hel\n",
    );
    run_success_test(
        "Byte mode: negative index",
        b"hello\n",
        &["--bytes", "-2"],
        b"l\n",
    );
    run_success_test(
        "Byte mode: negative range",
        b"hello\n",
        &["--bytes", "-3--1"],
        b"llo\n",
    );
    run_success_test(
        "Byte mode: multiple selections",
        b"hello\n",
        &["--bytes", "1", "3", "5"],
        b"hlo\n",
    );
    run_success_test(
        "Byte mode: full range",
        b"hello\n",
        &["--bytes", "1-5"],
        b"hello\n",
    );
    run_success_test(
        "Byte mode: no selections (output all)",
        b"hello\n",
        &["--bytes"],
        b"hello\n",
    );
    run_success_test(
        "Byte mode: newline works",
        b"hello\nworld\n",
        &["--bytes", "1"],
        b"h\nw\n",
    );
    run_success_test("Byte mode: empty input", b"\n", &["--bytes"], b"\n");
    run_success_test(
        "Byte mode: --count",
        b"hello\n",
        &["--count", "--bytes"],
        b"5\n",
    );
    run_success_test(
        "Byte mode: --count with empty",
        b"\n",
        &["--count", "--bytes"],
        b"0\n",
    );
    run_success_test(
        "Byte mode: --join",
        b"hello\n",
        &["--join", ",", "--bytes", "1", "3", "5"],
        b"h,l,o\n",
    );
    run_success_test(
        "Byte mode: --invert",
        b"hello\n",
        &["--invert", "--bytes", "2", "4"],
        b"hlo\n",
    );
    run_success_test(
        "Byte mode: --invert range",
        b"hello\n",
        &["--invert", "--bytes", "2-4"],
        b"ho\n",
    );
    run_success_test(
        "Byte mode: --strict-bounds valid",
        b"hello\n",
        &["--strict-bounds", "--bytes", "1-3"],
        b"hel\n",
    );
    run_error_test(
        "Byte mode: --strict-bounds invalid",
        b"hello\n",
        &["--strict-bounds", "--bytes", "10"],
    );
    run_success_test(
        "Byte mode: --placeholder out-of-bounds",
        b"hello\n",
        &["--placeholder=?", "--bytes", "10"],
        b"?\n",
    );
    run_hex_output_test(
        "Byte mode: --placeholder multiple",
        b"hello\n",
        &["--placeholder=0x00", "--bytes", "1", "10", "3"],
        "68 00 6c 0a",
    );
    run_success_test(
        "Byte mode: whole-string mode",
        b"hello\nworld\n",
        &["--whole-string", "--bytes", "1-5"],
        b"hello",
    );
    run_success_test(
        "Byte mode: whole-string mode with newline join",
        b"hello\nworld\n",
        &["--whole-string", "--bytes", "1", "2"],
        b"he",
    );
    run_success_test(
        "Byte mode: --strict-return empty output 0 placeholder",
        b"hello\n",
        &["--strict-return", "--placeholder=0x00", "--bytes", "10"],
        b"\0\n",
    );
    run_error_test(
        "Byte mode: --strict-return empty output",
        b"hello\n",
        &["--strict-return", "--bytes", "10"],
    );
}

#[test]
fn char_mode_tests() {
    run_success_test(
        "Char mode: single character",
        b"hello\n",
        &["--characters", "1"],
        b"h\n",
    );
    run_success_test(
        "Char mode: character range",
        b"hello\n",
        &["--characters", "1-3"],
        b"hel\n",
    );
    run_success_test(
        "Char mode: negative index",
        b"hello\n",
        &["--characters", "-2"],
        b"l\n",
    );
    run_success_test(
        "Char mode: negative range",
        b"hello\n",
        &["--characters", "-3--1"],
        b"llo\n",
    );
    run_success_test(
        "Char mode: multiple selections",
        b"hello\n",
        &["--characters", "1", "3", "5"],
        b"hlo\n",
    );
    run_success_test(
        "Char mode: full range",
        b"hello\n",
        &["--characters", "1-5"],
        b"hello\n",
    );
    run_success_test(
        "Char mode: no selections (output all)",
        b"hello\n",
        &["--characters"],
        b"hello\n",
    );
    run_success_test("Char mode: empty input", b"\n", &["--characters"], b"\n");
    run_success_test(
        "Char mode: --count",
        b"hello\n",
        &["--count", "--characters"],
        b"5\n",
    );
    run_success_test(
        "Char mode: --count with empty",
        b"\n",
        &["--count", "--characters"],
        b"0\n",
    );
    run_success_test(
        "Char mode: --count with graphemes",
        "café\n".as_bytes(),
        &["--count", "--characters"],
        b"4\n",
    );
    run_success_test(
        "Char mode: --join",
        b"hello\n",
        &["--join", ",", "--characters", "1", "3", "5"],
        b"h,l,o\n",
    );
    run_success_test(
        "Char mode: --invert",
        b"hello\n",
        &["--invert", "--characters", "2", "4"],
        b"hlo\n",
    );
    run_success_test(
        "Char mode: --invert range",
        b"hello\n",
        &["--invert", "--characters", "2-4"],
        b"ho\n",
    );
    run_success_test(
        "Char mode: --strict-bounds valid",
        b"hello\n",
        &["--strict-bounds", "--characters", "1-3"],
        b"hel\n",
    );
    run_error_test(
        "Char mode: --strict-bounds invalid",
        b"hello\n",
        &["--strict-bounds", "--characters", "10"],
    );
    run_success_test(
        "Char mode: --placeholder out-of-bounds",
        b"hello\n",
        &["--placeholder= ", "--characters", "10"],
        b" \n",
    );
    run_success_test(
        "Char mode: --placeholder multiple",
        b"hello\n",
        &["--placeholder= ", "--characters", "1", "10", "3"],
        b"h l\n",
    );
    run_success_test(
        "Char mode: whole-string mode",
        b"hello\nworld\n",
        &["--whole-string", "--characters", "1-5"],
        b"hello",
    );
    run_success_test(
        "Char mode: whole-string mode with newline join",
        b"hello\nworld\n",
        &["--whole-string", "--characters", "1", "2"],
        b"he",
    );
    run_success_test(
        "Char mode: --strict-return empty output",
        b"hello\n",
        &["--strict-return", "--placeholder= ", "--characters", "10"],
        b" \n",
    );
    run_success_test(
        "Char mode: grapheme cluster (café)",
        "café\n".as_bytes(),
        &["--characters", "1-4"],
        "café\n".as_bytes(),
    );
}

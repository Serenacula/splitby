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
        let error = "expected success, got status";
        let input = String::from_utf8_lossy(input_bytes);
        let stderr_text = String::from_utf8_lossy(&output.stderr);
        let stdout_text = String::from_utf8_lossy(&output.stdout);
        panic!(
            "-----\n DESC: {description}\nERROR: {error}\n\n ARGS: {arguments:?}\nINPUT: {input}\nSTDOUT: {stdout_text}\nSTDERR: {stderr_text}",
        );
    }

    if output.stdout != expected_stdout {
        let error = "stdout mismatch";
        let input = String::from_utf8_lossy(input_bytes);
        let expected_hex = bytes_to_hex_string(expected_stdout);
        let actual_hex = bytes_to_hex_string(&output.stdout);
        let expected_text = String::from_utf8_lossy(expected_stdout);
        let actual_text = String::from_utf8_lossy(&output.stdout);
        panic!(
            "-----\n DESC: {description}\nERROR: {error}\n\n ARGS: {arguments:?}\nINPUT: {input}\nEXPECTED (text): {expected_text}\n  ACTUAL (text): {actual_text}\nEXPECTED  (hex):  {expected_hex}\n  ACTUAL  (hex):  {actual_hex}"
        );
    }
}

fn run_error_test(description: &str, input_bytes: &[u8], arguments: &[&str]) {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("splitby"));
    command.args(arguments);
    command.write_stdin(input_bytes);

    let output = command
        .output()
        .unwrap_or_else(|error| panic!("{description}: failed to run: {error}"));

    if output.status.success() {
        let error = "expected failure, got success";
        let input = String::from_utf8_lossy(input_bytes);
        let actual_hex = bytes_to_hex_string(&output.stdout);
        let actual_text = String::from_utf8_lossy(&output.stdout);
        panic!(
            "-----\n DESC: {description}\nERROR: {error}\n\n ARGS: {arguments:?}\nINPUT: {input}\nACTUAL (text): {actual_text}\nACTUAL  (hex):  {actual_hex}"
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
            "-----\n DESC: {description}\nERROR: expected success, got status {}\n\n  ARGS: {arguments:?}\nSTDOUT: {stdout_text}\nSTDERR: {stderr_text}",
            output.status
        );
    }

    let hex_output = bytes_to_hex_string(&output.stdout);
    assert_eq!(
        hex_output, expected_hex,
        "DESC: {description}\nERROR: hex output mismatch\n\nARGS: {arguments:?}"
    );
}

mod basic_usage {
    use super::*;

    #[test]
    fn split_by_space() {
        run_success_test(
            "Split by space",
            b"this is a test\n",
            &["-d", "\\s+", "1"],
            b"this\n",
        );
    }

    #[test]
    fn split_by_comma() {
        run_success_test(
            "Split by comma",
            b"apple,banana,plum,cherry\n",
            &["-d", ",", "2"],
            b"banana\n",
        );
    }

    #[test]
    fn test_equals_syntax() {
        run_success_test(
            "Test equals syntax",
            b"this is a test\n",
            &["-w", "--delimiter= ", "--join=,"],
            b"this,is,a,test\n",
        );
    }

    #[test]
    fn per_line_default_extracts_index_2_from_every_row() {
        run_success_test(
            "Per-line default extracts index 2 from every row",
            b"u v w\nx y z\n",
            &["-d", " ", "2"],
            b"v\ny\n",
        );
    }

    #[test]
    fn test_with_newline_delimiter() {
        run_success_test(
            "Test with newline delimiter",
            b"this\nis\na\ntest\n",
            &["--whole-string", "-d", "\\n", "2"],
            b"is",
        );
    }
}

mod range_and_selection {
    use super::*;

    #[test]
    fn negative_number() {
        run_success_test(
            "Negative number",
            b"this is a test\n",
            &["-d", " ", "-1"],
            b"test\n",
        );
    }

    #[test]
    fn negative_split_by_comma() {
        run_success_test(
            "Negative split by comma",
            b"apple,banana,plum,cherry\n",
            &["-d", ",", "-2"],
            b"plum\n",
        );
    }

    #[test]
    fn split_by_space_empty_selection() {
        run_success_test(
            "Split by space, empty selection",
            b"this is a test\n",
            &["-d", " "],
            b"this is a test\n",
        );
    }

    #[test]
    fn split_by_space_empty_selection_whole_string() {
        run_success_test(
            "Split by space, empty selection whole-string",
            b"this is a test\n",
            &["-w", "-d", " "],
            b"this is a test\n",
        );
    }

    #[test]
    fn range_selection() {
        run_success_test(
            "Range selection",
            b"this is a test\n",
            &["-d", " ", "1-2"],
            b"this is\n",
        );
    }

    #[test]
    fn negative_range_selection() {
        run_success_test(
            "Negative range selection",
            b"this is a test\n",
            &["-d", " ", "-3--1"],
            b"is a test\n",
        );
    }

    #[test]
    fn positive_to_negative_range() {
        run_success_test(
            "Positive to negative range",
            b"this is a test\n",
            &["-d", " ", "2--1"],
            b"is a test\n",
        );
    }

    #[test]
    fn negative_to_positive_range() {
        run_success_test(
            "Negative to positive range",
            b"this is a test\n",
            &["-d", " ", "-3-4"],
            b"is a test\n",
        );
    }

    #[test]
    fn split_by_space_with_multiple_indexes() {
        run_success_test(
            "Split by space with multiple indexes",
            b"this is a test\n",
            &["-d", " ", "1", "2", "3-4"],
            b"this is a test\n",
        );
    }

    #[test]
    fn split_by_space_whole_string() {
        run_success_test(
            "Split by space whole-string",
            b"this is a test\n",
            &["-w", "-d", " ", "1", "3-4"],
            b"this a test\n",
        );
    }
}

mod comma_separated_selection {
    use super::*;

    #[test]
    fn basic() {
        run_success_test(
            "Comma-separated selections: basic",
            b"apple,banana,cherry,date\n",
            &["-d", ",", "1,2,3"],
            b"apple,banana,cherry\n",
        );
    }

    #[test]
    fn with_ranges() {
        run_success_test(
            "Comma-separated selections: with ranges",
            b"apple,banana,cherry,date,elderberry\n",
            &["-d", ",", "1-2,4,5"],
            b"apple,banana,date,elderberry\n",
        );
    }

    #[test]
    fn leading_comma() {
        run_success_test(
            "Comma-separated selections: leading comma",
            b"apple,banana,cherry\n",
            &["-d", ",", ",1"],
            b"apple\n",
        );
    }

    #[test]
    fn trailing_comma() {
        run_success_test(
            "Comma-separated selections: trailing comma",
            b"apple,banana,cherry\n",
            &["-d", ",", "1,"],
            b"apple\n",
        );
    }

    #[test]
    fn mixed_with_spaces() {
        run_success_test(
            "Comma-separated selections: mixed with spaces",
            b"apple,banana,cherry,date\n",
            &["-d", ",", "1,2", "3", "4"],
            b"apple,banana,cherry,date\n",
        );
    }

    #[test]
    fn multiple_comma_strings() {
        run_success_test(
            "Comma-separated selections: multiple comma strings",
            b"apple,banana,cherry,date\n",
            &["-d", ",", "1,2", "3,4"],
            b"apple,banana,cherry,date\n",
        );
    }

    #[test]
    fn negative_indices() {
        run_success_test(
            "Comma-separated selections: negative indices",
            b"apple,banana,cherry,date\n",
            &["-d", ",", "-3,-1"],
            b"banana,date\n",
        );
    }

    #[test]
    fn mixed_positive_and_negative() {
        run_success_test(
            "Comma-separated selections: mixed positive and negative",
            b"apple,banana,cherry,date\n",
            &["-d", ",", "1,-2"],
            b"apple,cherry\n",
        );
    }

    #[test]
    fn with_join() {
        run_success_test(
            "Comma-separated selections: with join",
            b"apple,banana,cherry\n",
            &["-d", ",", "--join=|", "1", "2", "3"],
            b"apple|banana|cherry\n",
        );
    }

    #[test]
    fn whole_string_mode() {
        run_success_test(
            "Comma-separated selections: whole-string mode",
            b"apple,banana\ncherry,date\n",
            &["-w", "-d", ",", "1,2"],
            b"apple,banana\ncherry",
        );
    }

    #[test]
    fn byte_mode() {
        run_success_test(
            "Comma-separated selections: byte mode",
            b"hello\n",
            &["--bytes", "1", "3", "5"],
            b"hlo\n",
        );
    }

    #[test]
    fn char_mode() {
        run_success_test(
            "Comma-separated selections: char mode",
            b"hello\n",
            &["--characters", "1", "3", "5"],
            b"hlo\n",
        );
    }

    #[test]
    fn with_invert() {
        run_success_test(
            "Comma-separated selections: with invert",
            b"a,b,c,d\n",
            &["-d", ",", "--invert", "1,3"],
            b"b,d\n",
        );
    }

    #[test]
    fn empty_parts_ignored() {
        run_success_test(
            "Comma-separated selections: empty parts ignored",
            b"apple,banana,cherry\n",
            &["-d", ",", ",,1,,"],
            b"apple\n",
        );
    }

    #[test]
    fn with_d_flag_letter_treated_as_selection_errors() {
        run_error_test(
            "Comma-separated selections: with -d flag, letter treated as selection (errors)",
            b"apple,banana,cherry\n",
            &["-d", ",", "a", "1"],
        );
    }

    #[test]
    fn with_d_flag_mixed_letter_number_letter_part_errors() {
        run_error_test(
            "Comma-separated selections: with -d flag, mixed letter-number (letter part errors)",
            b"apple,banana,cherry\n",
            &["-d", ",", "1,a", "1"],
        );
    }
}

mod optional_delimiter {
    use super::*;

    #[test]
    fn comma_as_first_argument() {
        run_success_test(
            "Optional delimiter: comma as first argument",
            b"apple,banana,cherry\n",
            &[",", "1"],
            b"apple\n",
        );
    }

    #[test]
    fn comma_with_multiple_selections() {
        run_success_test(
            "Optional delimiter: comma with multiple selections",
            b"apple,banana,cherry\n",
            &[",", "1", "3"],
            b"apple,cherry\n",
        );
    }

    #[test]
    fn regex_pattern_as_first_argument() {
        run_success_test(
            "Optional delimiter: regex pattern as first argument",
            b"this is a test\n",
            &["\\s+", "1", "2"],
            b"this is\n",
        );
    }

    #[test]
    fn d_flag_takes_priority() {
        run_error_test(
            "Optional delimiter: -d flag takes priority",
            b"apple,banana,cherry\n",
            &["-d", ",", ".", "1"],
        );
    }

    #[test]
    fn selection_takes_priority_over_delimiter() {
        run_error_test(
            "Optional delimiter: selection takes priority over delimiter",
            b"apple,banana,cherry\n",
            &["1", "2"],
        );
    }

    #[test]
    fn single_letter_as_delimiter() {
        run_success_test(
            "Optional delimiter: single letter as delimiter",
            b"apple,banana,cherry\n",
            &["a", "2"],
            b"pple,b\n",
        );
    }
}

mod edge_case {
    use super::*;

    #[test]
    fn single_field_with_out_of_range_index() {
        run_success_test(
            "Single field with out-of-range index",
            b"apple\n",
            &["-d", " ", "2"],
            b"\n",
        );
    }

    #[test]
    fn single_delimiter_at_beginning() {
        run_success_test(
            "Single delimiter at beginning",
            b" apple\n",
            &["-d", " ", "2"],
            b"apple\n",
        );
    }

    #[test]
    fn single_delimiter_at_end() {
        run_success_test(
            "Single delimiter at end",
            b"apple \n",
            &["-d", " ", "1"],
            b"apple\n",
        );
    }

    #[test]
    fn multiple_delimiters_with_spaces_and_commas() {
        run_success_test(
            "Multiple delimiters with spaces and commas",
            b"apple, orange  banana, pear\n",
            &["-d", "[, ]+", "1-3"],
            b"apple, orange  banana\n",
        );
    }

    #[test]
    fn delimiter_appears_multiple_times() {
        run_success_test(
            "Delimiter appears multiple times",
            b"apple,,orange\n",
            &["-d", ",", "3"],
            b"orange\n",
        );
    }

    #[test]
    fn delimiter_appears_multiple_times_with_range() {
        run_success_test(
            "Delimiter appears multiple times with range",
            b"apple,,orange\n",
            &["-d", ",", "1-3"],
            b"apple,,orange\n",
        );
    }
}

mod join_and_trim {
    use super::*;

    #[test]
    fn can_join_selections() {
        run_success_test(
            "Can join selections",
            b"boo hoo foo\n",
            &["-d", " ", "--join=,"],
            b"boo,hoo,foo\n",
        );
    }

    #[test]
    fn can_join_whole_string() {
        run_success_test(
            "Can join whole string",
            b"boo hoo foo\n",
            &["-w", "-d", " ", "--join=,"],
            b"boo,hoo,foo\n",
        );
    }

    #[test]
    fn doesnt_join_in_ranges() {
        run_success_test(
            "Doesn't join in ranges",
            b"boo hoo foo\n",
            &["-d", " ", "--join=,", "1", "2-3"],
            b"boo,hoo,foo\n",
        );
    }

    #[test]
    fn join_space() {
        run_success_test(
            "Join with @space",
            b"apple,banana,cherry\n",
            &["-d", ",", "--join=@space", "1", "2", "3"],
            b"apple banana cherry\n",
        );
    }

    #[test]
    fn join_space_single_selection() {
        run_success_test(
            "Join with @space (single selection, no join)",
            b"apple,banana,cherry\n",
            &["-d", ",", "--join=@space", "1"],
            b"apple\n",
        );
    }

    #[test]
    fn join_first() {
        run_success_test(
            "Join with @first (uses first delimiter)",
            b"apple,banana,cherry\n",
            &["-d", ",", "--join=@first", "1", "2", "3"],
            b"apple,banana,cherry\n",
        );
    }

    #[test]
    fn join_first_mixed_delimiters() {
        run_success_test(
            "Join with @first (mixed delimiters, uses first)",
            b"apple;banana,cherry\n",
            &["-d", "[,;]", "--join=@first", "1", "2", "3"],
            b"apple;banana;cherry\n",
        );
    }

    #[test]
    fn join_last() {
        run_success_test(
            "Join with @last (uses last delimiter)",
            b"apple,banana,cherry\n",
            &["-d", ",", "--join=@last", "1", "2", "3"],
            b"apple,banana,cherry\n",
        );
    }

    #[test]
    fn join_last_mixed_delimiters() {
        run_success_test(
            "Join with @last (mixed delimiters, uses last)",
            b"apple;banana,cherry\n",
            &["-d", "[,;]", "--join=@last", "1", "2", "3"],
            b"apple,banana,cherry\n",
        );
    }

    #[test]
    fn join_first_no_delimiters() {
        run_success_test(
            "Join with @first (no delimiters, falls back to space)",
            b"apple\n",
            &["-d", ",", "--join=@first", "1"],
            b"apple\n",
        );
    }

    #[test]
    fn join_last_no_delimiters() {
        run_success_test(
            "Join with @last (no delimiters, falls back to space)",
            b"apple\n",
            &["-d", ",", "--join=@last", "1"],
            b"apple\n",
        );
    }

    #[test]
    fn join_first_empty_fields() {
        run_success_test(
            "Join with @first (empty fields, still finds delimiter)",
            b",,,\n",
            &["-d", ",", "--join=@first", "1", "2", "3"],
            b",,\n",
        );
    }

    #[test]
    fn join_last_empty_fields() {
        run_success_test(
            "Join with @last (empty fields, still finds delimiter)",
            b",,,\n",
            &["-d", ",", "--join=@last", "1", "2", "3"],
            b",,\n",
        );
    }

    #[test]
    fn join_space_whole_string() {
        run_success_test(
            "Join with @space in whole-string mode",
            b"apple,banana,cherry",
            &["-w", "-d", ",", "--join=@space", "1", "2", "3"],
            b"apple banana cherry",
        );
    }

    #[test]
    fn join_first_whole_string() {
        run_success_test(
            "Join with @first in whole-string mode",
            b"apple,banana,cherry",
            &["-w", "-d", ",", "--join=@first", "1", "2", "3"],
            b"apple,banana,cherry",
        );
    }

    #[test]
    fn join_last_whole_string() {
        run_success_test(
            "Join with @last in whole-string mode",
            b"apple,banana,cherry",
            &["-w", "-d", ",", "--join=@last", "1", "2", "3"],
            b"apple,banana,cherry",
        );
    }
}

mod terminator_behavior {
    use super::*;

    #[test]
    fn per_line_keeps_final_newline_when_present() {
        run_success_test(
            "Per-line keeps final newline when present",
            b"alpha\nbeta\n",
            &["--bytes", "1"],
            b"a\nb\n",
        );
    }

    #[test]
    fn per_line_does_not_add_final_newline_when_absent() {
        run_success_test(
            "Per-line does not add final newline when absent",
            b"alpha\nbeta",
            &["--bytes", "1"],
            b"a\nb",
        );
    }

    #[test]
    fn per_line_preserves_empty_line_terminator() {
        run_success_test(
            "Per-line preserves empty line terminator",
            b"\n",
            &["--bytes"],
            b"\n",
        );
    }
}

mod count_and_invert {
    use super::*;

    #[test]
    fn using_count_to_count_fields() {
        run_success_test(
            "Using --count to count fields",
            b"this is a test\n",
            &["-d", " ", "--count"],
            b"4\n",
        );
    }

    #[test]
    fn using_count_with_newline_delimiter() {
        run_success_test(
            "Using --count with newline delimiter",
            b"this\nis\na\ntest\n",
            &["-d", "\\n", "--count"],
            b"1\n1\n1\n1\n",
        );
    }

    #[test]
    fn using_count_with_newline_delimiter_whole_string() {
        run_success_test(
            "Using --count with newline delimiter whole-string",
            b"this\nis\na\ntest\n",
            &["--whole-string", "-d", "\\n", "--count"],
            b"4",
        );
    }

    #[test]
    fn using_count_with_extra_newline() {
        run_success_test(
            "Using --count with extra newline",
            b"this\nis\na\ntest\n\n",
            &["-d", "\\n", "--count"],
            b"1\n1\n1\n1\n1\n",
        );
    }

    #[test]
    fn using_count_with_extra_newline_whole_string() {
        run_success_test(
            "Using --count with extra newline whole-string",
            b"this\nis\na\ntest\n\n",
            &["--whole-string", "-d", "\\n", "--count"],
            b"5",
        );
    }

    #[test]
    fn count_takes_precedence_over_join() {
        run_success_test(
            "Count takes precedence over join",
            b"a b c\n",
            &["-d", " ", "--count", "--join=,"],
            b"3\n",
        );
    }

    #[test]
    fn per_line_default_with_count_per_row() {
        run_success_test(
            "Per-line default with count (per row)",
            b"one two\nalpha beta gamma\n",
            &["-d", " ", "--count"],
            b"2\n3\n",
        );
    }

    #[test]
    fn invert_single_index() {
        run_success_test(
            "Invert single index",
            b"a b c d\n",
            &["-d", " ", "--invert", "2"],
            b"a c d\n",
        );
    }

    #[test]
    fn invert_single_index_whole_string() {
        run_success_test(
            "Invert single index whole-string",
            b"a b c d\n",
            &["-d", " ", "--whole-string", "--invert", "2"],
            b"a c d\n",
        );
    }

    #[test]
    fn invert_range_selection() {
        run_success_test(
            "Invert range selection",
            b"a b c d\n",
            &["-d", " ", "--invert", "2-3"],
            b"a d\n",
        );
    }

    #[test]
    fn invert_range_with_join() {
        run_success_test(
            "Invert range with join",
            b"a b c d\n",
            &["-d", " ", "--invert", "--join=,", "2-3"],
            b"a,d\n",
        );
    }

    #[test]
    fn invert_whole_set_empty_result() {
        run_success_test(
            "Invert whole set (empty result)",
            b"a b\n",
            &["-d", " ", "--invert", "1-2"],
            b"\n",
        );
    }

    #[test]
    fn invert_whole_set_with_placeholder() {
        run_success_test(
            "Invert whole set with placeholder",
            b"a b\n",
            &["-d", " ", "--invert", "--placeholder=?", "1-2"],
            b"\n",
        );
    }

    #[test]
    fn invert_with_count() {
        run_success_test(
            "Invert with count",
            b"a b c\n",
            &["-d", " ", "--count", "--invert", "2"],
            b"3\n",
        );
    }
}

mod strictness {
    use super::*;

    #[test]
    fn strict_bounds_feature() {
        run_success_test(
            "Strict bounds feature",
            b"this is a test\n",
            &["-d", " ", "--strict-bounds", "2-4"],
            b"is a test\n",
        );
    }

    #[test]
    fn strict_utf8_rejects_invalid_fields() {
        run_error_test(
            "Strict utf8 rejects invalid fields",
            b"\xFF,\n",
            &["-d", ",", "--strict-utf8", "1"],
        );
    }

    #[test]
    fn no_strict_utf8_allows_invalid_fields() {
        run_success_test(
            "No-strict-utf8 allows invalid fields",
            b"\xFF,\n",
            &["-d", ",", "--no-strict-utf8", "1"],
            b"\xEF\xBF\xBD\n",
        );
    }

    #[test]
    fn strict_bounds_with_out_of_range_index_0() {
        run_error_test(
            "Strict bounds with out-of-range index (0)",
            b"this is a test\n",
            &["-d", " ", "--strict-bounds", "0"],
        );
    }

    #[test]
    fn strict_bounds_with_out_of_range_index_5() {
        run_error_test(
            "Strict bounds with out-of-range index (5)",
            b"this is a test\n",
            &["-d", " ", "--strict-bounds", "5"],
        );
    }

    #[test]
    fn empty_string_with_strict_bounds() {
        run_success_test(
            "Empty string with strict bounds",
            b"\n",
            &["-d", " ", "--strict-bounds", "1"],
            b"\n",
        );
    }

    #[test]
    fn strict_return_feature() {
        run_error_test(
            "Strict return feature",
            b",boo\n",
            &["--strict-return", "-d", ",", "1"],
        );
    }

    #[test]
    fn strict_return_with_out_of_range_index() {
        run_error_test(
            "Strict return with out-of-range index",
            b"this is a test\n",
            &["--strict-return", "-d", "z", "2"],
        );
    }

    #[test]
    fn strict_return_doesnt_allow_empty_fields() {
        run_error_test(
            "Strict return doesn't allow empty fields",
            b",\n",
            &["--strict-return", "-d", ","],
        );
    }

    #[test]
    fn strict_return_counts() {
        run_success_test(
            "Strict return counts",
            b",\n",
            &["--strict-return", "--count", "-d", ","],
            b"2\n",
        );
    }

    #[test]
    fn start_after_end_no_strict_range_order() {
        run_success_test(
            "Start after end (no strict range order)",
            b"this is a test\n",
            &["--no-strict-range-order", "-d", " ", "2-1"],
            b"\n",
        );
    }

    #[test]
    fn start_after_end_negative_no_strict_range_order() {
        run_success_test(
            "Start after end negative (no strict range order)",
            b"this is a test\n",
            &["--no-strict-range-order", "-d", " ", "-1--2"],
            b"\n",
        );
    }

    #[test]
    fn start_after_end_positive_negative_no_strict_range_order() {
        run_success_test(
            "Start after end positive-negative (no strict range order)",
            b"this is a test\n",
            &["--no-strict-range-order", "-d", " ", "4--2"],
            b"\n",
        );
    }

    #[test]
    fn start_after_end_negative_positive_no_strict_range_order() {
        run_success_test(
            "Start after end negative-positive (no strict range order)",
            b"this is a test\n",
            &["--no-strict-range-order", "-d", " ", "-1-3"],
            b"\n",
        );
    }

    #[test]
    fn start_after_end_strict_range() {
        run_error_test(
            "Start after end (strict range)",
            b"this is a test\n",
            &["-d", " ", "2-1"],
        );
    }

    #[test]
    fn start_after_end_negative_strict_range() {
        run_error_test(
            "Start after end negative (strict range)",
            b"this is a test\n",
            &["-d", " ", "-1--2"],
        );
    }

    #[test]
    fn start_after_end_positive_negative_strict_range() {
        run_error_test(
            "Start after end positive-negative (strict range)",
            b"this is a test\n",
            &["-d", " ", "4--2"],
        );
    }

    #[test]
    fn start_after_end_negative_positive_strict_range() {
        run_error_test(
            "Start after end negative-positive (strict range)",
            b"this is a test\n",
            &["-d", " ", "-1-3"],
        );
    }

    #[test]
    fn works_with_correct_syntax() {
        run_success_test(
            "Works with correct syntax",
            b"this is a test\n",
            &["-d", " ", "1-2"],
            b"this is\n",
        );
    }

    #[test]
    fn works_with_no_range() {
        run_success_test(
            "Works with no range",
            b"this is a test\n",
            &["-w", "-d", " "],
            b"this is a test\n",
        );
    }

    #[test]
    fn strict_return_only_delimiter() {
        run_error_test(
            "Strict return fails with no valid fields",
            b",\n",
            &["--strict", "-d", ","],
        );
    }

    #[test]
    fn strict_enables_strict_return() {
        run_error_test(
            "Strict enables strict-return",
            b",\n",
            &["--strict", "-d", ","],
        );
    }

    #[test]
    fn no_strict_clears_strict_flags() {
        run_success_test(
            "No-strict clears strict flags",
            b"a b\n",
            &["--strict", "--no-strict", "-d", " ", "5"],
            b"\n",
        );
    }
}

mod skip_empty {
    use super::*;

    #[test]
    fn starting_empty_field() {
        run_success_test(
            "Starting empty field",
            b",orange\n",
            &["--skip-empty", "-d", ",", "1"],
            b"orange\n",
        );
    }

    #[test]
    fn middle_field_empty() {
        run_success_test(
            "Middle field empty",
            b"apple,,orange\n",
            &["--skip-empty", "-d", ",", "2"],
            b"orange\n",
        );
    }

    #[test]
    fn final_field_empty() {
        run_success_test(
            "Final field empty",
            b"orange,\n",
            &["--skip-empty", "-d", ",", "2"],
            b"\n",
        );
    }

    #[test]
    fn all_fields_empty() {
        run_success_test(
            "All fields empty",
            b",\n",
            &["--skip-empty", "-d", ","],
            b"\n",
        );
    }

    #[test]
    fn known_failure() {
        run_success_test(
            "Known failure",
            b"a  b   c\n",
            &["-d", " ", "--skip-empty", "1-3"],
            b"a b c\n",
        );
    }

    #[test]
    fn skip_with_strict_bounds_works() {
        run_success_test(
            "Skip with strict bounds works",
            b"orange,\n",
            &["--skip-empty", "--strict-bounds", "-d", ",", "1"],
            b"orange\n",
        );
    }

    #[test]
    fn skip_with_strict_bounds_fails() {
        run_error_test(
            "Skip with strict bounds fails",
            b"orange,\n",
            &["--skip-empty", "--strict-bounds", "-d", ",", "2"],
        );
    }

    #[test]
    fn skip_with_strict_return_works() {
        run_success_test(
            "Skip with strict return works",
            b"orange,\n",
            &["--skip-empty", "--strict-return", "-d", ",", "1"],
            b"orange\n",
        );
    }

    #[test]
    fn skip_with_strict_return_fails() {
        run_error_test(
            "Skip with strict return fails",
            b",,\n",
            &["--skip-empty", "--strict-return", "-d", ",", "1"],
        );
    }

    #[test]
    fn starting_empty_field_with_count() {
        run_success_test(
            "Starting empty field with count",
            b",orange\n",
            &["--skip-empty", "-d", ",", "--count"],
            b"1\n",
        );
    }

    #[test]
    fn middle_field_empty_with_count() {
        run_success_test(
            "Middle field empty with count",
            b"apple,,orange\n",
            &["--skip-empty", "-d", ",", "--count"],
            b"2\n",
        );
    }

    #[test]
    fn final_field_empty_with_count() {
        run_success_test(
            "Final field empty with count",
            b"orange,\n",
            &["--skip-empty", "-d", ",", "--count"],
            b"1\n",
        );
    }

    #[test]
    fn all_fields_empty_with_count() {
        run_success_test(
            "All fields empty with count",
            b",\n",
            &["--skip-empty", "-d", ",", "--count"],
            b"0\n",
        );
    }

    #[test]
    fn no_skip_empty_overrides_skip_empty() {
        run_success_test(
            "No-skip-empty overrides skip-empty",
            b"a,,b\n",
            &["-d", ",", "--count", "--skip-empty", "--no-skip-empty"],
            b"3\n",
        );
    }

    #[test]
    fn skip_empty_overrides_no_skip_empty() {
        run_success_test(
            "Skip-empty overrides no-skip-empty",
            b"a,,b\n",
            &["-d", ",", "--count", "--no-skip-empty", "--skip-empty"],
            b"2\n",
        );
    }
}

mod invalid_input {
    use super::*;

    #[test]
    fn delimiter_not_provided() {
        run_error_test("Delimiter not provided", b"this is a test\n", &["1"]);
    }

    #[test]
    fn delimiter_empty() {
        run_error_test("Delimiter empty", b"this is a test\n", &["-d", "", "1"]);
    }

    #[test]
    fn invalid_delimiter_regex() {
        run_error_test(
            "Invalid delimiter regex",
            b"this is a test\n",
            &["-d", "[[", "1"],
        );
    }

    #[test]
    fn empty_input() {
        run_success_test("Empty input", b"", &["-d", "\\s+", "1"], b"");
    }

    #[test]
    fn empty_i_input() {
        run_error_test("Empty -i input", b"", &["-i", "", "-d", ","]);
    }

    #[test]
    fn invalid_index_format() {
        run_error_test(
            "Invalid index format",
            b"this is a test\n",
            &["-d", "\\s+", "1a"],
        );
    }

    #[test]
    fn invalid_range_format() {
        run_error_test(
            "Invalid range format",
            b"this is a test\n",
            &["-d", "\\s+", "1-2a"],
        );
    }
}

mod zero_terminated_mode {
    use super::*;

    #[test]
    fn bytes_selection_keeps_terminators() {
        run_success_test(
            "Zero-terminated: bytes selection keeps terminators",
            b"alpha\0beta\0",
            &["-z", "--bytes", "1"],
            b"a\0b\0",
        );
    }

    #[test]
    fn missing_final_terminator_stays_missing() {
        run_success_test(
            "Zero-terminated: missing final terminator stays missing",
            b"alpha\0beta",
            &["-z", "--bytes", "1"],
            b"a\0b",
        );
    }

    #[test]
    fn empty_record_preserved() {
        run_success_test(
            "Zero-terminated: empty record preserved",
            b"\0",
            &["-z", "--bytes"],
            b"\0",
        );
    }

    #[test]
    fn field_selection() {
        run_success_test(
            "Zero-terminated: field selection",
            b"a,b\0c,d\0",
            &["-z", "-d", ",", "2"],
            b"b\0d\0",
        );
    }
}

mod byte_mode {
    use super::*;

    #[test]
    fn single_byte() {
        run_success_test(
            "Byte mode: single byte",
            b"hello\n",
            &["--bytes", "1"],
            b"h\n",
        );
    }

    #[test]
    fn byte_range() {
        run_success_test(
            "Byte mode: byte range",
            b"hello\n",
            &["--bytes", "1-3"],
            b"hel\n",
        );
    }

    #[test]
    fn negative_index() {
        run_success_test(
            "Byte mode: negative index",
            b"hello\n",
            &["--bytes", "-2"],
            b"l\n",
        );
    }

    #[test]
    fn negative_range() {
        run_success_test(
            "Byte mode: negative range",
            b"hello\n",
            &["--bytes", "-3--1"],
            b"llo\n",
        );
    }

    #[test]
    fn multiple_selections() {
        run_success_test(
            "Byte mode: multiple selections",
            b"hello\n",
            &["--bytes", "1", "3", "5"],
            b"hlo\n",
        );
    }

    #[test]
    fn full_range() {
        run_success_test(
            "Byte mode: full range",
            b"hello\n",
            &["--bytes", "1-5"],
            b"hello\n",
        );
    }

    #[test]
    fn no_selections_output_all() {
        run_success_test(
            "Byte mode: no selections (output all)",
            b"hello\n",
            &["--bytes"],
            b"hello\n",
        );
    }

    #[test]
    fn newline_works() {
        run_success_test(
            "Byte mode: newline works",
            b"hello\nworld\n",
            &["--bytes", "1"],
            b"h\nw\n",
        );
    }

    #[test]
    fn empty_input() {
        run_success_test("Byte mode: empty input", b"", &["--bytes"], b"");
    }

    #[test]
    fn count() {
        run_success_test(
            "Byte mode: --count",
            b"hello\n",
            &["--count", "--bytes"],
            b"5\n",
        );
    }

    #[test]
    fn count_with_empty() {
        run_success_test(
            "Byte mode: --count with empty",
            b"\n",
            &["--count", "--bytes"],
            b"0\n",
        );
    }

    #[test]
    fn join_not_supported() {
        run_error_test(
            "Byte mode: --join (not supported)",
            b"hello\n",
            &["--join", ",", "--bytes", "1", "3", "5"],
        );
    }

    #[test]
    fn invert() {
        run_success_test(
            "Byte mode: --invert",
            b"hello\n",
            &["--invert", "--bytes", "2", "4"],
            b"hlo\n",
        );
    }

    #[test]
    fn invert_range() {
        run_success_test(
            "Byte mode: --invert range",
            b"hello\n",
            &["--invert", "--bytes", "2-4"],
            b"ho\n",
        );
    }

    #[test]
    fn strict_bounds_valid() {
        run_success_test(
            "Byte mode: --strict-bounds valid",
            b"hello\n",
            &["--strict-bounds", "--bytes", "1-3"],
            b"hel\n",
        );
    }

    #[test]
    fn strict_bounds_invalid() {
        run_error_test(
            "Byte mode: --strict-bounds invalid",
            b"hello\n",
            &["--strict-bounds", "--bytes", "10"],
        );
    }

    #[test]
    fn placeholder_out_of_bounds() {
        run_success_test(
            "Byte mode: --placeholder out-of-bounds",
            b"hello\n",
            &["--placeholder=?", "--bytes", "10"],
            b"?\n",
        );
    }

    #[test]
    fn placeholder_multiple() {
        run_hex_output_test(
            "Byte mode: --placeholder multiple",
            b"hello\n",
            &["--placeholder=0x00", "--bytes", "1", "10", "3"],
            "68 00 6c 0a",
        );
    }

    #[test]
    fn whole_string_mode() {
        run_success_test(
            "Byte mode: whole-string mode",
            b"hello\nworld\n",
            &["--whole-string", "--bytes", "1-5"],
            b"hello",
        );
    }

    #[test]
    fn whole_string_mode_with_newline_join() {
        run_success_test(
            "Byte mode: whole-string mode with newline join",
            b"hello\nworld\n",
            &["--whole-string", "--bytes", "1", "2"],
            b"he",
        );
    }

    #[test]
    fn strict_return_empty_output_0_placeholder() {
        run_success_test(
            "Byte mode: --strict-return empty output 0 placeholder",
            b"hello\n",
            &["--strict-return", "--placeholder=0x00", "--bytes", "10"],
            b"\0\n",
        );
    }

    #[test]
    fn strict_return_empty_output() {
        run_error_test(
            "Byte mode: --strict-return empty output",
            b"hello\n",
            &["--strict-return", "--bytes", "10"],
        );
    }
}

mod char_mode {
    use super::*;

    #[test]
    fn single_character() {
        run_success_test(
            "Char mode: single character",
            b"hello\n",
            &["--characters", "1"],
            b"h\n",
        );
    }

    #[test]
    fn character_range() {
        run_success_test(
            "Char mode: character range",
            b"hello\n",
            &["--characters", "1-3"],
            b"hel\n",
        );
    }

    #[test]
    fn negative_index() {
        run_success_test(
            "Char mode: negative index",
            b"hello\n",
            &["--characters", "-2"],
            b"l\n",
        );
    }

    #[test]
    fn negative_range() {
        run_success_test(
            "Char mode: negative range",
            b"hello\n",
            &["--characters", "-3--1"],
            b"llo\n",
        );
    }

    #[test]
    fn multiple_selections() {
        run_success_test(
            "Char mode: multiple selections",
            b"hello\n",
            &["--characters", "1", "3", "5"],
            b"hlo\n",
        );
    }

    #[test]
    fn full_range() {
        run_success_test(
            "Char mode: full range",
            b"hello\n",
            &["--characters", "1-5"],
            b"hello\n",
        );
    }

    #[test]
    fn no_selections_output_all() {
        run_success_test(
            "Char mode: no selections (output all)",
            b"hello\n",
            &["--characters"],
            b"hello\n",
        );
    }

    #[test]
    fn empty_input() {
        run_success_test("Char mode: empty input", b"", &["--characters"], b"");
    }

    #[test]
    fn count() {
        run_success_test(
            "Char mode: --count",
            b"hello\n",
            &["--count", "--characters"],
            b"5\n",
        );
    }

    #[test]
    fn count_with_empty() {
        run_success_test(
            "Char mode: --count with empty",
            b"\n",
            &["--count", "--characters"],
            b"0\n",
        );
    }

    #[test]
    fn count_with_graphemes() {
        run_success_test(
            "Char mode: --count with graphemes",
            "café\n".as_bytes(),
            &["--count", "--characters"],
            b"4\n",
        );
    }

    #[test]
    fn join() {
        run_success_test(
            "Char mode: --join",
            b"hello\n",
            &["--join=,", "--characters", "1", "3", "5"],
            b"h,l,o\n",
        );
    }

    #[test]
    fn invert() {
        run_success_test(
            "Char mode: --invert",
            b"hello\n",
            &["--invert", "--characters", "2", "4"],
            b"hlo\n",
        );
    }

    #[test]
    fn invert_range() {
        run_success_test(
            "Char mode: --invert range",
            b"hello\n",
            &["--invert", "--characters", "2-4"],
            b"ho\n",
        );
    }

    #[test]
    fn strict_bounds_valid() {
        run_success_test(
            "Char mode: --strict-bounds valid",
            b"hello\n",
            &["--strict-bounds", "--characters", "1-3"],
            b"hel\n",
        );
    }

    #[test]
    fn strict_bounds_invalid() {
        run_error_test(
            "Char mode: --strict-bounds invalid",
            b"hello\n",
            &["--strict-bounds", "--characters", "10"],
        );
    }

    #[test]
    fn placeholder_out_of_bounds() {
        run_success_test(
            "Char mode: --placeholder out-of-bounds",
            b"hello\n",
            &["--placeholder= ", "--characters", "10"],
            b" \n",
        );
    }

    #[test]
    fn placeholder_multiple() {
        run_success_test(
            "Char mode: --placeholder multiple",
            b"hello\n",
            &["--placeholder= ", "--characters", "1", "10", "3"],
            b"h l\n",
        );
    }

    #[test]
    fn whole_string_mode() {
        run_success_test(
            "Char mode: whole-string mode",
            b"hello\nworld\n",
            &["--whole-string", "--characters", "1-5"],
            b"hello",
        );
    }

    #[test]
    fn whole_string_mode_with_newline_join() {
        run_success_test(
            "Char mode: whole-string mode with newline join",
            b"hello\nworld\n",
            &["--whole-string", "--characters", "1", "2"],
            b"he",
        );
    }

    #[test]
    fn strict_return_empty_output() {
        run_success_test(
            "Char mode: --strict-return empty output",
            b"hello\n",
            &["--strict-return", "--placeholder= ", "--characters", "10"],
            b" \n",
        );
    }

    #[test]
    fn grapheme_cluster_cafe() {
        run_success_test(
            "Char mode: grapheme cluster (café)",
            "café\n".as_bytes(),
            &["--characters", "1-4"],
            "café\n".as_bytes(),
        );
    }

    #[test]
    fn strict_utf8_rejects_invalid() {
        run_error_test(
            "Char mode: strict utf8 rejects invalid",
            b"\xFF\n",
            &["--characters", "--strict-utf8"],
        );
    }

    #[test]
    fn no_strict_utf8_allows_invalid() {
        run_success_test(
            "Char mode: no-strict-utf8 allows invalid",
            b"\xFF\n",
            &["--characters", "--no-strict-utf8"],
            b"\xEF\xBF\xBD\n",
        );
    }
}

mod hex_parsing {
    use super::*;

    #[test]
    fn placeholder_single_byte_hex() {
        run_success_test(
            "Placeholder: single-byte hex (0x2C = comma)",
            b"apple,banana\n",
            &["-d", ",", "--placeholder=0x2C", "1", "5"],
            b"apple,,\n",
        );
    }

    #[test]
    fn placeholder_single_byte_hex_uppercase() {
        run_success_test(
            "Placeholder: single-byte hex uppercase prefix (0X2C)",
            b"apple,banana\n",
            &["-d", ",", "--placeholder=0X2C", "1", "5"],
            b"apple,,\n",
        );
    }

    #[test]
    fn placeholder_multi_byte_hex() {
        run_success_test(
            "Placeholder: multi-byte hex (0x2C20 = comma+space)",
            b"apple,banana\n",
            &["-d", ",", "--placeholder=0x2C20", "1", "5"],
            b"apple,, \n",
        );
    }

    #[test]
    fn placeholder_multi_byte_hex_uppercase() {
        run_success_test(
            "Placeholder: multi-byte hex uppercase prefix (0X3A3A = ::)",
            b"apple,banana\n",
            &["-d", ",", "--placeholder=0X3A3A", "1", "5"],
            b"apple,::\n",
        );
    }

    #[test]
    fn placeholder_hex_four_bytes() {
        run_hex_output_test(
            "Placeholder: four-byte hex (0x48656C6C = Hell)",
            b"hello\n",
            &["--bytes", "--placeholder=0x48656C6C", "1", "10", "3"],
            "68 48 65 6c 6c 6c 0a",
        );
    }

    #[test]
    fn placeholder_hex_zero_byte() {
        run_hex_output_test(
            "Placeholder: hex zero byte (0x00)",
            b"hello\n",
            &["--bytes", "--placeholder=0x00", "1", "10", "3"],
            "68 00 6c 0a",
        );
    }

    #[test]
    fn placeholder_string_fallback() {
        run_success_test(
            "Placeholder: string fallback (not hex)",
            b"apple,banana\n",
            &["-d", ",", "--placeholder=N/A", "1", "5"],
            b"apple,N/A\n",
        );
    }

    #[test]
    fn placeholder_string_with_0x_prefix() {
        run_success_test(
            "Placeholder: string starting with 0x but not valid hex",
            b"apple,banana\n",
            &["-d", ",", "--placeholder=0xinvalid", "1", "5"],
            b"apple,0xinvalid\n",
        );
    }

    #[test]
    fn join_single_byte_hex() {
        run_success_test(
            "Join: single-byte hex (0x2C = comma)",
            b"apple,banana,cherry\n",
            &["-d", ",", "--join=0x2C", "1", "3"],
            b"apple,cherry\n",
        );
    }

    #[test]
    fn join_single_byte_hex_uppercase() {
        run_success_test(
            "Join: single-byte hex uppercase prefix (0X09 = tab)",
            b"apple,banana,cherry\n",
            &["-d", ",", "--join=0X09", "1", "3"],
            b"apple\tcherry\n",
        );
    }

    #[test]
    fn join_multi_byte_hex() {
        run_success_test(
            "Join: multi-byte hex (0x2C20 = comma+space)",
            b"apple,banana,cherry\n",
            &["-d", ",", "--join=0x2C20", "1", "3"],
            b"apple, cherry\n",
        );
    }

    #[test]
    fn join_multi_byte_hex_uppercase() {
        run_success_test(
            "Join: multi-byte hex uppercase prefix (0X3A20 = colon+space)",
            b"apple,banana,cherry\n",
            &["-d", ",", "--join=0X3A20", "1", "3"],
            b"apple: cherry\n",
        );
    }

    #[test]
    fn join_hex_four_bytes() {
        run_success_test(
            "Join: four-byte hex (0x48656C6C = Hell)",
            b"apple,banana,cherry\n",
            &["-d", ",", "--join=0x48656C6C", "1", "3"],
            b"appleHellcherry\n",
        );
    }

    #[test]
    fn join_string_fallback() {
        run_success_test(
            "Join: string fallback (not hex)",
            b"apple,banana,cherry\n",
            &["-d", ",", "--join=|", "1", "3"],
            b"apple|cherry\n",
        );
    }

    #[test]
    fn join_string_with_0x_prefix() {
        run_success_test(
            "Join: string starting with 0x but not valid hex",
            b"apple,banana,cherry\n",
            &["-d", ",", "--join=0xinvalid", "1", "3"],
            b"apple0xinvalidcherry\n",
        );
    }

    #[test]
    fn join_hex_odd_length() {
        run_success_test(
            "Join: odd-length hex falls back to string",
            b"apple,banana,cherry\n",
            &["-d", ",", "--join=0x2C2", "1", "3"],
            b"apple0x2C2cherry\n",
        );
    }

    #[test]
    fn placeholder_hex_odd_length() {
        run_success_test(
            "Placeholder: odd-length hex falls back to string",
            b"apple,banana\n",
            &["-d", ",", "--placeholder=0x2C2", "1", "5"],
            b"apple,0x2C2\n",
        );
    }

    #[test]
    fn join_hex_empty_after_prefix() {
        run_success_test(
            "Join: empty hex after 0x falls back to string",
            b"apple,banana,cherry\n",
            &["-d", ",", "--join=0x", "1", "3"],
            b"apple0xcherry\n",
        );
    }

    #[test]
    fn placeholder_hex_empty_after_prefix() {
        run_success_test(
            "Placeholder: empty hex after 0x falls back to string",
            b"apple,banana\n",
            &["-d", ",", "--placeholder=0x", "1", "5"],
            b"apple,0x\n",
        );
    }

    #[test]
    fn join_hex_invalid_characters() {
        run_success_test(
            "Join: invalid hex characters fall back to string",
            b"apple,banana,cherry\n",
            &["-d", ",", "--join=0xGH", "1", "3"],
            b"apple0xGHcherry\n",
        );
    }

    #[test]
    fn placeholder_hex_invalid_characters() {
        run_success_test(
            "Placeholder: invalid hex characters fall back to string",
            b"apple,banana\n",
            &["-d", ",", "--placeholder=0xGH", "1", "5"],
            b"apple,0xGH\n",
        );
    }

    #[test]
    fn join_hex_with_special_flags() {
        run_success_test(
            "Join: hex parsing doesn't interfere with special flags",
            b"apple,banana,cherry\n",
            &["-d", ",", "--join=@auto", "1", "3"],
            b"apple,cherry\n",
        );
    }

    #[test]
    fn placeholder_hex_in_char_mode() {
        run_success_test(
            "Placeholder: hex in char mode",
            b"hello\n",
            &["--characters", "--placeholder=0x2C20", "1", "10", "3"],
            b"h, l\n",
        );
    }

    #[test]
    fn join_hex_in_char_mode() {
        run_success_test(
            "Join: hex in char mode",
            b"hello\n",
            &["--characters", "--join=0x2C20", "1", "3", "5"],
            b"h, l, o\n",
        );
    }
}

mod align {
    use super::*;

    #[test]
    fn basic_alignment() {
        run_success_test(
            "Align: basic alignment",
            b"apple,banana,cherry\na,bb,ccc\nx,y,z\n",
            &["-d", ",", "--align", "1", "2", "3"],
            b"apple,banana,cherry\na,    bb,    ccc\nx,    y,     z\n",
        );
    }

    #[test]
    fn align_no_padding_after_final_field() {
        run_success_test(
            "Align: no padding after final field",
            b"apple,banana\na,bb\n",
            &["-d", ",", "--align", "1", "2"],
            b"apple,banana\na,    bb\n",
        );
    }

    #[test]
    fn align_with_join() {
        run_success_test(
            "Align: with join string",
            b"apple,banana,cherry\na,bb,ccc\n",
            &["-d", ",", "--align", "--join=|", "1", "2", "3"],
            b"apple|banana|cherry\na|    bb|    ccc\n",
        );
    }

    #[test]
    fn align_with_skip_empty() {
        run_success_test(
            "Align: with skip-empty",
            b"apple,,cherry\na,bb,\n",
            &["-d", ",", "--align", "--skip-empty", "1", "2"],
            b"apple,cherry\na,    bb\n",
        );
    }

    #[test]
    fn align_with_placeholder() {
        run_success_test(
            "Align: with placeholder",
            b"apple,banana,cherry\na,bb\n",
            &["-d", ",", "--align", "--placeholder=X", "1", "2", "3"],
            b"apple,banana,cherry\na,    bb,    X\n",
        );
    }

    #[test]
    fn align_empty_input() {
        run_success_test("Align: empty input", b"", &["-d", ",", "--align", "1"], b"");
    }

    #[test]
    fn align_single_field() {
        run_success_test(
            "Align: single field (no padding needed)",
            b"apple\na\n",
            &["-d", ",", "--align", "1"],
            b"apple\na\n",
        );
    }

    #[test]
    fn align_with_invert() {
        run_success_test(
            "Align: with invert",
            b"apple,banana,cherry\na,bb,ccc\n",
            &["-d", ",", "--align", "--invert", "2"],
            b"apple,cherry\na,    ccc\n",
        );
    }

    #[test]
    fn align_error_whole_string() {
        run_error_test(
            "Align: error in whole-string mode",
            b"apple,banana\n",
            &["-w", "-d", ",", "--align", "1", "2"],
        );
    }

    #[test]
    fn align_error_bytes_mode() {
        run_error_test(
            "Align: error in bytes mode",
            b"hello\n",
            &["--bytes", "--align", "1"],
        );
    }

    #[test]
    fn align_error_chars_mode() {
        run_error_test(
            "Align: error in chars mode",
            b"hello\n",
            &["--characters", "--align", "1"],
        );
    }
}

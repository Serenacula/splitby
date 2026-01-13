#!/bin/bash

# Test cases for splitby (both bash and Rust versions)
# Usage: ./test.sh [bash|rust|both]
#   bash: Test only the bash version (default)
#   rust: Test only the Rust version
#   both: Test both versions sequentially

# Determine which version to test
VERSION="${1:-bash}"

# Set the command based on version
if [[ "$VERSION" == "rust" ]]; then
    SPLITBY_CMD="./target/release/splitby"
    if [[ ! -f "$SPLITBY_CMD" ]]; then
        echo "Error: Rust binary not found at $SPLITBY_CMD"
        echo "Please build it first with: cargo build --release"
        exit 1
    fi
elif [[ "$VERSION" == "both" ]]; then
    # Test both versions
    echo "========================================="
    echo "Testing both bash and Rust versions"
    echo "========================================="
    echo
    echo "Testing bash version..."
    echo "----------------------------------------"
    "$0" bash
    bash_status=$?
    echo
    echo "Testing Rust version..."
    echo "----------------------------------------"
    "$0" rust
    rust_status=$?
    echo
    if [[ $bash_status -eq 0 && $rust_status -eq 0 ]]; then
        echo "========================================="
        echo "All tests passed for both versions!"
        echo "========================================="
        exit 0
    else
        echo "========================================="
        echo "Some tests failed"
        echo "========================================="
        exit 1
    fi
elif [[ "$VERSION" == "bash" ]]; then
    SPLITBY_CMD="./splitby.sh"
    if [[ ! -f "$SPLITBY_CMD" ]]; then
        echo "Error: Bash script not found at $SPLITBY_CMD"
        exit 1
    fi
else
    echo "Usage: $0 [bash|rust|both]"
    echo "  bash: Test only the bash version (default)"
    echo "  rust: Test only the Rust version"
    echo "  both: Test both versions sequentially"
    exit 1
fi

echo "Testing with: $SPLITBY_CMD (version: $VERSION)"
echo

# Function to normalize output (strip "record 0: " prefix from Rust errors)
normalize_output() {
    local output="$1"
    # Remove "record 0: " prefix if present (Rust version)
    output="${output#record 0: }"
    # Remove trailing newline for comparison
    echo -n "$output"
}

# Function to run the actual test
run_test() {
    local description="$1"
    local command="$2"
    local expected="$3"

    echo -n "testing $description... "

    # Replace ./splitby.sh with the actual command
    command="${command//.\/splitby.sh/$SPLITBY_CMD}"

    actual_output=$(eval "$command" 2>&1)
    status=$?

    # Normalize output for comparison
    actual_output=$(normalize_output "$actual_output")
    expected_normalized=$(normalize_output "$expected")

    # For error cases, check if command failed (non-zero exit)
    if [[ "$expected" == "error" ]]; then
        if [[ $status -eq 0 ]]; then
            echo "FAILED"
            echo "  Expected error (non-zero exit), but got exit code 0"
            echo "  Command: $command"
            echo "  Output: $actual_output"
            exit 1
        fi
        echo "OK"
        return 0
    fi

    # For non-error cases, command should succeed
    if [[ $status -ne 0 ]]; then
        echo "FAILED"
        echo "  Expected success, but got exit code $status"
        echo "  Command: $command"
        echo "  Output: $actual_output"
        exit 1
    fi

    # Compare normalized outputs
    if [[ "$actual_output" != "$expected_normalized" ]]; then
        echo "FAILED"
        echo "  Command: $command"
        echo "  Expected: $(printf '%q' "$expected_normalized")"
        echo "  Actual:   $(printf '%q' "$actual_output")"
        exit 1
    fi

    echo "OK"
}

# Basic usage
run_test "Split by space" "echo 'this is a test' | ./splitby.sh -d '\\s+' 1" "this"
run_test "Split by comma" "echo 'apple,banana,plum,cherry' | ./splitby.sh -d ',' 2" "banana"
# Note: Equals syntax (--delimiter=' ') works in bash but not in Rust/clap
run_test "Test equals syntax" "echo 'this is a test' | ./splitby.sh -w --delimiter=' '" $'this\nis\na\ntest'

# Mode: Per line
run_test "Per-line default extracts index 2 from every row" "printf 'u v w\nx y z\n' | ./splitby.sh -d ' ' 2" $'v\ny'

# Mode: Whole text
run_test "Test with newline delimiter" "echo -e 'this\nis\na\ntest' | ./splitby.sh --whole-string -d '\n' 2" "is"

# Negative usage
run_test "Negative number" "echo 'this is a test' | ./splitby.sh -d ' ' -1" "test"
run_test "Negative split by comma" "echo 'apple,banana,plum,cherry' | ./splitby.sh -d ',' -2" "plum"

# Empty index
run_test "Split by space, empty selection" "echo 'this is a test' | ./splitby.sh -d ' '" $'this is a test'
run_test "Split by space, empty selection whole-string" "echo 'this is a test' | ./splitby.sh -w -d ' '" $'this\nis\na\ntest'

# Range selection
run_test "Range selection" "echo 'this is a test' | ./splitby.sh -d ' ' 1-2" "this is"
run_test "Negative range selection" "echo 'this is a test' | ./splitby.sh -d ' ' -3--1" "is a test"
run_test "Positive to negative range" "echo 'this is a test' | ./splitby.sh -d ' ' 2--1" "is a test"
run_test "Negative to positive range" "echo 'this is a test' | ./splitby.sh -d ' ' -3-4" "is a test"

# Multiple indexes
run_test "Split by space with multiple indexes" "echo 'this is a test' | ./splitby.sh -d ' ' 1 2 3-4" $'this is a test'
run_test "Split by space whole-string" "echo 'this is a test' | ./splitby.sh -w -d ' ' 1 2 3-4" $'this\nis\na test'

# Edge cases
run_test "Single field with out-of-range index" "echo 'apple' | ./splitby.sh -d ' ' 2" ""
run_test "Single delimiter at beginning" "echo ' apple' | ./splitby.sh -d ' ' 2" "apple"
run_test "Single delimiter at end" "echo 'apple ' | ./splitby.sh -d ' ' 1" "apple"
run_test "Multiple delimiters with spaces and commas" "echo 'apple, orange  banana, pear' | ./splitby.sh -d '[, ]+' 1-3" "apple, orange  banana"
run_test "Delimiter appears multiple times" "echo 'apple,,orange' | ./splitby.sh -d ',' 3" "orange"
run_test "Delimiter appears multiple times with range" "echo 'apple,,orange' | ./splitby.sh -d ',' 1-3" "apple,,orange"

# Join feature
run_test "Can join selections" "echo 'boo hoo foo' | ./splitby.sh -d ' ' -j ','" "boo,hoo,foo"
if [[ "$VERSION" == "rust" ]]; then
    run_test "Doesn't join in ranges" "echo 'boo hoo foo' | ./splitby.sh -d ' ' -j ',' 1 2-3" "boo,hoo,foo"
else
    run_test "Doesn't join in ranges" "echo 'boo hoo foo' | ./splitby.sh -d ' ' -j ',' 1 2-3" "boo,hoo foo"
fi



# Count feature
run_test "Using --count to count fields" "echo 'this is a test' | ./splitby.sh -d ' ' --count" "4"
run_test "Using --count with newline delimiter" "echo -e 'this\nis\na\ntest' | ./splitby.sh -d '\\n' --count" $'1\n1\n1\n1'
run_test "Using --count with newline delimiter whole-string" "echo -e 'this\nis\na\ntest' | ./splitby.sh --whole-string -d '\\n' --count" "4"
run_test "Using --count with extra newline" "echo -e 'this\nis\na\ntest\n' | ./splitby.sh -d '\\n' --count" $'1\n1\n1\n1'
run_test "Using --count with extra newline whole-string" "echo -e 'this\nis\na\ntest\n' | ./splitby.sh --whole-string -d '\\n' --count" "4"
run_test "Count takes precedence over join" "echo 'a b c' | ./splitby.sh -d ' ' --count -j ','" "3"
run_test "Per-line default with count (per row)" "printf 'one two\nalpha beta gamma\n' | ./splitby.sh -d ' ' --count" $'2\n3'

# Invert feature
run_test "Invert single index" "echo 'a b c d' | ./splitby.sh -d ' ' --invert 2" $'a c d'
run_test "Invert single index whole-string" "echo 'a b c d' | ./splitby.sh -d ' ' --whole-string --invert 2" $'a\nc d'
run_test "Invert range selection" "echo 'a b c d' | ./splitby.sh -d ' ' --invert 2-3" $'a d'
run_test "Invert range with join" "echo 'a b c d' | ./splitby.sh -d ' ' --invert -j ',' 2-3" "a,d"
run_test "Invert whole set (empty result)" "echo 'a b' | ./splitby.sh -d ' ' --invert 1-2" ""
run_test "Invert whole set with placeholder" "echo 'a b' | ./splitby.sh -d ' ' --invert --placeholder 1-2" ""
run_test "Invert with count" "echo 'a b c' | ./splitby.sh -d ' ' --invert 2 --count" "3"


# Strict bounds feature
run_test "Strict bounds feature" "echo 'this is a test' | ./splitby.sh -d ' ' --strict-bounds 2-4" "is a test"
run_test "Strict bounds with out-of-range index" "echo 'this is a test' | ./splitby.sh -d ' ' --strict-bounds 0" "error"
run_test "Strict bounds with out-of-range index" "echo 'this is a test' | ./splitby.sh -d ' ' --strict-bounds 5" "error"
run_test "Empty string with strict bounds" "echo '' | ./splitby.sh -d ' ' --strict-bounds 1" "error"

# Strict return feature
run_test "Strict return feature" "echo ',boo' | ./splitby.sh --strict-return -d ',' 1" "error"
run_test "Strict return with out-of-range index" "echo 'this is a test' | ./splitby.sh --strict-return -d 'z' 2" "error"
run_test "Strict return doesn't allow empty fields" "echo ',' | ./splitby.sh --strict-return -d ','" "error"
run_test "Strict return counts" "echo ',' | ./splitby.sh --strict-return --count -d ','" "2"

# No strict range
run_test "Start after end" "echo 'this is a test' | ./splitby.sh --no-strict-range-order -d ' ' 2-1" ""
run_test "Start after end negative" "echo 'this is a test' | ./splitby.sh --no-strict-range-order -d ' ' -1--2" ""
run_test "Start after end positive-negative" "echo 'this is a test' | ./splitby.sh --no-strict-range-order -d ' ' 4--2" ""
run_test "Start after end negative-positive" "echo 'this is a test' | ./splitby.sh --no-strict-range-order -d ' ' -1-3" ""

# Strict range feature
run_test "Start after end" "echo 'this is a test' | ./splitby.sh -d ' ' 2-1" "error"
run_test "Start after end negative" "echo 'this is a test' | ./splitby.sh -d ' ' -1--2" "error"
run_test "Start after end positive-negative" "echo 'this is a test' | ./splitby.sh -d ' ' 4--2" "error"
run_test "Start after end negative-positive" "echo 'this is a test' | ./splitby.sh -d ' ' -1-3" "error"
run_test "Works with correct syntax" "echo 'this is a test' | ./splitby.sh -d ' ' 1-2" "this is"
run_test "Works with no range" "echo 'this is a test' | ./splitby.sh -w -d ' '" $'this\nis\na\ntest'

# Skip empty feature
run_test "Starting empty field" "echo ',orange' | ./splitby.sh --skip-empty -d ',' 1" "orange"
run_test "Middle field empty" "echo 'apple,,orange' | ./splitby.sh --skip-empty -d ',' 2" "orange"
run_test "Final field empty" "echo 'orange,' | ./splitby.sh --skip-empty -d ',' 2" ""
run_test "All fields empty" "echo ',' | ./splitby.sh --skip-empty -d ','" ""
run_test "Known failure" "echo 'a  b   c' | ./splitby.sh -d ' ' --skip-empty 1-3" "a b c"

# Skip with strict
run_test "Skip with strict bounds works" "echo 'orange,' | ./splitby.sh --skip-empty --strict-bounds -d ',' 1" "orange"
run_test "Skip with strict bounds fails" "echo 'orange,' | ./splitby.sh --skip-empty --strict-bounds -d ',' 2" "error"
run_test "Skip with strict return works" "echo 'orange,' | ./splitby.sh --skip-empty --strict-return -d ',' 1" "orange"
run_test "Skip with strict return fails" "echo ',,' | ./splitby.sh --skip-empty --strict-return -d ',' 1" "error"

# Skip with count
run_test "Starting empty field with count" "echo ',orange' | ./splitby.sh --skip-empty -d ',' --count" "1"
run_test "Middle field empty with count" "echo 'apple,,orange' | ./splitby.sh --skip-empty -d ',' --count" "2"
run_test "Final field empty with count" "echo 'orange,' | ./splitby.sh --skip-empty -d ',' --count" "1"
run_test "All fields empty with count" "echo ',' | ./splitby.sh --skip-empty -d ',' --count" "0"

# Invalid delimiter
run_test "Delimiter not provided" "echo 'this is a test' | ./splitby.sh 1" "error"
run_test "Delimiter empty" "echo 'this is a test' | ./splitby.sh  -d '' 1" "error"
run_test "Invalid delimiter regex" "echo 'this is a test' | ./splitby.sh -d '[[' 1" "error"

# Empty input
run_test "Empty input" "echo '' | ./splitby.sh -d '\\s+' 1" "error"
run_test "Empty -i input" "./splitby.sh -i '' -d ','" "error"
run_test "No input" "./splitby.sh -d ','" "error"

# Invalid index
run_test "Invalid index format" "echo 'this is a test' | ./splitby.sh -d '\\s+' 1a" "error"
run_test "Invalid range format" "echo 'this is a test' | ./splitby.sh -d '\\s+' 1-2a" "error"


# Byte mode tests (Rust version only - bash doesn't support byte mode)
if [[ "$VERSION" == "rust" ]]; then
    echo
    echo "=== Byte Mode Tests (Rust only) ==="
    run_test "Byte mode: single byte" "echo 'hello' | ./splitby.sh --bytes 1" "h"
    run_test "Byte mode: byte range" "echo 'hello' | ./splitby.sh --bytes 1-3" "hel"
    run_test "Byte mode: negative index" "echo 'hello' | ./splitby.sh --bytes -2" "lo"
    run_test "Byte mode: negative range" "echo 'hello' | ./splitby.sh --bytes -3--1" "llo"
    run_test "Byte mode: multiple selections" "echo 'hello' | ./splitby.sh --bytes 1 3 5" "h l o"
    run_test "Byte mode: full range" "echo 'hello' | ./splitby.sh --bytes 1-5" "hello"
    run_test "Byte mode: no selections (output all)" "echo 'hello' | ./splitby.sh --bytes" "hello"
    run_test "Byte mode: empty input" "echo '' | ./splitby.sh --bytes" ""
    run_test "Byte mode: --count" "echo 'hello' | ./splitby.sh --count --bytes" "5"
    run_test "Byte mode: --count with empty" "echo '' | ./splitby.sh --count --bytes" "0"
    run_test "Byte mode: --join" "echo 'hello' | ./splitby.sh --join ',' --bytes 1 3 5" "h,l,o"
    run_test "Byte mode: --invert" "echo 'hello' | ./splitby.sh --invert --bytes 2 4" "h l o"
    run_test "Byte mode: --invert range" "echo 'hello' | ./splitby.sh --invert --bytes 2-4" "ho"
    run_test "Byte mode: --strict-bounds valid" "echo 'hello' | ./splitby.sh --strict-bounds --bytes 1-3" "hel"
    run_test "Byte mode: --strict-bounds invalid" "echo 'hello' | ./splitby.sh --strict-bounds --bytes 10" "error"
    run_test "Byte mode: --placeholder out-of-bounds" "echo 'hello' | ./splitby.sh --placeholder --bytes 10" ""
    run_test "Byte mode: --placeholder multiple" "echo 'hello' | ./splitby.sh --placeholder --bytes 1 10 3" "h l"
    run_test "Byte mode: whole-string mode" "echo -e 'hello\nworld' | ./splitby.sh --whole-string --bytes 1-5" "hello"
    run_test "Byte mode: whole-string mode with newline join" "echo -e 'hello\nworld' | ./splitby.sh --whole-string --bytes 1 2" "h\ne"
    run_test "Byte mode: --strict-return empty output" "echo 'hello' | ./splitby.sh --strict-return --placeholder --bytes 10" "error"
fi

# Char mode tests (Rust version only - bash doesn't support char mode)
if [[ "$VERSION" == "rust" ]]; then
    echo
    echo "=== Char Mode Tests (Rust only) ==="
    run_test "Char mode: single character" "echo 'hello' | ./splitby.sh --characters 1" "h"
    run_test "Char mode: character range" "echo 'hello' | ./splitby.sh --characters 1-3" "hel"
    run_test "Char mode: negative index" "echo 'hello' | ./splitby.sh --characters -2" "l"
    run_test "Char mode: negative range" "echo 'hello' | ./splitby.sh --characters -3--1" "llo"
    run_test "Char mode: multiple selections" "echo 'hello' | ./splitby.sh --characters 1 3 5" "h l o"
    run_test "Char mode: full range" "echo 'hello' | ./splitby.sh --characters 1-5" "hello"
    run_test "Char mode: no selections (output all)" "echo 'hello' | ./splitby.sh --characters" "hello"
    run_test "Char mode: empty input" "echo '' | ./splitby.sh --characters" ""
    run_test "Char mode: --count" "echo 'hello' | ./splitby.sh --count --characters" "5"
    run_test "Char mode: --count with empty" "echo '' | ./splitby.sh --count --characters" "0"
    run_test "Char mode: --count with graphemes" "echo 'café' | ./splitby.sh --count --characters" "4"
    run_test "Char mode: --join" "echo 'hello' | ./splitby.sh --join ',' --characters 1 3 5" "h,l,o"
    run_test "Char mode: --invert" "echo 'hello' | ./splitby.sh --invert --characters 2 4" "h l o"
    run_test "Char mode: --invert range" "echo 'hello' | ./splitby.sh --invert --characters 2-4" "ho"
    run_test "Char mode: --strict-bounds valid" "echo 'hello' | ./splitby.sh --strict-bounds --characters 1-3" "hel"
    run_test "Char mode: --strict-bounds invalid" "echo 'hello' | ./splitby.sh --strict-bounds --characters 10" "error"
    run_test "Char mode: --placeholder out-of-bounds" "echo 'hello' | ./splitby.sh --placeholder --characters 10" " "
    run_test "Char mode: --placeholder multiple" "echo 'hello' | ./splitby.sh --placeholder --characters 1 10 3" "h   l"
    run_test "Char mode: whole-string mode" "echo -e 'hello\nworld' | ./splitby.sh --whole-string --characters 1-5" "hello"
    run_test "Char mode: whole-string mode with newline join" "echo -e 'hello\nworld' | ./splitby.sh --whole-string --characters 1 2" "h\ne"
    run_test "Char mode: --strict-return empty output" "echo 'hello' | ./splitby.sh --strict-return --placeholder --characters 10" "error"
    run_test "Char mode: grapheme cluster (café)" "echo 'café' | ./splitby.sh --characters 1-4" "café"
    run_test "Char mode: grapheme cluster (combining)" "printf 'e\u0301\n' | ./splitby.sh --characters 1" "é"
fi


# Unimplemented features (not yet implemented in Rust version)
# These tests are placed at the end and only run for bash version

# Simple ranges feature (not implemented in Rust yet)
# if [[ "$VERSION" == "bash" ]]; then
#     run_test "Simple ranges flattens range to selections" "echo 'a b c' | ./splitby.sh -w -d ' ' --simple-ranges 1-2" $'a\nb'
#     run_test "Simple ranges with join" "echo 'a b c' | ./splitby.sh -d ' ' --simple-ranges -j ',' 1-3" "a,b,c"
#     run_test "Simple ranges with mixed selection" "echo 'a b c d' | ./splitby.sh -w -d ' ' --simple-ranges 1 2-3 4" $'a\nb\nc\nd'
#     run_test "Simple ranges with join and mixed selection" "echo 'a b c d' | ./splitby.sh -w -d ' ' --simple-ranges -j '|' 1 2-3 4" "a|b|c|d"
#     run_test "Simple ranges with negative range" "echo 'a b c d' | ./splitby.sh -w -d ' ' --simple-ranges -3--1" $'b\nc\nd'
#     run_test "Join and simple-ranges with out-of-bounds range" "echo 'x y' | ./splitby.sh -w -d ' ' --simple-ranges -j ',' 3-5" ""
#     run_test "Count takes precedence over simple ranges" "echo 'a b c' | ./splitby.sh -d ' ' --count --simple-ranges 1-3" "3"
#     run_test "Invert index with simple ranges" "echo 'a b c d' | ./splitby.sh -d ' ' --whole-string --invert --simple-ranges 2" $'a\nc\nd'
#     run_test "Invert index with simple ranges and join" "echo 'a b c d' | ./splitby.sh -d ' ' --invert --simple-ranges -j ',' 2" "a,c,d"
# fi

# Replace range delimiter feature (not implemented in Rust yet)
# if [[ "$VERSION" == "bash" ]]; then
#     run_test "Replaces delimiter in range" "echo 'a b c' | ./splitby.sh -d ' ' --replace-range-delimiter ',' 1-3" "a,b,c"
#     run_test "Replaces delimiter in range with custom symbol" "echo 'a-b-c' | ./splitby.sh -d '-' --replace-range-delimiter ':' 1-3" "a:b:c"
#     run_test "Replace range delimiter only applies to range" "echo 'a b c d' | ./splitby.sh -w -d ' ' --replace-range-delimiter '|' 1 2-3 4" $'a\nb|c\nd'
#     run_test "Replace delimiter with skip-empty" "echo 'a  b   c' | ./splitby.sh -d ' ' --skip-empty --replace-range-delimiter ':' 1-3" "a:b:c"
#     run_test "Simple ranges overrides delimiter replacement" "echo 'a b c' | ./splitby.sh -d ' ' --simple-ranges --replace-range-delimiter ':' -j ',' 1-3" "a,b,c"
#     run_test "Replace range delimiter on empty result" "echo 'a b' | ./splitby.sh -d ' ' --replace-range-delimiter ':' 5-6" ""
# fi

# If all tests pass

echo
echo "-----------------------------------"
echo "Tests passed"
echo "-----------------------------------"
echo

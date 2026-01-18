#!/bin/bash

# Test cases for splitby (Rust version)
# Usage: ./test.sh

SPLITBY_CMD="./target/release/splitby"

echo "Building Rust release version..."
echo "----------------------------------------"
cargo build --release
if [[ $? -ne 0 ]]; then
    echo "Error: Failed to build Rust binary"
    exit 1
fi
echo "Build complete."
echo

# Function to normalize output (strip trailing newline for comparison)
normalize_output() {
    local output="$1"
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

# Comma-separated selections
run_test "Comma-separated selections: basic" "echo 'apple,banana,cherry,date' | ./splitby.sh -d ',' 1,2,3" "apple,banana,cherry"
run_test "Comma-separated selections: with ranges" "echo 'apple,banana,cherry,date,elderberry' | ./splitby.sh -d ',' 1-2,4,5" "apple,banana,date,elderberry"
run_test "Comma-separated selections: leading comma" "echo 'apple,banana,cherry' | ./splitby.sh -d ',' ,1" "apple"
run_test "Comma-separated selections: trailing comma" "echo 'apple,banana,cherry' | ./splitby.sh -d ',' 1," "apple"
run_test "Comma-separated selections: mixed with spaces" "echo 'apple,banana,cherry,date' | ./splitby.sh -d ',' 1,2 3 4" "apple,banana,cherry,date"
run_test "Comma-separated selections: multiple comma strings" "echo 'apple,banana,cherry,date' | ./splitby.sh -d ',' 1,2 3,4" "apple,banana,cherry,date"
run_test "Comma-separated selections: negative indices" "echo 'apple,banana,cherry,date' | ./splitby.sh -d ',' -3,-1" "banana,date"
run_test "Comma-separated selections: mixed positive and negative" "echo 'apple,banana,cherry,date' | ./splitby.sh -d ',' 1,-2" "apple,cherry"
run_test "Comma-separated selections: with join" "echo 'apple,banana,cherry' | ./splitby.sh -d ',' -j '|' 1,2,3" "apple|banana|cherry"
run_test "Comma-separated selections: whole-string mode" "echo -e 'apple,banana\ncherry,date' | ./splitby.sh -w -d ',' 1,2" $'apple\nbanana\ncherry'
run_test "Comma-separated selections: byte mode" "echo 'hello' | ./splitby.sh --bytes 1,3,5" "hlo"
run_test "Comma-separated selections: char mode" "echo 'hello' | ./splitby.sh --characters 1,3,5" "hlo"
run_test "Comma-separated selections: with invert" "echo 'a,b,c,d' | ./splitby.sh -d ',' --invert 1,3" "b,d"
run_test "Comma-separated selections: empty parts ignored" "echo 'apple,banana,cherry' | ./splitby.sh -d ',' ,,1,," "apple"
run_test "Comma-separated selections: with -d flag, comma-only splits to empty (skipped)" "echo 'apple,banana,cherry' | ./splitby.sh -d ',' ',' 1" "apple"
run_test "Comma-separated selections: with -d flag, letter treated as selection (errors)" "echo 'apple,banana,cherry' | ./splitby.sh -d ',' 'a' 1" "error"
run_test "Comma-separated selections: with -d flag, mixed letter-number (letter part errors)" "echo 'apple,banana,cherry' | ./splitby.sh -d ',' '1,a' 1" "error"

# Optional delimiter (automatic detection)
run_test "Optional delimiter: comma as first argument" "echo 'apple,banana,cherry' | ./splitby.sh , 1" "apple"
run_test "Optional delimiter: comma with multiple selections" "echo 'apple,banana,cherry' | ./splitby.sh , 1 3" "apple,cherry"
run_test "Optional delimiter: regex pattern as first argument" "echo 'this is a test' | ./splitby.sh '\\s+' 1 2" "this is"
run_test "Optional delimiter: -d flag takes priority" "echo 'apple,banana,cherry' | ./splitby.sh -d ',' . 1" "error"
run_test "Optional delimiter: selection takes priority over delimiter" "echo 'apple,banana,cherry' | ./splitby.sh 1 2" "error"
run_test "Optional delimiter: single letter as delimiter" "echo 'apple,banana,cherry' | ./splitby.sh a 2" "pple,b"

# Edge cases
run_test "Single field with out-of-range index" "echo 'apple' | ./splitby.sh -d ' ' 2" ""
run_test "Single delimiter at beginning" "echo ' apple' | ./splitby.sh -d ' ' 2" "apple"
run_test "Single delimiter at end" "echo 'apple ' | ./splitby.sh -d ' ' 1" "apple"
run_test "Multiple delimiters with spaces and commas" "echo 'apple, orange  banana, pear' | ./splitby.sh -d '[, ]+' 1-3" "apple, orange  banana"
run_test "Delimiter appears multiple times" "echo 'apple,,orange' | ./splitby.sh -d ',' 3" "orange"
run_test "Delimiter appears multiple times with range" "echo 'apple,,orange' | ./splitby.sh -d ',' 1-3" "apple,,orange"

# Join feature
run_test "Can join selections" "echo 'boo hoo foo' | ./splitby.sh -d ' ' -j ','" "boo,hoo,foo"
run_test "Doesn't join in ranges" "echo 'boo hoo foo' | ./splitby.sh -d ' ' -j ',' 1 2-3" "boo,hoo,foo"

# Trim newline feature
run_test "Trim newline: per-line mode" "echo -e 'a\nb\nc' | ./splitby.sh --trim-newline -d ' ' 1" $'a\nb\nc'
run_test "Trim newline: single line" "echo 'a' | ./splitby.sh --trim-newline -d ' ' 1" "a"
run_test "Trim newline: whole-string mode" "echo -e 'a\nb' | ./splitby.sh --trim-newline -w -d '\n' 1" "a"
run_test "Trim newline: without flag (has newline)" "echo -e 'a\nb\nc' | ./splitby.sh -d ' ' 1" $'a\nb\nc\n'

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
run_test "Invert whole set with placeholder" "echo 'a b' | ./splitby.sh -d ' ' --invert --placeholder="?" 1-2" ""
run_test "Invert with count" "echo 'a b c' | ./splitby.sh -d ' ' --count --invert 2" "3"


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
run_test "Empty input" "echo '' | ./splitby.sh -d '\\s+' 1" ""
run_test "Empty -i input" "./splitby.sh -i '' -d ','" "error"

# Invalid index
run_test "Invalid index format" "echo 'this is a test' | ./splitby.sh -d '\\s+' 1a" "error"
run_test "Invalid range format" "echo 'this is a test' | ./splitby.sh -d '\\s+' 1-2a" "error"


# Byte mode tests
echo
echo "=== Byte Mode Tests ==="
    run_test "Byte mode: single byte" "echo 'hello' | ./splitby.sh --bytes 1" "h"
    run_test "Byte mode: byte range" "echo 'hello' | ./splitby.sh --bytes 1-3" "hel"
    run_test "Byte mode: negative index" "echo 'hello' | ./splitby.sh --bytes -2" "l"
    run_test "Byte mode: negative range" "echo 'hello' | ./splitby.sh --bytes -3--1" "llo"
    run_test "Byte mode: multiple selections" "echo 'hello' | ./splitby.sh --bytes 1 3 5" "hlo"
    run_test "Byte mode: full range" "echo 'hello' | ./splitby.sh --bytes 1-5" "hello"
    run_test "Byte mode: no selections (output all)" "echo 'hello' | ./splitby.sh --bytes" "hello"
    run_test "Byte mode: newline works" "echo -e 'hello\nworld' | ./splitby.sh --bytes 1" $'h\nw'
    run_test "Byte mode: empty input" "echo '' | ./splitby.sh --bytes" ""
    run_test "Byte mode: --count" "echo 'hello' | ./splitby.sh --count --bytes" "5"
    run_test "Byte mode: --count with empty" "echo '' | ./splitby.sh --count --bytes" "0"
    run_test "Byte mode: --join" "echo 'hello' | ./splitby.sh --join ',' --bytes 1 3 5" "h,l,o"
    run_test "Byte mode: --invert" "echo 'hello' | ./splitby.sh --invert --bytes 2 4" "hlo"
    run_test "Byte mode: --invert range" "echo 'hello' | ./splitby.sh --invert --bytes 2-4" "ho"
    run_test "Byte mode: --strict-bounds valid" "echo 'hello' | ./splitby.sh --strict-bounds --bytes 1-3" "hel"
    run_test "Byte mode: --strict-bounds invalid" "echo 'hello' | ./splitby.sh --strict-bounds --bytes 10" "error"
    run_test "Byte mode: --placeholder out-of-bounds" "echo 'hello' | ./splitby.sh --placeholder=? --bytes 10" "?"
    run_test "Byte mode: --placeholder multiple" "echo 'hello' | ./splitby.sh --placeholder=0x00 --bytes 1 10 3 | hexdump -C | head -1 | sed 's/^[^ ]*  //; s/  .*$//'" "68 00 6c 0a"
    run_test "Byte mode: whole-string mode" "echo -e 'hello\nworld' | ./splitby.sh --whole-string --bytes 1-5" "hello"
    run_test "Byte mode: whole-string mode with newline join" "echo -e 'hello\nworld' | ./splitby.sh --whole-string --bytes 1 2" "he"
run_test "Byte mode: --strict-return empty output 0 placeholder" "echo 'hello' | ./splitby.sh --strict-return --placeholder=0x00 --bytes 10" ""
run_test "Byte mode: --strict-return empty output" "echo 'hello' | ./splitby.sh --strict-return --bytes 10" "error"

# Char mode tests
echo
echo "=== Char Mode Tests ==="
    run_test "Char mode: single character" "echo 'hello' | ./splitby.sh --characters 1" "h"
    run_test "Char mode: character range" "echo 'hello' | ./splitby.sh --characters 1-3" "hel"
    run_test "Char mode: negative index" "echo 'hello' | ./splitby.sh --characters -2" "l"
    run_test "Char mode: negative range" "echo 'hello' | ./splitby.sh --characters -3--1" "llo"
    run_test "Char mode: multiple selections" "echo 'hello' | ./splitby.sh --characters 1 3 5" "hlo"
    run_test "Char mode: full range" "echo 'hello' | ./splitby.sh --characters 1-5" "hello"
    run_test "Char mode: no selections (output all)" "echo 'hello' | ./splitby.sh --characters" "hello"
    run_test "Char mode: empty input" "echo '' | ./splitby.sh --characters" ""
    run_test "Char mode: --count" "echo 'hello' | ./splitby.sh --count --characters" "5"
    run_test "Char mode: --count with empty" "echo '' | ./splitby.sh --count --characters" "0"
    run_test "Char mode: --count with graphemes" "echo 'café' | ./splitby.sh --count --characters" "4"
    run_test "Char mode: --join" "echo 'hello' | ./splitby.sh --join ',' --characters 1 3 5" "h,l,o"
    run_test "Char mode: --invert" "echo 'hello' | ./splitby.sh --invert --characters 2 4" "hlo"
    run_test "Char mode: --invert range" "echo 'hello' | ./splitby.sh --invert --characters 2-4" "ho"
    run_test "Char mode: --strict-bounds valid" "echo 'hello' | ./splitby.sh --strict-bounds --characters 1-3" "hel"
    run_test "Char mode: --strict-bounds invalid" "echo 'hello' | ./splitby.sh --strict-bounds --characters 10" "error"
    run_test "Char mode: --placeholder out-of-bounds" "echo 'hello' | ./splitby.sh --placeholder=' ' --characters 10" " "
    run_test "Char mode: --placeholder multiple" "echo 'hello' | ./splitby.sh --placeholder=' ' --characters 1 10 3" "h l"
    run_test "Char mode: whole-string mode" "echo -e 'hello\nworld' | ./splitby.sh --whole-string --characters 1-5" "hello"
    run_test "Char mode: whole-string mode with newline join" "echo -e 'hello\nworld' | ./splitby.sh --whole-string --characters 1 2" "he"
    run_test "Char mode: --strict-return empty output" "echo 'hello' | ./splitby.sh --strict-return --placeholder=' ' --characters 10" " "
    run_test "Char mode: grapheme cluster (café)" "echo 'café' | ./splitby.sh --characters 1-4" "café"
    # Test broken because bash was interfering with it, needs manual confirmation when running
    # run_test "Char mode: grapheme cluster (combining)" "printf 'e\u0301\n' | ./splitby.sh --characters 1" "é"

# If all tests pass

echo
echo "-----------------------------------"
echo "Tests passed"
echo "-----------------------------------"
echo

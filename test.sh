#!/bin/bash

# Test cases for the splitby.sh script

# Function to run the actual test
run_test() {
    local description="$1"
    local command="$2"
    local expected="$3"
    
    echo "testing $1"

    actual_output=$(eval "$command" 2>&1)
    status=$?
    
    if [[ "$expected" == "error" ]] && [[ $status -ne 0 ]]; then
        return
    fi
    
    if [[ "$actual_output" != "$expected" ]]; then
        # Print the failed test result and exit immediately
        echo
        echo "-----------------------------------"
        echo "Test failed: $description"
        echo "Command: $command"
        echo "Expected output: $expected"
        echo "Actual output: $actual_output"
        echo "-----------------------------------"
        echo
        exit 1
    fi
}

# Basic usage
run_test "Split by space" "echo 'this is a test' | ./splitby.sh -d '\\s+' 1" "this"
run_test "Split by comma" "echo 'apple,banana,plum,cherry' | ./splitby.sh -d ',' 2" "banana"
run_test "Test equals syntax" "echo 'this is a test' | ./splitby.sh --delimiter=' '" $'this\nis\na\ntest'

# Mode: Per line
run_test "Per-line default extracts index 2 from every row" "printf 'u v w\nx y z\n' | ./splitby.sh -d ' ' 2" $'v\ny'

# Mode: Whole text
run_test "Test with newline delimiter" "echo -e 'this\nis\na\ntest' | ./splitby.sh --whole -d '\\n' 2" "is"

# Negative usage
run_test "Negative number" "echo 'this is a test' | ./splitby.sh -d ' ' -1" "test"
run_test "Negative split by comma" "echo 'apple,banana,plum,cherry' | ./splitby.sh -d ',' -2" "plum"

# Empty index
run_test "Split by space, empty selection" "echo 'this is a test' | ./splitby.sh -d ' '" $'this\nis\na\ntest'

# Range selection
run_test "Range selection" "echo 'this is a test' | ./splitby.sh -d ' ' 1-2" "this is"
run_test "Negative range selection" "echo 'this is a test' | ./splitby.sh -d ' ' -3--1" "is a test"
run_test "Positive to negative range" "echo 'this is a test' | ./splitby.sh -d ' ' 2--1" "is a test"
run_test "Negative to positive range" "echo 'this is a test' | ./splitby.sh -d ' ' -3-4" "is a test"

# Multiple indexes
run_test "Split by space" "echo 'this is a test' | ./splitby.sh -d ' ' 1 2 3-4" $'this\nis\na test'

# Edge cases
run_test "Single field with out-of-range index" "echo 'apple' | ./splitby.sh -d ' ' 2" ""
run_test "Single delimiter at beginning" "echo ' apple' | ./splitby.sh -d ' ' 2" "apple"
run_test "Single delimiter at end" "echo 'apple ' | ./splitby.sh -d ' ' 1" "apple"
run_test "Multiple delimiters with spaces and commas" "echo 'apple, orange  banana, pear' | ./splitby.sh -d '[, ]+' 1-3" "apple, orange  banana"
run_test "Delimiter appears multiple times" "echo 'apple,,orange' | ./splitby.sh -d ',' 3" "orange"
run_test "Delimiter appears multiple times with range" "echo 'apple,,orange' | ./splitby.sh -d ',' 1-3" "apple,,orange"

# Join feature
run_test "Can join selections" "echo 'boo hoo foo' | ./splitby.sh -d ' ' -j ','" "boo,hoo,foo"
run_test "Doesn't join in ranges" "echo 'boo hoo foo' | ./splitby.sh -d ' ' -j ',' 1 2-3" "boo,hoo foo"

# Simple ranges feature
run_test "Simple ranges flattens range to selections" "echo 'a b c' | ./splitby.sh -d ' ' --simple-ranges 1-2" $'a\nb'
run_test "Simple ranges with join" "echo 'a b c' | ./splitby.sh -d ' ' --simple-ranges -j ',' 1-3" "a,b,c"
run_test "Simple ranges with mixed selection" "echo 'a b c d' | ./splitby.sh -d ' ' --simple-ranges 1 2-3 4" $'a\nb\nc\nd'
run_test "Simple ranges with join and mixed selection" "echo 'a b c d' | ./splitby.sh -d ' ' --simple-ranges -j '|' 1 2-3 4" "a|b|c|d"
run_test "Simple ranges with negative range" "echo 'a b c d' | ./splitby.sh -d ' ' --simple-ranges -3--1" $'b\nc\nd'
run_test "Join and simple-ranges with out-of-bounds range" "echo 'x y' | ./splitby.sh -d ' ' --simple-ranges -j ',' 3-5" ""

# Replace range delimiter feature
run_test "Replaces delimiter in range" "echo 'a b c' | ./splitby.sh -d ' ' --replace-range-delimiter ',' 1-3" "a,b,c"
run_test "Replaces delimiter in range with custom symbol" "echo 'a-b-c' | ./splitby.sh -d '-' --replace-range-delimiter ':' 1-3" "a:b:c"
run_test "Replace range delimiter only applies to range" "echo 'a b c d' | ./splitby.sh -d ' ' --replace-range-delimiter '|' 1 2-3 4" $'a\nb|c\nd'
run_test "Replace delimiter with skip-empty" "echo 'a  b   c' | ./splitby.sh -d ' ' --skip-empty --replace-range-delimiter ':' 1-3" "a:b:c"
run_test "Simple ranges overrides delimiter replacement" "echo 'a b c' | ./splitby.sh -d ' ' --simple-ranges --replace-range-delimiter ':' -j ',' 1-3" "a,b,c"
run_test "Replace range delimiter on empty result" "echo 'a b' | ./splitby.sh -d ' ' --replace-range-delimiter ':' 5-6" ""


# Count feature
run_test "Using --count to count fields" "echo 'this is a test' | ./splitby.sh -d ' ' --count" "4"
run_test "Using --count with newline delimiter" "echo -e 'this\nis\na\ntest' | ./splitby.sh --whole -d '\\n' --count" "4"
run_test "Using --count with extra newline" "echo -e 'this\nis\na\ntest\n' | ./splitby.sh --whole -d '\\n' --count" "4"
run_test "Count takes precedence over join" "echo 'a b c' | ./splitby.sh -d ' ' --count -j ','" "3"
run_test "Count takes precedence over simple ranges" "echo 'a b c' | ./splitby.sh -d ' ' --count --simple-ranges 1-3" "3"
run_test "Per-line default with count (per row)" "printf 'one two\nalpha beta gamma\n' | ./splitby.sh -d ' ' --count" $'2\n3'

# Invert feature
run_test "Invert single index" "echo 'a b c d' | ./splitby.sh -d ' ' --invert 2" $'a\nc d'
run_test "Invert range selection" "echo 'a b c d' | ./splitby.sh -d ' ' --invert 2-3" $'a\nd'
run_test "Invert index with simple ranges" "echo 'a b c d' | ./splitby.sh -d ' ' --invert --simple-ranges 2" $'a\nc\nd'
run_test "Invert index with simple ranges and join" "echo 'a b c d' | ./splitby.sh -d ' ' --invert --simple-ranges -j ',' 2" "a,c,d"
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
run_test "Strict return feature" "echo 'this is a test' | ./splitby.sh --strict-return -d 'z'" "error"
run_test "Strict return with out-of-range index" "echo 'this is a test' | ./splitby.sh --strict-return -d 'z' 1" "error"
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
run_test "Works with no range" "echo 'this is a test' | ./splitby.sh -d ' '" $'this\nis\na\ntest'

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


# If all tests pass

echo
echo "-----------------------------------"
echo "Tests passed"
echo "-----------------------------------"
echo

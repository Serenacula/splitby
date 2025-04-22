#!/bin/bash

# Test cases for the splitby.sh script

# Function to run the actual test
run_test() {
    local description="$1"
    local command="$2"
    local expected="$3"

    actual_output=$(eval "$command" 2>&1)
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

# Test 1: Simple split
run_test "Split by space" "echo 'this is a test' | ./splitby.sh -d '\\s+' 1" "this"
run_test "Split by comma" "echo 'apple,banana,cherry' | ./splitby.sh -d ',' 2" "banana"

# Test 3: Range selection with space delimiter
run_test "Range selection with space delimiter" "echo 'this is a test' | ./splitby.sh -d '\\s+' 1-2" "this is"

# Test 4: Range with no end (open-ended range)
run_test "Range with no end" "echo 'this is a test' | ./splitby.sh -d '\\s+' 2-" "is a test"

# Test 5: Use --count flag to count the number of fields
run_test "Use --count to count fields" "echo 'this is a test' | ./splitby.sh -d '\\s+' --count" "4"

# Test 6: Edge case - Single field (index out of range)
run_test "Single field with out-of-range index" "echo 'apple' | ./splitby.sh -d '\\s+' 2" ""

# Test 7: Empty input with error handling
run_test "Empty input with error handling" "echo '' | ./splitby.sh -d '\\s+' 1" "No input provided. Use -i/--input or pipe data to stdin."

# Test 8: Invalid delimiter regex
run_test "Invalid delimiter regex" "echo 'this is a test' | ./splitby.sh -d '[[' 1" "Invalid delimiter regex: [["

# Test 9: Strict bounds checking with out-of-range index
run_test "Strict bounds with out-of-range index" "echo 'this is a test' | ./splitby.sh -d '\\s+' --strict-bounds 10" "Start index (10) out of bounds. Must be between 1 and 4"

# Test 10: Empty string with strict bounds
run_test "Empty string with strict bounds" "echo '' | ./splitby.sh -d '\\s+' --strict-bounds 1" "No input provided. Use -i/--input or pipe data to stdin."

# Test 11: Test with a different delimiter (newline)
run_test "Test with newline delimiter" "echo -e 'this\nis\na\ntest' | ./splitby.sh -d '\\n' 2" "is"

# Test 12: Using --count with different delimiters
run_test "Using --count with newline delimiter" "echo -e 'this\nis\na\ntest' | ./splitby.sh -d '\\n' --count" "4"

# Test 13: Input with spaces and multiple delimiters (whitespace and comma)
run_test "Multiple delimiters with spaces and commas" "echo 'apple, orange  banana, pear' | ./splitby.sh -d '[,\\s]+' 2-3" "orange  banana"

# Test 14: Delimiter is not provided (error handling)
run_test "Delimiter not provided" "echo 'this is a test' | ./splitby.sh 1" "Delimiter is required. Use -d or --delimiter to set one."
run_test "Delimiter not provided" "echo 'this is a test' | ./splitby.sh  -d '' 1" "Delimiter is required. Use -d or --delimiter to set one."

# Test 15: Invalid index format (e.g., '1a' instead of '1')
run_test "Invalid index format" "echo 'this is a test' | ./splitby.sh -d '\\s+' 1a" "Error: Index must be a number or range"
run_test "Invalid index format" "echo 'this is a test' | ./splitby.sh -d '\\s+' 1-2a" "Error: Index must be a number or range"

# If all tests pass

echo
echo "-----------------------------------"
echo "Tests passed"
echo "-----------------------------------"
echo

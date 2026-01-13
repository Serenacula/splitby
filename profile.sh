#!/usr/bin/env bash
# Performance profiling script for splitby

set -euo pipefail

echo "=== Performance Profiling Report ==="
echo

# Build release version
echo "Building release version..."
cargo build --release >/dev/null 2>&1
echo "Build complete."
echo

# Generate test data
echo "Generating test data..."
TMP_DATA=$(mktemp)
trap 'rm -f "$TMP_DATA"' EXIT

# Generate different test cases
generate_test() {
    local lines=$1
    local fields=$2
    awk -v lines="$lines" -v fields="$fields" '
    BEGIN {
        srand(42);
        for (i=1; i<=lines; i++) {
            for (f=1; f<=fields; f++) {
                printf "%d%s", rand()*1000, (f==fields ? ORS : ",");
            }
        }
    }' > "$TMP_DATA"
}

# Test 1: Small file, few fields
echo "Test 1: 1000 lines, 5 fields"
generate_test 1000 5
time (./target/release/splitby -i "$TMP_DATA" -d ',' 1 3 5 >/dev/null 2>&1) 2>&1 | grep real

# Test 2: Medium file, many fields
echo "Test 2: 10000 lines, 20 fields"
generate_test 10000 20
time (./target/release/splitby -i "$TMP_DATA" -d ',' 3 5 7 >/dev/null 2>&1) 2>&1 | grep real

# Test 3: Large file, few fields
echo "Test 3: 100000 lines, 10 fields"
generate_test 100000 10
time (./target/release/splitby -i "$TMP_DATA" -d ',' 3 5 7 >/dev/null 2>&1) 2>&1 | grep real

# Test 4: Small file, many fields
echo "Test 4: 1000 lines, 50 fields"
generate_test 1000 50
time (./target/release/splitby -i "$TMP_DATA" -d ',' 1 25 50 >/dev/null 2>&1) 2>&1 | grep real

echo
echo "=== Performance Analysis ==="
echo
echo "Key areas to profile:"
echo "1. Field extraction (regex matching)"
echo "2. Selection processing"
echo "3. String/byte operations"
echo "4. Memory allocations"
echo "5. Channel communication overhead"
echo

# Try to use flamegraph if available
if command -v cargo-flamegraph >/dev/null 2>&1; then
    echo "Attempting to generate flamegraph..."
    echo "Note: This may require sudo on macOS"
    echo

    generate_test 10000 20
    if sudo -n true 2>/dev/null; then
        echo "Running with sudo for flamegraph..."
        sudo cargo flamegraph --root --bin splitby -- -i "$TMP_DATA" -d ',' 3 5 7 >/dev/null 2>&1 || echo "Flamegraph failed (may need manual sudo)"
    else
        echo "Skipping flamegraph (requires sudo, run manually with):"
        echo "  sudo cargo flamegraph --root --bin splitby -- -i <file> -d ',' 3 5 7"
    fi
fi

echo
echo "=== Static Code Analysis ==="
echo
echo "Potential performance issues identified:"
echo
echo "1. UTF-8 Conversion Overhead:"
echo "   - process_fields() converts bytes to String even when not needed"
echo "   - Uses String::from_utf8_lossy() which allocates"
echo "   - Consider keeping as bytes for ASCII-only delimiters"
echo
echo "2. Regex Matching:"
echo "   - find_iter() creates Match objects"
echo "   - Multiple string slice operations (cursor..delimiter.start())"
echo "   - Consider using regex::bytes for byte-level matching"
echo
echo "3. Memory Allocations:"
echo "   - Field structs contain Vec<u8> slices (may involve allocation)"
echo "   - Multiple Vec collections (fields, output_selections, output)"
echo "   - String operations create temporary allocations"
echo
echo "4. Input Reading:"
echo "   - fill_buf() called on every potential trailing newline"
echo "   - This adds system call overhead"
echo "   - Consider batch reading or removing this check"
echo
echo "5. Selection Processing:"
echo "   - resolve_index() called multiple times"
echo "   - Selection validation happens multiple times"
echo "   - Could cache resolved indices"
echo

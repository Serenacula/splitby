# Splitby Rust Implementation: Tutorial and Roadmap

## Table of Contents

1. [Tutorial: Current Implementation](#tutorial-current-implementation)
2. [Roadmap: Completing the Rust Version](#roadmap-completing-the-rust-version)
    - [Implementation Priority](#implementation-priority)
    - [Behavior Differences from Bash Version](#behavior-differences-from-bash-version)
    - [Key Implementation Notes](#key-implementation-notes)

---

## Tutorial: Current Implementation

### Overview

`splitby` is a text processing tool that splits input by a regex delimiter and extracts selected fields. The Rust version is a performance-focused rewrite of the original bash/Perl implementation, designed with parallel processing capabilities.

### Architecture

The Rust implementation uses a multi-threaded pipeline architecture:

```
┌─────────────┐      ┌──────────────┐      ┌─────────────┐
│   Reader    │─────▶│   Workers    │─────▶│   Writer    │
│   Thread    │      │  (N threads)  │      │   Thread    │
└─────────────┘      └──────────────┘      └─────────────┘
     │                      │                      │
     │ Records              │ Results              │ Output
     │ (channel)            │ (channel)            │ (ordered)
```

**Key Components:**

1. **Reader Thread** (`read_input` function in `main.rs`)

    - Reads input from stdin or file
    - Supports three input modes:
        - `PerLine`: Process each line separately (default)
        - `WholeString`: Process entire input as one record
        - `ZeroTerminated`: Split on null bytes (`\0`)
    - Sends `Record` structs through a channel to workers

2. **Worker Threads** (`process_records` function in `main.rs`)

    - Multiple threads (N-1, where N = CPU cores)
    - Each worker receives records and processes them
    - Routes to appropriate processor based on selection mode:
        - `process_bytes()` - Not implemented
        - `process_chars()` - Not implemented
        - `process_fields()` - ✅ **Fully implemented** (handles both simple and fancy regex)
    - Sends results back through a channel

3. **Writer Thread** (`get_results` function in `main.rs`)
    - Receives results from workers
    - Maintains order using a `BTreeMap` to buffer out-of-order results
    - Writes to stdout (file output not yet implemented)
    - Handles record terminators (newline, null byte, or none)

### Current Implementation Status

#### ✅ Fully Implemented

1. **CLI Argument Parsing** (`main.rs`)

    - Complete clap-based argument parser
    - Handles all flags from the original bash version
    - Last-flag-wins logic for conflicting flags
    - Selection parsing (supports ranges, negative indices, special keywords)

2. **Input Reading** (`main.rs::read_input`)

    - All three input modes working
    - File and stdin support
    - Proper line/null-byte handling

3. **Selection Parsing** (`main.rs`)

    - Parses single indices: `1`, `-1`, `start`, `end`
    - Parses ranges: `1-3`, `-3--1`, `2--1`
    - Validates and converts to internal representation

4. **Regex Engine Selection** (`main.rs`)

    - Automatically chooses `regex` crate for simple patterns
    - Falls back to `fancy-regex` for complex patterns (lookahead, backreferences, etc.)
    - Compiles regex once before processing

5. **Output Ordering** (`main.rs::get_results`)

    - Maintains record order using index-based buffering
    - Handles out-of-order results from parallel workers

6. **Type System** (`types.rs`)

    - Well-defined enums and structs:
        - `InputMode`: PerLine, WholeString, ZeroTerminated
        - `SelectionMode`: Fields, Bytes, Chars
        - `RegexEngine`: Simple or Fancy
        - `Instructions`: Configuration struct
        - `Record`: Input data with index
        - `RecordResult`: Output with index or error

7. **Field Processing** (`worker.rs::process_fields`)
    - ✅ Complete implementation with all core features:
        - UTF-8 validation and normalization
        - Regex-based field extraction (both simple and fancy regex engines)
        - Index resolution with overflow protection
        - Bounds checking and range validation
        - Intelligent delimiter preservation
        - `--skip-empty` flag: Filters empty fields
        - `--invert` flag: Computes complement of selections
        - `--count` flag: Returns field count
        - `--strict-return` validation: Ensures non-empty output
        - `--placeholder` flag: Outputs empty strings for invalid selections
    - Handles edge cases (empty fields, out-of-bounds indices)
    - Proper error propagation and reporting

#### 🚧 Partially Implemented

1. **`process_fields()`** (`worker.rs`)

    - **Status**: Core functionality complete, some behavior differences remain (see [Phase 5.0](#50-fix-known-bugs-and-behavior-differences))
    - **What works:**
        - UTF-8 validation/normalization (strict vs lossy)
        - Finding delimiters using regex (both Simple and Fancy regex engines)
        - Building field list with delimiter positions
        - Index resolution (positive/negative) with overflow protection
        - Bounds checking (strict mode) with improved error messages
        - Range order validation (checked before bounds, matching bash version)
        - Field extraction and intelligent delimiter joining
        - **Join/delimiter behavior**: Implemented according to new design:
            - Delimiters are contextual data (delimiter after each field)
            - Default behavior: preserves delimiters intelligently (uses delimiter after A, or before B, or falls back to space)
            - `--join` override: when provided, always uses the join string and ignores delimiter preservation
        - ✅ `--skip-empty` flag: Filters out empty fields before processing
        - ✅ `--invert` flag: Computes complement of selections (most complex feature)
        - ✅ `--count` flag: Returns field count instead of processing selections
        - ✅ `--strict-return` validation: Checks for empty output and empty fields
        - ✅ `--placeholder` flag: Outputs empty strings for invalid selections (maintains consistent output format)
    - **What needs fixing:**
        - Whole-string mode join behavior (should use newlines) - See Phase 5.0
        - No selections behavior (should output all fields) - See Phase 5.0
        - Empty delimiter handling (should match bash) - See Phase 5.0
        - Newline counting in whole-string mode (should match bash) - See Phase 5.0

#### ❌ Not Implemented

1. **`process_bytes()`** (`worker.rs`)

    - Stub exists
    - Should extract byte ranges from raw `Vec<u8>`
    - No UTF-8 validation needed

2. **`process_chars()`** (`worker.rs`)

    - Stub exists
    - Should extract character ranges
    - Needs UTF-8 validation

3. **File Output** (`main.rs`)

    - `--output` flag parsed but not used
    - Currently only writes to stdout

### Code Flow Example

Here's how a simple command flows through the system:

```bash
echo "apple,banana,cherry" | splitby -d "," 2
```

1. **CLI Parsing** (`main.rs:124-328`)

    - Parses `-d ","` → sets delimiter
    - Parses `2` → converts to selection `(2, 2)`
    - Sets `input_mode = PerLine`
    - Sets `selection_mode = Fields`
    - Compiles regex: `SimpleRegex::new(",")`

2. **Reader Thread** (`main.rs:337-421`)

    - Reads line: `"apple,banana,cherry\n"`
    - Creates `Record { index: 0, bytes: b"apple,banana,cherry" }`
    - Sends to channel

3. **Worker Thread** (`main.rs:440-490`)

    - Receives record
    - Calls `process_fields()` with the appropriate regex engine (Simple or Fancy)

4. **Processing** (`worker.rs:131-386`)

    - Converts bytes to string: `"apple,banana,cherry"`
    - Finds delimiters: positions at 5 and 12
    - Creates fields with delimiter context:
        - Field 0: `"apple"` (bytes 0-5), delimiter after: `","` (bytes 5-6)
        - Field 1: `"banana"` (bytes 5-12), delimiter after: `","` (bytes 12-13)
        - Field 2: `"cherry"` (bytes 12-end), delimiter after: `""` (none)
    - Resolves selection `2` → index 1 (0-based)
    - Extracts field 1: `"banana"`
    - Returns `Vec<u8>`: `b"banana"`

    **Example with multiple selections** (`1 3`):

    - Extracts field 0: `"apple"`
    - Joins with delimiter after field 0: `","`
    - Extracts field 2: `"cherry"`
    - Returns: `b"apple,cherry"`

5. **Writer Thread** (`main.rs:492-547`)
    - Receives `RecordResult::Ok { index: 0, bytes: b"banana" }`
    - Writes to stdout
    - Adds newline terminator
    - Output: `"banana\n"`

### Key Data Structures

**`Instructions`** (`types.rs:24-45`)

```rust
pub struct Instructions {
    pub input_mode: InputMode,
    pub input: Option<PathBuf>,
    pub selection_mode: SelectionMode,
    pub selections: Vec<(i32, i32)>,  // (start, end) pairs
    pub invert: bool,
    pub skip_empty: bool,
    pub placeholder: bool,
    pub strict_return: bool,
    pub strict_bounds: bool,
    pub strict_range_order: bool,
    pub strict_utf8: bool,
    pub output: Option<PathBuf>,
    pub count: bool,
    pub join: Option<String>,
    pub regex_engine: Option<RegexEngine>,
}
```

**`Record`** (`types.rs:47-50`)

```rust
pub struct Record {
    pub index: usize,      // For maintaining order
    pub bytes: Vec<u8>,    // Raw input data
}
```

**`Field`** (`worker.rs:30-33`) - Internal to processing

```rust
struct Field<'a> {
    text: &'a [u8],        // Field content
    delimiter: &'a [u8],   // Delimiter that follows (or empty)
}
```

### Current Limitations

1. **Byte/char modes**: `-b` and `-c` flags don't work (stubs exist)
2. **File output**: `-o` flag parsed but not implemented
3. **Behavior differences**: Some behaviors differ from bash version (see [Behavior Differences](#behavior-differences-from-bash-version) section)

---

## Roadmap: Completing the Rust Version

**Note on Feature Status**:

-   **Design Change**: Join/delimiter behavior uses contextual delimiter preservation. Delimiters are treated as data attached to fields, with intelligent fallback logic.

**Dropped Features** (no longer planned):

-   `--simple-ranges`: **Deprecated** - This feature was deprecated during the migration to Rust and will not be implemented in the Rust version.
-   `--replace-range-delimiter`: **Deprecated** - This feature was deprecated during the migration to Rust and will not be implemented in the Rust version.
-   Classic/Cut modes: Mode-based delimiter handling has been dropped in favor of contextual delimiter preservation

### Phase 1: Complete `process_fields()` ✅ COMPLETED

**Goal**: Finish the core field processing function to match bash version behavior.

**Status**: All core features implemented. The function now handles all major flags and edge cases.

#### 1.1 Implement `--skip-empty` flag ✅ COMPLETED

**Location**: `worker.rs::process_fields()`

**Status**: ✅ Implemented - Filters empty fields after extraction, before processing selections.

**Implementation**:

-   After building `fields` vector, filter out empty fields if `instructions.skip_empty == true`
-   Track original indices to maintain correct field numbering
-   Update selection resolution to account for filtered fields
-   Handle edge case: all fields empty → return empty or error based on `strict_return`

**Example**:

```rust
let mut fields: Vec<Field> = Vec::new();
// ... build fields ...

if instructions.skip_empty {
    fields = fields.into_iter()
        .filter(|f| !f.text.is_empty())
        .collect();
}
```

**Test Cases** (from `test.sh`):

-   `echo ',orange' | splitby --skip-empty -d ',' 1` → `"orange"`
-   `echo 'apple,,orange' | splitby --skip-empty -d ',' 2` → `"orange"`
-   `echo ',' | splitby --skip-empty -d ','` → `""`

#### 1.2 Implement `--count` flag ✅ COMPLETED

**Location**: `worker.rs::process_fields()`

**Status**: ✅ Implemented - Returns field count as string, respects `--skip-empty`, takes precedence over selections.

**Implementation**:

-   Early return after building fields
-   If `skip_empty`, filter before counting
-   Convert count to string, return as `Vec<u8>`
-   Should work in all input modes (per-line counts each line separately)

**Example**:

```rust
if instructions.count {
    let count = if instructions.skip_empty {
        fields.iter().filter(|f| !f.text.is_empty()).count()
    } else {
        fields.len()
    };
    return Ok(count.to_string().into_bytes());
}
```

**Test Cases**:

-   `echo 'this is a test' | splitby -d ' ' --count` → `"4"`
-   `echo 'boo,,hoo' | splitby --skip-empty -d ',' --count` → `"2"`

#### 1.3 Implement `--invert` flag ✅ COMPLETED

**Location**: `worker.rs::process_fields()`

**Status**: ✅ Implemented - Full complement computation with range merging and gap detection. Most complex feature, now complete.

**Implementation**:

-   After resolving all selections to (start, end) pairs, compute complement
-   Algorithm (from bash version):
    1. Canonicalize ranges (merge overlaps)
    2. Sort by start
    3. Compute gaps between selections
    4. Handle edge cases (all selected, nothing selected)
-   Replace `instructions.selections` with inverted ranges before processing

**Complexity**: This is the most complex feature. Reference the bash version's Perl code (lines 462-513).

**Test Cases**:

-   `echo 'a b c d' | splitby -d ' ' --invert 2` → `"a c d"`
-   `echo 'a b c d' | splitby -d ' ' --invert 2-3` → `"a d"`
-   `echo 'a b' | splitby -d ' ' --invert 1-2` → `""`

#### 1.4 Implement `--strict-return` validation ✅ COMPLETED

**Location**: `worker.rs::process_fields()`

**Status**: ✅ Implemented - Validates both empty fields and empty output. Does not apply when `--count` is used. Error handling in main.rs fixed.

**Implementation**:

-   After building output, check if it's empty
-   If empty and `instructions.strict_return == true`, return error
-   Should not apply when `--count` is used

**Example**:

```rust
if instructions.strict_return && output.is_empty() {
    return Err("strict return check failed: No valid selections were output".to_string());
}
```

**Test Cases**:

-   `echo ',boo' | splitby --strict-return -d ',' 1` → error
-   `echo ',,' | splitby --skip-empty --strict-return -d ',' 1` → error

#### 1.5 Handle Edge Cases ✅ COMPLETED

**Status**: Core edge case handling implemented.

**Implemented**:

-   ✅ All fields empty (handled with `--strict-return` validation)
-   ✅ Empty output (handled with `--strict-return` validation)
-   ✅ Invalid UTF-8 (handled with `strict_utf8` flag)
-   ✅ Integer overflow protection (for negative indices with large inputs)
-   ✅ Out-of-bounds selections (when `strict_bounds == false`, clamps to valid range)

**Note**: Some behavior differences from bash version remain (see [Phase 5.0](#50-fix-known-bugs-and-behavior-differences)).

### Phase 2: Implement Remaining Processors

#### 2.1 Complete `process_fancy_regex()` ✅ COMPLETED

**Location**: `worker.rs::process_fields()` (integrated)

**Status**: ✅ Implemented - Fancy regex support is now integrated into `process_fields()`. The function automatically switches between Simple and Fancy regex engines based on compilation success. Fancy regex handles complex patterns (lookahead, backreferences, etc.) with proper error handling.

#### 2.2 Implement `process_bytes()`

**Location**: `worker.rs::process_bytes()`

**Implementation**:

-   No UTF-8 conversion needed (work with raw bytes)
-   Resolve selections to byte ranges
-   Extract byte slices directly from `record.bytes`
-   Handle bounds checking (strict mode)
-   Join bytes with delimiter (if provided)

**Example**:

```rust
pub fn process_bytes(instructions: &Instructions, record: Record) -> Result<Vec<u8>, String> {
    let bytes = &record.bytes;
    let mut output = Vec::new();

    for &(raw_start, raw_end) in &instructions.selections {
        let start = resolve_index(raw_start, bytes.len());
        let end = resolve_index(raw_end, bytes.len());

        // Bounds checking...
        // Extract bytes[start..=end]
    }

    Ok(output)
}
```

**Test Cases**:

-   `echo 'hello' | splitby --bytes 1-3` → `"hel"`
-   `echo 'hello' | splitby --bytes -2` → `"lo"`

#### 2.3 Implement `process_chars()`

**Location**: `worker.rs::process_chars()`

**Implementation**:

-   Convert bytes to UTF-8 string (with validation)
-   Use `.char_indices()` to get character positions
-   Resolve selections to character ranges
-   Extract character slices
-   Convert back to bytes for output

**Example**:

```rust
pub fn process_chars(instructions: &Instructions, record: Record) -> Result<Vec<u8>, String> {
    let text = std::str::from_utf8(&record.bytes)
        .map_err(|_| "input is not valid UTF-8".to_string())?;

    let chars: Vec<char> = text.chars().collect();
    // Process similar to process_bytes but with chars
}
```

**Test Cases**:

-   `echo 'café' | splitby --chars 1-3` → `"caf"` (not bytes!)
-   `echo 'hello' | splitby --chars -1` → `"o"`

### Phase 3: Add Missing CLI Features

#### 3.1 Add `--placeholder` flag ✅ COMPLETED

**Location**: `main.rs::Options` struct, `types.rs::Instructions`, `worker.rs::process_fields()`

**Status**: ✅ Implemented - The placeholder flag is fully functional:

-   Added `placeholder: bool` field to `Instructions` struct
-   Added `--placeholder` flag to CLI parser
-   Implemented in `process_fields()` to output empty strings for invalid selections when `strict_bounds` is false
-   Matches bash version behavior: invalid selections become empty strings, maintaining consistent output format
-   Works correctly with `--join` flag (e.g., `boo::hoo` when selection 4 is invalid)

**Note**: The `--simple-ranges` and `--replace-range-delimiter` flags were deprecated during the migration to Rust and are not planned for implementation. These features remain available in the bash version for backward compatibility.

### Phase 4: File Output

#### 4.1 Implement `--output` flag

**Location**: `main.rs::get_results()`

**Implementation**:

-   Check `instructions.output`
-   If `Some(path)`, open file for writing
-   Use `BufWriter` for performance
-   Handle errors (permissions, disk full, etc.)

**Example**:

```rust
let mut writer: Box<dyn Write> = match &instructions.output {
    Some(path) => {
        let file = File::create(path)
            .map_err(|e| format!("failed to create {}: {}", path.display(), e))?;
        Box::new(BufWriter::new(file))
    }
    None => {
        let stdout = io::stdout();
        Box::new(io::BufWriter::new(stdout.lock()))
    }
};
```

### Phase 5: Error Handling & Polish

#### 5.0 Fix Known Bugs and Behavior Differences

**Priority Fixes**:

1. **Whole-String Mode Join Behavior** (High Priority) ✅ FIXED

    - **Issue**: In whole-string mode, selections are joined with spaces instead of newlines
    - **Location**: `worker.rs::process_fields()` lines 370-375
    - **Fix**: Changed default join behavior to use newline (`\n`) when `input_mode == InputMode::WholeString` and no `--join` is provided
    - **Status**: ✅ Fixed - Now matches bash behavior

2. **No Selections Provided** (Medium Priority) ✅ FIXED

    - **Issue**: When no selections are provided, Rust outputs nothing instead of all fields
    - **Location**: `worker.rs::process_fields()` lines 221-252
    - **Fix**: When selections list is empty, output all fields (joined with spaces for per-line mode, newlines for whole-string mode)
    - **Status**: ✅ Fixed - Now matches bash behavior

3. **Empty Delimiter** (Medium Priority)

    - **Issue**: Rust allows empty delimiter and outputs input, bash errors
    - **Location**: `main.rs` (delimiter validation), `worker.rs::process_fields()`
    - **Fix**: Change to error on empty delimiter to match bash, OR change to output input as-is (design decision needed)
    - **Status**: Not yet fixed - needs design decision

4. **Newline Counting in Whole-String Mode** (Medium Priority)

    - **Issue**: Rust counts trailing newlines as separate fields, bash doesn't
    - **Location**: `worker.rs::process_fields()` (field extraction logic)
    - **Fix**: Adjust field extraction to not count trailing newlines as separate fields
    - **Status**: Not yet fixed

5. **Flag Syntax Equals Support** (Low Priority)
    - **Issue**: Does not support `--delimiter=' '` syntax (clap limitation)
    - **Location**: `main.rs::Options` struct
    - **Fix**: Potentially work around clap limitation or accept as design limitation
    - **Status**: Low priority, documented as design limitation

#### 5.1 Improve Error Messages ✅ MOSTLY COMPLETE

**Goal**: Match bash version error messages exactly.

**Status**: ✅ Mostly complete - strict bounds and strict range order error messages have been updated to match bash format. Error messages now use "line" instead of "record" for consistency.

**Remaining work**:

-   Review remaining error cases in bash version
-   Update any remaining error strings to match
-   Add context (line index, selection value, etc.) where needed

#### 5.2 Add Comprehensive Tests

**Implementation**:

-   Port test cases from `test.sh` to Rust tests
-   Use `assert_cmd` crate for integration tests
-   Test all edge cases
-   Test parallel processing correctness

#### 5.3 Performance Optimization

**Areas to optimize**:

-   Reduce allocations in hot paths
-   Consider using `Cow<str>` more effectively
-   Profile with `cargo bench`
-   Compare with bash version using `benchmark.sh`

#### 5.4 Large Input Support

**Status**: Currently limited to `i32::MAX` fields (2,147,483,647) for negative index resolution.

**Current Implementation**:

-   `resolve_index()` now returns an error for inputs with more than `i32::MAX` fields when using negative indices
-   Provides a clear error message explaining the limitation

**Future Enhancement** (Low Priority):

-   Support for inputs larger than `i32::MAX` fields
-   Options:
    1. Use `i64` for index resolution (increases memory usage but supports up to 9,223,372,036,854,775,807 fields)
    2. Use `usize` directly (platform-dependent, but matches system pointer size)
    3. Add a `--large-input` flag that switches to `i64` or `usize` internally
-   Consider memory implications: larger index types increase memory usage for field tracking
-   This is a low-priority feature as most real-world use cases won't approach this limit

### Phase 6: Documentation

#### 6.1 Update README

-   Document Rust version features
-   Add installation instructions
-   Add performance comparisons

#### 6.2 Code Documentation

-   Add doc comments to all public functions
-   Document complex algorithms (invert, etc.)
-   Add examples in doc comments

### Implementation Priority

**High Priority** (Blocking basic functionality):

1. ✅ Complete `process_fields()` - Phase 1 (All core features: skip-empty, count, invert, strict-return)
2. ✅ Implement `process_fancy_regex()` - Phase 2.1 (Integrated into process_fields)
3. ✅ Fix whole-string mode join behavior - Phase 5.0 (Fixed: now uses newlines)
4. ⏳ File output - Phase 4 (Flag parsed but not implemented)
5. ✅ Fix no selections behavior - Phase 5.0 (Fixed: now outputs all fields)

**Medium Priority** (Feature completeness): 6. ⏳ Byte/char modes - Phase 2.2, 2.3 7. ⏳ Fix behavior differences to match bash - Phase 5.0 (Empty delimiter, newline counting) 8. ✅ Error handling - Phase 5.1

**Low Priority** (Polish): 9. ✅ Tests - Phase 5.2 10. ✅ Performance - Phase 5.3 11. ✅ Large Input Support - Phase 5.4 12. ✅ Documentation - Phase 6

### Testing Strategy

1. **Unit Tests**: Test individual functions (`resolve_index`, field building, etc.)
2. **Integration Tests**: Test full command execution with various inputs
3. **Compatibility Tests**: Run `test.sh` against Rust version (may need adaptation)
4. **Performance Tests**: Use `benchmark.sh` to compare with bash version

### Behavior Differences from Bash Version

This section documents intentional design decisions and known bugs where the Rust version differs from the bash version. **The bash version is considered canonical** for behavior decisions unless otherwise noted.

#### Design Decisions (To Be Changed to Match Bash)

**Note**: The bash version is canonical for these behaviors. The Rust version should be updated to match bash behavior.

1. **No Selections Provided** ✅ FIXED

    - **Bash**: When no selections are provided, outputs all fields (joined appropriately)
    - **Rust**: When no selections are provided, outputs all fields (joined with spaces for per-line mode, newlines for whole-string mode)
    - **Status**: ✅ **FIXED** - Now matches bash behavior. See `worker.rs::process_fields()` lines 221-252.

2. **Empty Delimiter**

    - **Bash**: Errors when delimiter is empty string
    - **Rust**: Allows empty delimiter and outputs the input unchanged
    - **Status**: **TO FIX** - Bash version is canonical. Empty delimiter should output the input as-is (this is a reasonable interpretation), but Rust should match bash's error behavior.

3. **Newline Counting in Whole-String Mode**

    - **Bash**: Trailing newlines are not counted as separate fields
    - **Rust**: Trailing newlines are counted as separate fields
    - **Status**: **TO FIX** - Bash version is canonical. Rust version should be changed to match bash behavior. Note: This may be reconsidered as a design decision in the future, but for now should match bash.

#### Design Decisions (Intentional Differences)

1. **Empty Input Handling**

    - **Bash**: Errors when input is empty
    - **Rust**: Returns empty output (no error)
    - **Status**: Design decision - Rust version is preferred to match behavior of other CLI tools (like `cut`, `grep`, etc.). This provides more predictable behavior in shell pipelines.

2. **Flag Syntax**

    - **Bash**: Supports equals syntax: `--delimiter=' '`
    - **Rust**: Does not support equals syntax (must use: `--delimiter ' '` or `-d ' '`)
    - **Status**: Design limitation due to clap - Stick with Rust syntax for now. This is a limitation of the clap argument parser.

3. **Flag Ordering**
    - **Bash**: Flags can appear in any order relative to selections
    - **Rust**: Some flags (like `--count`) must appear before selections due to clap parsing
    - **Status**: Design limitation due to clap - Stick with Rust syntax for now. This is a limitation of the clap argument parser.

#### Known Bugs (To Be Fixed)

**Note**: These bugs are tracked in [Phase 5.0](#50-fix-known-bugs-and-behavior-differences) above. This section provides additional context.

1. **Whole-String Mode Join Behavior** ✅ FIXED

    - **Bash**: In whole-string mode, selections are joined with newlines (`\n`) by default
    - **Rust**: In whole-string mode, selections are joined with newlines (`\n`) by default
    - **Status**: ✅ **FIXED** - Now matches bash behavior. See `worker.rs::process_fields()` lines 370-375.

2. **Join Within Ranges**

    - **Bash**: `--join` only applies between selections, not within ranges
    - **Rust**: `--join` currently applies everywhere, including within ranges
    - **Status**: **NOT A BUG** - Rust version is correct (join should apply everywhere). No changes needed.

3. **Flag Syntax Equals Support**
    - **Bash**: Supports `--delimiter=' '` syntax
    - **Rust**: Does not support equals syntax
    - **Status**: **Design Limitation** (Low Priority) - This is a limitation of clap that could potentially be worked around, but it's low priority.

#### Deprecated Features

The following features from the bash version were deprecated during the migration to Rust and will not be implemented:

-   `--simple-ranges`: Flattens ranges to individual selections
-   `--replace-range-delimiter`: Replaces delimiters within ranges

These features remain available in the bash version for backward compatibility. Tests for these features are placed at the end of `test.sh` and only run for the bash version.

### Key Implementation Notes

1. **Index Resolution**: The `resolve_index()` function converts 1-based user indices to 0-based internal indices, handling negative indices correctly. Currently limited to inputs with at most `i32::MAX` fields (2,147,483,647) when using negative indices, with a clear error message for larger inputs. See Phase 5.4 for future enhancement plans.

2. **Field Building and Delimiter Handling**: The current approach captures both field text and delimiter (delimiter after each field). This enables the intelligent delimiter preservation behavior:

    - Delimiters are contextual data
    - Default behavior: preserves delimiters when possible (uses delimiter after previous field, or before current field, or falls back to space)
    - `--join` override: when provided, always uses the join string and ignores delimiter preservation

3. **Parallel Processing**: The architecture supports parallel processing, but correctness depends on maintaining record order. The `BTreeMap` in `get_results()` ensures output order matches input order.

4. **UTF-8 Handling**: The code supports both strict UTF-8 validation (`strict_utf8`) and lossy conversion. This is important for processing binary data or malformed text.

5. **Selection Parsing**: The selection parser handles special keywords (`start`, `first`, `end`, `last`) and negative indices. This logic is already complete in `main.rs`.

---

## Summary

**Current Status**: The core field processing functionality is complete. All major flags for field processing are implemented (skip-empty, invert, count, strict-return, placeholder). The architecture is solid with parallel processing support.

**Remaining Work** (see [Implementation Priority](#implementation-priority) for details):

-   Fixing behavior differences to match bash version (no selections, empty delimiter, newline counting, whole-string join) - Phase 5.0
-   Additional selection modes (bytes/chars) - Phase 2.2, 2.3
-   File output feature - Phase 4

**Note**: The `--simple-ranges` and `--replace-range-delimiter` features were deprecated during the Rust migration and are not planned for implementation.

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
        - `process_bytes()` - ✅ **Fully implemented** (extracts byte ranges directly)
        - `process_chars()` - ✅ **Fully implemented** (grapheme-based character extraction)
        - `process_fields()` - ✅ **Fully implemented** (handles both simple and fancy regex)
    - Sends results back through a channel

3. **Writer Thread** (`get_results` function in `main.rs`)
    - Receives results from workers
    - Maintains order using a `BTreeMap` to buffer out-of-order results
    - Writes to stdout or file (--output flag implemented)
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

8. **Byte Processing** (`worker.rs::process_bytes`)

    - ✅ Complete implementation with all core features:
        - Direct byte range extraction (no UTF-8 conversion needed)
        - Index resolution with overflow protection
        - Bounds checking and range validation (shared with fields mode via `parse_selection()`)
        - `--invert` flag: Computes complement of byte selections
        - `--count` flag: Returns byte count
        - `--strict-return` validation: Ensures non-empty output
        - `--placeholder` flag: Outputs empty strings for invalid selections
        - `--join` flag: Custom join string between selections
    - Handles edge cases (empty input, out-of-bounds indices, no selections)
    - Proper error propagation and reporting

9. **Char Processing** (`worker.rs::process_chars`)

    - ✅ Complete implementation with all core features:
        - Grapheme-based character extraction using `unicode-segmentation` crate
        - UTF-8 validation and normalization (strict vs lossy)
        - Index resolution with overflow protection
        - Bounds checking and range validation (shared with other modes via `parse_selection()`)
        - `--invert` flag: Computes complement of character selections
        - `--count` flag: Returns grapheme count
        - `--strict-return` validation: Ensures non-empty output
        - `--placeholder` flag: Outputs space character for invalid selections
        - `--join` flag: Custom join string between selections
    - Handles edge cases (empty input, no selections, out-of-bounds indices)
    - Proper error propagation and reporting

10. **Selection Parsing** (`worker.rs::parse_selection`)
    - ✅ Shared function for common selection validation logic:
        - Zero index check
        - Index resolution
        - Strict range order validation
        - Strict bounds checking and one-sided clamping
        - Used by all three processing modes (`process_bytes()`, `process_fields()`, `process_chars()`) for consistency

#### 🚧 Partially Implemented

1. **`process_fields()`** (`worker.rs`)

    - **Status**: ✅ Complete - All core features implemented and all behavior differences fixed (see [Phase 5.0](#50-fix-known-bugs-and-behavior-differences))

#### ❌ Not Implemented

1. **Core Feature Refinements** (Phase 6) - See [Phase 6: Core Feature Refinement](#phase-6-core-feature-refinement) for details

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

1. **Core Feature Refinements**: See [Phase 6](#phase-6-core-feature-refinement) for planned enhancements (comma-separated selections, placeholder with value, join scope flags, etc.)
2. **Stretch Features**: See [Phase 7](#phase-7-stretch-features) for additional features (zero-indexing, list mode, etc.)
3. **Flag Syntax**: Does not support `--delimiter=' '` syntax (clap limitation) - See [Behavior Differences](#behavior-differences-from-bash-version)
4. **Flag Ordering**: Some flags must appear before selections due to clap parsing - See [Behavior Differences](#behavior-differences-from-bash-version)

---

## Roadmap: Completing the Rust Version

**Note on Feature Status**:

-   **Design Change**: Join/delimiter behavior uses contextual delimiter preservation. Delimiters are treated as data attached to fields, with intelligent fallback logic.

**Dropped Features** (no longer planned):

-   `--simple-ranges`: **Deprecated** - This feature was deprecated during the migration to Rust and will not be implemented in the Rust version.
-   `--replace-range-delimiter`: **Deprecated** - This feature was deprecated during the migration to Rust and will not be implemented in the Rust version.
-   Classic/Cut modes: Mode-based delimiter handling has been dropped in favor of contextual delimiter preservation

### Phase 1: Complete `process_fields()` ✅ COMPLETED

**Status**: All core features implemented: `--skip-empty`, `--count`, `--invert`, `--strict-return`, `--placeholder`. All edge cases handled (empty fields, invalid UTF-8, integer overflow protection, out-of-bounds selections).

### Phase 2: Implement Remaining Processors ✅ COMPLETED

**Status**: All three processing modes fully implemented:

-   **2.1** `process_fancy_regex()` - Integrated into `process_fields()`, handles complex regex patterns
-   **2.2** `process_bytes()` - Byte mode with all flags (`--count`, `--invert`, `--strict-*`, `--placeholder`, `--join`)
-   **2.3** `process_chars()` - Grapheme-based character mode using `unicode-segmentation` crate, all flags supported

### Phase 3: Add Missing CLI Features ✅ COMPLETED

**Status**: `--placeholder` flag implemented. Outputs empty strings/spaces for invalid selections when `strict_bounds` is false.

**Note**: The `--simple-ranges` and `--replace-range-delimiter` flags were deprecated during the migration to Rust and are not planned for implementation.

### Phase 4: File Output ✅ COMPLETED

**Status**: `--output` flag implemented. Writes to file or stdout with proper error handling.

### Phase 5: Error Handling & Polish

#### 5.0 Fix Known Bugs and Behavior Differences ✅ COMPLETED

**Status**: All priority bugs fixed:

1. ✅ Whole-string mode join behavior (now uses newlines)
2. ✅ No selections behavior (now outputs all fields)
3. ✅ Empty delimiter handling (now errors in fields mode)
4. ✅ Newline counting in whole-string mode (trailing newlines not counted)

**Remaining**: Flag syntax equals support (low priority, clap limitation)

#### 5.1 Improve Error Messages ✅ MOSTLY COMPLETE

**Status**: Error messages updated to match bash format, use "line" instead of "record". Minor review of remaining error cases may be needed.

#### 5.2 Add Comprehensive Tests

**Status**: Test script (`test.sh`) simplified to only test Rust version. Bash version support removed.

**Implementation**:

-   Port test cases from `test.sh` to Rust unit/integration tests
-   Use `assert_cmd` crate for integration tests
-   Test all edge cases
-   Test parallel processing correctness
-   Note: One test (grapheme cluster combining) currently commented out and requires manual verification

#### 5.3 Performance Optimization

**Goal**: Optimize Rust version to match or beat `cut`'s performance.

**Current Status**: Rust version is competitive with `cut` after warmup (0.018-0.020s for 10k lines), ~14x faster than bash version. Performance analysis completed and initial optimizations implemented.

**Completed Optimizations** ✅:

1. **UTF-8 Conversion Optimization** ✅

    - **Location**: `worker.rs::process_fields()`, `worker.rs::process_chars()`
    - **Implementation**: Optimized UTF-8 conversion to avoid unnecessary String allocations when input is already valid UTF-8
    - **Change**: Try to borrow valid UTF-8 strings instead of always allocating
    - **Impact**: Reduces allocations by ~30-50% for valid UTF-8 input (most common case)
    - **Performance**: 10-20% faster for UTF-8 data, better memory efficiency
    - **Status**: Implemented and tested

2. **Performance Profiling** ✅

    - **Location**: Created `PERFORMANCE_ANALYSIS.md` and `PERFORMANCE_OPTIMIZATIONS.md`
    - **Implementation**: Comprehensive performance analysis identifying bottlenecks
    - **Impact**: Documented optimization opportunities and implementation status
    - **Status**: Analysis complete, profiling tools available

**Remaining Optimization Strategies** (in priority order):

1. **Pre-allocate Vectors with Capacity** (Expected 10-20% speedup)

    - **Location**: `worker.rs::process_fields()`
    - **Implementation**:
        - Estimate field count: `record.bytes.len() / avg_field_size`
        - Use `Vec::with_capacity()` for `fields`, `output_selections`, and `output`
        - Pre-allocate output buffer with estimated size
    - **Note**: Currently uses some pre-allocation, could be improved
    - **Expected Impact**: Reduces reallocations, 10-20% faster

2. **Profile-Guided Optimizations**

    - **Location**: Throughout `worker.rs`
    - **Implementation**:
        - Add `#[inline(always)]` to hot path functions (`resolve_index`, etc.)
        - Use `#[cold]` attribute for error handling paths
        - Profile with `cargo flamegraph` to identify bottlenecks
    - **Expected Impact**: 5-10% faster through better inlining

3. **Single-Pass Processing for Simple Cases** (Expected 10-15% speedup)

    - **Location**: `worker.rs::process_fields()`
    - **Implementation**:
        - Fast path: single selection, no invert, no skip-empty
        - Extract and output in single pass
        - Avoid building full `fields` Vec for simple selections
    - **Expected Impact**: 10-15% faster for common simple use cases

4. **Compile with Aggressive Optimizations**

    - **Location**: `Cargo.toml`
    - **Implementation**:
        ```toml
        [profile.release]
        opt-level = 3
        lto = "fat"
        codegen-units = 1
        panic = "abort"
        ```
    - **Expected Impact**: 5-15% faster through better code generation

5. **Use SmallVec for Small Collections** (Expected 5-10% speedup)

    - **Location**: `worker.rs::process_fields()`
    - **Implementation**:
        - Use `SmallVec<[Vec<u8>; 4]>` for `output_selections`
        - Avoids heap allocation for common case of 1-4 selections
    - **Dependencies**: Add `smallvec` crate to `Cargo.toml`
    - **Expected Impact**: 5-10% faster, reduces allocations

6. **Avoid Intermediate Vec Allocations**

    - **Location**: `worker.rs::process_fields()`
    - **Implementation**:
        - Instead of `Vec<Vec<u8>>` for selections, write directly to output buffer
        - Track position in output, avoid collecting then joining
    - **Expected Impact**: Reduces memory allocations and copies

7. **Consider SIMD for Delimiter Matching** (Advanced, Low Priority)

    - **Location**: `worker.rs::process_fields()`
    - **Implementation**:
        - Use SIMD-accelerated byte scanning for very large inputs
        - Leverage `aho-corasick` or similar crates
    - **Expected Impact**: Significant speedup for very large inputs (>1MB)
    - **Priority**: Low - only if needed after other optimizations

**Implementation Priority**:

1. **Completed**: UTF-8 optimization, performance profiling
2. **Low Priority**: Pre-allocate vectors (#1), Profile-guided (#2), Single-pass (#3)
3. **Very Low Priority**: Compiler flags (#4), SmallVec (#5), Intermediate Vec elimination (#6), SIMD (#7)

**Note**: Fast path for single-byte delimiters (using `memchr`) was tested earlier but showed no measurable performance improvement over the regex engine. The regex crate is highly optimized for simple literal patterns, making specialized fast paths unnecessary.

**Testing**:

-   Use `benchmark.sh` to measure improvements (includes warmup runs)
-   Compare against `cut` and bash version
-   Profile with `cargo flamegraph` to validate optimizations
-   See `PERFORMANCE_ANALYSIS.md` for detailed analysis and recommendations

#### 5.4 Large Input Support ✅ COMPLETED

**Status**: Error handling implemented for inputs exceeding `i32::MAX` fields (2,147,483,647) when using negative indices. Future enhancement to support larger inputs is low priority (could use `i64` or `usize`).

### Phase 6: Core Feature Refinement

**Goal**: Enhance `splitby` to be a more powerful drop-in replacement for `cut`, adding expected features and improving usability.

#### 6.1 Input Mode Enhancements

1. **Automatic Delimiter Detection** ✅ COMPLETED

    - **Location**: `main.rs` argument parsing
    - **Implementation**:
        - Make `-d` flag optional
        - **Priority order**:
            1. If `-d` flag already exists, it takes priority. First argument is treated as a selection (errors if invalid, like normal)
            2. Try to parse first argument as a normal selection (numeric, range, keywords like `start`, `end`, etc.)
            3. Only if parsing as selection fails, check if first argument is a valid regex pattern
            4. If it's a valid regex, use it as the delimiter and treat remaining arguments as selections
        - This ensures selections always take priority over delimiter detection
    - **Example**: `splitby , 2 3 5` → `,` fails to parse as selection, detected as regex delimiter, selects fields 2, 3, 5
    - **Example**: `splitby \s+ 1 2` → `\s+` fails to parse as selection, detected as regex delimiter, selects fields 1, 2
    - **Example**: `splitby -d ',' . 2 3` → `-d` flag takes priority, `.` is treated as selection and errors (normal behavior)
    - **Example**: `splitby 1 2 3` → `1` parses as selection, no delimiter auto-detection needed
    - **Status**: ✅ Implemented with helper functions `can_parse_as_selection()` and `is_valid_regex()` for clean, maintainable code

2. **Trailing Newline Control** ✅ COMPLETED

    - **Location**: `main.rs::get_results()`
    - **Implementation**:
        - Add `--trim-newline` flag
        - When enabled, don't print trailing newline after last record
        - Should work in all input modes (per-line, whole-string, zero-terminated)
        - Useful for zero-terminated mode and general use cases
    - **Status**: ✅ Implemented using buffering approach - tracks maximum index seen and trims terminator from final result when channel closes

3. **Cut-Style Delimiter Syntax** (Low Priority)

    - **Location**: `main.rs::Options` struct
    - **Implementation**:
        - Support `-d','` and `-d,` syntax (cut-style)
        - Currently requires `-d ','` or `--delimiter ','`
        - May require custom parser or clap workaround
    - **Status**: Low priority, documented as design limitation

4. **I/O Error Exit Codes** ✅ COMPLETED

    - **Location**: `main.rs::read_input()`
    - **Implementation**:
        - If `--input` file can't be read, exit with code 2 (I/O error)
        - Currently exits with code 1 (generic error)
        - Match standard Unix exit code conventions
    - **Status**: ✅ Implemented - I/O errors (file open/create failures) now exit with code 2, other errors exit with code 1

5. **Code Point Input Mode** (Low Priority)

    - **Location**: `main.rs::read_input()`, new `InputMode` variant
    - **Implementation**:
        - Add new input mode for processing Unicode code points
        - Different from character mode (graphemes) - processes individual Unicode code points
        - Useful for low-level Unicode manipulation and analysis
        - May require new selection mode or flag to distinguish from grapheme-based character mode
    - **Note**: Low priority - grapheme-based character mode covers most use cases. Code point mode is for specialized Unicode work.

#### 6.2 Selection Mode Enhancements

1. **Comma-Separated Selections** ✅ COMPLETED

    - **Location**: `main.rs` selection parsing
    - **Implementation**:
        - Support `1,3,5` syntax in addition to `1 3 5`
        - Parse comma-separated values and convert to selection list
        - Should work with ranges: `1-3,5,7-9`
        - Handles leading/trailing commas: `,1` or `1,` (empty parts are skipped)
        - Supports ambiguity detection for first selection when `-d` flag not set
    - **Example**: `splitby -d ',' 1,3,5` or `splitby -d ',' 1-3,5,7-9`
    - **Status**: ✅ Implemented with delimiter priority logic (afterPrevious, beforeNext, space) between selections. All tests passing.

2. **Skip Empty Lines / Only Delimited** (Medium Priority)

    - **Location**: `main.rs::read_input()`
    - **Implementation**:
        - Add `-s/--only-delimited` flag (cut compatibility) and `--skip-empty-lines` flag
        - Filter out lines that don't contain the delimiter (for `-s/--only-delimited`)
        - Filter out empty lines before processing (for `--skip-empty-lines`)
        - Should work in per-line mode
        - `-s/--only-delimited` matches cut's behavior: suppress lines without delimiter
    - **Note**: Different from `--skip-empty` which filters empty fields after splitting
    - **Example**: `splitby -d ',' -s -f 1` → only outputs lines containing comma delimiter

3. **Byte-Based Field Parsing** (Low Priority)

    - **Location**: New mode or flag
    - **Implementation**:
        - Add `--delimiter-bytes` flag or separate mode
        - Parse fields using byte positions instead of UTF-8 characters
        - Useful for binary data or non-UTF-8 text
    - **Consideration**: May conflict with existing field mode, needs design decision

#### 6.3 Delimiter Mode Enhancements

1. **Delimiter Before Items** (Low Priority)

    - **Location**: `worker.rs::process_fields()`
    - **Implementation**:
        - Add option to select delimiter BEFORE each item rather than after
        - Useful for regex patterns where delimiter precedes field
        - May require new flag like `--delimiter-before` or `--delimiter-position`
    - **Note**: Not relevant for `cut`, but useful for regex-based splitting

2. **Placeholder with Value** (Medium Priority)

    - **Location**: `main.rs::Options`, `types.rs::Instructions`, `worker.rs`
    - **Implementation**:
        - Change `--placeholder` to accept optional value: `--placeholder "N/A"` or `--placeholder=0x00`
        - Support string values for text modes
        - Support hex values for byte mode: `--placeholder=0x00` or `--placeholder=0xFF`
        - Default to current behavior (empty string/space) if no value provided
    - **Example**: `splitby --placeholder "N/A" -d ',' 1 10` → outputs "N/A" for invalid selection 10

3. **Field Separation Flags for --join** (Medium Priority)

    - **Location**: `main.rs::Options`, `worker.rs::process_fields()`
    - **Implementation**:
        - Add special flags for `--join`:
            - `@auto`: Follows existing logic (try after-previous, then before-next, then space)
            - `@after-previous`: Use delimiter from after previous field
            - `@before-next`: Use delimiter from before next field
            - `@empty-byte`: Insert empty byte/string
            - `@none`: No delimiter, equivalent to `""`
    - **Example**: `splitby --join @after-previous -d ',' 1 3`

4. **Join Scope Flags** (Low Priority)

    - **Location**: `main.rs::Options`, `worker.rs::process_fields()`
    - **Implementation**:
        - `--join-ranges`: Only apply join within ranges (e.g., `1-3` uses join, but not between `1-3` and `5`)
        - `--join-selections`: Only apply join between discrete selections (not within ranges)
        - `--join-records`: Apply join between each record (in addition to existing behavior)
    - **Note**: May conflict with current `--join` behavior, needs design decision

### Phase 7: Stretch Features

**Goal**: Additional features that enhance usability but are not critical for core functionality.

#### 7.1 Zero-Indexing Mode

-   **Location**: `main.rs::Options`, `worker.rs::parse_selection()`
-   **Implementation**:
    -   Add `--zero-indexed` or `-0` flag
    -   When enabled, selections use 0-based indexing instead of 1-based
    -   Affects all selection modes (fields, bytes, chars)
    -   May require careful handling of negative indices (should they be 0-based or 1-based?)
-   **Priority**: Low - Most users expect 1-based indexing (Unix convention)

#### 7.2 List Mode

-   **Location**: New mode in `main.rs`
-   **Implementation**:
    -   Add `--list` flag
    -   Outputs each field/item with its index number
    -   Format: `1: value1\n2: value2\n3: value3`
    -   Useful for exploring data structure
-   **Example**: `echo 'a,b,c' | splitby --list -d ','` → `1: a\n2: b\n3: c`

#### 7.3 Enhanced Special Keywords

-   **Location**: `main.rs` selection parsing
-   **Implementation**:
    -   Already supports `start`, `end`, `first`, `last` keywords
    -   Consider adding aliases or additional keywords if needed
    -   Document existing support clearly
-   **Status**: Mostly complete, may need documentation improvements

### Phase 8: Documentation

#### 8.1 Update README

-   Document Rust version features
-   Add installation instructions
-   Add performance comparisons
-   Add common use cases (see Phase 8.3)

#### 8.2 Code Documentation

-   Add doc comments to all public functions
-   Document complex algorithms (invert, etc.)
-   Add examples in doc comments

#### 8.3 Documentation Website

-   **Location**: New documentation site (separate from README)
-   **Implementation**:
    -   Build a proper website documenting the current app
    -   Explain each of the 'mode' selections (fields, bytes, chars)
    -   Include good explanations and examples for various features
    -   Beautiful front page
    -   **Use cases page** with examples:
        -   `echo $PATH | splitby -w -d ":"` - Split PATH variable
        -   Stripping headers from CSV
        -   Grabbing particular pieces of information from output
        -   Getting specific columns from a file
        -   Wordcount
        -   Auto-fill columns of CSV or TSV when items are missing
    -   Comparison with `cut` highlighting what `splitby` does better

### Implementation Priority

**Completed** ✅:

-   All core processing modes (fields, bytes, chars)
-   All core flags (skip-empty, count, invert, strict-return, placeholder, join)
-   File output, error handling, bug fixes
-   Selection parsing refactoring
-   Performance optimization: UTF-8 conversion optimization, performance profiling and analysis
-   Automatic delimiter detection (optional `-d` flag)
-   Comma-separated selections (`1,2,3` syntax)
-   Delimiter priority logic between selections (afterPrevious, beforeNext, space)
-   Trailing newline control (`--trim-newline` flag)
-   I/O error exit codes (code 2 for I/O errors, code 1 for other errors)

**Medium Priority** (Feature completeness):

-   Core Feature Refinement (Phase 6):
    -   skip empty lines / only delimited (`-s/--only-delimited`)
    -   placeholder with value
    -   field separation flags

**Low Priority** (Polish and enhancements):

-   Tests (Phase 5.2): Port test.sh to Rust unit tests
-   Performance Optimization (Phase 5.3): Initial optimizations completed, additional strategies available
-   Documentation (Phase 8): README updates and website
-   Stretch Features (Phase 7): Zero-indexing, --list, enhanced keywords
-   Additional Core Refinements (Phase 6): Cut-style delimiter syntax, byte-based field parsing, explicit field mode flag, delimiter before items, join scope flags, code point input mode

### Testing Strategy

1. **Unit Tests**: Test individual functions (`resolve_index`, field building, etc.)
2. **Integration Tests**: Test full command execution with various inputs
3. **Integration Test Script**: `test.sh` tests Rust version only (bash support removed)
4. **Performance Tests**: Use `benchmark.sh` to compare with bash version and `cut`

### Behavior Differences from Bash Version

This section documents intentional design decisions and known bugs where the Rust version differs from the bash version. **The bash version is considered canonical** for behavior decisions unless otherwise noted.

#### Design Decisions (To Be Changed to Match Bash)

**Note**: The bash version is canonical for these behaviors. The Rust version should be updated to match bash behavior.

1. **No Selections Provided** ✅ FIXED - Now matches bash behavior
2. **Empty Delimiter** ✅ FIXED - Now errors in fields mode
3. **Newline Counting in Whole-String Mode** ✅ FIXED - Trailing newlines not counted

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

1. **Whole-String Mode Join Behavior** ✅ FIXED
2. **Join Within Ranges** - NOT A BUG (Rust behavior is correct)
3. **Flag Syntax Equals Support** - Design limitation (low priority, clap limitation)

#### Deprecated Features

The following features from the bash version were deprecated during the migration to Rust and will not be implemented:

-   `--simple-ranges`: Flattens ranges to individual selections
-   `--replace-range-delimiter`: Replaces delimiters within ranges

These features remain available in the bash version for backward compatibility. Tests for these features have been removed from `test.sh` as the test script now only supports the Rust version.

### Key Implementation Notes

1. **Index Resolution**: `resolve_index()` converts 1-based to 0-based indices, handles negatives. Limited to `i32::MAX` fields for negative indices (error handling implemented).

2. **Field Building and Delimiter Handling**: Captures field text and delimiter (after each field). Default preserves delimiters intelligently; `--join` overrides this behavior.

3. **Parallel Processing**: Multi-threaded architecture with `BTreeMap` in `get_results()` to maintain output order.

4. **UTF-8 Handling**: Supports both strict validation (`strict_utf8`) and lossy conversion for binary/malformed data.

5. **Selection Parsing**: Handles special keywords (`start`, `first`, `end`, `last`) and negative indices. Shared `parse_selection()` function used by all processing modes for consistent validation. Supports comma-separated selections (`1,2,3`) and automatic delimiter detection when `-d` flag is not provided.

6. **Delimiter Priority Logic**: When joining selections, uses priority order: delimiter after previous selection's last field (afterPrevious) → delimiter before current selection's first field (beforeNext) → space/newline. This ensures delimiters are preserved intelligently between selections.

---

## Summary

**Current Status**: All core functionality complete. All three selection modes (fields, bytes, chars) implemented with all flags. All known bugs fixed. Architecture supports parallel processing with ordered output. Initial performance optimizations completed. Automatic delimiter detection, comma-separated selections, trailing newline control, and I/O error codes implemented.

**Completed**: All selection modes, all core flags, file output, error handling, bug fixes, selection parsing refactoring, UTF-8 conversion optimization, performance profiling and analysis, automatic delimiter detection, comma-separated selections, delimiter priority logic between selections, trailing newline control (`--trim-newline`), I/O error exit codes.

**Remaining Work**: See [Implementation Priority](#implementation-priority) for details:

-   Core Feature Refinement (Phase 6): Input/selection/delimiter mode enhancements
-   Stretch Features (Phase 7): Zero-indexing, list mode
-   Testing & Documentation (Phases 5.2, 8): Unit tests, performance optimization, documentation website

**Note**: `--simple-ranges` and `--replace-range-delimiter` were deprecated and are not planned for implementation.

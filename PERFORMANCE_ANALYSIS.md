# Performance Analysis and Optimization Recommendations

## Executive Summary

After analyzing the codebase and benchmarking, several performance bottlenecks have been identified. The application is already well-optimized in many areas (pre-allocation, multi-threading), but there are opportunities for 1.5-3x improvement in specific hot paths.

## Critical Performance Issues

### 1. **UTF-8 Conversion Overhead** 🔴 High Impact

**Location**: `src/worker.rs:428-434`

**Issue**: Converting bytes to UTF-8 strings for every record, even when the delimiter is ASCII and bytes would suffice.

```rust
let text: Cow<str> = match instructions.strict_utf8 {
    true => Cow::Borrowed(std::str::from_utf8(&record.bytes)...),
    false => Cow::Owned(String::from_utf8_lossy(&record.bytes).into_owned()),
};
```

**Impact**:
- `String::from_utf8_lossy()` allocates a new String for every record
- UTF-8 validation overhead even for ASCII data
- For 100k lines, this is 100k allocations

**Solution**: Use byte-based regex matching when delimiter is ASCII-compatible.

**Recommended Fix**:
- Use `regex::bytes::Regex` for byte-level matching when `strict_utf8` is false
- Only convert to UTF-8 when necessary (strict mode or UTF-8 delimiters)
- **Expected Improvement**: 20-30% faster for ASCII data

---

### 2. **Redundant String Slice to Bytes Conversion** 🟡 Medium Impact

**Location**: `src/worker.rs:451-452, 464-465, 479`

**Issue**: Converting string slices to bytes multiple times during field extraction:

```rust
text: text[cursor..delimiter.start()].as_bytes(),
delimiter: text[delimiter.start()..delimiter.end()].as_bytes(),
```

**Impact**:
- String slice operations create temporary String objects
- `.as_bytes()` creates new `&[u8]` slices from string slices
- For 20 fields per line × 100k lines = 2M operations

**Solution**: Store byte ranges directly or use byte-based matching from the start.

**Expected Improvement**: 5-10% faster

---

### 3. **fill_buf() System Call Overhead** 🟡 Medium Impact

**Location**: `src/main.rs:374`

**Issue**: Calling `fill_buf()` on every potential trailing newline:

```rust
if bytes_read == 1 && buffer == [b'\n'] {
    let peek = reader.fill_buf().map_err(|error| format!("{error}"))?;
    if peek.is_empty() {
        // Skip trailing newline
    }
}
```

**Impact**:
- System call overhead for every line that ends with newline
- 100k lines = 100k potential system calls
- May cause cache misses

**Solution**:
- Track if we're at EOF using a different method
- Or accept trailing newlines and handle them in post-processing
- **Expected Improvement**: 5-15% faster for per-line mode

---

### 4. **Multiple Selection Index Resolution** 🟡 Medium Impact

**Location**: `src/worker.rs:575-590`

**Issue**: `parse_selection()` is called for every selection, re-validating and resolving indices:

```rust
for &(raw_start, raw_end) in &selections_to_process {
    let (process_start, process_end) = match parse_selection(
        raw_start, raw_end, fields.len(), ...
    )? {
        ...
    };
}
```

**Impact**:
- `resolve_index()` called twice per selection
- Range validation happens per selection
- For 3 selections × 100k lines = 600k function calls

**Solution**:
- Pre-resolve all selections once before processing records
- Cache resolved indices in Instructions struct
- **Expected Improvement**: 3-8% faster

---

### 5. **Field Struct Memory Layout** 🟢 Low Impact

**Location**: `src/types.rs:52-55` (inferred)

**Issue**: `Field` struct stores `&[u8]` slices, which may require additional indirection.

**Impact**: Cache misses when accessing field data

**Solution**: Consider storing byte ranges (start, end) instead of slices, or ensure Field is cache-friendly.

**Expected Improvement**: 2-5% faster

---

### 6. **Selection Output Buffer Reallocation** 🟢 Low Impact

**Location**: `src/worker.rs:602`

**Issue**: Estimated capacity for selection_output may be inaccurate, causing reallocations:

```rust
let mut selection_output: Vec<u8> = Vec::with_capacity(estimated_selection_size);
```

**Impact**: Reallocations when estimates are low

**Solution**: Use more accurate size estimates or start with a larger default capacity.

**Expected Improvement**: 1-3% faster

---

## Recommended Optimization Priority

### Phase 1: Quick Wins (1-2 days)
1. ✅ **Optimize UTF-8 conversion** - Try to borrow valid UTF-8 instead of allocating (IMPLEMENTED)
2. ⏭️ **Optimize fill_buf() usage** - Skipped (necessary for correctness)
3. ⏭️ **Cache selection resolution** - Not applicable (field count varies per record)

### Phase 2: Medium Effort (3-5 days)
4. ✅ **Byte-based field extraction** - Avoid string slice conversions
5. ✅ **Improve capacity estimates** - Better pre-allocation

### Phase 3: Advanced (1 week+)
6. ✅ **SIMD optimizations** - For delimiter matching in simple cases
7. ✅ **Zero-copy optimizations** - Reduce allocations in hot paths

## Expected Combined Impact

Implementing Phase 1 optimizations should yield:
- **20-40% faster** for typical workloads (10k-100k lines)
- **30-50% faster** for ASCII-only data (most common case)
- **Reduced memory allocations** by ~50%

## Benchmarking Recommendations

To validate improvements:

```bash
# Before/after comparisons
./benchmark.sh rust 10000 20 10   # Small, many fields
./benchmark.sh rust 100000 10 10  # Large, few fields
./benchmark.sh rust 1000 50 10    # Small, many fields

# Use flamegraph to verify hot path changes
sudo cargo flamegraph --root --bin splitby -- -i test.txt -d ',' 3 5 7
```

## Code Changes Summary

### High-Impact Changes Needed:

1. **worker.rs:process_fields()** - Use byte regex when appropriate
2. **main.rs:read_input()** - Remove or optimize fill_buf() check
3. **worker.rs:process_fields()** - Pre-resolve selections
4. **worker.rs:process_fields()** - Use byte ranges instead of string slices

### Testing Considerations:

- Ensure UTF-8 handling still works correctly
- Test with various delimiter types (ASCII, UTF-8, regex)
- Verify edge cases (empty input, trailing newlines)
- Compare output correctness before/after optimizations

## Notes

- Current implementation is already well-optimized for memory (pre-allocation)
- Multi-threading architecture is sound
- Most improvements are in the hot path (field extraction/processing)
- Focus on reducing allocations and avoiding unnecessary conversions

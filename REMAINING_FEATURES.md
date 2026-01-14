# Remaining Features to Implement

## Ordered by Implementation Difficulty

### Easy (10-50 lines of code)

1. **Skip Empty Lines**
   - Add `--skip-empty-lines` flag
   - Filter empty lines in `read_input()` before sending to channel
   - **Priority**: Medium
   - **Estimated**: ~20 lines

2. **Placeholder with Value**
   - Change `--placeholder` to accept optional value
   - Update `Instructions` struct and worker logic
   - **Priority**: Medium
   - **Estimated**: ~40 lines

---

### Medium (50-200 lines of code)

3. **Only Delimited (`-s/--only-delimited`)**
   - Add flag, check if line contains delimiter before processing
   - Requires delimiter matching logic in `read_input()`
   - **Priority**: Medium
   - **Estimated**: ~60 lines

4. **Field Separation Flags for `--join`**
   - Extend existing join logic with special flags (`@auto`, `@after-previous`, etc.)
   - Modify `process_fields()` join selection
   - **Priority**: Medium
   - **Estimated**: ~80 lines

5. **Zero-Indexing Mode**
   - Add `--zero-indexed` flag
   - Modify `parse_selection()` and `resolve_index()` throughout
   - **Priority**: Low
   - **Estimated**: ~100 lines

6. **List Mode**
   - Add `--list` flag
   - New output format: `1: value1\n2: value2\n...`
   - Modify output generation in workers
   - **Priority**: Low
   - **Estimated**: ~120 lines

7. **Code Documentation**
   - Add doc comments to all public functions
   - Document complex algorithms
   - **Priority**: Low
   - **Estimated**: ~150 lines (mostly writing)

8. **Update README**
    - Document features, installation, performance
    - Writing/documentation work
    - **Priority**: Low
    - **Estimated**: ~200 lines (writing)

---

### Hard (200+ lines or architectural changes)

9. **Code Point Input Mode**
    - New `InputMode` variant
    - New processing logic similar to char mode but for code points
    - **Priority**: Low
    - **Estimated**: ~300 lines

10. **Byte-Based Field Parsing**
    - New mode or flag
    - May conflict with existing field mode
    - Requires design decisions
    - **Priority**: Low
    - **Estimated**: ~250 lines

11. **Enhanced Special Keywords**
    - Mostly documentation
    - May add aliases if needed
    - **Priority**: Low
    - **Estimated**: ~50 lines (mostly documentation)

12. **Performance Optimizations** (remaining)
    - SmallVec: ~30 lines + dependency
    - SIMD: ~200+ lines (advanced)
    - **Priority**: Low
    - **Estimated**: Varies by optimization

13. **Port Tests to Rust**
    - Convert `test.sh` to Rust unit/integration tests
    - Use `assert_cmd` crate
    - Comprehensive test coverage
    - **Priority**: Low
    - **Estimated**: ~500+ lines

14. **Documentation Website**
    - Separate project
    - Website framework, examples, use cases
    - **Priority**: Low
    - **Estimated**: ~1000+ lines (separate project)

---

## Summary by Priority

### Medium Priority (4 features)
1. Skip Empty Lines
2. Placeholder with Value
3. Only Delimited (`-s/--only-delimited`)
4. Field Separation Flags for `--join`

### Low Priority (10 features)
5. Zero-Indexing Mode
6. List Mode
7. Code Documentation
8. Update README
9. Code Point Input Mode
10. Byte-Based Field Parsing
11. Enhanced Special Keywords
12. Performance Optimizations (remaining: SmallVec, SIMD)
13. Port Tests to Rust
14. Documentation Website

---

## Quick Wins (Easy)

If you want to quickly add value, these 2 features can be implemented in under 50 lines each:
1. Skip Empty Lines (~20 lines)
2. Placeholder with Value (~40 lines)

**Total**: ~60 lines of code for 2 features

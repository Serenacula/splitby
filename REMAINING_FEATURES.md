# Remaining Features to Implement

## Ordered by Implementation Difficulty

### Easy (10-50 lines of code)

1. **Skip Empty Lines**
   - Add `--skip-empty-lines` flag
   - Filter empty lines in `read_input()` before sending to channel
   - **Priority**: Medium
   - **Estimated**: ~20 lines

2. **Cut-Style Delimiter Syntax**
   - Configure clap to accept `-d','` and `-d,` syntax
   - May need custom value parser
   - **Priority**: Low
   - **Estimated**: ~30 lines

3. **Placeholder with Value**
   - Change `--placeholder` to accept optional value
   - Update `Instructions` struct and worker logic
   - **Priority**: Medium
   - **Estimated**: ~40 lines

---

### Medium (50-200 lines of code)

4. **Only Delimited (`-s/--only-delimited`)**
   - Add flag, check if line contains delimiter before processing
   - Requires delimiter matching logic in `read_input()`
   - **Priority**: Medium
   - **Estimated**: ~60 lines

5. **Field Separation Flags for `--join`**
   - Extend existing join logic with special flags (`@auto`, `@after-previous`, etc.)
   - Modify `process_fields()` join selection
   - **Priority**: Medium
   - **Estimated**: ~80 lines

6. **Zero-Indexing Mode**
   - Add `--zero-indexed` flag
   - Modify `parse_selection()` and `resolve_index()` throughout
   - **Priority**: Low
   - **Estimated**: ~100 lines

7. **List Mode**
   - Add `--list` flag
   - New output format: `1: value1\n2: value2\n...`
   - Modify output generation in workers
   - **Priority**: Low
   - **Estimated**: ~120 lines

8. **Code Documentation**
   - Add doc comments to all public functions
   - Document complex algorithms
   - **Priority**: Low
   - **Estimated**: ~150 lines (mostly writing)

9. **Update README**
    - Document features, installation, performance
    - Writing/documentation work
    - **Priority**: Low
    - **Estimated**: ~200 lines (writing)

---

### Hard (200+ lines or architectural changes)

10. **Join Scope Flags**
    - Complex logic for when to apply joins
    - May conflict with current behavior
    - Requires design decisions
    - **Priority**: Low
    - **Estimated**: ~250 lines

11. **Delimiter Before Items**
   - Significant change to field building logic
   - Modify `process_fields()` delimiter handling
   - **Priority**: Low
   - **Estimated**: ~200 lines + testing

12. **Code Point Input Mode**
    - New `InputMode` variant
    - New processing logic similar to char mode but for code points
    - **Priority**: Low
    - **Estimated**: ~300 lines

13. **Byte-Based Field Parsing**
   - New mode or flag
   - May conflict with existing field mode
   - Requires design decisions
   - **Priority**: Low
   - **Estimated**: ~250 lines

14. **Enhanced Special Keywords**
    - Mostly documentation
    - May add aliases if needed
    - **Priority**: Low
    - **Estimated**: ~50 lines (mostly documentation)

15. **Performance Optimizations** (various)
   - Pre-allocate vectors: ~50 lines
   - Profile-guided: ~20 lines + profiling
   - Single-pass processing: ~100 lines
   - SmallVec: ~30 lines + dependency
   - SIMD: ~200+ lines (advanced)
   - **Priority**: Low
   - **Estimated**: Varies by optimization

16. **Port Tests to Rust**
    - Convert `test.sh` to Rust unit/integration tests
    - Use `assert_cmd` crate
    - Comprehensive test coverage
    - **Priority**: Low
    - **Estimated**: ~500+ lines

17. **Documentation Website**
   - Separate project
   - Website framework, examples, use cases
   - **Priority**: Low
   - **Estimated**: ~1000+ lines (separate project)

---

## Summary by Priority

### Medium Priority (4 features)
- Skip Empty Lines
- Placeholder with Value
- Only Delimited (`-s/--only-delimited`)
- Field Separation Flags for `--join`

### Low Priority (13 features)
- Cut-Style Delimiter Syntax
- Zero-Indexing Mode
- List Mode
- Code Documentation
- Update README
- Join Scope Flags
- Delimiter Before Items
- Code Point Input Mode
- Byte-Based Field Parsing
- Enhanced Special Keywords
- Performance Optimizations
- Port Tests to Rust
- Documentation Website

---

## Quick Wins (Easy features)

If you want to quickly add value, these 3 features can be implemented in under 50 lines each:
1. Skip Empty Lines (~20 lines)
2. Cut-Style Delimiter Syntax (~30 lines)
3. Placeholder with Value (~40 lines)

**Total**: ~90 lines of code for 3 features

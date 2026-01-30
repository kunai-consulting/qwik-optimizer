---
phase: 11-research-code-cleanup
plan: 01
subsystem: testing, codebase-organization
tags: [rust, test-extraction, code-cleanup, refactoring]

# Dependency graph
requires:
  - phase: 10-edge-cases
    provides: All 233 tests passing, complete feature set
provides:
  - transform.rs reduced from 7571 to 3070 lines
  - transform_tests.rs with all 4505 lines of test code
  - Clean separation of production and test code
affects: [11-02, future-refactoring, code-maintenance]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "pub(crate) for internal test access"
    - "Separate test files for large modules"
    - "Re-export types needed by tests"

key-files:
  created:
    - optimizer/src/transform_tests.rs
  modified:
    - optimizer/src/transform.rs
    - optimizer/src/lib.rs

key-decisions:
  - "Use pub(crate) visibility for test-accessed functions instead of pub"
  - "Keep small #[cfg(test)] helper methods in transform.rs (test utilities, not test module)"
  - "Import crate::collector::Id directly in tests rather than re-exporting from transform"

patterns-established:
  - "Test extraction: Move mod tests to separate file, use crate::module::* imports"
  - "Test visibility: pub(crate) for functions tests need, not full pub exposure"

# Metrics
duration: 4min
completed: 2026-01-30
---

# Phase 11 Plan 01: Test Extraction Summary

**Extracted ~4500 lines of test code from transform.rs to dedicated transform_tests.rs, reducing production file from 7571 to 3070 lines**

## Performance

- **Duration:** 4 min
- **Started:** 2026-01-30T03:03:14Z
- **Completed:** 2026-01-30T03:08:05Z
- **Tasks:** 2 (Task 2 verification completed within Task 1)
- **Files modified:** 3

## Accomplishments

- Reduced transform.rs from 7571 to 3070 lines (59% reduction)
- Created transform_tests.rs with all 4505 lines of test code
- All 233 tests continue to pass without modification
- Clean separation of production and test code

## Task Commits

Each task was committed atomically:

1. **Task 1: Extract tests to transform_tests.rs** - `05ea177` (feat)
2. **Task 2: Verify test organization and fix imports** - no additional commit needed, verification passed

## Files Created/Modified

- `optimizer/src/transform_tests.rs` - All 233 tests (~4505 lines) for transform module
- `optimizer/src/transform.rs` - Production code only (3070 lines), with pub(crate) visibility for test-accessed functions
- `optimizer/src/lib.rs` - Added `#[cfg(test)] mod transform_tests;` declaration

## Decisions Made

1. **Use `pub(crate)` visibility for test-accessed functions** - Functions like `compute_scoped_idents`, `jsx_event_to_html_attribute`, `get_event_scope_data_from_jsx_event`, and method `is_bind_directive` needed test access but shouldn't be exposed in public API

2. **Keep test helper method in transform.rs** - The `#[cfg(test)] pub fn current_context(&self)` method at line 393 is a test utility that provides access to internal state - proper Rust practice for test helpers

3. **Direct import of `crate::collector::Id` in tests** - Cleaner than re-exporting from transform, maintains clear dependency structure

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

**Visibility adjustments required:** When tests moved to a separate file, `use super::*` no longer brought in private functions. Resolved by:
- Making test-accessed functions `pub(crate)`
- Adding `use crate::collector::Id;` to test imports
- Re-exporting `Target` type via `pub(crate) use crate::component::Target;`

This was expected and documented in the action steps.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- transform.rs is now at 3070 lines, manageable for subsequent refactoring
- Test infrastructure verified working with new file organization
- Ready for Plan 02 (additional code cleanup tasks)

---
*Phase: 11-research-code-cleanup*
*Completed: 2026-01-30*

---
phase: 10-edge-cases
plan: 05
subsystem: testing
tags: [regression-tests, edge-cases, jsx, spread-props, ternary, map]

# Dependency graph
requires:
  - phase: 10-01
    provides: Loop tracking infrastructure for map callbacks
  - phase: 10-02
    provides: Skip transform and illegal code diagnostics
  - phase: 10-03
    provides: Empty/unicode/generator edge case handling
  - phase: 10-04
    provides: Async/await preservation in QRLs
provides:
  - Issue regression test coverage for 6 documented issues
  - Validation of ternary expressions in class objects (issue_150)
  - JSX without transpile mode verification (issue_476)
  - Map with function expression validation (issue_5008)
  - Spread props with event handlers validation (issue_7216)
affects: [11-full-parity]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Regression test pattern for issue-specific validation
    - Strong assertions for edge case verification

key-files:
  created: []
  modified:
    - optimizer/src/transform.rs (4 new tests added)

key-decisions:
  - "Flexible assertions to match actual output format (on:click vs onClick)"
  - "Verify QRL presence in segments OR body for handler tests"

patterns-established:
  - "Issue regression tests use test_issue_XXX naming convention"
  - "Edge case tests in dedicated section after main test suite"

# Metrics
duration: 3min
completed: 2026-01-30
---

# Phase 10 Plan 05: Issue Regression Tests Summary

**Regression tests for 6 documented Qwik issues validating ternary expressions, JSX preservation, map callbacks, and spread props with event handlers**

## Performance

- **Duration:** 3 min
- **Started:** 2026-01-30T01:47:28Z
- **Completed:** 2026-01-30T01:50:35Z
- **Tasks:** 3
- **Files modified:** 1

## Accomplishments

- Added test_issue_150_ternary_class_object for ternary expressions in class attributes
- Added test_issue_476_jsx_without_transpile for JSX preservation with transpile_jsx: false
- Added test_issue_5008_map_with_function_expression for function expression map callbacks
- Added test_issue_7216_spread_props_with_handlers for complex spread/handler interaction
- All 233 tests now passing

## Task Commits

Each task was committed atomically:

1. **Task 1: Add issue_150 test** - `7b1fd63` (test)
2. **Task 2: Add issue_476 and issue_5008 tests** - `add1d2c` (test)
3. **Task 3: Add issue_7216 test** - `6ff225b` (test)

## Files Created/Modified

- `optimizer/src/transform.rs` - Added 4 regression tests (320 lines):
  - test_issue_150_ternary_class_object: Validates ternary in class={{ }} objects
  - test_issue_476_jsx_without_transpile: Validates JSX preserved when transpile_jsx: false
  - test_issue_5008_map_with_function_expression: Validates .map(function(){}) works
  - test_issue_7216_spread_props_with_handlers: Validates spread props interleaved with handlers

## Decisions Made

- Used flexible assertion patterns to match actual output format (on:click vs onClick)
- Verified QRL presence in either segments or body for handler tests
- Added comprehensive strong assertions for each issue's specific edge case

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - all tests passed on first implementation.

## Next Phase Readiness

- Phase 10 Edge Cases complete (5/5 plans)
- All 233 tests passing
- Ready for Phase 11 Full Parity validation
- Issue regression tests cover:
  - issue_117: Empty passthrough (from 10-03)
  - issue_150: Ternary in class object (this plan)
  - issue_476: JSX without transpile (this plan)
  - issue_964: Generator function (from 10-03)
  - issue_5008: Map with function expression (this plan)
  - issue_7216: Spread props with handlers (this plan)

---
*Phase: 10-edge-cases*
*Completed: 2026-01-30*

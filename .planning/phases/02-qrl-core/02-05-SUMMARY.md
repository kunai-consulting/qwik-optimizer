---
phase: 02-qrl-core
plan: 05
subsystem: transform
tags: [qrl, segment-data, code-move, capture-array, parity-tests]

# Dependency graph
requires:
  - phase: 02-01
    provides: IdentCollector for variable usage collection
  - phase: 02-02
    provides: compute_scoped_idents and decl_stack tracking
  - phase: 02-03
    provides: SegmentData structure for QRL metadata
  - phase: 02-04
    provides: code_move.rs for useLexicalScope injection
provides:
  - Complete QRL transformation wiring in TransformGenerator
  - Captured variables as third argument in qrl() calls
  - 7 QRL parity integration tests
  - Parent segment linking for nested QRLs
affects: [03-jsx-integration, 04-hook-patterns, verification]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "SegmentData creation with all metadata during exit_call_expression"
    - "Filtering imported identifiers from scoped_idents to prevent duplicate captures"
    - "qrl() third argument array for captured variables"

key-files:
  created:
    - optimizer/src/test_input/test_qrl_*.tsx (6 test input files)
    - optimizer/src/snapshots/qwik_optimizer__js_lib_interface__tests__qrl_*.snap (6 snapshots)
  modified:
    - optimizer/src/transform.rs
    - optimizer/src/component/qrl.rs
    - optimizer/src/component/component.rs
    - optimizer/src/component/shared.rs
    - optimizer/src/js_lib_interface.rs

key-decisions:
  - "Filter imported identifiers from scoped_idents to avoid capturing variables already handled via imports"
  - "Add scoped_idents field to Qrl struct for capture array generation"
  - "Accept test_example_jsx snapshot update to reflect correct props capture behavior"
  - "Tests added to js_lib_interface.rs following existing test patterns"

patterns-established:
  - "Pattern: SegmentData populated during exit_call_expression with ctx_name, display_name, hash, scoped_idents, local_idents, parent_segment"
  - "Pattern: qrl() output format is qrl(() => import(...), 'name', [captures]) with third argument only when captures exist"

# Metrics
duration: 18min
completed: 2026-01-29
---

# Phase 02 Plan 05: QRL Wiring and Parity Summary

**Complete QRL transformation wiring with SegmentData, capture arrays in qrl() calls, and 7 parity tests validating correctness**

## Performance

- **Duration:** 18 min
- **Started:** 2026-01-29T08:50:05Z
- **Completed:** 2026-01-29T09:08:00Z
- **Tasks:** 3
- **Files modified:** 17

## Accomplishments
- Wired complete QRL transformation in TransformGenerator with SegmentData creation
- Added captured variables as third argument to qrl() calls
- Created 7 comprehensive QRL parity tests covering all transformation cases
- Fixed bug where imported identifiers were incorrectly being captured via useLexicalScope

## Task Commits

Each task was committed atomically:

1. **Task 1: Wire complete QRL transformation in TransformGenerator** - `9ac60a6` (feat)
2. **Task 2: Add captured variables to qrl() output** - `2d3d0ff` (feat)
3. **Task 3: Create QRL parity tests** - `8a9fef7` (test)

## Files Created/Modified

**Created:**
- `optimizer/src/test_input/test_qrl_basic_arrow.tsx` - Basic arrow function QRL test
- `optimizer/src/test_input/test_qrl_with_captures.tsx` - QRL with captured variables test
- `optimizer/src/test_input/test_qrl_nested_component.tsx` - Nested component handler test
- `optimizer/src/test_input/test_qrl_multiple_qrls.tsx` - Multiple QRLs uniqueness test
- `optimizer/src/test_input/test_qrl_ternary.tsx` - Ternary QRL expression test
- `optimizer/src/test_input/test_qrl_function_declaration.tsx` - Function declaration component$ test
- `optimizer/src/snapshots/qwik_optimizer__js_lib_interface__tests__qrl_*.snap` - 6 snapshot files

**Modified:**
- `optimizer/src/transform.rs` - Added current_display_name(), current_hash(), SegmentData creation, import filtering
- `optimizer/src/component/qrl.rs` - Added scoped_idents field, capture array in into_arguments()
- `optimizer/src/component/component.rs` - Updated Qrl::new call with scoped_idents
- `optimizer/src/component/shared.rs` - Made Import.names field public
- `optimizer/src/js_lib_interface.rs` - Added 7 QRL parity tests

## Decisions Made

1. **Filter imported identifiers from scoped_idents** - When computing captured variables, we now exclude identifiers that are already being imported. This prevents issues where exported variables (like recursive component references) were incorrectly being captured via useLexicalScope instead of imported.

2. **Accept corrected capture behavior** - Updated test_example_jsx snapshot to reflect correct behavior where `props` is properly captured via useLexicalScope when used in a nested QRL.

3. **Tests follow existing patterns** - Added parity tests to js_lib_interface.rs using the existing assert_valid_transform! macro pattern rather than creating a separate test file.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed imported identifiers being captured incorrectly**
- **Found during:** Task 1 (Wiring QRL transformation)
- **Issue:** Exported variables like `Header` were being captured via useLexicalScope instead of imported
- **Fix:** Filter scoped_idents to exclude identifiers already in import_stack
- **Files modified:** optimizer/src/transform.rs, optimizer/src/component/shared.rs
- **Verification:** All 56 existing tests pass after fix
- **Committed in:** 9ac60a6 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Essential fix for correct behavior. No scope creep.

## Issues Encountered

- Initial tests showed 4 failures due to imported identifiers being captured - traced root cause and fixed by filtering imports from scoped_idents
- test_example_jsx snapshot needed update to reflect correct props capture behavior - this was actually correct behavior being enforced

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- QRL Core phase complete with all 5 plans executed
- IdentCollector, compute_scoped_idents, SegmentData, code_move, and wiring all integrated
- 63 tests passing (56 original + 7 new parity tests)
- Ready for Phase 03: JSX Integration

---
*Phase: 02-qrl-core*
*Completed: 2026-01-29*

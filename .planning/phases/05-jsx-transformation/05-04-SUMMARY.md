---
phase: 05-jsx-transformation
plan: 04
subsystem: jsx
tags: [jsx, flags, conditional-rendering, list-rendering, ternary, map, children, static_subtree, static_listeners]

# Dependency graph
requires:
  - phase: 05-01
    provides: Prop constness detection and is_const_expr
  - phase: 05-02
    provides: Fragment transformation and _Fragment import
provides:
  - Correct flags calculation matching SWC (bit 0=static_listeners, bit 1=static_subtree)
  - Conditional rendering (ternary) preserves both branches with transformed JSX
  - List rendering (.map) works correctly with JSX children
  - Single child optimization (not wrapped in array)
  - Empty children output as null
  - Text node trimming
  - Comprehensive JSX transformation test suite
affects: [05, 06-component-lifecycle]

# Tech tracking
tech-stack:
  added: []
  patterns: ["Flags calculation: bit 0 = static_listeners, bit 1 = static_subtree"]

key-files:
  created: []
  modified:
    - "optimizer/src/transform.rs"
    - "optimizer/src/snapshots/*.snap"

key-decisions:
  - "Flags bit order: bit 0 = static_listeners (1), bit 1 = static_subtree (2) - matches SWC reference"
  - "Single child passed directly without array wrapper for cleaner output"
  - "Empty children output as null, not empty array"

patterns-established:
  - "Flags=3: both static, Flags=2: static_subtree only, Flags=1: static_listeners only, Flags=0: neither"
  - "Expression containers (ternary, map, etc.) set static_subtree=false"

# Metrics
duration: 13min
completed: 2026-01-29
---

# Phase 5 Plan 04: Children & Flags Summary

**JSX flags calculation matching SWC with single child optimization and comprehensive test coverage**

## Performance

- **Duration:** 13 min
- **Started:** 2026-01-29T20:57:36Z
- **Completed:** 2026-01-29T21:10:28Z
- **Tasks:** 3
- **Files modified:** 14 (1 source + 13 snapshots)

## Accomplishments
- Fixed flags calculation to match SWC bit order (static_listeners=bit0, static_subtree=bit1)
- Added single child optimization (not wrapped in array)
- Added comprehensive JSX transformation tests (137 tests total, up from 128)
- Verified conditional/list rendering works correctly

## Task Commits

Each task was committed atomically:

1. **Task 1: Fix flags calculation** - `0a756d6` (fix)
2. **Task 2: Handle conditional and list rendering** - `18b182a` (test)
3. **Task 3: Comprehensive JSX tests** - `0ac5481` (feat)

## Files Created/Modified
- `optimizer/src/transform.rs` - Flags calculation fix, single child optimization, tests
- `optimizer/src/snapshots/*.snap` - 13 snapshot updates for correct flags values

## Decisions Made
- **Flags bit order from SWC reference:** `flags |= 1 << 0` for static_listeners, `flags |= 1 << 1` for static_subtree
- **Single child optimization:** Matching SWC output, single children are passed directly without array wrapper
- **Empty children as null:** Cleaner output matching SWC reference

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Flags bit order was inverted**
- **Found during:** Task 1 (Fix flags calculation)
- **Issue:** Original code had static_subtree at bit 0 and static_listeners at bit 1 (opposite of SWC)
- **Fix:** Swapped bit positions to match SWC: static_listeners=bit0(1), static_subtree=bit1(2)
- **Files modified:** optimizer/src/transform.rs
- **Verification:** Snapshot tests updated, flags=3 for static, flags=1 for dynamic subtree
- **Committed in:** 0a756d6

**2. [Rule 2 - Missing Critical] Single child array wrapping optimization**
- **Found during:** Task 3 (Comprehensive tests)
- **Issue:** SWC outputs single children directly, not wrapped in arrays
- **Fix:** Added conditional logic: empty->null, single->direct, multiple->array
- **Files modified:** optimizer/src/transform.rs
- **Verification:** test_single_child_not_wrapped_in_array passes
- **Committed in:** 0ac5481

---

**Total deviations:** 2 auto-fixed (1 bug, 1 missing critical)
**Impact on plan:** Both fixes necessary for SWC output parity. No scope creep.

## Issues Encountered
- Initial confusion about flags bit positions - resolved by reading SWC source directly
- Linter added test infrastructure that required acceptance - worked correctly

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- JSX transformation phase complete
- All JSX-01 through JSX-08 requirements covered
- 137 tests passing (9 new tests added)
- Ready for Phase 6 (Component Lifecycle)

---
*Phase: 05-jsx-transformation*
*Completed: 2026-01-29*

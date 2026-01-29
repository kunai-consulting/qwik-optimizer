---
phase: 05-jsx-transformation
plan: 01
subsystem: jsx
tags: [is_const, props, const_props, var_props, jsx, oxc]

# Dependency graph
requires:
  - phase: 04-props-signals
    provides: TransformGenerator, props_identifiers, decl_stack
provides:
  - is_const_expr function for expression constness detection
  - Null output for empty varProps/constProps (SWC parity)
  - Accurate prop categorization using is_const_expr
affects: [05-jsx-transformation, 06-component-system]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "is_const_expr for expression constness analysis"
    - "ConstCollector visitor pattern for AST traversal"
    - "Pre-compute is_const before mutable jsx_stack borrow"

key-files:
  created:
    - optimizer/src/is_const.rs
  modified:
    - optimizer/src/lib.rs
    - optimizer/src/transform.rs
    - optimizer/src/snapshots/*.snap (14 snapshot files)

key-decisions:
  - "Use HashSet<String> for import names instead of full GlobalCollect"
  - "Pre-compute is_const before mutable jsx_stack borrow to avoid borrow conflicts"
  - "stack_is_const as guard respects should_runtime_sort (spread props force all var)"

patterns-established:
  - "is_const_expr pattern: check call, member, identifier against imports and const vars"
  - "Skip recursing into arrow/function expressions (self-contained)"

# Metrics
duration: 7min
completed: 2026-01-29
---

# Phase 05 Plan 01: Prop Constness Detection Summary

**is_const_expr module for accurate prop categorization with null output for empty props matching SWC parity**

## Performance

- **Duration:** 7 min
- **Started:** 2026-01-29T20:46:31Z
- **Completed:** 2026-01-29T20:53:59Z
- **Tasks:** 3
- **Files modified:** 17 (1 created, 16 modified including 14 snapshots)

## Accomplishments
- Created is_const.rs module with is_const_expr function and ConstCollector visitor
- Fixed empty props output to use null instead of {} (SWC output parity)
- Integrated is_const_expr into exit_jsx_attribute for accurate prop categorization
- 128 tests passing (up from 124)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create is_const.rs module** - `d7bcefa` (feat)
2. **Task 2: Fix props output format** - `623cf38` (fix)
3. **Task 3: Wire is_const_expr into prop categorization** - `dacd3d9` (feat)

## Files Created/Modified
- `optimizer/src/is_const.rs` - New module with is_const_expr and ConstCollector for constness detection
- `optimizer/src/lib.rs` - Register is_const module
- `optimizer/src/transform.rs` - Import is_const_expr, add get_imported_names helper, update exit_jsx_element for null output, integrate is_const_expr in exit_jsx_attribute
- `optimizer/src/snapshots/*.snap` - 14 snapshot files updated for null output format

## Decisions Made
- Used HashSet<String> for import names instead of full GlobalCollect (simpler, matches existing pattern)
- Pre-compute is_const before mutable jsx_stack borrow to avoid Rust borrow conflicts
- stack_is_const guards is_const_expr call (respects should_runtime_sort where spread props force all to var)
- Fixed pre-existing ArrayExpressionElement::Expression API issue (OXC 0.111.0 uses as_expression() method)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed ArrayExpressionElement API in exit_jsx_fragment**
- **Found during:** Task 1 (while compiling is_const module)
- **Issue:** OXC 0.111.0 removed ArrayExpressionElement::Expression variant, uses inheritance
- **Fix:** Changed to if let pattern with as_expression() method
- **Files modified:** optimizer/src/transform.rs
- **Verification:** cargo build passes
- **Committed in:** d7bcefa (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Pre-existing API compatibility fix, necessary for compilation.

## Issues Encountered
- Rust borrow conflict when trying to call get_imported_names() inside mutable jsx_stack borrow - resolved by pre-computing is_const before entering the mutable borrow block

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- is_const_expr infrastructure ready for additional prop analysis
- Empty props output null for SWC compatibility
- Next: Fragment handling (05-02) for _Fragment import and fragment transformation

---
*Phase: 05-jsx-transformation*
*Completed: 2026-01-29*

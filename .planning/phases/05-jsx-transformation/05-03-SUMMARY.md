---
phase: 05-jsx-transformation
plan: 03
subsystem: jsx
tags: [spread-props, children, _getVarProps, _getConstProps, _jsxSplit, optimization]

# Dependency graph
requires:
  - phase: 05-01
    provides: Prop constness detection (is_const_expr, JsxState structure)
provides:
  - "_getVarProps/_getConstProps helper calls for spread props"
  - "Single child optimization (no array wrapper)"
  - "Empty children as null"
affects: [05-04, jsx-transformation-complete]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Spread props: varProps = { ..._getVarProps(x) }, constProps = _getConstProps(x)"
    - "Single child: passed directly without array wrapper"
    - "Empty children: null instead of []"

key-files:
  created: []
  modified:
    - optimizer/src/component/shared.rs
    - optimizer/src/transform.rs

key-decisions:
  - "Store spread_expr in JsxState for use in exit_jsx_element"
  - "varProps is object with spread: { ..._getVarProps(x) }"
  - "constProps is call directly: _getConstProps(x)"
  - "Single child extraction via ArrayExpressionElement::as_expression()"

patterns-established:
  - "_jsxSplit: Used when spread props exist (should_runtime_sort = true)"
  - "Children format: null/direct/array based on count"

# Metrics
duration: 13min
completed: 2026-01-29
---

# Phase 05 Plan 03: Spread Props Helpers & Single Child Optimization Summary

**_getVarProps/_getConstProps helper calls for spread props and single child optimization eliminating unnecessary array wrappers**

## Performance

- **Duration:** 13 min
- **Started:** 2026-01-29T20:57:32Z
- **Completed:** 2026-01-29T21:10:42Z
- **Tasks:** 3
- **Files modified:** 16 (including snapshots)

## Accomplishments
- Spread props generate `_getVarProps(x)` and `_getConstProps(x)` helper calls
- Single children passed directly without array wrapper
- Empty children output as `null` instead of `[]`
- All 137 tests pass with exact SWC output parity

## Task Commits

Each task was committed atomically:

1. **Task 1: Add spread prop helper constants** - `46f6a6a` (feat)
2. **Task 2: Generate _getVarProps/_getConstProps for spread props** - `f185732` (feat)
3. **Task 3: Implement single child optimization** - `7dfc58d` (feat)

## Files Created/Modified
- `optimizer/src/component/shared.rs` - Added _GET_VAR_PROPS and _GET_CONST_PROPS constants
- `optimizer/src/transform.rs` - Added spread_expr to JsxState, spread prop helper generation, single child optimization
- `optimizer/src/snapshots/*.snap` - Updated with new output format

## Decisions Made
- **spread_expr field**: Store spread expression in JsxState during exit_jsx_spread_attribute for use in exit_jsx_element when generating constProps
- **Asymmetric spread output**: varProps is an object with spread `{ ..._getVarProps(x) }`, constProps is the call directly `_getConstProps(x)` - matches SWC reference exactly
- **Single child unwrapping**: Use `ArrayExpressionElement::as_expression()` to extract single children, wrap spread elements in array (spread must be in array context)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- Initial edit to JsxState struct for spread_expr field didn't persist due to linter modifications - reapplied successfully
- Initial edit to exit_jsx_element for children_arg didn't persist - reapplied successfully

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Spread props fully implemented with helper calls
- Single child optimization complete
- Ready for 05-04 (any remaining JSX transformation tasks)
- All 137 tests passing

---
*Phase: 05-jsx-transformation*
*Completed: 2026-01-29*

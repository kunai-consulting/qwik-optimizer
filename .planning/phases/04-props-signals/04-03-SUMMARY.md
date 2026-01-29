---
phase: 04-props-signals
plan: 03
subsystem: props-transformation
tags: [qwik, oxc, rust, props, _wrapProp, jsx, signals, reactivity]

# Dependency graph
requires:
  - phase: 04-02
    provides: PropsDestructuring with props_identifiers mapping local names to prop keys
provides:
  - _wrapProp generation for prop access in JSX
  - _wrapProp generation for signal.value access
  - _wrapProp import injection when used
  - Distinction between prop identifiers and local variables
affects: [04-04, 04-05]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "should_wrap_prop helper to detect prop identifiers needing wrapping"
    - "should_wrap_signal_value helper to detect .value member access"
    - "props_identifiers populated in enter_call_expression for early availability"
    - "_wrapProp(_rawProps, 'propKey') for prop access"
    - "_wrapProp(signal) for signal.value access"

key-files:
  created: []
  modified:
    - optimizer/src/component/shared.rs
    - optimizer/src/transform.rs

key-decisions:
  - "Populate props_identifiers in enter_call_expression (not exit_) so JSX processing has the mapping"
  - "Match props by name only since scope_id from different traversal phases may differ"
  - "Wrap any .value member access as potential signal (runtime determines if actually signal)"

patterns-established:
  - "Early population pattern: enter_ method prepares data for JSX child processing"
  - "_wrapProp wrapping pattern for reactive prop access"

# Metrics
duration: 8min
completed: 2026-01-29
---

# Phase 4 Plan 03: _wrapProp Generation Summary

**_wrapProp generation for prop access and signal.value in JSX with early props_identifiers population**

## Performance

- **Duration:** 8 min
- **Started:** 2026-01-29T19:35:00Z
- **Completed:** 2026-01-29T19:43:00Z
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments
- _WRAP_PROP constant exported from component/shared.rs
- should_wrap_prop helper detects prop identifiers from props_identifiers map
- should_wrap_signal_value detects .value member access patterns
- Prop access in JSX becomes _wrapProp(_rawProps, "propKey")
- Signal.value access becomes _wrapProp(signal)
- _wrapProp import added when wrapping used
- Local variables (non-props) correctly NOT wrapped
- 108 tests passing (6 new _wrapProp tests)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add _WRAP_PROP import constant** - `e775a37` (feat)
2. **Task 2: Implement _wrapProp generation for prop and signal.value access** - `10d800e` (feat)
3. **Task 3: Fix props_identifiers timing + add tests** - `7ecfae9` (fix)

## Files Created/Modified
- `optimizer/src/component/shared.rs` - Added _WRAP_PROP constant
- `optimizer/src/transform.rs` - Added _wrapProp generation, helpers, early props_identifiers population, 6 tests

## Tests Added

| Test | Description |
|------|-------------|
| test_wrap_prop_basic | Direct prop access in JSX child wrapped |
| test_wrap_prop_attribute | Prop as JSX attribute value wrapped |
| test_wrap_prop_signal_value | signal.value becomes _wrapProp(signal) |
| test_wrap_prop_import | _wrapProp import added when used |
| test_no_wrap_local_vars | Local variables NOT wrapped |
| test_wrap_prop_aliased | Aliased props use original key |

## Decisions Made
- **Early props_identifiers population:** Moved from exit_call_expression to enter_call_expression so JSX children have access during traversal
- **Name-only matching:** Match prop identifiers by name only (ignoring ScopeId) since scopes from different traversal phases may differ
- **Wrap all .value access:** Treat any .value member access as potential signal - runtime determines actual signal status

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] props_identifiers empty during JSX processing**
- **Found during:** Task 3 (tests were failing)
- **Issue:** props_identifiers was being populated in exit_call_expression, but JSX children/attributes are processed BEFORE the exit. The map was empty during JSX traversal.
- **Fix:** Moved props_identifiers population to enter_call_expression so it's available when processing JSX children and attributes
- **Files modified:** optimizer/src/transform.rs
- **Commit:** 7ecfae9

---

**Total deviations:** 1 auto-fixed (1 bug - timing issue)
**Impact on plan:** Critical bug fix required for _wrapProp to function correctly.

## Issues Encountered
The props_identifiers timing bug was the main issue - tests helped identify this quickly.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- _wrapProp generation complete for props and signals
- Ready for Plan 04-04: _fnSignal generation for computed expressions
- All 108 tests passing

---
*Phase: 04-props-signals*
*Completed: 2026-01-29*

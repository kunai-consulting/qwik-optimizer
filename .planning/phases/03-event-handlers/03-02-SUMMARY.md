---
phase: 03-event-handlers
plan: 02
subsystem: transform
tags: [rust, event-handlers, jsx, qrl, native-elements, ast-traversal]

# Dependency graph
requires:
  - phase: 03-01
    provides: jsx_event_to_html_attribute for event name transformation
  - phase: 02-qrl-core
    provides: QRL infrastructure, Qrl::new, import_stack, decl_stack, compute_scoped_idents
provides:
  - jsx_element_is_native stack for tracking native vs component elements
  - Event handler attribute name transformation on native elements
  - Event handler QRL transformation for arrow/function expressions
affects: [03-03, event-handler-captures, component-extraction]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Native element detection via JSXElementName case analysis"
    - "Event handler QRL creation in exit_jsx_attribute"
    - "Import stack frame management for event handler functions"

key-files:
  created: []
  modified:
    - optimizer/src/transform.rs
    - optimizer/src/snapshots/qwik_optimizer__js_lib_interface__tests__qrl_nested_component.snap

key-decisions:
  - "Native element detection uses JSXElementName variant matching with case-sensitivity"
  - "Event handler QRL transformation mirrors exit_call_expression pattern"
  - "QRL import added via import_stack.last_mut() for event handlers"

patterns-established:
  - "jsx_element_is_native stack pushed in enter_jsx_element, popped in exit_jsx_element"
  - "Event handler detection: attr ends with MARKER_SUFFIX and value is arrow/function expression"
  - "QRL creation flow: collect idents -> compute scoped_idents -> filter imports -> create Qrl -> transform"

# Metrics
duration: 8min
completed: 2026-01-29
---

# Phase 03 Plan 02: JSX Attribute Transformation Summary

**Native element detection and event handler QRL transformation converting onClick$={() => {}} on <button> to on:click={qrl(...)} with capture array support**

## Performance

- **Duration:** 8 min
- **Started:** 2026-01-29T00:00:00Z
- **Completed:** 2026-01-29T00:08:00Z
- **Tasks:** 4
- **Files modified:** 2

## Accomplishments
- Added `jsx_element_is_native` stack to track native HTML vs component elements during JSX traversal
- Implemented event handler attribute name transformation (onClick$ -> on:click) on native elements only
- Integrated with QRL infrastructure to transform arrow/function expressions to QRL calls
- Added 2 integration tests verifying transformation behavior on native and component elements
- Updated snapshot to reflect correct event handler transformation output

## Task Commits

Each task was committed atomically:

1. **Task 1: Add native element tracking stack** - `fd0fffb` (feat)
2. **Task 2: Transform event handler attribute names on native elements** - `0a98f35` (feat)
3. **Task 3: Set up QRL context for event handler functions** - `a4c95c4` (feat)
4. **Task 4: Add integration test for event handler transformation** - `b4211a7` (test)

## Files Created/Modified
- `optimizer/src/transform.rs` - Added jsx_element_is_native stack, event handler transformation logic, and tests
- `optimizer/src/snapshots/qwik_optimizer__js_lib_interface__tests__qrl_nested_component.snap` - Updated to reflect new event handler QRL output

## Decisions Made
- Native element detection based on JSXElementName variant: JSXIdentifier (lowercase) = native, IdentifierReference checks first char case, MemberExpression/ThisExpression = component
- Event handler QRL transformation uses same pattern as exit_call_expression (IdentCollector -> compute_scoped_idents -> filter imports -> Qrl::new)
- Using container.expression.as_expression() for OXC 0.111.0 API compatibility

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed OXC API for JSXExpression pattern matching**
- **Found during:** Task 3 (QRL context setup)
- **Issue:** Plan used JSXExpression::Expression which doesn't exist in OXC 0.111.0
- **Fix:** Used container.expression.as_expression() method instead of pattern matching
- **Files modified:** optimizer/src/transform.rs
- **Verification:** cargo check passes, tests pass
- **Committed in:** a4c95c4 (Task 3 commit)

**2. [Rule 3 - Blocking] Fixed Source API in integration tests**
- **Found during:** Task 4 (integration tests)
- **Issue:** Source::from_string doesn't exist, Language::Tsx doesn't exist
- **Fix:** Used Source::from_source with Language::Typescript
- **Files modified:** optimizer/src/transform.rs
- **Verification:** Tests compile and pass
- **Committed in:** b4211a7 (Task 4 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking - API compatibility)
**Impact on plan:** Both fixes necessary for API compatibility with actual codebase. No scope creep.

## Issues Encountered
- Snapshot test `test_qrl_nested_component` initially failed because expected output changed - this was expected behavior showing transformation works correctly. Snapshot updated with `cargo insta accept`.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Native element tracking works correctly for nested JSX
- Event handler attribute names transform on native elements
- Event handler functions create QRL calls with capture arrays
- All 70 tests pass including 2 new event handler tests
- Ready for Plan 03-03: Event Handler Edge Cases and Validation

---
*Phase: 03-event-handlers*
*Completed: 2026-01-29*

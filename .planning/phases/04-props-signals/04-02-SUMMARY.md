---
phase: 04-props-signals
plan: 02
subsystem: props-transformation
tags: [qwik, oxc, rust, props, destructuring, rest-pattern, _restProps]

# Dependency graph
requires:
  - phase: 04-01
    provides: PropsDestructuring struct with identifiers HashMap, transform_component_props method
provides:
  - Rest props extraction (rest_id, omit_keys fields)
  - _restProps statement generation
  - _restProps import injection
  - Aliased props tracking
affects: [04-03, 04-04, 04-05]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "rest_id Option<Id> pattern for optional rest extraction"
    - "omit_keys Vec<String> for collecting prop names to exclude"
    - "generate_rest_stmt method for AST node creation"
    - "Statement injection via arrow.body.statements.insert(0, stmt)"

key-files:
  created: []
  modified:
    - optimizer/src/component/shared.rs
    - optimizer/src/props_destructuring.rs
    - optimizer/src/transform.rs

key-decisions:
  - "Use ScopeId::new(0) for rest_id since we match by name later"
  - "Generate _restProps call with or without omit array based on presence of explicit props"
  - "Handle expression body by converting to block with rest stmt + return"

patterns-established:
  - "Statement injection pattern: handle expression vs block body separately"
  - "Rest props generation: _restProps(_rawProps) or _restProps(_rawProps, [omit_list])"

# Metrics
duration: 5min
completed: 2026-01-29
---

# Phase 4 Plan 02: Rest Props Summary

**Rest props extraction and _restProps statement generation with omit array for explicit prop names**

## Performance

- **Duration:** 5 min
- **Started:** 2026-01-29T19:27:46Z
- **Completed:** 2026-01-29T19:33:03Z
- **Tasks:** 4
- **Files modified:** 3

## Accomplishments
- PropsDestructuring extracts rest pattern and generates _restProps call
- Omit array includes all explicit prop names (["message", "id", "count"])
- Rest-only pattern works (empty omit array)
- _restProps import added from @qwik.dev/core when rest pattern used
- 86 tests passing (4 new rest props tests)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add _REST_PROPS import constant** - `1b8f3d0` (chore)
2. **Task 2: Implement rest props extraction and statement generation** - `49108f6` (feat)
3. **Task 3: Integrate rest statement injection into transform** - `1a46626` (feat)
4. **Task 4: Add tests for rest props and aliasing** - `eac74b1` (test)

## Files Created/Modified
- `optimizer/src/component/shared.rs` - Added _REST_PROPS constant
- `optimizer/src/props_destructuring.rs` - Extended struct with rest_id, omit_keys; added generate_rest_stmt method
- `optimizer/src/transform.rs` - Added rest statement injection and _restProps import; added 4 tests

## Decisions Made
- **ScopeId for rest_id:** Used ScopeId::new(0) since we match identifiers by name later, not by scope
- **OXC 0.111.0 API:** Used expression_identifier() not expression_identifier_reference(); expression_array() takes 2 args not 3; variable_declarator() has type_annotation parameter
- **Body handling:** Handle arrow.expression flag to determine if body is expression or block statement

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] OXC 0.111.0 API differences**
- **Found during:** Task 2 (generate_rest_stmt implementation)
- **Issue:** Plan used `expression_identifier_reference` and `expression_array` with wrong args
- **Fix:** Used correct OXC 0.111.0 APIs: `expression_identifier()`, `expression_array(SPAN, elements)` without trailing None
- **Files modified:** optimizer/src/props_destructuring.rs
- **Verification:** cargo build succeeds
- **Committed in:** 49108f6

**2. [Rule 3 - Blocking] FunctionBody structure in OXC 0.111.0**
- **Found during:** Task 3 (rest statement injection)
- **Issue:** Plan assumed FunctionBody is enum with BlockStatement/Expression variants; it's actually a struct with statements field
- **Fix:** Used arrow.expression flag and arrow.body.statements directly, following code_move.rs pattern
- **Files modified:** optimizer/src/transform.rs
- **Verification:** cargo build succeeds
- **Committed in:** 1a46626

---

**Total deviations:** 2 auto-fixed (2 blocking - OXC API)
**Impact on plan:** Both auto-fixes necessary for OXC 0.111.0 compatibility. No scope creep.

## Issues Encountered
None - all issues were API compatibility deviations handled automatically.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Rest props transformation complete
- Ready for Plan 04-03: Identifier replacement with _wrapProp
- Aliased props correctly tracked (c -> "count"), ready for replacement logic

---
*Phase: 04-props-signals*
*Completed: 2026-01-29*

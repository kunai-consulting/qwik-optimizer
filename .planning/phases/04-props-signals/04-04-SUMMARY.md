---
phase: 04-props-signals
plan: 04
subsystem: optimizer
tags: [fnSignal, hoisted-functions, computed-expressions, oxc, codegen]

# Dependency graph
requires:
  - phase: 04-02
    provides: props_destructuring module, _rawProps transformation
provides:
  - _fnSignal module for computed expression wrapping
  - should_wrap_in_fn_signal detection function
  - convert_inlined_fn hoisted function generation
  - TransformGenerator hoisted function tracking
  - InlinedFnResult structure with captures
affects: [04-05-computed-signals, JSX-attribute-processing]

# Tech tracking
tech-stack:
  added: []
  patterns: [hoisted-function-generation, identifier-replacement, expression-serialization]

key-files:
  created:
    - optimizer/src/inlined_fn.rs
  modified:
    - optimizer/src/lib.rs
    - optimizer/src/component/shared.rs
    - optimizer/src/transform.rs

key-decisions:
  - "Use is_used_as_object_or_call for dual detection of member access and call patterns"
  - "Filter call expressions from _fnSignal wrapping (can't serialize function calls)"
  - "Use IdentifierReplacer for AST-level identifier transformation to positional params"
  - "Expression serialization via OXC Codegen with minify option"
  - "MAX_EXPR_LENGTH 150 chars for _fnSignal wrapping threshold"

patterns-established:
  - "Hoisted function naming: _hfN for function, _hfN_str for string representation"
  - "Positional parameters: p0, p1, p2, ... for captured identifiers"
  - "InlinedFnResult struct: hoisted_fn, hoisted_name, hoisted_str, captures, is_const"

# Metrics
duration: 8min
completed: 2026-01-29
---

# Phase 4 Plan 04: _fnSignal Generation Summary

**_fnSignal infrastructure module with should_wrap_in_fn_signal detection, convert_inlined_fn hoisted function generation, and TransformGenerator integration for computed expression wrapping**

## Performance

- **Duration:** 8 min
- **Started:** 2026-01-29
- **Completed:** 2026-01-29
- **Tasks:** 4
- **Files modified:** 4

## Accomplishments
- Created inlined_fn.rs module with complete _fnSignal generation infrastructure
- Implemented should_wrap_in_fn_signal for detecting member access patterns requiring wrapping
- Implemented convert_inlined_fn for generating hoisted arrow functions with positional params
- Integrated hoisted function tracking into TransformGenerator (fields, counter, import flag)
- Added 16 unit tests for _fnSignal functionality (9 in inlined_fn.rs, 7 in transform.rs)
- 102 tests passing total

## Task Commits

Each task was committed atomically:

1. **Task 1: Add _FN_SIGNAL import constant and create module** - `04a0316` (feat)
2. **Task 2: Implement convert_inlined_fn for hoisted function generation** - `a7fd92d` (test)
3. **Task 3: Integrate _fnSignal into JSX attribute processing** - `4f00f7a` (feat)
4. **Task 4: Add tests for _fnSignal generation** - `f192c72` (test)

## Files Created/Modified
- `optimizer/src/inlined_fn.rs` - New module for _fnSignal generation with should_wrap_in_fn_signal, convert_inlined_fn, ObjectUsageChecker, IdentifierReplacer, render_expression
- `optimizer/src/lib.rs` - Added inlined_fn module export
- `optimizer/src/component/shared.rs` - Added _FN_SIGNAL constant
- `optimizer/src/transform.rs` - Added hoisted_fns, hoisted_fn_counter, needs_fn_signal_import fields and create_fn_signal_call helper

## Decisions Made
- Use dual detection (is_used_as_object_or_call) to filter both member access patterns and call expressions
- Filter call expressions from _fnSignal wrapping since function calls cannot be serialized
- Use IdentifierReplacer visitor for AST-level identifier transformation rather than string manipulation
- Use OXC Codegen with minify option for expression serialization
- MAX_EXPR_LENGTH of 150 chars matches SWC reference implementation
- Hoisted functions stored as Vec<(String, Expression, String)> for name/fn/str tuple

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed BinaryOperator::LogicalOr pattern matching**
- **Found during:** Task 1 (Module creation)
- **Issue:** OXC 0.111.0 uses Expression::LogicalExpression not BinaryOperator::LogicalOr
- **Fix:** Changed pattern matching to use LogicalExpression variant
- **Files modified:** optimizer/src/inlined_fn.rs
- **Verification:** Build passes
- **Committed in:** 04a0316 (Task 1 commit)

**2. [Rule 3 - Blocking] Fixed OXC 0.111.0 API differences**
- **Found during:** Task 1-3
- **Issue:** Multiple API differences from SWC reference (binding_pattern, FormalParameter fields, expression_identifier vs expression_identifier_reference)
- **Fix:** Updated to OXC 0.111.0 patterns: direct struct construction, expression_identifier()
- **Files modified:** optimizer/src/inlined_fn.rs
- **Verification:** All tests pass
- **Committed in:** Multiple task commits

**3. [Rule 1 - Bug] Fixed syntax error in test module**
- **Found during:** Task 4 (Test addition)
- **Issue:** Misplaced closing brace at line 2730 closed test module prematurely
- **Fix:** Removed errant closing brace before _wrapProp tests section comment
- **Files modified:** optimizer/src/transform.rs
- **Verification:** 102 tests pass
- **Committed in:** f192c72 (Task 4 commit)

---

**Total deviations:** 3 auto-fixed (2 bugs, 1 blocking)
**Impact on plan:** All auto-fixes necessary for OXC 0.111.0 compatibility and test correctness. No scope creep.

## Issues Encountered
- ObjectProperty struct in OXC 0.111.0 has no 'init' field - removed it
- Codegen print_expression returns () not String - changed to into_source_text()
- Lifetime issues with string parameters in create_fn_signal_call - added 'b lifetime annotation

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- _fnSignal infrastructure complete with detection and conversion functions
- TransformGenerator has fields for tracking hoisted functions
- Ready for Plan 05 (computed signal JSX attribute integration)
- Actual wrapping in exit_jsx_attribute not yet implemented - infrastructure only

---
*Phase: 04-props-signals*
*Completed: 2026-01-29*

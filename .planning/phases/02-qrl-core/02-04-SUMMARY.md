---
phase: 02-qrl-core
plan: 04
subsystem: optimizer
tags: [oxc, ast, code-generation, useLexicalScope, qrl]

# Dependency graph
requires:
  - phase: 02-02
    provides: compute_scoped_idents and decl_stack tracking
  - phase: 02-03
    provides: SegmentData structure with scoped_idents field
provides:
  - transform_function_expr for useLexicalScope injection
  - create_use_lexical_scope statement generation
  - Arrow expression body to block body conversion
  - QrlComponent integration with code_move
affects: [03-segment-output, 04-qrl-emit]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - OXC AST builder pattern for array destructuring (binding_pattern_array_pattern)
    - Expression body to block body conversion for arrow functions

key-files:
  created:
    - optimizer/src/code_move.rs
  modified:
    - optimizer/src/lib.rs
    - optimizer/src/component/component.rs

key-decisions:
  - "Used OXC's binding_pattern_array_pattern for destructuring pattern creation"
  - "scoped_idents passed as slice reference to avoid ownership issues"
  - "Transformation applied conditionally when scoped_idents is non-empty"

patterns-established:
  - "Arrow function expression body conversion: () => expr becomes () => { const [...] = useLexicalScope(); return expr; }"
  - "Function body prepending: useLexicalScope statement inserted at index 0"

# Metrics
duration: 5min
completed: 2026-01-29
---

# Phase 02 Plan 04: Code Move Summary

**useLexicalScope injection via transform_function_expr with OXC AST builder for const [a, b, c] destructuring**

## Performance

- **Duration:** 5 min
- **Started:** 2026-01-29T08:42:39Z
- **Completed:** 2026-01-29T08:47:44Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- Created code_move.rs module porting SWC's code_move.rs lines 175-290
- Implemented transform_function_expr dispatching to arrow/function transformers
- Arrow expression body `() => x` converted to `() => { const [...] = useLexicalScope(); return x; }`
- Arrow block body has useLexicalScope prepended
- Function expressions have useLexicalScope prepended to body
- QrlComponent::gen applies transformation when segment has captures

## Task Commits

Each task was committed atomically:

1. **Task 1: Create code_move.rs with useLexicalScope injection** - `6c06532` (feat)
2. **Task 2: Add unit tests for code_move transformations** - `ec3a25e` (test)
3. **Task 3: Integrate code_move into QrlComponent code generation** - `0f6f8f6` (feat)

## Files Created/Modified

- `optimizer/src/code_move.rs` - New module with transform_function_expr, transform_arrow_fn, transform_fn, create_use_lexical_scope
- `optimizer/src/lib.rs` - Added `pub mod code_move;`
- `optimizer/src/component/component.rs` - Imports transform_function_expr, applies in gen() when scoped_idents non-empty

## Decisions Made

1. **Used OXC's binding_pattern_array_pattern**: This creates the array destructuring pattern `[a, b, c]` directly as a BindingPattern, avoiding the need to manually construct BindingPatternKind enum variants.

2. **Atom allocation for string lifetimes**: Used `ast.atom(name.as_str())` to allocate identifier names in the arena, ensuring proper lifetime management for scoped_idents strings.

3. **Conditional transformation**: transform_function_expr returns expression unchanged when scoped_idents is empty, avoiding unnecessary AST modifications.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

1. **OXC API differences from SWC**: The OXC `binding_pattern_array_pattern` API differs from SWC's manual ArrayPat construction. Resolved by using the builder method directly.

2. **ParenthesizedExpression in tests**: When parsing `(function() {...})`, OXC wraps it in ParenthesizedExpression. Added unwrapping logic in test helper.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- code_move.rs ready for use in segment file generation
- transform_function_expr can be called from any code path with Expression + scoped_idents
- Ready for Plan 05 (QRL reference generation) or Phase 03 (Segment Output)

---
*Phase: 02-qrl-core*
*Completed: 2026-01-29*

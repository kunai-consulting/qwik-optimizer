---
phase: 02-qrl-core
plan: 01
subsystem: compiler
tags: [oxc, ast-visitor, identifier-collection, qrl, captures]

# Dependency graph
requires:
  - phase: 01-oxc-foundation
    provides: OXC crate setup and Visit trait patterns
provides:
  - IdentCollector visitor for variable usage collection in QRL expressions
  - ExprOrSkip context tracking for expression vs statement contexts
  - JSX element and fragment tracking (use_h, use_fragment flags)
affects: [02-qrl-core plans 02-05, lexical-capture, qrl-generation]

# Tech tracking
tech-stack:
  added: []
  patterns: [oxc_ast_visit::Visit trait implementation, walk functions for visitor pattern]

key-files:
  created:
    - optimizer/src/collector.rs
  modified:
    - optimizer/src/lib.rs

key-decisions:
  - "Used (String, ScopeId) for Id type to match OXC's identifier model"
  - "Included unit tests in same file using Rust #[cfg(test)] convention"
  - "Used walk functions (walk_expression, etc.) for explicit child traversal"

patterns-established:
  - "OXC Visit pattern: push context, walk children, pop context"
  - "Builtin exclusion via const array lookup for undefined, NaN, Infinity, null"

# Metrics
duration: 8min
completed: 2026-01-29
---

# Phase 2 Plan 1: IdentCollector Summary

**OXC-based IdentCollector for variable usage collection with expression context tracking and JSX support**

## Performance

- **Duration:** 8 min
- **Started:** 2026-01-29
- **Completed:** 2026-01-29
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Ported IdentCollector from SWC to OXC with full Visit trait implementation
- Implemented expression context tracking (ExprOrSkip enum) for accurate identifier collection
- Built-in identifier exclusion (undefined, NaN, Infinity, null)
- Property key and member expression property skipping
- JSX element and fragment tracking (use_h, use_fragment flags)
- 7 comprehensive unit tests all passing

## Task Commits

Each task was committed atomically:

1. **Task 1: Create collector.rs with IdentCollector** - `50bcecd` (feat)
2. **Task 2: Add unit tests for IdentCollector** - included in Task 1 commit (Rust #[cfg(test)] pattern)

_Note: Tests included inline per Rust convention for unit tests_

## Files Created/Modified
- `optimizer/src/collector.rs` - IdentCollector visitor for variable usage collection
- `optimizer/src/lib.rs` - Module export for collector

## Decisions Made
- Used `(String, ScopeId)` as the Id type, adapting from SWC's `(Atom, SyntaxContext)` pattern to OXC's identifier model
- Included unit tests in the same file using Rust's `#[cfg(test)] mod tests` convention rather than a separate file
- Used explicit `oxc_ast_visit::walk::*` functions for child traversal instead of relying on default implementations

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - implementation followed SWC reference directly with OXC API adaptations.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- IdentCollector ready for use in QRL expression analysis
- get_words() provides sorted identifiers for deterministic capture arrays
- use_h and use_fragment flags ready for JSX import generation
- Next: GlobalCollect for module-level scope tracking (02-02-PLAN.md)

---
*Phase: 02-qrl-core*
*Completed: 2026-01-29*

---
phase: 08-ssr-build-modes
plan: 02
subsystem: compiler
tags: [ssr, build-modes, const-replacement, dead-code-elimination, oxc]

# Dependency graph
requires:
  - phase: 08-01
    provides: "ImportTracker, IS_SERVER/IS_BROWSER/IS_DEV constants, TransformOptions.is_server"
provides:
  - "ConstReplacerVisitor for SSR/build mode const replacement"
  - "isServer/isBrowser/isDev identifier replacement with boolean literals"
  - "Support for aliased imports and both @qwik.dev/core sources"
affects: [08-03-ssr-integration]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Custom AST visitor with allocator for OXC expression replacement"
    - "Import tracking for aliased identifier resolution"

key-files:
  created:
    - optimizer/src/const_replace.rs
  modified:
    - optimizer/src/lib.rs

key-decisions:
  - "Used manual visitor pattern with allocator instead of VisitMut trait to create BooleanLiteral nodes"
  - "OXC Expression variants StaticMemberExpression/ComputedMemberExpression accessed directly (not via MemberExpression wrapper)"
  - "isBrowser value is inverse of is_server (!is_server) matching SWC behavior"

patterns-established:
  - "ConstReplacerVisitor pattern: store allocator reference for AST node creation during traversal"
  - "Identifier matching via ImportTracker.get_imported_local() for aliased import support"

# Metrics
duration: 4min
completed: 2026-01-29
---

# Phase 08 Plan 02: Const Replacement Summary

**ConstReplacerVisitor replaces isServer/isBrowser/isDev imports with boolean literals enabling bundler dead code elimination**

## Performance

- **Duration:** 4 min
- **Started:** 2026-01-29T23:50:42Z
- **Completed:** 2026-01-29T23:54:51Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Created ConstReplacerVisitor module for SSR/build mode const replacement
- isServer identifiers replaced with is_server boolean literal
- isBrowser identifiers replaced with !is_server (inverse of isServer)
- isDev identifiers replaced with is_dev boolean literal
- Aliased imports handled correctly (import { isServer as s })
- Both @qwik.dev/core and @qwik.dev/core/build sources supported
- 12 unit tests covering all functionality and edge cases

## Task Commits

Each task was committed atomically:

1. **Task 1: Create const_replace.rs module** - `0b9e521` (feat)
2. **Task 2: Verify edge cases** - `84295ed` (test)

## Files Created/Modified
- `optimizer/src/const_replace.rs` - ConstReplacerVisitor with visit_program/visit_expression for const replacement
- `optimizer/src/lib.rs` - Added `pub mod const_replace` export

## Decisions Made
- **Manual visitor over VisitMut:** OXC's VisitMut trait doesn't provide allocator access needed to create BooleanLiteral nodes. Implemented custom visit_* methods that store allocator reference.
- **Direct member expression variants:** OXC flattens MemberExpression subtypes directly onto Expression enum (StaticMemberExpression, ComputedMemberExpression, PrivateFieldExpression) rather than nesting them.
- **Import tracking pattern:** Leveraged ImportTracker from 08-01 to resolve aliased imports, ensuring `import { isServer as s }` correctly replaces `s` with the boolean value.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] OXC allocator Box type mismatch**
- **Found during:** Task 1 (Initial implementation)
- **Issue:** OXC Expression::BooleanLiteral expects `oxc_allocator::Box`, not `std::boxed::Box`
- **Fix:** Used `OxcBox::new_in(BooleanLiteral {...}, self.allocator)` pattern
- **Files modified:** optimizer/src/const_replace.rs
- **Verification:** cargo build succeeds, tests pass
- **Committed in:** 0b9e521 (Task 1 commit)

**2. [Rule 3 - Blocking] Private module access for constants**
- **Found during:** Task 1 (Initial implementation)
- **Issue:** `crate::component::shared::` is private; constants re-exported via `crate::component::`
- **Fix:** Changed import to `use crate::component::{IS_BROWSER, IS_DEV, ...}`
- **Files modified:** optimizer/src/const_replace.rs
- **Verification:** cargo build succeeds
- **Committed in:** 0b9e521 (Task 1 commit)

**3. [Rule 3 - Blocking] MemberExpression variant structure**
- **Found during:** Task 1 (Initial implementation)
- **Issue:** OXC doesn't have `Expression::MemberExpression` wrapper; subtypes are direct variants
- **Fix:** Changed to `Expression::StaticMemberExpression`, `Expression::ComputedMemberExpression`, `Expression::PrivateFieldExpression`
- **Files modified:** optimizer/src/const_replace.rs
- **Verification:** cargo build succeeds, tests pass
- **Committed in:** 0b9e521 (Task 1 commit)

---

**Total deviations:** 3 auto-fixed (3 blocking - OXC API differences)
**Impact on plan:** All auto-fixes necessary to adapt plan's SWC-style code to OXC. No scope creep.

## Issues Encountered
- OXC's allocator-based AST requires different patterns than SWC's standard Rust types. Resolved by referencing existing codebase patterns (IdentifierReplacer in inlined_fn.rs, OxcBox usage in transform.rs).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- ConstReplacerVisitor ready for integration in transform pipeline
- Next plan (08-03) will integrate visitor into TransformGenerator and add integration tests
- ImportTracker population in TransformGenerator already exists from 08-01

---
*Phase: 08-ssr-build-modes*
*Completed: 2026-01-29*

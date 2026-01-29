---
phase: 05-jsx-transformation
plan: 02
subsystem: jsx
tags: [fragment, jsx-runtime, _jsxSorted, _Fragment]

# Dependency graph
requires:
  - phase: 05-01
    provides: JSX element transformation infrastructure
provides:
  - Fragment handling for implicit (<></>) and explicit (<Fragment>) JSX fragments
  - _Fragment import from @qwik.dev/core/jsx-runtime
  - Key generation for fragments inside components
affects: [05-03, 05-04, 05-05, jsx-transformation]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Fragment transformation via _jsxSorted(_Fragment, null, null, children, flags, key)"
    - "JSX_RUNTIME_SOURCE constant for jsx-runtime imports"

key-files:
  created: []
  modified:
    - optimizer/src/component/shared.rs
    - optimizer/src/transform.rs

key-decisions:
  - "is_fn=true for fragments to enable key generation inside components"
  - "Single child passed directly, multiple children as array (matching SWC)"
  - "Explicit <Fragment> uses user-imported identifier, implicit uses _Fragment"

patterns-established:
  - "JSX_RUNTIME_SOURCE: @qwik.dev/core/jsx-runtime for Fragment import"
  - "_FRAGMENT constant for implicit fragment identifier"
  - "Fragment import: ImportId::NamedWithAlias('Fragment', '_Fragment')"

# Metrics
duration: 4min
completed: 2026-01-29
---

# Phase 05 Plan 02: Fragment Handling Summary

**Implicit fragments (<></>) transformed to _jsxSorted(_Fragment, ...) with proper jsx-runtime import**

## Performance

- **Duration:** 4 min
- **Started:** 2026-01-29T20:46:24Z
- **Completed:** 2026-01-29T20:50:51Z
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments
- Implicit fragments transform to `_jsxSorted(_Fragment, null, null, children, flags, key)`
- `Fragment as _Fragment` import added from `@qwik.dev/core/jsx-runtime`
- Explicit `<Fragment>` uses user-imported identifier (not _Fragment)
- Keys generated for fragments inside components
- 127 tests passing (3 new fragment tests)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add Fragment constants** - `239459f` (feat)
2. **Task 2: Transform implicit fragments to _jsxSorted(_Fragment, ...)** - `f1b6128` (feat)
3. **Task 3: Handle explicit Fragment component** - `18b733f` (test)

## Files Created/Modified
- `optimizer/src/component/shared.rs` - Added JSX_RUNTIME_SOURCE and _FRAGMENT constants
- `optimizer/src/transform.rs` - Updated exit_jsx_fragment to generate _jsxSorted call, added tests

## Decisions Made
- **is_fn=true for fragments:** Changed enter_jsx_fragment to set is_fn=true so fragments generate keys like component elements when inside components
- **Single child optimization:** Pass single child directly instead of wrapping in array (matches SWC output)
- **Flags calculation:** Same as elements (static_subtree | static_listeners)

## Deviations from Plan
None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Fragment transformation complete and tested
- Ready for Phase 05-03 (null prop handling) and 05-04 (children optimization)
- All fragment tests passing with exact SWC output parity

---
*Phase: 05-jsx-transformation*
*Completed: 2026-01-29*

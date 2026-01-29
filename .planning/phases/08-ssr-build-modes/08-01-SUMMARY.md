---
phase: 08-ssr-build-modes
plan: 01
subsystem: optimizer
tags: [ssr, build-modes, is-server, is-browser, is-dev, const-replacement]

# Dependency graph
requires:
  - phase: 07-entry-strategies
    provides: TransformGenerator infrastructure, stack_ctxt tracking
provides:
  - TransformOptions with is_server and is_dev fields
  - Build mode constants (QWIK_CORE_BUILD, IS_SERVER, IS_BROWSER, IS_DEV)
  - ImportTracker for tracking imported identifiers by source and specifier
  - get_imported_local method for finding aliased imports
affects: [08-02-const-replacement-visitor, 08-03-import-removal]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - ImportTracker pattern for tracking aliased imports by (source, specifier) key

key-files:
  created: []
  modified:
    - optimizer/src/component/shared.rs
    - optimizer/src/transform.rs
    - optimizer/src/js_lib_interface.rs

key-decisions:
  - "is_server defaults to true (safe default - server code is safer)"
  - "is_dev derived from target (not stored separately) matching SWC implementation"
  - "ImportTracker uses (source, specifier) tuple as key for efficient lookup"

patterns-established:
  - "ImportTracker pattern: track imports by source+specifier for const replacement"
  - "Build mode fields: is_server field with is_dev() method pattern"

# Metrics
duration: 5min
completed: 2026-01-29
---

# Phase 08 Plan 01: Infrastructure Summary

**TransformOptions extended with is_server/is_dev fields, build mode constants defined, and ImportTracker added for tracking aliased imports from @qwik.dev/core/build**

## Performance

- **Duration:** 5 min
- **Started:** 2026-01-29T23:42:01Z
- **Completed:** 2026-01-29T23:46:30Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments
- Added QWIK_CORE_BUILD, IS_SERVER, IS_BROWSER, IS_DEV constants to shared.rs
- Extended TransformOptions with is_server field (default: true) and is_dev() method
- Added ImportTracker struct for tracking imports by source and specifier
- Integrated ImportTracker into TransformGenerator for const replacement support
- Added 3 unit tests for ImportTracker functionality
- All 180 tests passing (177 existing + 3 new)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add build mode constants to shared.rs** - `af01028` (feat)
2. **Task 2: Extend TransformOptions with is_server and is_dev** - `48a5ec0` (feat)
3. **Task 3: Add get_imported_local helper for import tracking** - `0b95a9f` (feat)

## Files Created/Modified
- `optimizer/src/component/shared.rs` - Added QWIK_CORE_BUILD, IS_SERVER, IS_BROWSER, IS_DEV constants
- `optimizer/src/transform.rs` - Added is_server field, is_dev() method, ImportTracker struct
- `optimizer/src/js_lib_interface.rs` - Updated TransformOptions construction with is_server field

## Decisions Made
- is_server defaults to true (safe default - server code won't break on server, client code might)
- is_dev() is derived from target == Target::Dev (not stored separately), matching SWC implementation
- ImportTracker uses (source, specifier) tuple as HashMap key for efficient O(1) lookup

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated js_lib_interface.rs TransformOptions construction**
- **Found during:** Task 2 (Extend TransformOptions)
- **Issue:** Adding is_server field caused compilation error in js_lib_interface.rs where TransformOptions was constructed without the new field
- **Fix:** Added is_server: true to TransformOptions construction in js_lib_interface.rs
- **Files modified:** optimizer/src/js_lib_interface.rs
- **Verification:** cargo build succeeds
- **Committed in:** 48a5ec0 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary fix for compilation. No scope creep.

## Issues Encountered
None - plan executed as specified with one auto-fix for struct field addition.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Infrastructure complete for const replacement visitor (plan 08-02)
- ImportTracker ready to find isServer/isBrowser/isDev imports
- TransformOptions.is_server and is_dev() ready to provide replacement values
- Constants defined for identifier matching

---
*Phase: 08-ssr-build-modes*
*Completed: 2026-01-29*

---
phase: 08-ssr-build-modes
plan: 03
subsystem: transform
tags: [ssr, const-replacement, dead-code-elimination, isServer, isBrowser, isDev]

# Dependency graph
requires:
  - phase: 08-02
    provides: ConstReplacerVisitor module for SSR/build mode const replacement
provides:
  - Integrated const replacement in transform pipeline
  - SSR integration tests verifying SSR-01 through SSR-05
  - Export declaration handling for const replacement
affects: [09-validation-testing, 10-error-recovery, 11-optimization]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Const replacement runs after import collection, before QRL transformation"
    - "Test mode skips const replacement (matching SWC behavior)"

key-files:
  created: []
  modified:
    - "optimizer/src/transform.rs"
    - "optimizer/src/const_replace.rs"

key-decisions:
  - "Import collection and const replacement happen before semantic analysis"
  - "Export declarations (ExportNamedDeclaration, ExportDefaultDeclaration) require explicit handling in visitor"
  - "Test mode skips const replacement to match SWC behavior"

patterns-established:
  - "SSR const replacement: isServer/isBrowser/isDev replaced with boolean literals based on build config"
  - "DCE pattern: if(false) for bundler dead code elimination"

# Metrics
duration: 7min
completed: 2026-01-30
---

# Phase 8 Plan 3: Transform Pipeline Integration Summary

**ConstReplacerVisitor integrated into transform pipeline with full SSR requirements coverage (SSR-01 through SSR-05)**

## Performance

- **Duration:** 7 min
- **Started:** 2026-01-29T23:58:27Z
- **Completed:** 2026-01-30T00:05:31Z
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments

- Integrated ConstReplacerVisitor into transform pipeline
- Added 10 SSR integration tests covering all requirements
- Fixed const replacement to handle export declarations
- Documented SSR requirements in module docs

## Task Commits

Each task was committed atomically:

1. **Task 1: Integrate const replacement into transform pipeline** - `c1add4e` (feat)
2. **Task 2: Add integration tests for SSR requirements** - `e52402b` (test)
3. **Task 3: Verify all tests pass and document requirements** - `9694a77` (docs)

## Files Created/Modified

- `optimizer/src/transform.rs` - Integrated const replacement, added 10 SSR integration tests
- `optimizer/src/const_replace.rs` - Added export declaration handling, updated module documentation

## Decisions Made

- **Import collection before const replacement**: Collect imports from program body before running const replacement, enables aliased import tracking
- **Export declaration handling**: Added `visit_export_named_declaration` and `visit_export_default_declaration` methods to handle `export const foo = isServer;` patterns
- **Test mode skip**: Const replacement skips when `target == Target::Test` to match SWC behavior

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed const replacement not handling export declarations**
- **Found during:** Task 2 (SSR integration tests)
- **Issue:** `export const serverCheck = isServer;` was not being transformed because ExportNamedDeclaration wraps the VariableDeclaration
- **Fix:** Added `visit_export_named_declaration` and `visit_export_default_declaration` methods to ConstReplacerVisitor
- **Files modified:** optimizer/src/const_replace.rs
- **Verification:** All 10 SSR integration tests pass
- **Committed in:** e52402b (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Bug fix necessary for correct const replacement in exported declarations. No scope creep.

## Issues Encountered

- Initial SSR tests failed because export declarations weren't being visited by const replacer - added proper export statement handling

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 8 SSR & Build Modes COMPLETE
- All 5 SSR requirements satisfied (SSR-01 through SSR-05)
- 202 total tests passing (192 existing + 10 SSR integration)
- Ready for Phase 9: Validation & Testing

---
*Phase: 08-ssr-build-modes*
*Completed: 2026-01-30*

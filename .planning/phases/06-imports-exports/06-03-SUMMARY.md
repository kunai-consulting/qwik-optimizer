---
phase: 06-imports-exports
plan: 03
subsystem: transform
tags: [imports, exports, segment-files, qrl, oxc, ast]

# Dependency graph
requires:
  - phase: 06-01
    provides: export_by_name HashMap for tracking module exports
provides:
  - referenced_exports field on Qrl struct
  - Segment file import generation from source exports
  - Default export and aliased export handling
affects: [06-04, testing, segment-generation]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "ExportInfo to ImportId conversion for segment file imports"
    - "Qrl struct stores referenced_exports for segment generation"
    - "QrlComponent::generate_source_file_imports for import generation"

key-files:
  created: []
  modified:
    - "optimizer/src/component/qrl.rs"
    - "optimizer/src/component/segment_data.rs"
    - "optimizer/src/component/component.rs"
    - "optimizer/src/transform.rs"
    - "optimizer/src/collector.rs"

key-decisions:
  - "referenced_exports stored on Qrl struct (not SegmentData) since QrlComponent uses it for import generation"
  - "SegmentData also stores referenced_exports for transport from transform to QrlComponent"
  - "ExportInfo derives PartialOrd, Ord, Serialize for Qrl struct compatibility"
  - "Aliased exports import using exported_name as local_name (expr2 as internal)"

patterns-established:
  - "Default export: import { default as LocalName } from ./source"
  - "Aliased export: import { exported_name as local_name } from ./source"
  - "Direct export: import { name } from ./source"

# Metrics
duration: 13min
completed: 2026-01-29
---

# Phase 06 Plan 03: Segment Import Generation from Source Exports Summary

**Segment files import referenced exports from source file with correct syntax for named, aliased, and default exports**

## Performance

- **Duration:** 13 min
- **Started:** 2026-01-29T22:05:47Z
- **Completed:** 2026-01-29T22:18:33Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments
- Added referenced_exports field to Qrl and SegmentData structs for tracking source exports used in QRL
- Populated referenced_exports during QRL creation in both exit_call_expression and exit_jsx_attribute
- Generated segment file imports from referenced exports with correct syntax for default, aliased, and named exports
- Added 3 comprehensive tests verifying segment import generation

## Task Commits

Each task was committed atomically:

1. **Task 1: Add referenced_exports to Qrl struct** - `5b93cf2` (feat)
2. **Task 2: Populate referenced_exports during QRL creation** - `643c482` (feat)
3. **Task 3: Generate segment file imports from exports** - `4331f61` (feat)

**Tests:** `9a4ca79` (test: add segment file import generation tests)

## Files Created/Modified
- `optimizer/src/component/qrl.rs` - Added referenced_exports field and new_with_exports constructor
- `optimizer/src/component/segment_data.rs` - Added referenced_exports field and new_with_exports constructor
- `optimizer/src/component/component.rs` - Added generate_source_file_imports helper
- `optimizer/src/transform.rs` - Collect referenced_exports in QRL creation paths
- `optimizer/src/collector.rs` - Added PartialOrd, Ord, Serialize derives to ExportInfo

## Decisions Made
- **referenced_exports on both Qrl and SegmentData:** SegmentData transports it from transform.rs to QrlComponent, then QrlComponent extracts it for import generation
- **Import syntax matching SWC reference:** Default exports use `import { default as Name }` pattern, aliased exports use `import { exported_name as local_name }`
- **ExportInfo derives for Qrl compatibility:** Added PartialOrd, Ord, Serialize to ExportInfo since Qrl derives these

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- Pre-existing uncommitted changes from previous session (06-04 tests) were present but did not affect plan execution

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Segment file import generation complete
- 151 tests passing (148 previous + 3 new)
- Ready for Plan 06-04 (Unused import removal)

---
*Phase: 06-imports-exports*
*Completed: 2026-01-29*

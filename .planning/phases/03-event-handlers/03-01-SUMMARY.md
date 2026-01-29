---
phase: 03-event-handlers
plan: 01
subsystem: transform
tags: [rust, event-handlers, jsx, html-attributes, string-transformation]

# Dependency graph
requires:
  - phase: 02-qrl-core
    provides: transform.rs base infrastructure
provides:
  - jsx_event_to_html_attribute function for event name transformation
  - get_event_scope_data_from_jsx_event for prefix extraction
  - create_event_name for camelCase to kebab-case conversion
affects: [03-02, 03-03, event-handler-integration]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Event name transformation from JSX to HTML attribute format"
    - "Prefix-based scope detection (on:, on-document:, on-window:)"
    - "Case-preserving events with dash prefix marker"

key-files:
  created: []
  modified:
    - optimizer/src/transform.rs

key-decisions:
  - "Used usize::MAX as sentinel value for invalid event detection"
  - "Case-preserving events trigger on dash prefix before event name"

patterns-established:
  - "Event transformation: onClick$ -> on:click, document:onFocus$ -> on-document:focus"
  - "Scope prefixes: on: (local), on-document: (document), on-window: (window)"

# Metrics
duration: 3min
completed: 2026-01-29
---

# Phase 03 Plan 01: Event Name Transformation Summary

**Event name transformation utilities for converting Qwik JSX event attributes (onClick$) to HTML attribute format (on:click) with scope prefix and case handling support**

## Performance

- **Duration:** 3 min
- **Started:** 2026-01-29T00:00:00Z
- **Completed:** 2026-01-29T00:03:00Z
- **Tasks:** 3
- **Files modified:** 1

## Accomplishments
- Implemented `get_event_scope_data_from_jsx_event` for extracting prefix and event name start index
- Implemented `create_event_name` for camelCase to kebab-case conversion with case-preserving support
- Implemented `jsx_event_to_html_attribute` main function combining both utilities
- Added 5 comprehensive unit tests covering all event transformation cases

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement event scope data extraction** - `cd6f4fa` (feat)
2. **Task 2: Implement event name creation with case handling** - `40a63ce` (feat)
3. **Task 3: Implement main jsx_event_to_html_attribute function and tests** - `80eecb0` (feat)

## Files Created/Modified
- `optimizer/src/transform.rs` - Added three event transformation functions and unit tests

## Decisions Made
None - followed plan as specified

## Deviations from Plan
None - plan executed exactly as written

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Event name transformation utilities ready for use in JSX attribute processing
- Functions can be integrated into JSX element exit handler in Plan 03-02
- All success criteria met:
  - `jsx_event_to_html_attribute("onClick$")` returns `Some("on:click")`
  - `jsx_event_to_html_attribute("document:onFocus$")` returns `Some("on-document:focus")`
  - `jsx_event_to_html_attribute("window:onClick$")` returns `Some("on-window:click")`
  - `jsx_event_to_html_attribute("on-cLick$")` returns `Some("on:c-lick")`
  - `jsx_event_to_html_attribute("custom$")` returns `None`
  - All 5 unit tests pass

---
*Phase: 03-event-handlers*
*Completed: 2026-01-29*

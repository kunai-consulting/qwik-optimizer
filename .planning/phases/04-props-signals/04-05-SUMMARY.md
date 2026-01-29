---
phase: 04-props-signals
plan: 05
subsystem: jsx-transform
tags: [bind-directives, two-way-binding, signals, jsx, event-handlers]

# Dependency graph
requires:
  - phase: 04-03
    provides: _wrapProp infrastructure and props_identifiers tracking
provides:
  - bind:value transformation to value prop + on:input handler
  - bind:checked transformation to checked prop + on:input handler
  - Handler merging for existing onInput$ handlers
  - _val, _chk, inlinedQrl import generation
affects: [05-advanced-patterns, phase completion]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Bind directive detection via is_bind_directive helper
    - inlinedQrl for inline QRL creation with helpers
    - Handler merging for order-independent event combinations

key-files:
  created: []
  modified:
    - optimizer/src/component/shared.rs
    - optimizer/src/transform.rs

key-decisions:
  - "Process bind directives in exit_jsx_attribute for proper prop insertion"
  - "Check existing on:input in const_props for handler merging"
  - "Order-independent merging handles onInput$ before or after bind"
  - "Unknown bind: directives (not value/checked) pass through unchanged"

patterns-established:
  - "Bind directive pattern: bind:X -> X prop + on:input handler"
  - "Handler merging: combine existing handlers with bind handlers in array"

# Metrics
duration: 8min
completed: 2026-01-29
---

# Phase 04 Plan 05: Bind Directives Summary

**Two-way binding with bind:value and bind:checked transforming to value/checked props plus inlinedQrl event handlers with automatic handler merging**

## Performance

- **Duration:** 8 min
- **Started:** 2026-01-29T20:06:56Z
- **Completed:** 2026-01-29T20:14:49Z
- **Tasks:** 4
- **Files modified:** 2

## Accomplishments
- Bind directive constants (_VAL, _CHK, INLINED_QRL, BIND_PREFIX) added to shared.rs
- Complete bind:value and bind:checked transformation in exit_jsx_attribute
- Handler merging with existing onInput$ handlers (order-independent)
- Import generation for _val, _chk, and inlinedQrl
- 7 new bind directive tests (115 total tests passing)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add _VAL and _CHK import constants** - `e17ee57` (feat)
2. **Task 2+3: Implement bind directive transformation + imports** - `8f5d1d8` (feat)
3. **Task 4: Add tests for bind directives** - `0425771` (test)

## Files Created/Modified
- `optimizer/src/component/shared.rs` - Added _VAL, _CHK, INLINED_QRL, BIND_PREFIX, BIND_VALUE, BIND_CHECKED constants
- `optimizer/src/transform.rs` - Added bind directive transformation:
  - is_bind_directive, create_bind_handler, merge_event_handlers methods
  - pending_bind_directives, pending_on_input_handlers tracking fields
  - needs_val_import, needs_chk_import, needs_inlined_qrl_import flags
  - Bind directive detection in enter_jsx_attribute
  - Bind directive processing in exit_jsx_attribute
  - Import generation in exit_program
  - 7 new tests for bind directive functionality

## Decisions Made
- Merged Tasks 2 and 3 into single commit since import tracking is integral to bind directive implementation
- Process bind directives in exit_jsx_attribute (not element level) to properly add props in order
- Check const_props for existing on:input to enable order-independent handler merging
- Unknown bind: directives (like bind:stuff) pass through unchanged to match SWC behavior
- Added Clone derive to TransformOptions to support test cloning

## Deviations from Plan
None - plan executed exactly as written.

## Issues Encountered
None.

## Next Phase Readiness
- Phase 04 (Props & Signals) complete with all 5 plans executed
- All bind directive transformation working correctly
- 115 tests passing
- Ready for Phase 05 (Advanced Patterns)

---
*Phase: 04-props-signals*
*Completed: 2026-01-29*

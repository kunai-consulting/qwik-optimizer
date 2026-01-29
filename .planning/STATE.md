# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-01-29)

**Core value:** All 162 tests from qwik-core pass with exact output parity to the SWC implementation.
**Current focus:** Phase 4 - Props & Signals (COMPLETE)

## Current Position

Phase: 4 of 11 (Props & Signals) - COMPLETE
Plan: 5 of 5 in Phase 4 COMPLETE
Status: Phase complete - all 5 plans executed
Last activity: 2026-01-29 - Completed 04-05-PLAN.md (Bind Directives)

Progress: [========            ] 38.6% (4/11 phases complete, 17/44 total plans)

## Performance Metrics

**Velocity:**
- Total plans completed: 17
- Average duration: 6.4 min
- Total execution time: 1.9 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-oxc-foundation | 2/2 | 15 min | 7.5 min |
| 02-qrl-core | 7/7 | 51 min | 7.3 min |
| 03-event-handlers | 3/3 | 15 min | 5.0 min |
| 04-props-signals | 5/5 | 36 min | 7.2 min |

**Recent Trend:**
- Last 5 plans: 03-03 (4 min), 04-01 (7 min), 04-02 (5 min), 04-04 (8 min), 04-05 (8 min)
- Phase 4 Props & Signals COMPLETE

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Init]: Update OXC before porting tests - access latest APIs, avoid rework on outdated patterns
- [Init]: Target exact output parity - prevents production regressions when replacing SWC optimizer
- [01-01]: Removed unused CommentKind import rather than updating to renamed enum variant
- [01-01]: OXC 0.111.0 API patterns: binding_pattern_binding_identifier(), FormalParameterRest, Ident->Atom conversion
- [02-01]: Used (String, ScopeId) for Id type to match OXC's identifier model
- [02-01]: Used walk functions for explicit child traversal in Visit implementations
- [02-02]: decl_stack push/pop at function, arrow, class boundaries (not block statements)
- [02-02]: Parameters tracked as Var(false) since they can be reassigned
- [02-02]: Sort output of compute_scoped_idents for deterministic hash computation
- [02-04]: Used OXC's binding_pattern_array_pattern for destructuring pattern creation
- [02-04]: scoped_idents passed as slice reference to avoid ownership issues
- [02-04]: Transformation applied conditionally when scoped_idents is non-empty
- [02-05]: Filter imported identifiers from scoped_idents to avoid capturing variables already handled via imports
- [02-05]: Add scoped_idents field to Qrl struct for capture array generation
- [02-06]: Compare identifiers by name only in compute_scoped_idents (item.0.0 == ident.0), ignoring ScopeId mismatch
- [03-01]: Used usize::MAX as sentinel value for invalid event detection
- [03-01]: Case-preserving events trigger on dash prefix before event name
- [03-02]: Native element detection via JSXElementName variant matching with case-sensitivity
- [03-02]: Event handler QRL transformation mirrors exit_call_expression pattern
- [03-02]: Using container.expression.as_expression() for OXC 0.111.0 API compatibility
- [03-03]: Namespaced JSX attributes (document:onFocus$) require full name helper function
- [03-03]: Property keys use transformed names after event handler processing
- [04-01]: Props transformation must occur BEFORE QRL component extraction
- [04-01]: Use in_component_props flag for detection in enter_ and apply in exit_
- [04-01]: OXC 0.111.0 FormalParameter has 10 fields including initializer, optional, type_annotation
- [04-02]: Use ScopeId::new(0) for rest_id since we match by name later
- [04-02]: Handle arrow.expression flag to determine if body is expression or block statement
- [04-02]: OXC 0.111.0 expression_identifier() not expression_identifier_reference()
- [04-03]: Populate props_identifiers in enter_call_expression (not exit_) so JSX processing has the mapping
- [04-03]: Match props by name only since scope_id from different traversal phases may differ
- [04-04]: Use is_used_as_object_or_call for dual detection of member access and call patterns
- [04-04]: Filter call expressions from _fnSignal wrapping (can't serialize function calls)
- [04-04]: Use IdentifierReplacer visitor for AST-level identifier transformation
- [04-04]: MAX_EXPR_LENGTH 150 chars for _fnSignal wrapping threshold
- [04-05]: Process bind directives in exit_jsx_attribute for proper prop insertion
- [04-05]: Check existing on:input in const_props for order-independent handler merging
- [04-05]: Unknown bind: directives (not value/checked) pass through unchanged

### Pending Todos

None yet.

### Blockers/Concerns

None yet.

## Session Continuity

Last session: 2026-01-29
Stopped at: Completed 04-05-PLAN.md (Bind Directives) - Phase 4 COMPLETE
Resume file: None

## Phase 2 QRL Core Summary

Phase 2 QRL Core complete with all 7 plans executed:

1. **02-01:** IdentCollector for variable usage collection
2. **02-02:** compute_scoped_idents and decl_stack tracking
3. **02-03:** SegmentData structure for QRL metadata
4. **02-04:** code_move.rs for useLexicalScope injection
5. **02-05:** Complete wiring and parity tests
6. **02-06:** Fix ScopeId mismatch in capture detection (gap closure)
7. **02-07:** Verify hash stability and uniqueness (gap closure)

**Key Deliverables:**
- IdentCollector collects all referenced identifiers in QRL bodies
- decl_stack tracks variable declarations across scope boundaries
- compute_scoped_idents determines captured variables (by name matching)
- SegmentData stores all QRL metadata (ctx_name, hash, scoped_idents, parent_segment)
- code_move.rs injects useLexicalScope for captured variables
- qrl() calls include capture arrays as third argument
- 63 tests passing (all QRL requirements satisfied)
- Hash generation stable and unique per QRL

**Requirements satisfied:** QRL-01 through QRL-10 (10/10)

## Phase 3 Event Handlers Summary

Phase 3 Event Handlers complete with all 3 plans executed:

1. **03-01:** Event name transformation utilities - COMPLETE
   - jsx_event_to_html_attribute: onClick$ -> on:click
   - get_event_scope_data_from_jsx_event: prefix extraction
   - create_event_name: camelCase to kebab-case conversion
   - 5 unit tests passing

2. **03-02:** JSX attribute transformation integration - COMPLETE
   - jsx_element_is_native stack for native element tracking
   - Event handler attribute name transformation on native elements
   - Event handler QRL transformation for arrow/function expressions
   - 2 integration tests passing, 70 total tests passing

3. **03-03:** Event handler edge cases and validation - COMPLETE
   - Comprehensive tests for all EVT requirements (EVT-01 through EVT-08)
   - Strong assertions for multiple handlers, captured state, document/window scopes
   - Component element negative tests, prevent default, custom events
   - Requirements traceability documentation
   - 77 total tests passing

**Key Deliverables:**
- Event handler transformation: onClick$ -> on:click (on native elements)
- Document/window scopes: document:onFocus$ -> on-document:focus
- Case preservation: on-cLick$ -> on:c-lick
- Component elements preserve original attribute names
- Captured state variables included in QRL capture arrays
- Prevent default patterns preserved

**Requirements satisfied:** EVT-01 through EVT-08 (8/8)

## Phase 4 Props & Signals Summary

Phase 4 Props & Signals COMPLETE with all 5 plans executed:

1. **04-01:** Props destructuring detection - COMPLETE (7 min)
   - PropsDestructuring struct with component_ident tracking and identifiers HashMap
   - transform_component_props detects ObjectPattern and replaces with _rawProps
   - Integration into TransformGenerator enter/exit_call_expression
   - props_identifiers map populated with prop name -> key mappings
   - 82 total tests passing (5 new props tests)

2. **04-02:** Rest props and aliasing - COMPLETE (5 min)
   - rest_id and omit_keys fields added to PropsDestructuring
   - generate_rest_stmt creates _restProps call with omit array
   - Statement injection at function body start
   - _restProps import added when rest pattern present
   - 86 total tests passing (4 new rest props tests)

3. **04-03:** Identifier replacement with _wrapProp - COMPLETE (8 min)
   - _WRAP_PROP constant added to shared.rs
   - should_wrap_prop and should_wrap_signal_value helpers
   - Props_identifiers populated in enter_call_expression (critical fix)
   - _wrapProp(_rawProps, "propKey") for prop access
   - _wrapProp(signal) for signal.value access
   - 108 total tests passing (6 new _wrapProp tests)

4. **04-04:** _fnSignal generation - COMPLETE (8 min)
   - inlined_fn.rs module with should_wrap_in_fn_signal, convert_inlined_fn
   - ObjectUsageChecker, IdentifierReplacer for AST traversal
   - TransformGenerator integration with hoisted_fns tracking
   - 108 total tests passing

5. **04-05:** Bind directives - COMPLETE (8 min)
   - is_bind_directive, create_bind_handler, merge_event_handlers methods
   - bind:value -> value prop + on:input with inlinedQrl(_val)
   - bind:checked -> checked prop + on:input with inlinedQrl(_chk)
   - Handler merging for existing onInput$ (order-independent)
   - _val, _chk, inlinedQrl import generation
   - 115 total tests passing (7 new bind directive tests)

**Key Deliverables:**
- Props parameter transformation: `({ message, id })` -> `(_rawProps)`
- Rest props: `({ message, ...rest })` -> `const rest = _restProps(_rawProps, ["message"])`
- Rest-only: `({ ...props })` -> `const props = _restProps(_rawProps)`
- Aliased props tracked: c -> "count" for later replacement
- _wrapProp for prop access: `{message}` -> `_wrapProp(_rawProps, "message")`
- _wrapProp for signals: `{count.value}` -> `_wrapProp(count)`
- _fnSignal infrastructure: hoisted functions with positional params (p0, p1, ...)
- Member access detection for computed expression wrapping
- bind:value/bind:checked two-way binding transformation
- Handler merging with existing event handlers

**Requirements satisfied:** PROP-01 through PROP-08 (estimated 8/8)

**Ready for:** Phase 05 (Advanced Patterns)

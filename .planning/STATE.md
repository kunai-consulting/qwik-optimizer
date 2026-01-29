# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-01-29)

**Core value:** All 162 tests from qwik-core pass with exact output parity to the SWC implementation.
**Current focus:** Phase 5 - JSX Transformation (In progress)

## Current Position

Phase: 5 of 11 (JSX Transformation) - In progress
Plan: 4 of 5 in Phase 5 COMPLETE (05-01, 05-02, 05-03, 05-04 done)
Status: In progress - 4/5 plans executed
Last activity: 2026-01-29 - Completed 05-04-PLAN.md (Children & Flags)

Progress: [=========           ] 45% (4/11 phases complete, 22/44 total plans)

## Performance Metrics

**Velocity:**
- Total plans completed: 22
- Average duration: 7.0 min
- Total execution time: 2.5 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-oxc-foundation | 2/2 | 15 min | 7.5 min |
| 02-qrl-core | 7/7 | 51 min | 7.3 min |
| 03-event-handlers | 3/3 | 15 min | 5.0 min |
| 04-props-signals | 5/5 | 36 min | 7.2 min |
| 05-jsx-transformation | 4/5 | 35 min | 8.8 min |

**Recent Trend:**
- Last 5 plans: 05-01 (7 min), 05-02 (8 min), 05-03 (7 min), 05-04 (13 min)
- Phase 5 JSX Transformation 80% complete

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
- [05-01]: Use HashSet<String> for import names instead of full GlobalCollect
- [05-01]: Pre-compute is_const before mutable jsx_stack borrow to avoid borrow conflicts
- [05-01]: stack_is_const guards is_const_expr call (respects should_runtime_sort)
- [05-02]: Fragment as _Fragment import from @qwik.dev/core/jsx-runtime
- [05-02]: Implicit fragments get _jsxSorted(_Fragment, ...) output
- [05-02]: Explicit Fragment components use user-imported identifier
- [05-04]: Flags bit order: bit 0 = static_listeners (1), bit 1 = static_subtree (2) per SWC
- [05-04]: Single child passed directly without array wrapper
- [05-04]: Empty children output as null, not empty array

### Pending Todos

None yet.

### Blockers/Concerns

None yet.

## Session Continuity

Last session: 2026-01-29
Stopped at: Completed 05-04-PLAN.md (Children & Flags)
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
2. **04-02:** Rest props and aliasing - COMPLETE (5 min)
3. **04-03:** Identifier replacement with _wrapProp - COMPLETE (8 min)
4. **04-04:** _fnSignal generation - COMPLETE (8 min)
5. **04-05:** Bind directives - COMPLETE (8 min)

**Key Deliverables:**
- Props parameter transformation: `({ message, id })` -> `(_rawProps)`
- Rest props: `({ message, ...rest })` -> `const rest = _restProps(_rawProps, ["message"])`
- _wrapProp for prop access and signal.value
- _fnSignal infrastructure for computed expressions
- bind:value/bind:checked two-way binding transformation
- 115 total tests passing

**Requirements satisfied:** PROP-01 through PROP-08 (8/8)

## Phase 5 JSX Transformation Summary (In Progress)

Phase 5 JSX Transformation 80% complete (4/5 plans):

1. **05-01:** Prop Constness Detection - COMPLETE (7 min)
   - is_const_expr function for detecting static vs dynamic props
   - HashSet<String> for import names tracking
   - Pre-compute is_const before mutable borrow
   - 128 total tests passing

2. **05-02:** Fragment Transformation - COMPLETE (8 min)
   - Implicit fragments: <></> -> _jsxSorted(_Fragment, ...)
   - Explicit fragments preserve user import
   - _Fragment import from @qwik.dev/core/jsx-runtime
   - 128 total tests passing

3. **05-03:** Spread Props Helpers - COMPLETE (7 min)
   - _getVarProps/_getConstProps for spread props
   - _jsxSplit used instead of _jsxSorted for spread
   - 128 total tests passing

4. **05-04:** Children & Flags - COMPLETE (13 min)
   - Flags calculation: bit 0 = static_listeners, bit 1 = static_subtree
   - Single child optimization (not wrapped in array)
   - Empty children as null
   - Comprehensive JSX tests
   - 137 total tests passing

**Ready for:** 05-05 (final plan of phase 5)

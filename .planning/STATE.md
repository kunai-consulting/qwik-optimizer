# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-01-29)

**Core value:** All 162 tests from qwik-core pass with exact output parity to the SWC implementation.
**Current focus:** Phase 8 - SSR & Build Modes (IN PROGRESS)

## Current Position

Phase: 8 of 11 (SSR & Build Modes)
Plan: 2 of 3 in Phase 8 COMPLETE
Status: In Progress
Last activity: 2026-01-29 - Completed 08-02-PLAN.md

Progress: [==============      ] 68.2% (7/11 phases complete + 2 plans, 30/44 total plans)

## Performance Metrics

**Velocity:**
- Total plans completed: 30
- Average duration: 7.1 min
- Total execution time: 3.5 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-oxc-foundation | 2/2 | 15 min | 7.5 min |
| 02-qrl-core | 7/7 | 51 min | 7.3 min |
| 03-event-handlers | 3/3 | 15 min | 5.0 min |
| 04-props-signals | 5/5 | 36 min | 7.2 min |
| 05-jsx-transformation | 4/4 | 37 min | 9.3 min |
| 06-imports-exports | 4/4 | 45 min | 11.3 min |
| 07-entry-strategies | 3/3 | 29 min | 9.7 min |
| 08-ssr-build-modes | 2/3 | 9 min | 4.5 min |

**Recent Trend:**
- Last 5 plans: 07-02 (8 min), 07-03 (6 min), 08-01 (5 min), 08-02 (4 min)
- Phase 8 SSR & Build Modes IN PROGRESS

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
- [06-01]: export_by_name keyed by local name for lookup during QRL body analysis
- [06-01]: ExportInfo includes source field for re-exports tracking
- [06-01]: Duplicate local name exports overwrite (latest wins)
- [06-02]: Use exported_name as key for export specifiers (export { foo as bar })
- [06-02]: BTreeSet provides automatic deduplication for ImportId
- [06-02]: finalize_imports() converts HashMap to merged Import statements
- [06-03]: referenced_exports stored on both Qrl and SegmentData for transport and generation
- [06-03]: Default exports use import { default as LocalName } pattern
- [06-03]: Aliased exports use import { exported_name as local_name } pattern
- [06-03]: ExportInfo derives PartialOrd, Ord, Serialize for Qrl struct compatibility
- [06-04]: Side-effect import preservation verified via existing ImportCleanUp logic (specifiers: None)
- [06-04]: Re-exports pass through unchanged (source field present)
- [06-04]: Dynamic import generation verified via () => import('./segment.js') pattern
- [06-04]: Import order maintained for polyfill/CSS dependencies
- [07-01]: Added stacked_ctxt flag to JsxState to track whether JSX element pushed to stack_ctxt
- [07-01]: EntryPolicy::get_entry_for_sym takes &SegmentData instead of &Segment
- [07-02]: PerComponentStrategy returns entry_segments for top-level QRLs (empty context)
- [07-02]: SmartStrategy checks scoped_idents.is_empty() AND ctx_kind for event handler detection
- [07-02]: Entry grouping format: {origin}_entry_{root} for component-grouped segments
- [07-03]: Entry field added to QrlComponent struct, not SegmentData, for proper serialization
- [07-03]: Entry computed at QRL extraction time using entry_policy.get_entry_for_sym(stack_ctxt, segment_data)
- [07-03]: JSX event handlers (onClick$) don't produce segments - only component$() calls do
- [08-01]: is_server defaults to true (safe default - server code is safer)
- [08-01]: is_dev derived from target (not stored separately) matching SWC implementation
- [08-01]: ImportTracker uses (source, specifier) tuple as key for efficient lookup
- [08-02]: Used manual visitor pattern with allocator instead of VisitMut trait for OXC BooleanLiteral creation
- [08-02]: OXC Expression variants StaticMemberExpression/ComputedMemberExpression accessed directly (not via MemberExpression wrapper)
- [08-02]: isBrowser value is inverse of is_server (!is_server) matching SWC behavior

### Pending Todos

None yet.

### Blockers/Concerns

None yet.

## Session Continuity

Last session: 2026-01-29 23:54:51Z
Stopped at: Completed 08-02-PLAN.md (Const Replacement)
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

## Phase 5 JSX Transformation Summary

Phase 5 JSX Transformation COMPLETE with all 4 plans executed:

1. **05-01:** Prop Constness Detection - COMPLETE (7 min)
   - is_const.rs module with is_const_expr function and ConstCollector
   - HashSet<String> for import names tracking
   - Empty props output as null, not {}
   - 128 total tests passing

2. **05-02:** Fragment Handling - COMPLETE (4 min)
   - Implicit fragments: <></> -> _jsxSorted(_Fragment, ...)
   - Explicit fragments preserve user-imported identifier
   - _Fragment import from @qwik.dev/core/jsx-runtime
   - 127 total tests passing

3. **05-03:** Spread Props & Single Child - COMPLETE (13 min)
   - _getVarProps/_getConstProps helpers for spread props
   - Single child optimization (not wrapped in array)
   - Empty children as null
   - 137 total tests passing

4. **05-04:** Children & Flags - COMPLETE (13 min)
   - Flags: bit 0 = static_listeners (1), bit 1 = static_subtree (2)
   - Conditional (ternary) and list (.map) rendering verified
   - Comprehensive JSX tests added
   - 137 total tests passing

**Key Deliverables:**
- is_const_expr for accurate prop constness detection
- Empty props/children output as null
- Fragment transformation with proper imports
- Spread props with _getVarProps/_getConstProps helpers
- Single child optimization
- Correct flags calculation matching SWC

**Requirements satisfied:** JSX-01 through JSX-08 (8/8)
**Verification:** 17/17 must-haves passed

## Phase 6 Imports & Exports Summary (COMPLETE)

Phase 6 Imports & Exports COMPLETE with all 4 plans executed:

1. **06-01:** Export Tracking - COMPLETE (15 min)
   - ExportInfo struct with local_name, exported_name, is_default, source
   - collect_exports function for AST export collection
   - export_by_name HashMap in TransformGenerator
   - enter_export_named_declaration and enter_export_default_declaration hooks
   - 140 total tests passing

2. **06-02:** Synthesized Import Tracking - COMPLETE (18 min)
   - synthesized_imports HashMap for tracking imports by source
   - add_synthesized_import() and finalize_imports() helper methods
   - Fixed export specifier keying bug (aliased exports)
   - Tests for synthesized import deduplication and merging
   - 142 total tests passing

3. **06-03:** Segment Import Generation - COMPLETE (13 min)
   - referenced_exports populated during QRL creation
   - ExportInfo tracked in Qrl and SegmentData structs
   - generate_source_file_imports helper in QrlComponent
   - Default, aliased, and named export import syntax handled
   - 151 total tests passing

4. **06-04:** Side-Effects & Re-Exports - COMPLETE (12 min)
   - Side-effect import preservation verified
   - Re-exports pass through unchanged
   - Dynamic import generation (IMP-08) verified
   - Import order and mixed import types tested
   - 148 total tests passing

**Key Deliverables:**
- Export tracking infrastructure for segment import generation
- Synthesized imports properly tracked and deduplicated
- Side-effect imports preserved (import './x')
- Re-exports pass through unchanged (export { x } from './y')
- Dynamic import generation for QRL lazy-loading verified

**Requirements satisfied:** IMP-01 through IMP-08 (8/8)

## Phase 7 Entry Strategies Summary (COMPLETE)

Phase 7 Entry Strategies COMPLETE with all 3 plans executed:

1. **07-01:** Context Stack Infrastructure - COMPLETE (15 min)
   - stack_ctxt: Vec<String> field added to TransformGenerator
   - Push/pop in enter/exit_variable_declarator, enter/exit_function, enter/exit_class
   - Push/pop in enter/exit_jsx_element, enter/exit_jsx_attribute
   - Push/pop in enter/exit_call_expression for marker functions
   - EntryPolicy trait updated to use &SegmentData
   - 6 stack_ctxt tests added, 157 total tests passing

2. **07-02:** Strategy Implementations - COMPLETE (8 min)
   - PerComponentStrategy: groups QRLs by root component ({origin}_entry_{root})
   - SmartStrategy: separates stateless event handlers for independent loading
   - 11 unit tests covering all 5 entry strategies (ENT-01 through ENT-05)
   - 168 total tests passing

3. **07-03:** Integration & Validation - COMPLETE (6 min)
   - entry_policy integration in TransformGenerator
   - entry field in QrlComponent and SegmentAnalysis
   - Entry flows from TransformOptions through segment output
   - 9 integration tests verifying all strategies
   - 177 total tests passing

**Key Deliverables:**
- stack_ctxt field tracks component hierarchy for entry strategy grouping
- EntryPolicy::get_entry_for_sym accepts SegmentData for full QRL metadata access
- All 5 entry strategies implemented and integration tested
- Entry value flows from TransformOptions to SegmentAnalysis output

**Requirements satisfied:** ENT-01 through ENT-05 (5/5)

## Phase 8 SSR & Build Modes Summary (IN PROGRESS)

Phase 8 SSR & Build Modes in progress:

1. **08-01:** Infrastructure - COMPLETE (5 min)
   - QWIK_CORE_BUILD, IS_SERVER, IS_BROWSER, IS_DEV constants
   - TransformOptions.is_server field (default: true)
   - TransformOptions.is_dev() method (derived from target)
   - ImportTracker struct for aliased import tracking
   - get_imported_local() method for const replacement lookup
   - 180 total tests passing (177 existing + 3 new)

2. **08-02:** Const Replacement - COMPLETE (4 min)
   - ConstReplacerVisitor for SSR/build mode const replacement
   - isServer replaced with is_server boolean literal
   - isBrowser replaced with !is_server (inverse)
   - isDev replaced with is_dev boolean literal
   - Aliased imports handled (import { isServer as s })
   - Both @qwik.dev/core and @qwik.dev/core/build sources supported
   - 12 unit tests, 192 total tests passing

**Key Deliverables so far:**
- Build mode constants defined for @qwik.dev/core/build imports
- TransformOptions extended with SSR/build mode configuration
- ImportTracker ready to find isServer/isBrowser/isDev imports
- ConstReplacerVisitor replaces imported consts with boolean literals

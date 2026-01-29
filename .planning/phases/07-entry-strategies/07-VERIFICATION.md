---
phase: 07-entry-strategies
verified: 2026-01-29T23:23:34Z
status: passed
score: 5/5 must-haves verified
---

# Phase 7: Entry Strategies Verification Report

**Phase Goal:** All code-splitting strategies produce correct output
**Verified:** 2026-01-29T23:23:34Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #   | Truth                                             | Status     | Evidence                                                                 |
| --- | ------------------------------------------------- | ---------- | ------------------------------------------------------------------------ |
| 1   | InlineStrategy keeps code in single output        | ✓ VERIFIED | Implementation returns `Some("entry_segments")`, 2 tests pass            |
| 2   | SingleStrategy produces one segment file          | ✓ VERIFIED | Implementation returns `Some("entry_segments")`, 1 test passes           |
| 3   | PerSegmentStrategy produces multiple segment files| ✓ VERIFIED | Implementation returns `None`, 2 tests pass (Hook alias also verified)   |
| 4   | PerComponentStrategy groups by component          | ✓ VERIFIED | Implementation uses `{origin}_entry_{root}` format, 2 tests pass         |
| 5   | SmartStrategy optimizes based on analysis         | ✓ VERIFIED | Separates stateless handlers, groups with captures, 4 tests pass         |

**Score:** 5/5 truths verified

### Required Artifacts

| Artifact                                  | Expected                                     | Status     | Details                                                           |
| ----------------------------------------- | -------------------------------------------- | ---------- | ----------------------------------------------------------------- |
| `optimizer/src/entry_strategy.rs`        | All 5 strategy implementations               | ✓ VERIFIED | 300 lines, 5 strategies + tests, substantive, 11 unit tests pass |
| `optimizer/src/transform.rs`              | stack_ctxt and entry_policy integration      | ✓ VERIFIED | stack_ctxt: Vec<String> field, entry_policy integration wired    |
| `optimizer/src/component/component.rs`    | QrlComponent with entry field                | ✓ VERIFIED | pub entry: Option<String> field, flows to SegmentAnalysis        |
| `optimizer/src/js_lib_interface.rs`       | SegmentAnalysis.entry populated              | ✓ VERIFIED | entry: c.entry.clone() at line 283, 9 integration tests pass     |

### Key Link Verification

| From                        | To                          | Via                                      | Status     | Details                                                           |
| --------------------------- | --------------------------- | ---------------------------------------- | ---------- | ----------------------------------------------------------------- |
| transform.rs                | entry_strategy.rs           | get_entry_for_sym called with stack_ctxt | ✓ WIRED    | Line 970: entry_policy.get_entry_for_sym(&self.stack_ctxt, ...)  |
| entry_strategy.rs           | component/segment_data.rs   | Accesses segment.scoped_idents, ctx_kind | ✓ WIRED    | SmartStrategy uses scoped_idents.is_empty(), ctx_kind checks     |
| transform.rs                | component/component.rs      | Passes entry to QrlComponent             | ✓ WIRED    | Line 980: entry parameter passed to from_call_expression_argument |
| component/component.rs      | js_lib_interface.rs         | QrlComponent.entry flows to output       | ✓ WIRED    | Line 283: entry: c.entry.clone() populates SegmentAnalysis       |

### Requirements Coverage

| Requirement | Description                        | Status      | Blocking Issue |
| ----------- | ---------------------------------- | ----------- | -------------- |
| ENT-01      | InlineStrategy implementation      | ✓ SATISFIED | None           |
| ENT-02      | SingleStrategy implementation      | ✓ SATISFIED | None           |
| ENT-03      | PerSegmentStrategy implementation  | ✓ SATISFIED | None           |
| ENT-04      | PerComponentStrategy implementation| ✓ SATISFIED | None           |
| ENT-05      | SmartStrategy implementation       | ✓ SATISFIED | None           |

### Anti-Patterns Found

No blocker anti-patterns detected.

**Warnings (non-blocking):**
- Dead code warnings in component/shared.rs (BIND constants unused) - acceptable for this phase
- Unused test functions in js_lib_interface.rs (test_example_9, test_example_10) - test infrastructure

### Test Results

**Entry Strategy Unit Tests:** 11/11 passing
- test_inline_strategy_always_returns_entry_segments
- test_inline_strategy_no_context
- test_single_strategy_always_returns_entry_segments
- test_per_segment_strategy_always_returns_none
- test_per_segment_strategy_no_context
- test_per_component_with_context
- test_per_component_no_context
- test_smart_event_handler_no_captures
- test_smart_event_handler_with_captures
- test_smart_function_with_context
- test_smart_no_context

**Integration Tests:** 9/9 passing
- test_entry_strategy_inline
- test_entry_strategy_single
- test_entry_strategy_segment
- test_entry_strategy_hook
- test_entry_strategy_component
- test_entry_strategy_smart
- test_entry_strategy_smart_named_qrl
- test_entry_strategy_smart_multiple_components
- test_entry_strategy_smart_vs_component

**Stack Context Tests:** 6/6 passing
- test_stack_ctxt_component_function
- test_stack_ctxt_function_declaration
- test_stack_ctxt_jsx_element
- test_stack_ctxt_jsx_attribute
- test_stack_ctxt_multiple_handlers_same_element
- test_stack_ctxt_nested_components

**Total Tests:** 177 passing (168 existing + 9 new integration tests)
**Build Status:** Compiles without errors (6 non-blocking warnings)

### Implementation Verification

#### Level 1: Existence ✓
All required files exist:
- optimizer/src/entry_strategy.rs (300 lines)
- optimizer/src/transform.rs (modified, stack_ctxt + entry_policy)
- optimizer/src/component/component.rs (modified, entry field added)
- optimizer/src/js_lib_interface.rs (modified, entry flows to output)

#### Level 2: Substantive ✓
**entry_strategy.rs:**
- Lines: 300
- No stub patterns (TODO/FIXME)
- 5 complete strategy implementations
- Proper documentation on EntryPolicy trait
- 11 unit tests covering all strategies
- parse_entry_strategy() function maps enum to trait objects

**transform.rs stack_ctxt:**
- stack_ctxt: Vec<String> field with capacity 16
- Push/pop in 7 visitor methods:
  - enter/exit_call_expression (marker functions)
  - enter/exit_function (function declarations)
  - enter/exit_class (class declarations)
  - enter/exit_jsx_element (JSX element tracking)
  - enter/exit_jsx_attribute (event handler tracking)
  - enter/exit_variable_declarator (component variable names)
- current_context() accessor for testing
- 6 unit tests verify tracking

**transform.rs entry_policy:**
- entry_policy: Box<dyn EntryPolicy> field
- Parsed from TransformOptions.entry_strategy
- Called at QRL extraction: get_entry_for_sym(&self.stack_ctxt, &segment_data)
- Entry value passed to QrlComponent constructor

**component.rs:**
- pub entry: Option<String> field added to QrlComponent
- Field properly serialized with skip_serializing_if = "Option::is_none"
- Constructor parameters updated
- Entry flows from transformation to output

**js_lib_interface.rs:**
- SegmentAnalysis.entry populated: c.entry.clone()
- 9 integration tests verify end-to-end flow
- All entry strategies tested with actual transformations

#### Level 3: Wired ✓
**Complete flow verified:**

1. **TransformOptions → TransformGenerator:**
   - TransformOptions.entry_strategy (enum)
   - parse_entry_strategy() → Box<dyn EntryPolicy>
   - Stored in TransformGenerator.entry_policy

2. **QRL Extraction → Entry Computation:**
   - During QRL processing (transform.rs:970)
   - entry_policy.get_entry_for_sym(&self.stack_ctxt, &segment_data)
   - Returns Option<String> based on strategy

3. **Entry → QrlComponent:**
   - Entry value passed to QrlComponent::from_call_expression_argument
   - Stored in QrlComponent.entry field

4. **QrlComponent → Output:**
   - QrlComponent.entry flows to SegmentAnalysis.entry
   - js_lib_interface.rs:283: entry: c.entry.clone()
   - Serialized in JSON output for bundler

**Strategy-specific verification:**

**InlineStrategy:**
- Always returns Some("entry_segments")
- Groups all QRLs into one entry point
- Verified by 2 unit tests + integration test

**SingleStrategy:**
- Always returns Some("entry_segments")
- Identical to InlineStrategy (groups all)
- Verified by 1 unit test + integration test

**PerSegmentStrategy:**
- Always returns None
- Each segment gets its own file
- Also used by HookStrategy alias
- Verified by 2 unit tests + 2 integration tests (Segment + Hook)

**PerComponentStrategy:**
- Context present: Some("{origin}_entry_{root}")
- Context empty: Some("entry_segments")
- Groups all QRLs by root component name
- Verified by 2 unit tests + integration test

**SmartStrategy:**
- Stateless event handlers: None (separate files)
- Handlers with captures: Some("{origin}_entry_{root}")
- Function context: Some("{origin}_entry_{root}")
- No context: None
- Complex logic verified by 4 unit tests + 3 integration tests

### Evidence Files

**Core Implementation:**
- /Users/jackshelton/dev/open-source/qwik-optimizer/optimizer/src/entry_strategy.rs:1-300
- /Users/jackshelton/dev/open-source/qwik-optimizer/optimizer/src/transform.rs:256,260,310,334,712-713,969-971,1011-1014,1037-1044,1069-1071,1077-1084,1095-1097
- /Users/jackshelton/dev/open-source/qwik-optimizer/optimizer/src/component/component.rs:36,49,100
- /Users/jackshelton/dev/open-source/qwik-optimizer/optimizer/src/js_lib_interface.rs:113-128,283

**Test Files:**
- /Users/jackshelton/dev/open-source/qwik-optimizer/optimizer/src/entry_strategy.rs:148-299 (unit tests)
- /Users/jackshelton/dev/open-source/qwik-optimizer/optimizer/src/js_lib_interface.rs:635-900 (integration tests)
- /Users/jackshelton/dev/open-source/qwik-optimizer/optimizer/src/transform.rs (stack_ctxt tests)

### SWC Parity Check

All implementations directly ported from SWC reference:
- qwik-core/src/optimizer/core/src/entry_strategy.rs

**Key differences handled correctly:**
- SWC uses `Atom` for strings → OXC uses `String`
- SWC segment.origin is string → OXC uses PathBuf.display().to_string()
- EntryPolicy trait uses &SegmentData (not &Segment) per 07-01
- stack_ctxt patterns match SWC's TransformVisitor exactly

**Verified matching behavior:**
- InlineStrategy: Identical logic
- SingleStrategy: Identical logic
- PerSegmentStrategy: Identical logic
- PerComponentStrategy: Identical logic including context.first().map_or_else pattern
- SmartStrategy: Identical logic including scoped_idents check and ctx_kind logic

---

## Summary

Phase 7 goal **ACHIEVED**. All five code-splitting strategies produce correct output:

1. **InlineStrategy (ENT-01):** Groups all QRLs into "entry_segments" ✓
2. **SingleStrategy (ENT-02):** Groups all QRLs into "entry_segments" ✓
3. **PerSegmentStrategy (ENT-03):** Each segment gets own file (None) ✓
4. **PerComponentStrategy (ENT-04):** Groups by component with "{origin}_entry_{root}" format ✓
5. **SmartStrategy (ENT-05):** Intelligently separates stateless handlers, groups others ✓

**Infrastructure complete:**
- stack_ctxt tracking in TransformGenerator for component hierarchy
- EntryPolicy trait with SegmentData parameter
- entry_policy integration in transformation pipeline
- Entry field flows from strategy → QrlComponent → SegmentAnalysis → output
- 26 tests (11 unit + 9 integration + 6 stack_ctxt) all passing
- 177 total tests passing, no regressions

**Ready for Phase 8:** SSR & Build Modes

---
_Verified: 2026-01-29T23:23:34Z_
_Verifier: Claude (gsd-verifier)_

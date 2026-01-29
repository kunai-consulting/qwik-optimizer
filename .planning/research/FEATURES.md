# Feature Landscape: Qwik Optimizer

**Domain:** JavaScript/TypeScript code transformation for Qwik framework
**Researched:** 2026-01-29
**Source:** Analysis of 162 snapshot tests in qwik-core SWC implementation

## Executive Summary

The Qwik optimizer transforms Qwik component code by extracting QRL (Qwik Resource Locator) functions into separate modules, enabling lazy-loading and resumability. The 162 snapshot tests in the reference implementation cover 10 major feature categories, ranging from basic QRL extraction to complex JSX transformations, server/client code stripping, and edge cases.

The current OXC implementation has 19 snapshot tests covering basic examples (1-8, 11), import capture, JSX transformation, and TypeScript support. Approximately **143 tests remain to be ported** to achieve feature parity.

---

## Table Stakes (Must Have for Parity)

Features users expect. Missing = optimizer is incomplete.

### 1. QRL Extraction (Core Feature)

The fundamental transformation: extracting `$()` wrapped functions into separate modules.

| Test | Description | Complexity | Status |
|------|-------------|------------|--------|
| `example_1` - `example_11` | Basic QRL extraction patterns | Low | **8 of 11 DONE** |
| `should_extract_single_qrl` | Single QRL in complex context | Med | MISSING |
| `should_extract_single_qrl_2` | Nested QRL extraction | Med | MISSING |
| `should_extract_single_qrl_with_index` | QRL with index parameter | Med | MISSING |
| `should_extract_single_qrl_with_nested_components` | Nested component QRLs | Med | MISSING |

**Priority:** HIGH - Core functionality

### 2. Component Transformation

Transform `component$()` wrappers and their inner QRLs.

| Test | Description | Complexity | Status |
|------|-------------|------------|--------|
| `example_functional_component` | Basic component$ | Low | MISSING |
| `example_functional_component_2` | Component with state | Med | MISSING |
| `example_functional_component_capture_props` | Capturing props in QRLs | Med | MISSING |
| `should_transform_component_with_normal_function` | Named function components | Low | MISSING |
| `example_lightweight_functional` | Lightweight (non-component$) functions | Med | MISSING |

**Priority:** HIGH - Essential for any Qwik app

### 3. Event Handler Transformation

Transform event handlers (`onClick$`, etc.) into QRLs.

| Test | Description | Complexity | Status |
|------|-------------|------------|--------|
| `example_jsx_listeners` | Various event listener types | Med | MISSING |
| `should_convert_jsx_events` | JSX event conversion | Med | MISSING |
| `should_transform_event_names_without_jsx_transpile` | Event names without JSX | Med | MISSING |
| `should_transform_multiple_event_handlers` | Multiple handlers on element | Med | MISSING |
| `should_transform_multiple_event_handlers_case2` | Multiple with index | Med | MISSING |
| `should_not_transform_events_on_non_elements` | Skip non-element events | Med | MISSING |
| `example_component_with_event_listeners_inside_loop` | Events in loops (complex) | High | MISSING |
| `should_transform_nested_loops` | Nested loop event handlers | High | MISSING |

**Priority:** HIGH - Every interactive component uses event handlers

### 4. JSX Transformation

Transform JSX into Qwik's optimized format.

| Test | Description | Complexity | Status |
|------|-------------|------------|--------|
| `example_jsx` | Basic JSX transformation | Med | **DONE** |
| `example_jsx_keyed` | Keyed JSX elements | Med | MISSING |
| `example_jsx_keyed_dev` | Keyed elements in dev mode | Med | MISSING |
| `example_jsx_import_source` | Custom JSX import source | Med | MISSING |
| `example_spread_jsx` | Spread props in JSX | Med | MISSING |
| `special_jsx` | Non-plain-object props | Low | MISSING |
| `should_handle_dangerously_set_inner_html` | dangerouslySetInnerHTML | Med | MISSING |

**Priority:** HIGH - JSX is the primary UI authoring format

### 5. Import/Export Handling

Handle various import/export patterns.

| Test | Description | Complexity | Status |
|------|-------------|------------|--------|
| `example_capture_imports` | Capturing imported values | Med | **DONE** |
| `example_capturing_fn_class` | Capturing functions/classes | Med | **DONE** |
| `example_exports` | Various export patterns | Med | MISSING |
| `example_renamed_exports` | Renamed exports | Med | MISSING |
| `example_default_export` | Default export handling | Med | MISSING |
| `example_default_export_index` | Default from index file | Med | MISSING |
| `example_default_export_invalid_ident` | Invalid identifier default | Med | MISSING |
| `example_fix_dynamic_import` | Dynamic import paths | Med | MISSING |
| `example_export_issue` | Export edge cases | Med | MISSING |
| `example_import_assertion` | Import assertions | Low | MISSING |
| `rename_builder_io` | Rename @builder.io imports | Low | MISSING |

**Priority:** HIGH - Module system is fundamental

### 6. TypeScript Support

Handle TypeScript-specific constructs.

| Test | Description | Complexity | Status |
|------|-------------|------------|--------|
| `example_transpile_ts_only` | TS without JSX | Low | MISSING |
| `example_transpile_jsx_only` | JSX without TS | Low | MISSING |
| `example_ts_enums` | Enum handling | Med | MISSING |
| `example_ts_enums_no_transpile` | Enums without transpilation | Med | MISSING |
| `example_ts_enums_issue_1341` | Enum edge case | Med | MISSING |
| `issue_964` | Generator function types | Med | MISSING |

**Priority:** HIGH - Most Qwik apps use TypeScript

### 7. Entry Strategy Modes

Support different code-splitting strategies.

| Test | Description | Complexity | Status |
|------|-------------|------------|--------|
| `example_inlined_entry_strategy` | Inline strategy | Med | MISSING |
| `example_manual_chunks` | Manual chunk control | Med | MISSING |
| `example_preserve_filenames` | Preserve original names | Low | MISSING |
| `example_preserve_filenames_segments` | Segment with filenames | Low | MISSING |
| `example_parsed_inlined_qrls` | Pre-inlined QRLs | Med | MISSING |
| `relative_paths` | Relative path handling | Med | MISSING |
| `consistent_hashes` | Hash consistency | High | MISSING |

**Priority:** MEDIUM - Affects build output structure

### 8. Props Handling

Handle component props and their optimization.

| Test | Description | Complexity | Status |
|------|-------------|------------|--------|
| `example_props_optimization` | Props optimization | Med | MISSING |
| `example_props_wrapping` | Props wrapping | Med | MISSING |
| `example_props_wrapping2` | Props wrapping variant | Med | MISSING |
| `example_props_wrapping_children` | Children props wrapping | Med | MISSING |
| `example_props_wrapping_children2` | Children wrapping variant | Med | MISSING |
| `should_destructure_args` | Arg destructuring | Med | MISSING |
| `destructure_args_inline_cmp_block_stmt` | Inline component args | Med | MISSING |
| `destructure_args_inline_cmp_block_stmt2` | Block statement args | Med | MISSING |
| `destructure_args_inline_cmp_expr_stmt` | Expression args | Med | MISSING |
| `destructure_args_colon_props` | Colon props (bind:value) | Med | MISSING |
| `destructure_args_colon_props2` | Colon props variant | Med | MISSING |
| `destructure_args_colon_props3` | Colon props with rest | Med | MISSING |
| `should_convert_rest_props` | Rest props handling | Med | MISSING |
| `should_wrap_inner_inline_component_prop` | Inner component props | Med | MISSING |
| `should_wrap_prop_from_destructured_array` | Destructured array props | High | MISSING |
| `should_wrap_object_with_fn_signal` | Object with signal | Med | MISSING |
| `should_mark_props_as_var_props_for_inner_cmp` | Var props marking | Med | MISSING |
| `should_not_wrap_fn` | Skip function wrapping | Med | MISSING |
| `should_not_wrap_var_template_string` | Skip template strings | Med | MISSING |
| `should_wrap_type_asserted_variables_in_template` | Type assertion vars | Med | MISSING |
| `should_wrap_logical_expression_in_template` | Logical expressions | Med | MISSING |
| `should_not_wrap_ternary_function_operator_with_fn` | Ternary functions | Med | MISSING |
| `should_move_props_related_to_iteration_variables_to_var_props` | Iteration var props | High | MISSING |

**Priority:** HIGH - Props handling is essential for component authoring

### 9. Spread Props & JSX Split

Handle spread props and JSX splitting scenarios.

| Test | Description | Complexity | Status |
|------|-------------|------------|--------|
| `should_split_spread_props` | Basic spread split | Med | MISSING |
| `should_split_spread_props_with_additional_prop` | Spread + prop | Med | MISSING |
| `should_split_spread_props_with_additional_prop2` - `5` | Various spread combos | Med | MISSING |
| `should_merge_attributes_with_spread_props` | Merge attrs with spread | Med | MISSING |
| `should_merge_attributes_with_spread_props_before_and_after` | Before/after spreads | Med | MISSING |
| `issue_7216_add_test` | Spread with events | High | MISSING |

**Priority:** MEDIUM - Common pattern in component libraries

### 10. Bind Directive Support

Handle `bind:value` and `bind:checked` directives.

| Test | Description | Complexity | Status |
|------|-------------|------------|--------|
| `example_input_bind` | Basic input binding | Med | MISSING |
| `should_merge_bind_value_and_on_input` | bind:value + onInput | Med | MISSING |
| `should_merge_bind_checked_and_on_input` | bind:checked + onInput | Med | MISSING |
| `should_merge_on_input_and_bind_value` | onInput + bind:value | Med | MISSING |
| `should_merge_on_input_and_bind_checked` | onInput + bind:checked | Med | MISSING |
| `should_not_transform_bind_value_in_var_props_for_jsx_split` | bind:value in var props | Med | MISSING |
| `should_not_transform_bind_checked_in_var_props_for_jsx_split` | bind:checked in var props | Med | MISSING |
| `should_move_bind_value_to_var_props` | Move bind to var props | Med | MISSING |

**Priority:** MEDIUM - Important for form handling

---

## Server/Client Code Handling

Features for server-side rendering and client hydration.

| Test | Description | Complexity | Status |
|------|-------------|------------|--------|
| `example_build_server` | Server build mode | Med | MISSING |
| `example_strip_server_code` | Strip server-only code | Med | MISSING |
| `example_strip_client_code` | Strip client-only code | Med | MISSING |
| `example_strip_exports_unused` | Strip unused exports | Med | MISSING |
| `example_strip_exports_used` | Handle used exports | Med | MISSING |
| `example_server_auth` | Server auth patterns | Med | MISSING |
| `example_use_server_mount` | useTask$ server side | Med | MISSING |
| `example_reg_ctx_name_segments` | Registered context names | Med | MISSING |
| `example_reg_ctx_name_segments_inlined` | Inlined ctx names | Med | MISSING |
| `example_reg_ctx_name_segments_hoisted` | Hoisted ctx names | Med | MISSING |
| `example_drop_side_effects` | Drop side effects | Med | MISSING |

**Priority:** MEDIUM - Required for SSR/SSG

---

## Dev/Prod Mode Handling

| Test | Description | Complexity | Status |
|------|-------------|------------|--------|
| `example_dev_mode` | Dev mode output | Med | MISSING |
| `example_dev_mode_inlined` | Dev mode inlined | Med | MISSING |
| `example_prod_node` | Prod mode output | Med | MISSING |
| `example_noop_dev_mode` | No-op in dev mode | Med | MISSING |

**Priority:** MEDIUM - Affects DX and debugging

---

## Signal & Store Handling (Derived Signals)

| Test | Description | Complexity | Status |
|------|-------------|------------|--------|
| `example_derived_signals_div` | Div signal props | Med | MISSING |
| `example_derived_signals_children` | Children signals | Med | MISSING |
| `example_derived_signals_multiple_children` | Multiple signal children | Med | MISSING |
| `example_derived_signals_complext_children` | Complex signal children | Med | MISSING |
| `example_derived_signals_cmp` | Component signal props | Med | MISSING |
| `should_wrap_store_expression` | Store expressions | Med | MISSING |
| `hoisted_fn_signal_in_loop` | Signals in loops | High | MISSING |
| `lib_mode_fn_signal` | Lib mode signals | Med | MISSING |

**Priority:** HIGH - Signals are core to Qwik reactivity

---

## Immutability Analysis

| Test | Description | Complexity | Status |
|------|-------------|------------|--------|
| `example_immutable_analysis` | Immutable prop analysis | High | MISSING |
| `example_immutable_function_components` | Immutable function cmps | Med | MISSING |
| `example_mutable_children` | Mutable children handling | Med | MISSING |

**Priority:** MEDIUM - Performance optimization

---

## Edge Cases & Bug Fixes

Tests for specific issues and edge cases.

| Test | Description | Complexity | Status |
|------|-------------|------------|--------|
| `issue_117` | Pattern cache issue | Low | MISSING |
| `issue_150` | Class object issue | Med | MISSING |
| `issue_476` | HTML structure issue | Low | MISSING |
| `issue_964` | Generator function | Med | MISSING |
| `issue_5008` | Store array issue | Med | MISSING |
| `example_issue_33443` | Ternary in title | Med | MISSING |
| `example_issue_4438` | $localize issue | Med | MISSING |
| `example_optimization_issue_3542` | Optimization bug | Med | MISSING |
| `example_optimization_issue_3561` | Destructuring bug | Med | MISSING |
| `example_optimization_issue_3795` | Variable mutation bug | Med | MISSING |
| `example_optimization_issue_4386` | Mapping optimization | Low | MISSING |
| `support_windows_paths` | Windows path handling | Low | MISSING |

**Priority:** MEDIUM-LOW - Important for stability

---

## Miscellaneous Features

| Test | Description | Complexity | Status |
|------|-------------|------------|--------|
| `example_custom_inlined_functions` | Custom $ functions | Med | MISSING |
| `example_missing_custom_inlined_functions` | Missing wrap() | Med | MISSING |
| `example_skip_transform` | Skip transformation | Low | MISSING |
| `example_explicit_ext_transpile` | Explicit extensions | Low | MISSING |
| `example_explicit_ext_no_transpile` | Explicit ext no transpile | Low | MISSING |
| `example_qwik_conflict` | Qwik name conflicts | Med | MISSING |
| `example_qwik_react` | Qwik React integration | High | MISSING |
| `example_qwik_react_inline` | Qwik React inlined | High | MISSING |
| `example_qwik_router_inline` | Qwik Router | High | MISSING |
| `example_use_client_effect` | useBrowserVisibleTask$ | Med | MISSING |
| `example_use_optimization` | Use optimization | Med | MISSING |
| `example_multi_capture` | Multiple captures | Med | MISSING |
| `example_dead_code` | Dead code elimination | Med | MISSING |
| `example_with_tagname` | Component tagName | Low | MISSING |
| `example_with_style` | useStyles$ handling | Med | MISSING |
| `example_class_name` | className prop | Med | MISSING |
| `example_getter_generation` | Getter generation | Med | MISSING |
| `example_of_synchronous_qrl` | sync$ function | Med | MISSING |
| `example_invalid_references` | Invalid ref detection | Med | MISSING |
| `example_invalid_segment_expr1` | Invalid segment expr | Med | MISSING |
| `ternary_prop` | Ternary in prop | Med | MISSING |
| `transform_qrl_in_regular_prop` | QRL in regular prop | Med | MISSING |
| `impure_template_fns` | Impure template fns | Med | MISSING |
| `should_not_move_over_side_effects` | Side effect ordering | Med | MISSING |
| `should_ignore_null_inlined_qrl` | Null inlined QRL | Low | MISSING |
| `should_not_generate_conflicting_props_identifiers` | Prop ID conflicts | Med | MISSING |
| `should_transform_qrls_in_ternary_expression` | Ternary QRLs | Med | MISSING |

**Priority:** LOW-MEDIUM - Various features and edge cases

---

## Gap Analysis Summary

### Current OXC Implementation Coverage (19 tests)

1. **Basic QRL Extraction:** examples 1-8, 11 (9 tests)
2. **Import Capture:** `example_capture_imports`, `example_capturing_fn_class` (2 tests)
3. **JSX:** `example_jsx` (1 test)
4. **TypeScript:** `example_ts` (1 test)
5. **Project-level:** `test_project_1` (1 test)
6. **Transform-level:** 5 individual segment tests

### Missing for Feature Parity (143 tests)

| Category | Count | Priority |
|----------|-------|----------|
| QRL Extraction (remaining) | 4 | HIGH |
| Component Transformation | 5 | HIGH |
| Event Handlers | 8 | HIGH |
| JSX (remaining) | 6 | HIGH |
| Import/Export | 10 | HIGH |
| TypeScript (remaining) | 5 | HIGH |
| Props Handling | 24 | HIGH |
| Entry Strategies | 7 | MEDIUM |
| Spread Props | 8 | MEDIUM |
| Bind Directives | 8 | MEDIUM |
| Server/Client | 11 | MEDIUM |
| Dev/Prod Mode | 4 | MEDIUM |
| Signals/Stores | 8 | HIGH |
| Immutability | 3 | MEDIUM |
| Edge Cases | 12 | MEDIUM |
| Miscellaneous | 30+ | LOW-MEDIUM |

---

## Recommended Porting Priority

### Phase 1: Core Functionality (HIGH PRIORITY)
1. **Component Transformation** (5 tests) - Every app needs components
2. **Event Handlers** (8 tests) - Every interactive app needs events
3. **Remaining QRL Extraction** (4 tests) - Core extraction edge cases
4. **Props Handling** (24 tests) - Essential for component authoring
5. **Signals/Stores** (8 tests) - Core reactivity system

### Phase 2: Build System Features (MEDIUM PRIORITY)
6. **Import/Export** (10 tests) - Module system completeness
7. **Entry Strategies** (7 tests) - Build output control
8. **Server/Client** (11 tests) - SSR support
9. **TypeScript** (5 tests) - TS completeness

### Phase 3: Advanced Features (MEDIUM PRIORITY)
10. **JSX (remaining)** (6 tests) - JSX edge cases
11. **Spread Props** (8 tests) - Component library patterns
12. **Bind Directives** (8 tests) - Form handling
13. **Dev/Prod Mode** (4 tests) - DX features

### Phase 4: Edge Cases & Polish (LOW-MEDIUM PRIORITY)
14. **Immutability** (3 tests) - Performance optimization
15. **Edge Cases** (12 tests) - Bug fixes and stability
16. **Miscellaneous** (30+ tests) - Feature completeness

---

## Sources

- `/Users/jackshelton/dev/open-source/qwik-optimizer/qwik-core/src/optimizer/core/src/test.rs` - Reference implementation tests
- `/Users/jackshelton/dev/open-source/qwik-optimizer/qwik-core/src/optimizer/core/src/snapshots/` - 162 snapshot files
- `/Users/jackshelton/dev/open-source/qwik-optimizer/optimizer/src/js_lib_interface.rs` - OXC implementation tests
- `/Users/jackshelton/dev/open-source/qwik-optimizer/optimizer/src/snapshots/` - 19 current OXC snapshots

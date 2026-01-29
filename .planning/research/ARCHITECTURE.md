# Architecture Patterns: Qwik Optimizer SWC vs OXC

**Domain:** Compiler/Transformer tooling for Qwik framework
**Researched:** 2026-01-29
**Confidence:** HIGH (direct source code analysis)

## Overview

This document compares the architecture of two Qwik optimizer implementations:
- **SWC implementation** (qwik-core): The production optimizer built on SWC
- **OXC implementation** (optimizer): A port/rewrite using the OXC toolchain

Both are Rust-based AST transformation systems that extract QRL (Qwik Resource Locator) segments from source code for lazy-loading optimization.

---

## File Mapping Between Implementations

### Core Entry Points

| Purpose | SWC (qwik-core) | OXC (optimizer) |
|---------|-----------------|-----------------|
| Library root | `lib.rs` | `lib.rs` |
| Main transform | `parse.rs` (transform_code) | `transform.rs` (transform) |
| Public API | `lib.rs` (transform_modules) | `js_lib_interface.rs` (transform_modules) |

### Transformation Pipeline

| Purpose | SWC (qwik-core) | OXC (optimizer) |
|---------|-----------------|-----------------|
| AST traversal | `transform.rs` (QwikTransform) | `transform.rs` (TransformGenerator) |
| Symbol collection | `collector.rs` (GlobalCollect) | `transform.rs` + `component/` |
| Code movement | `code_move.rs` | `component/component.rs` |
| Segment naming | `transform.rs` (Segment, SegmentData) | `segment.rs` (Segment, SegmentBuilder) |
| Entry strategy | `entry_strategy.rs` | `entry_strategy.rs` |

### Supporting Modules

| Purpose | SWC (qwik-core) | OXC (optimizer) |
|---------|-----------------|-----------------|
| QRL generation | `transform.rs` (inline) | `component/qrl.rs` (Qrl struct) |
| Import handling | `collector.rs`, `rename_imports.rs` | `component/shared.rs` (Import), `import_clean_up.rs` |
| Source info | `parse.rs` (PathData) | `source.rs`, `component/source_info.rs` |
| ID/hash generation | `transform.rs` (inline) | `component/id.rs` |
| Const analysis | `const_replace.rs`, `is_const.rs` | Not yet implemented |
| Props destructuring | `props_destructuring.rs` | Not yet implemented |
| Dead code elimination | `clean_side_effects.rs` | `dead_code.rs` |
| Side effects | `add_side_effect.rs` | Not yet implemented |
| Error handling | `errors.rs` | `error.rs`, `processing_failure.rs` |
| Illegal code detection | Inline in transform | `illegal_code.rs` |

### SWC-only Modules (Need Porting)

| Module | Purpose | Priority |
|--------|---------|----------|
| `const_replace.rs` | Replace const values (isServer, isDev) | Medium |
| `props_destructuring.rs` | Reconstruct destructured props for signal forwarding | High |
| `add_side_effect.rs` | Add side effect markers for bundler | Medium |
| `clean_side_effects.rs` | Treeshaker implementation | Medium |
| `filter_exports.rs` | Strip specified exports | Low |
| `inlined_fn.rs` | Handle inlined functions | Medium |

---

## Transformation Pipeline Stages

### SWC Pipeline (parse.rs::transform_code)

```
1. Parse source code (Lexer -> Parser)
2. Strip exports (if configured)
3. TypeScript strip (if transpileTs)
4. JSX transform (if transpileJsx, uses React runtime)
5. Rename imports (old Qwik imports -> new)
6. Resolve with marks (unresolved_mark, top_level_mark)
7. Global collect (imports, exports, root declarations)
8. Props destructuring transform
9. Const replacer (isServer, isDev)
10. QwikTransform (main segment extraction via Fold trait)
11. Treeshaker mark (client-side only)
12. Simplify/DCE pass
13. Side effect visitor (Inline/Hoist strategies)
14. Second treeshake clean pass
15. Hygiene cleanup
16. Fixer pass
17. Generate segment modules (code_move::new_module)
18. Emit source code for each module
```

### OXC Pipeline (transform.rs::transform)

```
1. Parse source code (Parser::new)
2. TypeScript transform (if transpile_ts, via Transformer)
3. Semantic analysis (SemanticBuilder)
4. TransformGenerator traversal (via traverse_mut)
   - enter_program: Initialize
   - enter/exit_call_expression: Detect QRL markers
   - enter/exit_variable_declarator: Track declarations
   - enter/exit_jsx_element: JSX transformation
   - enter/exit_jsx_attribute: JSX prop handling
   - exit_identifier_reference: Track imports for segments
5. ImportCleanUp
6. Codegen for main body and components
```

### Key Architectural Differences

| Aspect | SWC | OXC |
|--------|-----|-----|
| Traversal pattern | `Fold` trait (immutable transform) | `Traverse` trait (mutable in-place) |
| Symbol tracking | SyntaxContext-based Id tuple | SymbolId/ReferenceId |
| AST allocation | Owned boxes | Arena allocator (`Allocator`) |
| Module generation | Separate pass via `new_module` | Inline during traversal |
| JSX handling | Via React transform, then process | Direct JSX node processing |

---

## Test Structure

### SWC Test Format (test.rs)

**Input format:** Inline Rust string in test function
```rust
#[test]
fn example_1() {
    test_input!(TestInput {
        code: r#"
import { $, component, onRender } from '@qwik.dev/core';
export const renderHeader = $(() => { ... });
"#.to_string(),
        ..TestInput::default()
    });
}
```

**TestInput struct fields:**
- `code`: Source code string
- `filename`: Path for source file (default: "test.tsx")
- `src_dir`: Source directory
- `root_dir`: Root directory for source maps
- `transpile_ts/transpile_jsx`: Transform flags
- `entry_strategy`: EntryStrategy enum
- `mode`: EmitMode (Prod/Lib/Dev/Test)
- `minify`: MinifyMode
- `strip_exports`, `strip_ctx_name`, `reg_ctx_name`: Filter options
- `is_server`: Server/client mode

**Snapshot format:** Uses `insta` crate
```
==INPUT==
[source code]

============================= [path] (ENTRY POINT)==
[generated code]
[source map as JSON string]
/*
{
  "origin": "...",
  "name": "...",
  "hash": "...",
  ...
}
*/

== DIAGNOSTICS ==
[JSON array]
```

### OXC Test Format (js_lib_interface.rs + macros.rs)

**Input format:** External files in `src/test_input/`
```
optimizer/src/test_input/test_example_1.tsx
optimizer/src/test_input/test_example_2.tsx
...
```

**Test invocation:**
```rust
#[test]
fn test_example_1() {
    assert_valid_transform_debug!(EntryStrategy::Segment);
}
```

**The macro:**
1. Derives filename from function name (`test_example_1` -> `test_example_1.tsx`)
2. Reads file from `./src/test_input/`
3. Calls `transform_modules` with standard options
4. Snapshots result via `insta`

**Snapshot location:** `optimizer/src/snapshots/`

**Snapshot naming:** `qwik_optimizer__{module}__tests__{test_name}.snap`

### Test Comparison

| Aspect | SWC | OXC |
|--------|-----|-----|
| Input location | Inline in test file | External .tsx/.js files |
| Test definition | Per-test TestInput struct | Macro with defaults |
| Snapshot format | Similar (both use insta) | Similar |
| Source maps | Included | Currently None |
| SegmentAnalysis | Full metadata | Full metadata |

---

## Behavioral Differences to Watch

### 1. Hash Generation
- **SWC:** Uses `DefaultHasher` with file path + scope, Base64 encoded
- **OXC:** Similar approach but verify hash algorithm matches

### 2. Symbol Naming
- **SWC:** Uses `{origin}_{name}_{hash}` pattern
- **OXC:** Same pattern but segment naming in `SegmentBuilder`

### 3. Import Hoisting
- **SWC:** Hoists dynamic import arrows: `const i_xxx = () => import(...)`
- **OXC:** Inline arrow in qrl call: `qrl(() => import(...), "name")`

### 4. JSX Transformation
- **SWC:** Uses `_jsxQ` and related Qwik-specific JSX functions
- **OXC:** Uses `_jsxSorted` and `_jsxSplit`

### 5. Source Maps
- **SWC:** Full source map generation
- **OXC:** Not yet implemented (returns `None`)

### 6. Const Props Analysis
- **SWC:** Separates const vs var props in JSX
- **OXC:** Implements this with `expr_is_const_stack` tracking

### 7. Scoped Identifiers / Captures
- **SWC:** Uses `useLexicalScope` for captured variables
- **OXC:** `captures` field present but not fully implemented

---

## Component Boundaries

### SWC Architecture
```
lib.rs (public API)
  |
  v
parse.rs (orchestration)
  |
  +-> transform.rs (QwikTransform - main logic)
  |     |
  |     +-> collector.rs (GlobalCollect - imports/exports)
  |     +-> entry_strategy.rs (bundling decisions)
  |
  +-> code_move.rs (new_module - segment generation)
  |
  +-> Supporting modules (const_replace, props_destructuring, etc.)
```

### OXC Architecture
```
lib.rs (module declarations)
  |
  v
js_lib_interface.rs (public API, compatible types)
  |
  v
transform.rs (TransformGenerator - main orchestration + logic)
  |
  +-> component/ (QRL/segment data structures)
  |     +-> component.rs (QrlComponent generation)
  |     +-> qrl.rs (Qrl struct, call expression building)
  |     +-> id.rs (Id struct, hash generation)
  |     +-> shared.rs (Import, Target)
  |     +-> source_info.rs (SourceInfo)
  |
  +-> segment.rs (Segment enum, SegmentBuilder)
  +-> entry_strategy.rs (bundling decisions)
  +-> import_clean_up.rs (import normalization)
  +-> illegal_code.rs (validation)
```

---

## Porting Strategy Recommendations

### Phase 1: Core Parity (Current State)
- Basic QRL extraction works
- JSX transformation works
- Import tracking works

### Phase 2: Feature Completion
1. **Source maps** - Add OXC source map generation
2. **Captures/scoped idents** - Implement `useLexicalScope` injection
3. **Const replacement** - Port `const_replace.rs` logic
4. **Props destructuring** - Port `props_destructuring.rs`

### Phase 3: Optimization Features
1. **Treeshaking** - Port `clean_side_effects.rs`
2. **Side effect markers** - Port `add_side_effect.rs`
3. **Entry strategies** - Complete `Component` and `Smart` strategies

### Phase 4: Edge Cases
1. **Filter exports** - Port `strip_exports` support
2. **Inlined functions** - Port `inlined_fn.rs`
3. **All test cases** - Ensure snapshot parity

---

## Sources

- Direct analysis of:
  - `/Users/jackshelton/dev/open-source/qwik-optimizer/qwik-core/src/optimizer/core/src/`
  - `/Users/jackshelton/dev/open-source/qwik-optimizer/optimizer/src/`

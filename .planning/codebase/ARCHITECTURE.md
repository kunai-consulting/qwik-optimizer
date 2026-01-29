# Architecture

**Analysis Date:** 2026-01-29

## Pattern Overview

**Overall:** AST-based Code Transformer with Entry Strategy Pattern

**Key Characteristics:**
- Two-tier Rust architecture: core optimizer library (`optimizer`) + Node.js N-API bindings (`napi`)
- AST manipulation using Oxc (Oxide/JavaScript parser ecosystem) for high-performance code transformation
- Modular visitor pattern for traversing and transforming JavaScript/TypeScript code
- Strategy pattern for entry point generation and code segmentation
- Heavy use of Rust traits and extension methods for type-safe AST operations

## Layers

**Entry Point Layer (Node.js Integration):**
- Purpose: Expose Rust optimizer to JavaScript/TypeScript environments via N-API bridge
- Location: `napi/src/lib.rs`
- Contains: JavaScript function bindings, async/tokio coordination, serde JSON marshalling
- Depends on: `qwik-optimizer` (core optimizer), `napi` crate, `tokio` runtime
- Used by: Node.js/JavaScript applications building Qwik projects

**Public Interface Layer:**
- Purpose: Define JS-compatible input/output types and the main transformation orchestrator
- Location: `optimizer/src/js_lib_interface.rs`
- Contains: `TransformModulesOptions`, `TransformOutput`, `TransformModule`, `SegmentAnalysis`, minify modes, entry strategies
- Depends on: Core transformation modules, component types, error handling
- Used by: NAPI bridge, direct Rust consumers

**Core Transformation Layer:**
- Purpose: Main AST visitor and code transformation engine
- Location: `optimizer/src/transform.rs`
- Contains: `TransformGenerator` (large visitor implementation), `OptimizedApp`, JSX state tracking, symbol resolution
- Depends on: All extension modules, component system, segment tracking, import cleanup, illegal code detection
- Used by: Public interface to perform actual code transformation

**Component System Layer:**
- Purpose: Represent and manage component abstractions in the code
- Location: `optimizer/src/component/` (mod.rs, component.rs, id.rs, qrl.rs, shared.rs, source_info.rs, language.rs)
- Contains: Component definitions (`QrlComponent`), identifier generation (`Id`), QRL (qualified runtime literal) representations
- Depends on: Segment system, AST builder extensions
- Used by: Transformation layer to create extracted components

**Analysis & Optimization Layer:**
- Purpose: Detect patterns and apply optimizations
- Location: `optimizer/src/dead_code.rs`, `illegal_code.rs`, `import_clean_up.rs`, `ref_counter.rs`
- Contains: Dead code detection, illegal construct validation, unused import removal, reference counting
- Depends on: Oxc AST types
- Used by: Transformation layer to guide optimization decisions

**Segmentation Layer:**
- Purpose: Uniquely identify and track code segments for extraction
- Location: `optimizer/src/segment.rs`
- Contains: `Segment` enum (Named, NamedQrl, IndexQrl), `SegmentBuilder` for unique naming
- Depends on: Component/ID system
- Used by: Transformation layer for scope and extraction tracking

**Entry Strategy Layer:**
- Purpose: Determine how extracted functions map to entry points
- Location: `optimizer/src/entry_strategy.rs`
- Contains: `EntryPolicy` trait, `InlineStrategy`, `SingleStrategy`, `PerSegmentStrategy`, `PerComponentStrategy`, `SmartStrategy`
- Depends on: Segment system
- Used by: Transformation layer to place extracted code

**Extension Layer:**
- Purpose: Provide type-safe helpers for AST manipulation
- Location: `optimizer/src/ext/` (ast_builder_ext.rs, expression_ext.rs)
- Contains: Trait implementations on `AstBuilder` and expression types for common patterns
- Depends on: Oxc AST types
- Used by: Component, QRL, and transformation layers

**Error & Source Layer:**
- Purpose: Type-safe error handling and source code management
- Location: `optimizer/src/error.rs`, `prelude.rs`, `source.rs`, `processing_failure.rs`
- Contains: Custom error enum with thiserror, Result type alias, Source enum for file/string input, ProcessingFailure for non-fatal issues
- Depends on: Oxc error types, std I/O
- Used by: All layers for error propagation

## Data Flow

**Module Transformation Flow:**

1. **Input Ingestion:**
   - JavaScript/TypeScript code arrives via N-API as `TransformModulesOptions` JSON
   - Each `TransformModuleInput` contains file path, dev path, and source code

2. **Parsing & Semantic Analysis:**
   - `TransformGenerator` parses code with Oxc parser using language-appropriate `SourceType` (JavaScript, JSX, TypeScript, TSX)
   - Oxc `SemanticBuilder` constructs symbol tables, scope trees, and reference information
   - `SourceInfo` extracts file metadata (language, relative path, file name)

3. **Traversal & Transformation:**
   - `TransformGenerator` implements `Traverse` trait to visit AST nodes in depth-first order
   - `enter_*` methods descend into AST subtrees, tracking segment stack and import state
   - `exit_*` methods build extracted components and replace original code with QRL references
   - `TraverseCtx` provides mutable AST access and semantic information throughout

4. **Segment Extraction:**
   - When QRL marker (`$`) is encountered, a `Segment` is pushed to track context
   - Scope identifiers are collected as segment path for unique naming
   - `Id` is generated using segment path + hash of file + display name
   - Expression is wrapped in `QrlComponent` with imports

5. **Component Generation:**
   - `Qrl` objects create call expressions like `qrl(() => import("./module"), "displayName")`
   - `QrlComponent::gen()` uses `AstBuilder` to reconstruct variable declarations with proper exports
   - Codegen with minification produces final component code string

6. **Import Management:**
   - Imports are tracked in `import_stack` (per-scope)
   - `ImportCleanUp` removes unused imports at program exit
   - Synthesized imports (e.g., for `qrl` function) are added if not present

7. **Output Generation:**
   - Transformed program body is code-generated with optional minification
   - `TransformModule` objects collect path, code, optional source map, segment analysis, entry flag
   - `TransformOutput` aggregates modules and diagnostics

**State Management:**

- **Segment Stack:** `Vec<Segment>` tracks nesting of extracted functions (context)
- **Component Stack:** `Vec<QrlComponent>` accumulates extracted components during traversal
- **Import Stack:** `Vec<BTreeSet<Import>>` maintains scope-aware imports
- **Symbol Table:** `HashMap<String, SymbolId>` maps names to their semantic identities
- **Const Stack:** `Vec<BTreeSet<SymbolId>>` tracks which identifiers are constant (immutable)

## Key Abstractions

**TransformGenerator:**
- Purpose: Central visitor orchestrating the entire transformation
- Examples: `optimizer/src/transform.rs:169-201`
- Pattern: Visitor pattern (via `Traverse` trait) with mutable state for AST walking

**QrlComponent:**
- Purpose: Represents an extracted, serializable component with code and metadata
- Examples: `optimizer/src/component/component.rs:14-19`
- Pattern: Data structure wrapping AST-to-string compilation with QRL metadata

**Qrl:**
- Purpose: Represents a lazy-loaded reference to extracted code via dynamic import
- Examples: `optimizer/src/component/qrl.rs:37-50`
- Pattern: Builder-like methods to construct call expressions (`qrl(() => import(...), "name")`)

**Segment:**
- Purpose: Uniquely identifies a code extraction point in scope hierarchy
- Examples: `optimizer/src/segment.rs:9-24`
- Pattern: Enum with variants for named scopes and indexed QRL slots

**EntryPolicy (Strategy Pattern):**
- Purpose: Determines code placement strategy for extracted functions
- Examples: `optimizer/src/entry_strategy.rs:19-80`
- Pattern: Strategy trait with implementations for different bundling approaches

**Id:**
- Purpose: Generates deterministic, unique identifiers for components
- Examples: `optimizer/src/component/id.rs:130-195`
- Pattern: Builder with hash-based uniqueness; sanitization of names

## Entry Points

**N-API Module Export:**
- Location: `napi/src/lib.rs:37-42`
- Triggers: Node.js `require()` or ES module import of compiled NAPI module
- Responsibilities: Exposes single `transform_modules` function to JavaScript

**Transform Modules Function:**
- Location: `napi/src/lib.rs:18-35`
- Triggers: JavaScript call with `TransformModulesOptions` JSON
- Responsibilities: Parse options, spawn blocking task, convert result to JS value

**Core Rust Entry (js_lib_interface::transform_modules):**
- Location: `optimizer/src/js_lib_interface.rs:XXX` (not shown in excerpt but called by NAPI)
- Triggers: NAPI wrapper
- Responsibilities: Accept options, orchestrate per-module transformation, aggregate outputs

**Program Traversal Entry:**
- Location: `optimizer/src/transform.rs:266-271` (enter_program method)
- Triggers: AST visitor begins traversing Program node
- Responsibilities: Initialize transformation state, log entry

## Error Handling

**Strategy:** Custom error enum with `thiserror` for automatic `Display` and `Error` trait impl

**Patterns:**

- **Recoverable Errors (Non-Fatal):** Stored in `ProcessingFailure` objects accumulated in `errors: Vec<ProcessingFailure>` within `OptimizationResult`; transformation continues producing partial results
  - Location: `optimizer/src/processing_failure.rs`

- **Fatal Errors (Unrecoverable):** Propagated via `Result<T>` with custom `Error` enum; cause immediate halt
  - Location: `optimizer/src/error.rs`
  - Examples: I/O errors, invalid language/file extension, illegal code in QRL scope

- **Illegal Code Detection:** Raises `Error::IllegalCode(IllegalCodeType)` when classes/functions appear inside QRL scope
  - Location: `optimizer/src/illegal_code.rs:37-58`
  - Error message includes identifier name and expression type (class/function)

## Cross-Cutting Concerns

**Logging:** Debug-only println! statements controlled by `DEBUG` const in `transform.rs:262`
- Includes scope ID, recording state, and segment path information
- Conditionally compiled and disabled in release builds

**Validation:** Multi-phase validation
- Syntax validation during parsing
- Semantic validation via Oxc semantic builder (symbol resolution, scope correctness)
- Legal code validation for QRL-scoped expressions (via `IllegalCode` trait)

**Authentication:** Not applicable (core transformation is stateless)

**AST Lifecycle:**
- Allocator: Oxc `Allocator` manages all AST node memory (arena allocation)
- Cloning: Nodes are cloned into target allocator with `CloneIn`, `FromIn`, `IntoIn` traits
- Codegen: Final AST rendered to JavaScript string via `Codegen` with optional minification settings

---

*Architecture analysis: 2026-01-29*

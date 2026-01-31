# Codebase Structure

**Analysis Date:** 2026-01-29

## Directory Layout

```
qwik-optimizer/
├── Cargo.toml              # Workspace manifest (members: napi, optimizer)
├── Cargo.lock              # Dependency lock file
├── README.md               # Project overview
├── LICENSE                 # MIT license
├── flake.nix               # Nix development environment
├── flake.lock              # Nix lock file
├── .envrc                  # Direnv configuration
├── .gitignore              # Git ignore patterns
├── .cargo/                 # Cargo configuration directory
│   └── config.toml
│
├── optimizer/              # Core Rust optimizer library
│   ├── Cargo.toml          # Package manifest: qwik-optimizer v0.1.0
│   └── src/
│       ├── lib.rs          # Library entry: module declarations, public exports
│       ├── prelude.rs      # Type aliases: Result<T> = Result<T, Error>
│       ├── error.rs        # Error enum: thiserror-based custom errors
│       ├── source.rs       # Source enum: ScriptFile abstraction
│       ├── macros.rs       # Macro definitions and utilities
│       │
│       ├── js_lib_interface.rs  # PUBLIC: Options/output types for JS binding
│       │   ├── TransformModulesOptions (input config)
│       │   ├── TransformOutput (aggregated result)
│       │   ├── TransformModule (per-module output)
│       │   ├── SegmentAnalysis (metadata)
│       │   ├── MinifyMode enum
│       │   └── Diagnostic type
│       │
│       ├── transform.rs    # Core transformation: TransformGenerator visitor
│       │   ├── OptimizedApp (final output)
│       │   ├── TransformOptions (input options)
│       │   ├── TransformGenerator (main Traverse impl)
│       │   ├── JsxState (JSX tracking)
│       │   └── AST visitor methods (enter_*/exit_*)
│       │
│       ├── component/      # Component system
│       │   ├── mod.rs      # Module flattening (re-exports all submodules)
│       │   ├── component.rs   # QrlComponent: code + metadata
│       │   ├── id.rs          # Id: unique component identifier generation
│       │   ├── qrl.rs         # Qrl: lazy import call construction
│       │   ├── shared.rs      # Import, ImportId, Target enum
│       │   ├── source_info.rs # SourceInfo: file metadata
│       │   └── language.rs    # Language enum: JS/TS/JSX/TSX detection
│       │
│       ├── segment.rs      # Segment: code location uniquification
│       │   ├── Segment enum (Named, NamedQrl, IndexQrl)
│       │   ├── SegmentBuilder (unique naming state)
│       │   └── Helper functions (make_fq_name, make_unique_segment_name)
│       │
│       ├── entry_strategy.rs  # EntryPolicy trait + strategy implementations
│       │   ├── EntryStrategy enum
│       │   ├── EntryPolicy trait
│       │   ├── InlineStrategy
│       │   ├── SingleStrategy
│       │   ├── PerSegmentStrategy
│       │   ├── PerComponentStrategy (unimplemented)
│       │   └── SmartStrategy (unimplemented)
│       │
│       ├── ext/           # Extension traits for AST manipulation
│       │   ├── mod.rs     # Re-exports ast_builder_ext, expression_ext
│       │   ├── ast_builder_ext.rs  # AstBuilder helper methods
│       │   └── expression_ext.rs   # Expression type helpers
│       │
│       ├── dead_code.rs   # DeadCode trait: empty body detection
│       ├── illegal_code.rs  # IllegalCode trait: forbidden constructs in QRL
│       ├── import_clean_up.rs  # ImportCleanUp: unused import removal
│       ├── ref_counter.rs    # RefCounter: reference tracking
│       ├── processing_failure.rs  # ProcessingFailure: non-fatal error collection
│       │
│       └── test_input/    # Test data directory
│           ├── test_project_1/
│           │   ├── package.json
│           │   ├── tsconfig.json
│           │   └── package-lock.json
│           ├── test_example_1.tsx  # Example TSX file for tests
│           └── snapshots/  # Insta snapshot test data
│
├── napi/                   # Node.js N-API bindings
│   ├── Cargo.toml          # Package manifest: qwik-optimizer-napi v0.1.0
│   ├── build.rs            # Build script for N-API
│   └── src/
│       └── lib.rs          # NAPI export: transform_modules function
│           ├── transform_modules (JS-facing function)
│           └── module_exports (initialization)
│
└── .planning/              # GSD planning directory
    └── codebase/           # Codebase analysis documents
        ├── ARCHITECTURE.md  # (this analysis)
        └── STRUCTURE.md     # (this analysis)
```

## Directory Purposes

**optimizer/:**
- Purpose: Core Rust optimization library, language and platform-independent
- Contains: AST parsing/transformation, component extraction, code generation
- Key files: `transform.rs` (largest, most complex), `js_lib_interface.rs` (public API), `component/` (domain model)

**optimizer/src/component/:**
- Purpose: Component abstraction layer - represents extracted functions as serializable units
- Contains: Component identity, QRL construction, language detection, source metadata
- Key files: `id.rs` (deterministic ID generation), `qrl.rs` (lazy-load representation), `shared.rs` (core types)

**optimizer/src/ext/:**
- Purpose: Type-safe extension methods on Oxc AST builder and expressions
- Contains: Helper functions to simplify AST node construction
- Key files: `ast_builder_ext.rs` (import/function statement builders), `expression_ext.rs` (utility methods)

**napi/:**
- Purpose: JavaScript interop layer - exposes Rust optimizer to Node.js
- Contains: NAPI bindings, async coordination, JSON marshalling
- Key files: `lib.rs` (single entry point for JS)

**optimizer/src/test_input/:**
- Purpose: Test data and fixtures for unit/integration tests
- Contains: Sample TypeScript/JSX files, project configurations, snapshot expectations
- Key files: Snapshots from Insta testing framework, example source files

## Key File Locations

**Entry Points:**

- `napi/src/lib.rs:18-35`: JavaScript-facing `transform_modules` N-API function
- `napi/src/lib.rs:37-42`: Module initialization that exposes the function
- `optimizer/src/js_lib_interface.rs`: Public interface types and orchestration function (not shown but called by NAPI)

**Configuration:**

- `Cargo.toml` (root): Workspace configuration, member specification
- `Cargo.toml` (optimizer): Dependencies: Oxc v0.94, serde, thiserror, base64
- `Cargo.toml` (napi): Dependencies: napi v2, tokio (async runtime), qwik-optimizer (local)
- `flake.nix`: Nix environment setup (Rust toolchain, Cargo)

**Core Logic:**

- `optimizer/src/transform.rs`: Main `TransformGenerator` struct (600+ lines); implements `Traverse` trait for AST walking
- `optimizer/src/component/component.rs`: `QrlComponent` definition and code generation
- `optimizer/src/component/id.rs`: Deterministic identifier generation with hashing (348 lines with tests)
- `optimizer/src/component/qrl.rs`: QRL call expression construction (330 lines with examples)

**Testing:**

- `optimizer/src/test_input/`: Test fixtures and project examples
- `optimizer/src/*/mod.rs` with `#[cfg(test)]` blocks: Unit tests within modules
- Insta snapshots: `optimizer/src/snapshots/` (snapshot-based testing)

## Naming Conventions

**Files:**

- `snake_case.rs` for modules: `dead_code.rs`, `import_clean_up.rs`, `entry_strategy.rs`
- `lib.rs`: Single entry point for crate
- `mod.rs`: Module flattening/re-exports in subdirectories (e.g., `component/mod.rs`)

**Directories:**

- `src/` prefix convention not used; modules are at root of `src/`
- Thematic grouping: `component/` clusters component-related modules
- `ext/` for extension traits
- `test_input/` for test data

**Type Naming:**

- PascalCase for structs, enums, traits: `TransformGenerator`, `QrlComponent`, `Segment`, `Id`, `EntryPolicy`
- camelCase for enum variants: `Inline`, `Single`, `Component`, `Smart` (in `EntryStrategy`)
- SCREAMING_SNAKE_CASE for constants: `DEBUG`, `ENTRY_SEGMENTS`, `QWIK_CORE_SOURCE`, `QRL_SUFFIX`

**Function Naming:**

- snake_case for functions: `new_segment()`, `enter_program()`, `exit_expression()`, `into_identifier_reference()`
- `is_*` for predicates: `is_recording()`, `is_dead_code()`, `is_illegal_code_in_qrl()`
- `into_*` for conversions (Rust idiom): `into_identifier_reference()`, `into_call_expression()`, `into_statement()`
- `with_*` for builder methods: `with_options()`

## Where to Add New Code

**New Feature (e.g., New Optimization Pass):**
- Primary code: `optimizer/src/new_feature.rs` (create new module at top level)
- Integration point: Add module declaration to `optimizer/src/lib.rs`
- Tests: Include `#[cfg(test)]` mod within the same file or test_input fixtures
- Example: Following pattern of `dead_code.rs` or `illegal_code.rs` (single trait + implementations)

**New Component Type (e.g., New Segment Variant):**
- Implementation: `optimizer/src/component/mod_name.rs`
- Module flattening: Add to `optimizer/src/component/mod.rs` pub(crate) use statement
- Tests: Unit tests in module file; integration in `transform.rs` tests

**Utilities/Helpers:**
- Shared helpers: `optimizer/src/ext/new_ext.rs` if extending Oxc types
- Or: `optimizer/src/prelude.rs` if adding to Result/Error handling
- Example: Follow `ast_builder_ext.rs` pattern for adding AstBuilder methods

**AST Visitor Methods (e.g., New Node Type):**
- Add to `TransformGenerator` impl block in `optimizer/src/transform.rs`
- Follow naming: `enter_node_type()` and `exit_node_type()`
- State management: Use `self.component_stack`, `self.segment_stack`, `self.import_stack` as needed

**JavaScript Binding:**
- Modify `napi/src/lib.rs` to expose new function (use `#[js_function]` macro)
- Add types to `optimizer/src/js_lib_interface.rs` for serialization/deserialization
- Test via Node.js require and Jest/Vitest

## Special Directories

**snapshots/:**
- Purpose: Insta snapshot testing framework data for regression testing
- Generated: Yes (auto-generated by Insta, reviewed in PRs)
- Committed: Yes (committed to git for regression detection)
- Location: `optimizer/src/snapshots/`

**test_input/:**
- Purpose: Fixture files and projects for unit/integration testing
- Generated: No (manually created and maintained)
- Committed: Yes (part of test suite)
- Location: `optimizer/src/test_input/`

**.cargo/:**
- Purpose: Cargo configuration (manifest), workspace settings
- Generated: No (committed to git)
- Committed: Yes
- Content: Cargo config for build behavior

**.planning/codebase/:**
- Purpose: GSD codebase analysis documents (architecture, structure, testing, etc.)
- Generated: Yes (by `/gsd:map-codebase` agent)
- Committed: Yes (part of knowledge base for other GSD commands)
- Content: ARCHITECTURE.md, STRUCTURE.md, CONVENTIONS.md, TESTING.md, CONCERNS.md (as written)

---

*Structure analysis: 2026-01-29*

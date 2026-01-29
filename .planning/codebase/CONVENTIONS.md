# Coding Conventions

**Analysis Date:** 2026-01-29

## Naming Patterns

**Files:**
- Snake case for Rust files: `js_lib_interface.rs`, `processing_failure.rs`, `entry_strategy.rs`
- Tests co-located in modules with `#[cfg(test)] mod tests { ... }`
- Test functions prefixed with `test_`: `test_example_1()`, `test_calculate_hash()`, `test_project_1()`
- Snapshot files stored in `src/snapshots/` directory with naming pattern: `<module_path>__<test_name>.snap`

**Functions:**
- Snake case for function names: `transform_modules()`, `error_to_diagnostic()`, `can_load_from_file()`
- Helper/private functions use private visibility: `fn sanitize()`, `fn calculate_hash()`
- Public API functions use `pub fn`: `pub fn transform_modules()`
- Method names follow action verbs: `get_component()`, `create_import_statement()`, `is_dead_code()`

**Variables:**
- Snake case for local variables and parameters: `src_dir`, `root_dir`, `display_name`, `symbol_name`
- Short, descriptive names: `path`, `code`, `opts`, `env`, `result`
- Prefixed names for iterations: `func_name`, `source_info`, `local_file_name`

**Types:**
- PascalCase for struct names: `TransformModulesOptions`, `OptimizedApp`, `ProcessingFailure`, `QrlComponent`
- PascalCase for enum names: `DiagnosticCategory`, `MinifyMode`, `Target`, `Language`
- Generic lifetimes use single letter: `'a`, `'_`
- Allocator type parameters often use generic: `Box as OxcBox`, `Vec as OxcVec`

**Constants:**
- SCREAMING_SNAKE_CASE: `TEST_FILE` (only constant found in tests)

## Code Style

**Formatting:**
- Rust edition 2021 (`edition = "2021"` in Cargo.toml)
- 4-space indentation (standard Rust)
- Line length appears to follow standard Rust conventions
- No explicit formatter configured (no `.rustfmt.toml` found)

**Linting:**
- Clippy warnings treated as hard denials via `#![deny(clippy::all)]` in `napi/src/lib.rs`
- Additional Clippy categories denied: `clippy::perf`, `clippy::nursery`
- Allows specific features: `#![allow(unused)]` in `transform.rs` for conditional compilation

**Visibility:**
- Public API marked with `pub`: `pub mod component`, `pub struct OptimizedApp`
- Private internals use `mod` without pub or `#[allow(dead_code)]`
- Types intended for FFI/NAPI use `pub struct` with serde derives

## Import Organization

**Order:**
1. Internal crate modules: `use crate::error::Error;`
2. External crates: `use oxc_ast::ast::*;`, `use serde::{Deserialize, Serialize};`
3. Standard library: `use std::fs;`, `use std::path::Path;`
4. Module flattening via re-exports: `pub(crate) use component::*;` in `mod.rs` files

**Module Structure:**
- `pub mod <name>` for public modules
- `pub(crate) mod <name>` for crate-private modules
- `mod <name>` for fully private modules
- Barrel files (mod.rs) aggregate and flatten submodules for cleaner imports

**Path Aliases:**
- No path aliases detected; uses direct `crate::` prefix imports
- Type aliases for common patterns: `pub type Result<T> = core::result::Result<T, Error>;` in `prelude.rs`

## Error Handling

**Patterns:**
- Custom error enum with `thiserror` derive: `#[derive(thiserror::Error, Debug)]` in `error.rs`
- Error variants use `#[error(...)]` attribute for display messages
- Transparent error forwarding with `#[from]` for IO and parsing errors
- Result type alias defined in prelude: `pub type Result<T> = core::result::Result<T, Error>;`
- Errors converted to diagnostics in public API layer: `fn error_to_diagnostic(error: ProcessingFailure, path: &Path) -> Diagnostic`
- Processing failures logged as diagnostics with category (Error/Warning) in output struct

**Error Recovery:**
- Errors wrapped in `Result` and propagated with `?` operator
- Configuration validation happens at boundaries (js_lib_interface.rs)
- Partial results allowed via `Option` in some transforms: `Option<TransformOutput>` summed into final result

## Logging

**Framework:** No logging framework detected (no log, tracing, or env_logger crate)

**Patterns:**
- Diagnostic output structured in `TransformOutput` and `Diagnostic` types
- Debug output via `println!` macros in test helpers: `println!("Loading test input file from path: {:?}", &path);`
- `#[derive(Debug)]` used on public types for inspection
- Snapshot testing via `insta` crate captures full output for regression testing

## Comments

**When to Comment:**
- Used for documentation comments on public APIs: `/// Represents a component identifier...`
- Explanatory comments for complex logic: `// Never push consecutive underscores.`
- Comments on disabled tests explain known issues: `// Not removing:`, `// Not converting:`
- Large comment blocks explain test requirements and design decisions

**Doc Comments:**
- Triple-slash `///` for public types and methods
- Structured with sections: `# Segments`, `# Target`, `# Hash Generation Semantics`
- Example code in doc comments for public functions
- Field documentation where needed: `pub optional: bool, // Meaning explained here`

**Inline Comments:**
- Sparse but present for non-obvious logic
- TODO comments present for tracked improvements: `// TODO: Set this flag correctly`
- Explains conditional logic and edge cases

## Function Design

**Size:**
- Range from tiny (5 lines) to large (100+ lines for complex transforms)
- Most utility functions 10-50 lines
- Larger functions have helper sub-functions or trait implementations
- Longest file is `transform.rs` with 1186 lines (top-level struct with many impl blocks)

**Parameters:**
- Typically 2-5 parameters for most functions
- Complex options passed as `Struct` rather than many parameters: `TransformModulesOptions`
- Reference parameters for borrowed data: `&self`, `&str`, `&Option<String>`
- Generic lifetimes used for borrowed AST nodes: `Expression<'_>`, `Statement<'_>`

**Return Values:**
- `Result<T>` for fallible operations
- `Option<T>` for optional results (e.g., `get_component()` returns `Option<&QrlComponent>`)
- Owned values returned: `String`, `Vec<Module>`, `TransformOutput`
- Trait implementations implement standard protocols: `Display`, `PartialOrd`, `From`

## Module Design

**Exports:**
- `pub mod` for public modules in library API
- `pub(crate) mod` for internal modules
- Explicit re-exports via `pub use` in barrel files
- Types marked `pub` for serialization boundaries (NAPI)

**Barrel Files:**
- `component/mod.rs` flattens submodules: `pub(crate) use component::*;`
- Allows `use crate::component::Id` instead of `use crate::component::id::Id`
- Simplifies imports for users of the library

**Visibility Layers:**
- Public API in `js_lib_interface.rs`: `pub fn transform_modules(config: TransformModulesOptions) -> Result<TransformOutput>`
- Core logic in `transform.rs` as `pub(crate)` or module-private
- Supporting modules rarely cross-reference; primarily hierarchical
- NAPI bindings in separate crate (`napi/`) expose safe Rust interface

## Type Serialization

**Serde Usage:**
- `#[derive(Serialize, Deserialize)]` on public types
- `#[serde(rename_all = "camelCase")]` for JavaScript interop (CamelCase)
- Field skipping: `#[serde(skip_serializing)]` for internal-only fields
- Works with both JSON and YAML (insta supports both)

---

*Convention analysis: 2026-01-29*

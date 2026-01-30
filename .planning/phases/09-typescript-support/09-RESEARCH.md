# Phase 9: TypeScript Support - Research

**Researched:** 2026-01-29
**Domain:** TypeScript TSX parsing, type annotation handling, type-only import detection
**Confidence:** HIGH

## Summary

Phase 9 implements comprehensive TypeScript support for the Qwik optimizer, ensuring TSX files parse correctly, type annotations are handled appropriately, and type-only imports are excluded from QRL capture arrays. The implementation builds on existing OXC infrastructure already in the codebase.

The current implementation already has most TypeScript support in place:
1. **TSX Parsing**: OXC 0.111.0 parses TSX files correctly via `SourceType::tsx()` (verified in `component/language.rs`)
2. **Type Stripping**: The `oxc_transformer` with `TypeScriptOptions::default()` strips types when `transpile_ts: true` (verified in `transform.rs` lines 2862-2878)
3. **Type-Only Imports**: OXC provides `ImportOrExportKind::Type` enum to identify type-only imports

The main work is ensuring type-only imports are excluded from QRL capture, as captured imports must be runtime values. Generic component types need validation that they work correctly with the existing JSX transformation.

**Primary recommendation:** Extend the existing import tracking to filter out type-only imports (`ImportOrExportKind::Type`) before they can be captured in QRL arrays. The OXC infrastructure already handles parsing and stripping; this phase focuses on correct capture behavior.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| oxc_parser | 0.111.0 | Parse TSX with type annotations | Already parsing TSX correctly |
| oxc_transformer | 0.111.0 | Strip TypeScript types | Already integrated in transform() |
| oxc_ast | 0.111.0 | AST types including ImportOrExportKind | Provides type-only detection |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| oxc_semantic | 0.111.0 | Symbol resolution | For tracking type vs value bindings |
| oxc_traverse | 0.111.0 | AST traversal | For import collection |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| OXC TypeScript transform | Manual type stripping | OXC handles edge cases correctly |
| Custom type detection | ImportOrExportKind enum | OXC already parses and exposes this |

**Installation:**
No new dependencies needed - uses existing OXC 0.111.0 infrastructure.

## Architecture Patterns

### Recommended Project Structure
```
optimizer/src/
├── transform.rs           # Existing: Already applies TypeScript transform
├── collector.rs           # UPDATE: Filter type-only imports from capture
├── import_clean_up.rs     # UPDATE: Skip type-only imports in cleanup
└── component/language.rs  # Existing: Already handles TSX source type
```

### Pattern 1: Type-Only Import Detection
**What:** Check `import_kind` field on ImportDeclaration and ImportSpecifier nodes
**When to use:** During import collection before QRL capture
**Example:**
```rust
// Source: OXC AST documentation at docs.rs/oxc_ast
use oxc_ast::ast::{ImportDeclaration, ImportOrExportKind};

// Check whole-statement type import: `import type { Foo } from '...'`
if import.import_kind == ImportOrExportKind::Type {
    // Skip - this is a type-only import
    continue;
}

// Check inline type specifier: `import { type Foo, bar } from '...'`
if let ImportDeclarationSpecifier::ImportSpecifier(spec) = specifier {
    if spec.import_kind == ImportOrExportKind::Type {
        // Skip - this specific specifier is type-only
        continue;
    }
}
```

### Pattern 2: Type-Aware Import Tracking
**What:** Extend ImportTracker to track whether imports are value or type
**When to use:** During initial import collection pass
**Example:**
```rust
// Extend existing ImportTracker in transform.rs
impl ImportTracker {
    /// Record an import, tracking whether it's a type-only import
    pub fn add_import(&mut self, source: &str, specifier: &str, local: &str, is_type_only: bool) {
        if is_type_only {
            return; // Don't track type-only imports for QRL capture
        }
        self.imports.insert(
            (source.to_string(), specifier.to_string()),
            local.to_string(),
        );
    }
}
```

### Pattern 3: TypeScript Transform Integration
**What:** Apply oxc_transformer for type stripping before main transform
**When to use:** When `transpile_ts: true` in TransformOptions
**Example:**
```rust
// Already implemented in transform.rs lines 2862-2878
if options.transpile_ts {
    let SemanticBuilderReturn { semantic, .. } = SemanticBuilder::new().build(&program);
    let scoping = semantic.into_scoping();
    Transformer::new(
        &allocator,
        source_info.rel_path.as_path(),
        &OxcTransformOptions {
            typescript: TypeScriptOptions::default(),
            jsx: JsxOptions::disable(),
            ..OxcTransformOptions::default()
        },
    )
    .build_with_scoping(scoping, &mut program);
}
```

### Anti-Patterns to Avoid
- **Capturing type-only imports:** Will cause runtime errors as types don't exist at runtime
- **Manual type stripping:** OXC's transformer handles all edge cases correctly
- **Ignoring inline type specifiers:** `import { type Foo, bar }` has both type and value

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| TypeScript parsing | Custom TS parser | OXC parser with tsx() | OXC handles all TS syntax |
| Type annotation stripping | Manual AST removal | oxc_transformer TypeScriptOptions | Handles generics, decorators, edge cases |
| Type vs value import detection | Heuristics | ImportOrExportKind enum | OXC already parses and exposes this |
| Generic type handling | Custom generic resolution | OXC type stripping | Generics on components just disappear after stripping |

**Key insight:** OXC already does the hard work of TypeScript parsing and type stripping. This phase is about ensuring the optimizer correctly handles the type-stripped output and doesn't capture type-only references.

## Common Pitfalls

### Pitfall 1: Inline Type Specifiers
**What goes wrong:** `import { type Foo, bar } from '...'` captures `Foo` as if it were a value
**Why it happens:** Only checking `ImportDeclaration.import_kind`, not `ImportSpecifier.import_kind`
**How to avoid:** Check BOTH the declaration-level and specifier-level `import_kind`
**Warning signs:** Runtime errors about undefined `Foo` in QRL segment

### Pitfall 2: Type-Only Re-exports
**What goes wrong:** `export type { Foo }` treated as value export
**Why it happens:** Not checking `export_kind` on ExportNamedDeclaration
**How to avoid:** Check `ExportNamedDeclaration.export_kind` for Type variant
**Warning signs:** Type references appearing in segment imports

### Pitfall 3: Generic Component Types Lost
**What goes wrong:** `const Comp: Component<Props>` loses type after transform
**Why it happens:** Expecting type annotations to survive transformation
**How to avoid:** Types are correctly stripped by OXC - this is expected behavior
**Warning signs:** None - this is correct behavior, just verify it works

### Pitfall 4: Import Tracking Before Type Stripping
**What goes wrong:** Collecting imports before TypeScript transform, then imports are stripped
**Why it happens:** Import collection happens on pre-transform AST
**How to avoid:** Either:
  1. Filter type-only imports during collection (check import_kind)
  2. Re-collect imports after TypeScript transform
**Warning signs:** Captured imports that don't exist in output

### Pitfall 5: TypeScript Transform Order
**What goes wrong:** TypeScript transform runs after QRL extraction
**Why it happens:** Wrong order in transform pipeline
**How to avoid:** TypeScript stripping must happen BEFORE main transformation (already correct in codebase)
**Warning signs:** Type annotations appearing in QRL segment code

## Code Examples

Verified patterns from official sources:

### ImportOrExportKind Enum
```rust
// Source: https://docs.rs/oxc_ast/latest/oxc_ast/ast/enum.ImportOrExportKind.html
#[repr(u8)]
pub enum ImportOrExportKind {
    Value = 0,  // Regular import: import { foo } from '...'
    Type = 1,   // Type-only: import type { foo } from '...' or import { type foo } from '...'
}

// Usage methods
impl ImportOrExportKind {
    pub fn is_value(&self) -> bool;
    pub fn is_type(&self) -> bool;
}
```

### ImportDeclaration Fields
```rust
// Source: https://docs.rs/oxc_ast/latest/oxc_ast/ast/struct.ImportDeclaration.html
pub struct ImportDeclaration<'a> {
    pub span: Span,
    pub specifiers: Option<Vec<'a, ImportDeclarationSpecifier<'a>>>,
    pub source: StringLiteral<'a>,
    pub phase: Option<ImportPhase>,
    pub with_clause: Option<Box<'a, WithClause<'a>>>,
    pub import_kind: ImportOrExportKind,  // Type for `import type { Foo }`
}
```

### ImportSpecifier Fields
```rust
// Source: https://docs.rs/oxc_ast/latest/oxc_ast/ast/struct.ImportSpecifier.html
pub struct ImportSpecifier<'a> {
    pub span: Span,
    pub imported: ModuleExportName<'a>,
    pub local: BindingIdentifier<'a>,
    pub import_kind: ImportOrExportKind,  // Type for `import { type Foo }`
}
```

### Filtering Type-Only Imports During Collection
```rust
// Pattern for transform.rs import collection (lines 2880-2905)
for stmt in &program.body {
    if let Statement::ImportDeclaration(import) = stmt {
        // Skip entire type-only import declarations
        if import.import_kind.is_type() {
            continue;
        }

        let source = import.source.value.to_string();
        if let Some(specifiers) = &import.specifiers {
            for specifier in specifiers {
                match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                        // Skip type-only specifiers within mixed imports
                        if spec.import_kind.is_type() {
                            continue;
                        }
                        let imported = spec.imported.name().to_string();
                        let local = spec.local.name.to_string();
                        import_tracker.add_import(&source, &imported, &local);
                    }
                    // Default and namespace specifiers can't be type-only at specifier level
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                        let local = spec.local.name.to_string();
                        import_tracker.add_import(&source, "default", &local);
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                        let local = spec.local.name.to_string();
                        import_tracker.add_import(&source, "*", &local);
                    }
                }
            }
        }
    }
}
```

### TypeScript Transform Configuration
```rust
// Source: Current transform.rs implementation (lines 2862-2878)
// Already correctly integrated - no changes needed
use oxc_transformer::{TransformOptions as OxcTransformOptions, TypeScriptOptions};

if options.transpile_ts {
    let semantic_return = SemanticBuilder::new().build(&program);
    let scoping = semantic_return.semantic.into_scoping();
    Transformer::new(
        &allocator,
        source_info.rel_path.as_path(),
        &OxcTransformOptions {
            typescript: TypeScriptOptions::default(),
            jsx: JsxOptions::disable(),  // JSX handled separately
            ..OxcTransformOptions::default()
        },
    )
    .build_with_scoping(scoping, &mut program);
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| SWC noop_visit_type!() macro | OXC TypeScriptOptions transform | OXC migration | More declarative type handling |
| Manual type annotation removal | oxc_transformer | OXC 0.100+ | Handles all TS features automatically |

**Deprecated/outdated:**
- `noop_visit_type!()` / `noop_fold_type!()`: SWC macros not available in OXC - use TypeScriptOptions instead
- Manual type node filtering: OXC's transformer handles this

## Open Questions

1. **Namespace Import Type Handling**
   - What we know: `import * as Foo from '...'` with `Foo.Type` usage
   - What's unclear: Does OXC track which namespace members are type vs value?
   - Recommendation: Test this case; may need to capture namespace but let runtime handle

2. **Re-export Chains**
   - What we know: `export { type Foo } from './other'` has `export_kind`
   - What's unclear: Complex re-export chains with mixed types
   - Recommendation: The TypeScript transformer strips these before main transform

## Sources

### Primary (HIGH confidence)
- [OXC ImportDeclaration](https://docs.rs/oxc_ast/latest/oxc_ast/ast/struct.ImportDeclaration.html) - import_kind field documentation
- [OXC ImportSpecifier](https://docs.rs/oxc_ast/latest/oxc_ast/ast/struct.ImportSpecifier.html) - Specifier-level import_kind
- [OXC ImportOrExportKind](https://docs.rs/oxc_ast/latest/oxc_ast/ast/enum.ImportOrExportKind.html) - Value vs Type enum
- [OXC TypeScript transformer docs](https://oxc.rs/docs/guide/usage/transformer/typescript) - onlyRemoveTypeImports option
- `/Users/jackshelton/dev/open-source/qwik-optimizer/optimizer/src/transform.rs` - Existing TypeScript transform integration (lines 2862-2878)
- `/Users/jackshelton/dev/open-source/qwik-optimizer/optimizer/src/component/language.rs` - TSX source type handling

### Secondary (MEDIUM confidence)
- `/Users/jackshelton/dev/open-source/qwik-optimizer/qwik-core/src/optimizer/core/src/collector.rs` - SWC noop_visit_type!() usage (line 128)
- `/Users/jackshelton/dev/open-source/qwik-optimizer/qwik-core/src/optimizer/core/src/parse.rs` - SWC TypeScript transform (lines 244-246)

### Tertiary (LOW confidence)
- None - all patterns verified from official documentation or codebase

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Using existing OXC 0.111.0, patterns verified in docs
- Architecture: HIGH - Existing TypeScript transform already works, minor extension needed
- Pitfalls: HIGH - All derived from OXC documentation and SWC reference

**Research date:** 2026-01-29
**Valid until:** 60 days (OXC API stable, patterns established)

---

## Implementation Summary for Planner

### Required Changes

1. **Update import collection in transform.rs (lines 2880-2905):**
   - Check `import.import_kind.is_type()` to skip type-only declarations
   - Check `spec.import_kind.is_type()` to skip type-only specifiers
   - This prevents type imports from being tracked for QRL capture

2. **Validate existing TypeScript transform:**
   - Verify `transpile_ts: true` path works correctly (already implemented)
   - Add tests for generic component types: `Component<Props>`
   - Add tests for type annotations on function parameters

3. **Add type-only import test cases:**
   - `import type { Foo } from '...'` - whole declaration type-only
   - `import { type Foo, bar } from '...'` - mixed type and value
   - `import { Component } from '@qwik.dev/core'` with only type usage

4. **Verify export type handling:**
   - `export type { Foo }` should not create runtime exports
   - Check `ExportNamedDeclaration.export_kind` if needed

### Existing Functionality (No Changes Needed)

- TSX parsing: `SourceType::tsx()` in `component/language.rs`
- TypeScript transform: `oxc_transformer` integration in `transform.rs`
- Type stripping: `TypeScriptOptions::default()` handles all type removal

### Test Cases Needed

1. **Type-Only Import Exclusion:**
   - `import type { Component } from '@qwik.dev/core'` - not captured
   - `import { type Signal, $ } from '@qwik.dev/core'` - Signal not captured, $ captured

2. **Type Annotation Stripping:**
   - `const comp: Component = component$(() => ...)` - type removed
   - `function foo(props: Props): JSX.Element` - types removed

3. **Generic Components:**
   - `Component<MyProps>` type parameter handled
   - Generic function components work

4. **Mixed Import Handling:**
   - Same import with type and value specifiers
   - Type specifier not in capture array, value specifier is

### Key Files to Modify

| File | Change |
|------|--------|
| `transform.rs` | Filter type-only imports in collection loop (lines 2880-2905) |
| Tests | Add TSX-specific test cases for type handling |

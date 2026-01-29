# Phase 6: Imports & Exports - Research

**Researched:** 2026-01-29
**Domain:** Module system transforms (imports, exports, dynamic imports) for SWC-to-OXC port
**Confidence:** HIGH

## Summary

Phase 6 focuses on module system transformations: capturing imports for QRL scope resolution, cleaning up unused imports after transformation, synthesizing new imports (like `qrl`, `useLexicalScope`), and correctly handling all export types. This phase depends heavily on infrastructure built in phases 1-5.

The SWC reference implementation has well-defined patterns for import/export handling spread across `collector.rs` (GlobalCollect), `transform.rs` (synthetic imports, module body finalization), `code_move.rs` (segment file import generation), and `add_side_effect.rs` (side-effect import preservation). The OXC port already has partial infrastructure: `ImportCleanUp` for unused import removal and Qwik package renaming, `Import`/`ImportId` types in `shared.rs`, and `import_by_symbol`/`symbols_by_name` tracking in `transform.rs`.

The key gap is completing the integration: ensuring imports flow correctly into QRL segment files, synthesized imports are emitted at the right place in the module, side-effect imports are preserved, and all export variants (named, default, re-exports) work correctly.

**Primary recommendation:** Complete the import tracking integration in `QwikTransform`, then implement module finalization to emit synthesized imports and clean up unused ones, using the existing `ImportCleanUp` infrastructure.

## Standard Stack

### Core Components (OXC - Already Implemented)
| Component | File | Purpose | Status |
|-----------|------|---------|--------|
| ImportCleanUp | `optimizer/src/import_clean_up.rs` | Unused import removal + Qwik package renaming | Complete |
| Import | `optimizer/src/component/shared.rs:141-186` | Import statement model | Complete |
| ImportId | `optimizer/src/component/shared.rs:56-139` | Import specifier variants | Complete |
| import_by_symbol | `optimizer/src/transform.rs:165` | Symbol-to-Import mapping for tracking | Partial |
| import_stack | `optimizer/src/transform.rs` | Tracks imports used within current scope | Partial |

### SWC Reference (Specification)
| Component | File | Purpose | Port Priority |
|-----------|------|---------|---------------|
| GlobalCollect | `qwik-core/.../collector.rs:37-125` | Import/export collection at module level | HIGH |
| global_collect.synthetic | `qwik-core/.../collector.rs:38` | Tracks synthesized imports to emit | HIGH |
| fold_module (finalization) | `qwik-core/.../transform.rs:2600-2614` | Module body assembly with imports | HIGH |
| SideEffectVisitor | `qwik-core/.../add_side_effect.rs` | Preserves side-effect imports | MEDIUM |
| create_synthetic_named_import | `qwik-core/.../transform.rs:3300-3318` | Creates import statement AST | HIGH |
| new_module | `qwik-core/.../code_move.rs:35-151` | Segment file generation with imports | HIGH |

### Constants (Already Ported)
| Constant | OXC Location | Value | Used For |
|----------|--------------|-------|----------|
| QWIK_CORE_SOURCE | `shared.rs:11` | `"@qwik.dev/core"` | Synthesized import source |
| QRL | `shared.rs:15` | `"qrl"` | QRL function import |
| _REST_PROPS | `shared.rs:19` | `"_restProps"` | Rest props helper |
| _WRAP_PROP | `shared.rs:22` | `"_wrapProp"` | Signal prop wrapping |
| _FN_SIGNAL | `shared.rs:25` | `"_fnSignal"` | Computed signals |
| _FRAGMENT | `shared.rs:47` | `"_Fragment"` | JSX fragments |

## Architecture Patterns

### Pattern 1: Import Tracking Flow

**What:** Track imports as identifiers are referenced, collect for segment files and cleanup

**Current OXC state:**
```rust
// optimizer/src/transform.rs
struct QwikTransform<'a> {
    import_by_symbol: HashMap<SymbolId, Import>,  // Symbol -> Import mapping
    import_stack: Vec<BTreeSet<Import>>,          // Imports used in current scope
    symbols_by_name: HashMap<String, SymbolId>,   // Name -> Symbol lookup
}
```

**Flow:**
1. `enter_import_declaration`: Register each import specifier in `import_by_symbol`
2. `exit_identifier_reference`: When identifier references an import, add to `import_stack.last()`
3. When creating segment: Clone `import_stack.last()` for segment's required imports
4. End of module: Emit synthesized imports, run ImportCleanUp

**SWC Reference (collector.rs:63-115):**
```rust
impl GlobalCollect {
    pub fn import(&mut self, specifier: &Atom, source: &Atom) -> Id {
        // Check if already imported, return existing local
        // Or create new synthetic import
        let local = id!(private_ident!(specifier.clone()));
        self.add_import(local.clone(), Import { ... synthetic: true ... });
        local
    }
}
```

### Pattern 2: Synthesized Import Emission

**What:** Imports created during transformation (not in original source) must be emitted

**When to use:** When `qrl`, `useLexicalScope`, `_jsxSorted`, `_restProps`, etc. are used but not imported

**SWC Pattern (transform.rs:2600-2608):**
```rust
fn fold_module(node: ast::Module) -> ast::Module {
    // ... main body transformation ...

    // Add synthesized imports at top
    body.extend(
        self.options.global_collect.synthetic.iter()
            .map(|(new_local, import)| {
                create_synthetic_named_import(new_local, &import.source)
            }),
    );
    // Add extra_top_items (hoisted declarations)
    body.extend(self.extra_top_items.values().cloned());
    // Then original transformed body
    body.append(&mut module_body);
    // Add extra_bottom_items (if any)
    body.extend(self.extra_bottom_items.values().cloned());

    ast::Module { body, ..node }
}
```

**OXC Implementation Strategy:**
```rust
// In exit_program or similar:
fn finalize_module(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a, ()>) {
    // 1. Collect synthesized imports from import_by_symbol where symbol was created during transform
    let synthesized: Vec<_> = self.import_by_symbol.iter()
        .filter(|(sym, _)| self.synthesized_symbols.contains(sym))
        .collect();

    // 2. Create import statements
    for (_, import) in synthesized {
        let stmt = import.into_statement(ctx.ast.allocator);
        // Insert at beginning of body
    }

    // 3. Run ImportCleanUp to remove unused imports
    ImportCleanUp::clean_up(program, ctx.ast.allocator);
}
```

### Pattern 3: Segment File Import Generation

**What:** When extracting QRL to segment file, determine required imports

**SWC Pattern (code_move.rs:58-135):**
```rust
fn new_module(ctx: NewModuleCtx) -> Result<(ast::Module, SingleThreadedComments), Error> {
    // For each identifier used in the segment:
    for id in ctx.local_idents {
        if let Some(import) = ctx.global.imports.get(id) {
            // Import from original source
            module.body.push(create_import(import));
        } else if let Some(export) = ctx.global.exports.get(id) {
            // Import from the original file (re-import the export)
            module.body.push(create_import_from_self(export, ctx.path));
        }
    }

    // If has captured variables, import useLexicalScope
    if !ctx.scoped_idents.is_empty() {
        module.body.push(create_synthetic_named_import(&use_lexical_scope, ctx.core_module));
    }
}
```

**Key insight:** Segment files import from:
1. Original import sources (third-party libraries)
2. The original file itself (for exports the segment needs)
3. `@qwik.dev/core` for runtime functions (useLexicalScope)

### Pattern 4: Side-Effect Import Preservation

**What:** Imports without specifiers (`import './styles.css'`) must be preserved

**SWC Pattern (add_side_effect.rs:35-65):**
```rust
fn visit_mut_import_decl(&mut self, node: &mut ast::ImportDecl) {
    // Track existing side-effect imports
    if node.src.value.starts_with('.') {
        self.imports.insert(node.src.value.clone());
    }
}

fn visit_mut_module(&mut self, node: &mut ast::Module) {
    // After transformation, add any missing side-effect imports
    for import in imports_that_should_exist {
        if !self.imports.contains(&import.source) {
            // Insert side-effect import (no specifiers)
            node.body.insert(0, create_side_effect_import(import.source));
        }
    }
}
```

**OXC Handling:**
The current `ImportCleanUp` preserves side-effect imports by checking `if let Some(specifiers) = &import.specifiers { ... } else { true }` - imports without specifiers are kept.

### Pattern 5: Export Handling

**What:** Different export types require different handling during transformation

**Export Types:**
1. **Named export with declaration:** `export const Foo = component$(...)`
2. **Named export list:** `export { foo, bar as baz }`
3. **Default export:** `export default component$(...)`
4. **Re-export:** `export { foo } from './other'`
5. **Re-export all:** `export * from './other'`

**SWC Pattern (collector.rs:203-292 for export collection):**
```rust
fn visit_export_decl(&mut self, node: &ast::ExportDecl) {
    match &node.decl {
        ast::Decl::Var(var) => {
            for decl in &var.decls {
                self.add_export(id_from_pat(&decl.name), None);
            }
        }
        ast::Decl::Fn(func) => self.add_export(id!(func.ident), None),
        ast::Decl::Class(class) => self.add_export(id!(class.ident), None),
        _ => {}
    }
}

fn visit_export_default_decl(&mut self, node: &ast::ExportDefaultDecl) {
    // Track that there's a default export
    match &node.decl {
        ast::DefaultDecl::Fn(func) => {
            if let Some(ident) = &func.ident {
                self.add_export(id!(ident), Some(atom!("default")));
            }
        }
        // ...
    }
}
```

**Current OXC Status:**
- Named exports with declarations are transformed in `exit_expression`
- The transform handles `export const X = component$(...)` correctly
- Export tracking for segment scope resolution needs completion

### Anti-Patterns to Avoid

- **Removing side-effect imports:** These have no specifiers but are intentional
- **Duplicating imports:** Check before adding synthesized imports
- **Wrong import source for re-exports:** Re-exports keep original source, not local file
- **Forgetting type-only imports:** `import type { T }` should be preserved/cleaned differently
- **Import order changes:** Order affects side effects, maintain original order where possible

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Import statement AST | Manual construction | `Import::into_statement` | Handles all specifier variants |
| Unused import removal | Custom visitor | `ImportCleanUp::clean_up` | Already implemented and tested |
| Qwik package renaming | String replace | `ImportCleanUp::rename_qwik_imports` | Handles all @builder.io -> @qwik.dev |
| Import deduplication | HashSet by string | `BTreeSet<Import>` with proper Ord | Import includes specifiers |
| Synthesized import creation | Manual AST | `Import::new` + `into_statement` | Type-safe, handles aliases |

**Key insight:** The `shared.rs` Import/ImportId types and `ImportCleanUp` cover most needs. The gap is orchestration, not AST construction.

## Common Pitfalls

### Pitfall 1: Synthesized Import Duplication
**What goes wrong:** `import { qrl } from "@qwik.dev/core"` appears multiple times
**Why it happens:** Each QRL creation tries to synthesize the import
**How to avoid:** Check `symbols_by_name.contains_key("qrl")` before creating (OXC qrl.rs:75 does this)
**Warning signs:** Duplicate import declarations in output

### Pitfall 2: Lost Import Aliases
**What goes wrong:** `import { $ as myDollar } from "@qwik.dev/core"` loses the alias
**Why it happens:** Tracking by symbol name only, not including alias
**How to avoid:** ImportId::NamedWithAlias preserves both names
**Warning signs:** Transformed code uses wrong identifier name

### Pitfall 3: Side-Effect Import Removal
**What goes wrong:** `import './styles.css'` disappears after transformation
**Why it happens:** ImportCleanUp removes "unused" imports
**How to avoid:** ImportCleanUp already handles this - don't modify its behavior
**Warning signs:** CSS/side-effect modules not loaded at runtime

### Pitfall 4: Re-export Source Confusion
**What goes wrong:** `export { foo } from './a'` becomes `import { foo } from './original-file'`
**Why it happens:** Confusing re-export source with local import source
**How to avoid:** Re-exports maintain their original source in GlobalCollect
**Warning signs:** Circular imports or wrong module resolution

### Pitfall 5: Type-Only Import Handling
**What goes wrong:** Type imports removed but their values still referenced
**Why it happens:** TypeScript type imports should be removed, but some are values too
**How to avoid:** OXC's ImportOrExportKind::Type flag distinguishes them
**Warning signs:** Runtime errors about undefined types that should be values

### Pitfall 6: Import Order Matters for Side Effects
**What goes wrong:** Module initialization order changes
**Why it happens:** Synthesized imports inserted at wrong position
**How to avoid:** Insert synthesized imports BEFORE original imports that depend on them
**Warning signs:** Initialization order bugs, undefined at import time

## Code Examples

### Import Tracking (Current OXC)
```rust
// Source: optimizer/src/transform.rs:2264-2291
fn enter_import_declaration(&mut self, node: &mut ImportDeclaration<'a>, ctx: &mut TraverseCtx<'a, ()>) {
    if let Some(specifiers) = &mut node.specifiers {
        for specifier in specifiers.iter_mut() {
            let symbol_id = /* get symbol from specifier */;
            let source = node.source.value.to_string();

            // Track import for later reference
            self.import_by_symbol.insert(
                symbol_id,
                Import::new(vec![specifier.into()], source)
            );

            // Also track by name for synthesis checks
            self.symbols_by_name.insert(local_name, symbol_id);
        }
    }

    // Rename @builder.io/qwik -> @qwik.dev/core
    let source = ImportCleanUp::rename_qwik_imports(node.source.value);
    node.source.value = source.into_in(ctx.ast.allocator);
}
```

### Synthesized Import Creation (SWC Reference)
```rust
// Source: qwik-core/.../transform.rs:3300-3318
pub fn create_synthetic_named_import(local: &Id, src: &Atom) -> ast::ModuleItem {
    ast::ModuleItem::ModuleDecl(ast::ModuleDecl::Import(ast::ImportDecl {
        span: DUMMY_SP,
        src: Box::new(ast::Str { value: src.clone(), .. }),
        specifiers: vec![ast::ImportSpecifier::Named(ast::ImportNamedSpecifier {
            local: new_ident_from_id(local),
            imported: None,  // Same name
            ..
        })],
        type_only: false,
        ..
    }))
}
```

### Import for Segment File (SWC Reference)
```rust
// Source: qwik-core/.../code_move.rs:58-99
for id in ctx.local_idents {
    if let Some(import) = ctx.global.imports.get(id) {
        let specifier = match import.kind {
            ImportKind::Named => ast::ImportSpecifier::Named(...),
            ImportKind::Default => ast::ImportSpecifier::Default(...),
            ImportKind::All => ast::ImportSpecifier::Namespace(...),
        };
        module.body.push(ast::ModuleItem::ModuleDecl(ast::ModuleDecl::Import(
            ast::ImportDecl {
                src: Box::new(ast::Str { value: import.source.clone(), .. }),
                specifiers: vec![specifier],
                ..
            }
        )));
    }
}
```

### Dynamic Import Generation (OXC Existing)
```rust
// Source: optimizer/src/ext/ast_builder_ext.rs
// The dynamic import `() => import("./path.js")` is created in qrl.rs:152-179
fn into_arrow_function<'a>(&self, ast_builder: &AstBuilder<'a>) -> ArrowFunctionExpression<'a> {
    let filename = format!("./{}.js", self.rel_path.file_name()...);

    // Create: import("./filename.js")
    let mut statements = ast_builder.vec_with_capacity(1);
    statements.push(ast_builder.create_simple_import(filename.as_ref()));

    // Wrap in arrow: () => import(...)
    ast_builder.arrow_function_expression(SPAN, true, false, None, params, None, body)
}
```

## State of the Art

| Requirement | SWC Location | OXC Status | What's Needed |
|-------------|--------------|------------|---------------|
| IMP-01: Import capture | collector.rs:156-200 | Partial (import_by_symbol) | Complete identifier reference tracking |
| IMP-02: Unused import cleanup | N/A (post-transform) | Complete (ImportCleanUp) | Integration in module finalization |
| IMP-03: Synthesized imports | transform.rs:2600-2608 | Partial (qrl synthesis works) | Module finalization to emit all |
| IMP-04: Named exports | collector.rs:239-261 | Works via exit_expression | Verify edge cases |
| IMP-05: Default exports | collector.rs:263-279 | Works via exit_expression | Verify edge cases |
| IMP-06: Re-exports | collector.rs:203-237 | Not fully tested | Test and verify |
| IMP-07: Side-effect preservation | add_side_effect.rs | Automatic via ImportCleanUp | Verify behavior |
| IMP-08: Dynamic import generation | qrl.rs:152-179 | Complete | Already working |

## Open Questions

1. **Module finalization hook**
   - What we know: OXC traverse has `exit_program` for module-level finalization
   - What's unclear: Exact AST mutation API for inserting statements at program level
   - Recommendation: Use `exit_program` with `program.body.insert(0, ...)` for synthesized imports

2. **Import cleanup timing**
   - What we know: ImportCleanUp requires semantic analysis (symbol usage)
   - What's unclear: Whether to run after traverse or as separate pass
   - Recommendation: Run as separate pass after traverse completes, matching SWC pattern

3. **Re-export handling scope**
   - What we know: Re-exports like `export { foo } from './other'` exist
   - What's unclear: Whether QRL transformation affects re-exports
   - Recommendation: Re-exports without local usage should pass through unchanged

## Sources

### Primary (HIGH confidence)
- `/optimizer/src/import_clean_up.rs` - OXC import cleanup implementation (verified)
- `/optimizer/src/component/shared.rs` - OXC Import/ImportId types (verified)
- `/optimizer/src/transform.rs:2264-2323` - OXC import tracking (verified)
- `/optimizer/src/component/qrl.rs:68-101` - OXC synthesized import for qrl (verified)
- `/qwik-core/.../collector.rs:37-292` - SWC GlobalCollect (verified)
- `/qwik-core/.../transform.rs:2600-2614` - SWC module finalization (verified)
- `/qwik-core/.../code_move.rs:35-151` - SWC segment file imports (verified)
- `/qwik-core/.../add_side_effect.rs` - SWC side-effect preservation (verified)

### Secondary (MEDIUM confidence)
- `/qwik-core/.../test.rs:1074-1127` - Export test cases (verified)
- `/qwik-core/.../test.rs:1767-1819` - strip_exports tests (verified)
- `/qwik-core/.../test.rs:821-862` - Side effect tests (verified)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Codebase fully examined, existing infrastructure documented
- Architecture patterns: HIGH - SWC patterns clear, OXC partial implementations exist
- Pitfalls: HIGH - Derived from SWC code analysis and import_clean_up tests
- Code examples: HIGH - Direct excerpts from working code

**Research date:** 2026-01-29
**Valid until:** 2026-03-01 (stable infrastructure, incremental completion)

## Requirements Mapping

| Requirement | SWC Implementation | OXC Status | Priority |
|-------------|-------------------|------------|----------|
| IMP-01: Import capture for QRL scope | collector.rs GlobalCollect | Partial - import_by_symbol exists | HIGH |
| IMP-02: Unused import cleanup | transform.rs fold_module | Complete - ImportCleanUp | MEDIUM |
| IMP-03: Synthesized imports | transform.rs:2600-2608 | Partial - qrl works, others not | HIGH |
| IMP-04: Named exports | collector.rs:239-261 | Works | LOW |
| IMP-05: Default exports | collector.rs:263-279 | Works | LOW |
| IMP-06: Re-exports | collector.rs:203-237 | Needs testing | MEDIUM |
| IMP-07: Side-effect preservation | add_side_effect.rs | Automatic | LOW |
| IMP-08: Dynamic import generation | code_move.rs + qrl.rs | Complete | DONE |

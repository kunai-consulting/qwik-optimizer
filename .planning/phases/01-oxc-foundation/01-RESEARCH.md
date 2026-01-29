# Phase 1: OXC Foundation - Research

**Researched:** 2026-01-29
**Domain:** OXC Rust crate upgrade (0.94.0 -> 0.111.0)
**Confidence:** HIGH

## Summary

This phase involves upgrading all OXC crates from version 0.94.0 to 0.111.0 in the Qwik optimizer codebase. The current codebase uses 12 OXC crates and has 31 passing tests (19 snapshot tests). Research reveals that the codebase is already well-positioned for the upgrade - key APIs like `TraverseCtx<'a, ()>` with State parameter, `semantic.into_scoping()`, and `Option<Atom>` for StringLiteral raw fields are already in use.

The breaking changes between 0.94.0 and 0.111.0 are well-documented in OXC changelogs. Critical changes include: removal of `Scoping::scope_build_child_ids` (not used in codebase), `CommentKind::Block` renamed to `CommentKind::SinglelineBlock` (imported but not used in codebase), and addition of new `Ident` AST type.

**Primary recommendation:** Bump all OXC crate versions to 0.111.0 in Cargo.toml, compile, fix any errors (expected to be minimal), and verify all 31 tests pass.

## Standard Stack

The established libraries/tools for this domain:

### Core OXC Crates
| Crate | Current | Target | Purpose | Impact Level |
|-------|---------|--------|---------|--------------|
| oxc_parser | 0.94.0 | 0.111.0 | JavaScript/TypeScript parsing | LOW |
| oxc_ast | 0.94.0 | 0.111.0 | AST node definitions | MEDIUM |
| oxc_semantic | 0.94.0 | 0.111.0 | Scope/symbol analysis | MEDIUM |
| oxc_traverse | 0.94.0 | 0.111.0 | AST traversal with mutation | LOW |
| oxc_transformer | 0.94.0 | 0.111.0 | Code transformation (TS strip) | LOW |
| oxc_codegen | 0.94.0 | 0.111.0 | Code generation from AST | LOW |
| oxc_allocator | 0.94.0 | 0.111.0 | Arena allocator | LOW |
| oxc_span | 0.94.0 | 0.111.0 | Source spans and atoms | LOW |
| oxc_syntax | 0.94.0 | 0.111.0 | Syntax constants/enums | LOW |
| oxc_minifier | 0.94.0 | 0.111.0 | Code minification | LOW |
| oxc_ast_visit | 0.94.0 | 0.111.0 | Visitor pattern traits | LOW |

### Supporting Crates
| Crate | Current | Target | Purpose | Notes |
|-------|---------|--------|---------|-------|
| oxc_index | 4.1.0 | 4.1.0 | Index types | No change needed |
| oxc-browserslist | 2.1.2 | 2.1.x | Browser targets | Check for compatible version |

**Cargo.toml update:**
```toml
[dependencies]
oxc_index = "4.1.0"  # Keep same
oxc_parser = "0.111.0"
oxc_ast = "0.111.0"
oxc_codegen = "0.111.0"
oxc_allocator = "0.111.0"
oxc_semantic = "0.111.0"
oxc_span = "0.111.0"
oxc_traverse = "0.111.0"
oxc_minifier = "0.111.0"
oxc_ast_visit = "0.111.0"
oxc_syntax = "0.111.0"
oxc_transformer = "0.111.0"
oxc-browserslist = "2.1.2"  # Verify compatibility
```

## Architecture Patterns

### Current Codebase Patterns (Already Compatible)

The codebase already uses the patterns expected by OXC 0.111.0:

**Pattern 1: Traverse with State Type Parameter**
```rust
// Source: /optimizer/src/transform.rs:265
impl<'a> Traverse<'a, ()> for TransformGenerator<'a> {
    fn enter_program(&mut self, node: &mut Program<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        // ...
    }
}
```

**Pattern 2: Scoping API via into_scoping()**
```rust
// Source: /optimizer/src/transform.rs:1156, 1180
let scoping = semantic.into_scoping();
traverse_mut(&mut transform, &allocator, &mut program, scoping, ());
```

**Pattern 3: Scoping Access in TraverseCtx**
```rust
// Source: /optimizer/src/transform.rs:1019, 1090
ctx.scoping_mut().rename_symbol(symbol_id, scope_id, new_name.into());
ctx.scoping().get_reference(ref_id).symbol_id()
```

**Pattern 4: StringLiteral with Option<Atom> raw**
```rust
// Source: /optimizer/src/ext/ast_builder_ext.rs:29-30
self.builder.expression_string_literal(span, value, Some(raw))
```

### traverse_mut Signature (0.111.0)
```rust
// Source: docs.rs/oxc_traverse/0.111.0
pub fn traverse_mut<'a, State, Tr: Traverse<'a, State>>(
    traverser: &mut Tr,
    allocator: &'a Allocator,
    program: &mut Program<'a>,
    scoping: Scoping,
    state: State,
) -> Scoping
```

### Anti-Patterns to Avoid

- **Do not use `into_symbol_table_and_scope_tree()`**: Deprecated, use `into_scoping()` instead
- **Do not use `scope_build_child_ids()`**: Removed in 0.111.0, use ancestor traversal instead
- **Do not use `CommentKind::Block`**: Renamed to `CommentKind::SinglelineBlock`
- **Do not construct AST nodes directly**: Use AstBuilder methods

## Don't Hand-Roll

Problems with existing OXC solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| AST node construction | Manual struct creation | `AstBuilder` methods | Handles allocator, proper initialization |
| Symbol lookup | Custom hash maps | `Scoping::get_resolved_references()` | Integrated with semantic analysis |
| Reference tracking | Manual reference counting | `Scoping::symbol_is_unused()` | Accurate dead code detection |
| Scope traversal | Child ID iteration | `scope_ancestors()` or parent references | `scope_build_child_ids` removed |

**Key insight:** OXC provides complete semantic analysis infrastructure. The codebase already uses these properly.

## Common Pitfalls

### Pitfall 1: CommentKind Enum Change
**What goes wrong:** Compilation error on `CommentKind::Block`
**Why it happens:** Renamed to `CommentKind::SinglelineBlock` in v0.103.0, new `MultilineBlock` variant added
**How to avoid:** Search for `CommentKind::Block` usage and update. Note: Current codebase imports `CommentKind` but doesn't use the `Block` variant
**Warning signs:** Import exists at transform.rs:15 but grep shows no `CommentKind::` usage - likely unused import

### Pitfall 2: Scoping Child ID APIs Removed
**What goes wrong:** `scope_build_child_ids` method not found
**Why it happens:** Removed in v0.111.0 as part of optimization
**How to avoid:** Use `scope_ancestors()` for traversal patterns. Verify codebase doesn't call removed methods
**Warning signs:** Any direct iteration over child scope IDs
**Codebase status:** Not used - codebase uses `ancestor_scopes()` at transform.rs:367

### Pitfall 3: oxc-browserslist Version Mismatch
**What goes wrong:** Cargo dependency resolution conflicts
**Why it happens:** oxc_transformer may expect specific oxc-browserslist version
**How to avoid:** Check if explicit oxc-browserslist dependency is still needed after upgrade
**Warning signs:** Cargo.lock conflicts, duplicate dependency versions

### Pitfall 4: Ident Type Addition
**What goes wrong:** Pattern matching on identifier AST nodes may break
**Why it happens:** New `Ident` type added in v0.111.0
**How to avoid:** Review any exhaustive pattern matches on identifier-related enums
**Warning signs:** Non-exhaustive match warnings after upgrade

### Pitfall 5: TSEnumDeclaration Scope Moved
**What goes wrong:** TypeScript enum scope resolution differs
**Why it happens:** Scope moved from `TSEnumDeclaration` to `TSEnumBody` in v0.109.0
**How to avoid:** Low impact for this codebase - primarily JSX/JS transformation
**Warning signs:** TS enum handling tests failing

## Code Examples

Verified patterns from current codebase (already 0.111.0-compatible):

### Semantic Analysis Setup
```rust
// Source: /optimizer/src/transform.rs:1169-1176
let SemanticBuilderReturn {
    semantic,
    errors: semantic_errors,
} = SemanticBuilder::new()
    .with_check_syntax_error(true)
    .with_cfg(true)
    .build(&program);
```

### Transformer with Scoping
```rust
// Source: /optimizer/src/transform.rs:1157-1167
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
```

### Symbol Renaming
```rust
// Source: /optimizer/src/transform.rs:1018-1023
let scope_id = ctx.current_scope_id();
ctx.scoping_mut().rename_symbol(
    symbol_id,
    scope_id,
    local_name.as_str().into(),
);
```

### Reference Resolution
```rust
// Source: /optimizer/src/transform.rs:1090
if let Some(symbol_id) = ctx.scoping.scoping().get_reference(ref_id).symbol_id() {
    // ...
}
```

### Import Cleanup with Scoping
```rust
// Source: /optimizer/src/import_clean_up.rs:113-115
ctx.scoping()
    .symbol_is_unused(specifier.local().symbol_id())
```

## State of the Art

| Old Approach (pre-0.111.0) | Current Approach (0.111.0) | When Changed | Impact |
|---------------------------|---------------------------|--------------|--------|
| `into_symbol_table_and_scope_tree()` | `into_scoping()` | v0.57.0 | Codebase OK |
| `CommentKind::Block` | `CommentKind::SinglelineBlock` | v0.103.0 | Unused import |
| `scope_build_child_ids()` | `scope_ancestors()` | v0.111.0 | Codebase OK |
| `TraverseCtx<'a>` | `TraverseCtx<'a, State>` | v0.74.0 | Codebase OK |

**Already deprecated/updated in codebase:**
- Scoping API: Already using `into_scoping()`
- Traverse State: Already using `TraverseCtx<'a, ()>`
- StringLiteral raw: Already using `Option<Atom>`

## Open Questions

Things that require verification during implementation:

1. **oxc-browserslist compatibility**
   - What we know: Explicit 2.1.2 dependency exists "to avoid version mismatch in build"
   - What's unclear: Whether this is still needed with 0.111.0
   - Recommendation: Try removing explicit dependency first, add back if conflicts arise

2. **New Ident AST type impact**
   - What we know: New type added in v0.111.0
   - What's unclear: Whether any pattern matches in codebase are affected
   - Recommendation: Compile and check for exhaustive match warnings

3. **Double scoping() call pattern**
   - What we know: Line 1090 has `ctx.scoping.scoping().get_reference()`
   - What's unclear: If this is intentional or a bug
   - Recommendation: Verify this compiles and works correctly after upgrade

## Sources

### Primary (HIGH confidence)
- [docs.rs/oxc_semantic Scoping API](https://docs.rs/oxc_semantic/latest/oxc_semantic/struct.Scoping.html) - Method signatures verified
- [docs.rs/oxc_traverse](https://docs.rs/oxc_traverse/latest/oxc_traverse/trait.Traverse.html) - Traverse trait with State parameter verified
- [OXC GitHub Releases](https://github.com/oxc-project/oxc/releases) - Breaking changes v0.94.0-v0.111.0
- [OXC Codegen CHANGELOG](https://raw.githubusercontent.com/oxc-project/oxc/main/crates/oxc_codegen/CHANGELOG.md) - CommentKind rename confirmed

### Secondary (MEDIUM confidence)
- Codebase analysis at `/optimizer/src/` - Current API usage patterns
- [OXC Transformer docs.rs](https://docs.rs/oxc_transformer/0.111.0/oxc_transformer/) - build_with_scoping signature

### Tertiary (LOW confidence)
- WebSearch for ecosystem patterns - General migration guidance

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Versions documented in changelogs
- Architecture: HIGH - Codebase already uses target patterns
- Pitfalls: HIGH - Verified against official changelogs
- Open questions: MEDIUM - Require compilation to verify

**Research date:** 2026-01-29
**Valid until:** 2026-02-28 (OXC releases frequently, verify if delayed)

## Migration Checklist

Pre-verified items (codebase already compatible):
- [x] `Traverse<'a, ()>` with State parameter
- [x] `TraverseCtx<'a, ()>` in method signatures
- [x] `semantic.into_scoping()` usage
- [x] `ctx.scoping()` / `ctx.scoping_mut()` access
- [x] `StringLiteral` with `Option<Atom>` raw field
- [x] `Transformer::new()` with `&TransformOptions`
- [x] `traverse_mut()` with Scoping parameter

Items to verify during upgrade:
- [ ] CommentKind import - may be unused, can remove if causes issues
- [ ] oxc-browserslist explicit dependency - may be removable
- [ ] Any exhaustive pattern matches on identifier types
- [ ] Double `scoping()` call at line 1090

Test verification:
- [ ] All 31 unit tests pass
- [ ] 19 snapshot tests unchanged
- [ ] No new warnings beyond current 14

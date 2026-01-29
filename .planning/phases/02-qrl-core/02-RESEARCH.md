# Phase 2: QRL Core - Research

**Researched:** 2026-01-29
**Domain:** QRL extraction and transformation (SWC-to-OXC port)
**Confidence:** HIGH

## Summary

This phase ports QRL (Qwik Resource Locator) extraction and transformation from the existing SWC-based optimizer to the new OXC-based implementation. The SWC reference implementation at `qwik-core/src/optimizer/core/src/transform.rs` provides the complete specification - no design decisions needed, only faithful replication.

QRL transformation involves: (1) identifying `$`-suffixed function calls (markers like `component$`, `onClick$`), (2) extracting the function argument into a separate lazy-loadable segment, (3) generating a unique hash/display name, (4) tracking captured lexical scope variables, and (5) replacing the original call with a `qrl()` call that lazy-imports the extracted segment.

The OXC port already has partial infrastructure in place (`/optimizer/src/component/qrl.rs`, `/optimizer/src/segment.rs`, `/optimizer/src/component/id.rs`) but needs completion of the core transformation logic to match SWC output exactly. The existing 90+ snapshot tests in `qwik-core/src/optimizer/core/src/snapshots/` serve as the ground truth for parity verification.

**Primary recommendation:** Port the SWC transformation logic section-by-section, using the existing OXC infrastructure and snapshot tests to verify parity at each step.

## Standard Stack

The established components for this domain (all local, no external dependencies):

### Core Components (OXC Port)
| Component | File | Purpose | Status |
|-----------|------|---------|--------|
| Qrl | `/optimizer/src/component/qrl.rs` | QRL expression generation | Implemented |
| QrlType | `/optimizer/src/component/qrl.rs:14-18` | QRL variant types (Qrl, PrefixedQrl, IndexedQrl) | Implemented |
| Id | `/optimizer/src/component/id.rs` | Hash/display name generation | Implemented |
| Segment | `/optimizer/src/segment.rs` | Segment name uniqueness | Implemented |
| SegmentBuilder | `/optimizer/src/segment.rs:26-159` | Unique segment naming | Implemented |
| TransformGenerator | `/optimizer/src/transform.rs` | Main traverse implementation | Partial |

### SWC Reference (Specification)
| Component | File | Purpose | Port Priority |
|-----------|------|---------|---------------|
| QwikTransform | `qwik-core/.../transform.rs:98-132` | Main transformation state | HIGH |
| create_synthetic_qsegment | `qwik-core/.../transform.rs:650-786` | QRL creation from expressions | HIGH |
| register_context_name | `qwik-core/.../transform.rs:323-375` | Hash/display name generation | HIGH |
| compute_scoped_idents | `qwik-core/.../transform.rs:3582-3596` | Lexical scope capture | HIGH |
| IdentCollector | `qwik-core/.../collector.rs:300-394` | Variable usage collection | MEDIUM |
| transform_function_expr | `qwik-core/.../code_move.rs:175-251` | useLexicalScope injection | HIGH |

### Constants (Already Ported)
| Constant | OXC Location | SWC Location | Value |
|----------|--------------|--------------|-------|
| QWIK_CORE_SOURCE | `/optimizer/src/component/shared.rs:11` | `words.rs:21` | `"@qwik.dev/core"` |
| MARKER_SUFFIX | `/optimizer/src/component/shared.rs:14` | `words.rs:4` | `"$"` |
| QRL | `/optimizer/src/component/shared.rs:15` | `words.rs:11` | `"qrl"` |
| QRL_SUFFIX | `/optimizer/src/component/shared.rs:16` | `words.rs:5` | `"Qrl"` |

## Architecture Patterns

### Pattern 1: QRL Transformation Flow (SWC Reference)

**What:** Convert `marker$(() => ...)` to `markerQrl(qrl(() => import(...), "symbol_name", [captures]))`

**When to use:** Every time a `$`-suffixed call expression is encountered

**Example (from SWC transform.rs:650-786):**
```
INPUT:  component$(() => { return <div onClick$={() => console.log("hi")} /> })
OUTPUT: componentQrl(qrl(() => import("./file_Component_hash"), "Component_hash"))
        // Plus separate segment file with the extracted function
```

**Steps:**
1. Detect marker function (ends with `$`)
2. Collect descendent identifiers using IdentCollector
3. Partition into (scoped_idents, invalid_decl)
4. Generate symbol_name, display_name, hash via register_context_name
5. Fold/traverse the first argument recursively (handles nested QRLs)
6. Create segment with extracted expression
7. Generate QRL call expression with lazy import

### Pattern 2: Hash Generation (SWC Reference)

**What:** Generate stable, unique hashes for QRL identification

**When to use:** For every extracted segment

**Algorithm (from SWC transform.rs:353-374, OXC id.rs:39-57):**
```rust
// Source: SWC transform.rs and OXC id.rs
fn calculate_hash(local_file_name: &str, display_name: &str, scope: &Option<String>) -> (u64, String) {
    let mut hasher = DefaultHasher::new();
    if let Some(scope) = scope {
        hasher.write(scope.as_bytes());
    }
    hasher.write(local_file_name.as_bytes());
    hasher.write(display_name.as_bytes());
    let hash = hasher.finish();
    // Base64 encode and sanitize
    (hash, base64_encode(hash).replace(['-', '_'], "0"))
}
```

**Display name format:** `{stack_context}_joined_by_underscores`
**Symbol name format (dev):** `{display_name}_{hash}`
**Symbol name format (prod):** `s_{hash}`

### Pattern 3: Lexical Scope Capture (SWC Reference)

**What:** Track which variables from enclosing scope are used inside a QRL

**When to use:** When a QRL function references variables from parent scope

**Example (from SWC code_move.rs:260-290):**
```javascript
// INPUT:
const count = useSignal(0);
return <button onClick$={() => count.value++}>

// OUTPUT (segment file):
import { useLexicalScope } from "@qwik.dev/core";
export const onClick_hash = () => {
    const [count] = useLexicalScope();  // Injected
    count.value++;
};

// OUTPUT (main file):
qrl(() => import("./segment"), "onClick_hash", [count])  // 3rd arg = captures
```

**Algorithm (from SWC transform.rs:3582-3596):**
```rust
fn compute_scoped_idents(all_idents: &[Id], all_decl: &[IdPlusType]) -> (Vec<Id>, bool) {
    let mut set: HashSet<Id> = HashSet::new();
    let mut is_const = true;
    for ident in all_idents {
        if let Some(item) = all_decl.iter().find(|item| item.0 == *ident) {
            set.insert(ident.clone());
            if !matches!(item.1, IdentType::Var(true)) {
                is_const = false;
            }
        }
    }
    let mut output: Vec<Id> = set.into_iter().collect();
    output.sort();  // Deterministic ordering for stable output
    (output, is_const)
}
```

### Pattern 4: Nested QRL Handling

**What:** QRLs can contain other QRLs, requiring recursive transformation

**When to use:** When a component$ contains onClick$ handlers, etc.

**Example (from snapshot example_2):**
```javascript
// INPUT:
export const Header = component$(() => {
    return <div onClick={$((ctx) => console.log(ctx))}/>
});

// OUTPUT: 3 segments generated
// 1. Header_component_hash - the component function
// 2. Header_component_div_onClick_hash - the click handler
// 3. Main file with componentQrl() and nested qrl()
```

**Key insight:** The `segment_stack` tracks nesting depth; parent_segment field links nested QRLs.

### Pattern 5: Ternary QRL Handling

**What:** QRLs inside ternary expressions must both be transformed

**When to use:** `condition ? $(() => a) : $(() => b)` or `condition ? $(...) : undefined`

**Example (from snapshot should_transform_qrls_in_ternary_expression):**
```javascript
// INPUT:
onInput$={enabled.value ? $((ev, el) => { input.value = el.value; }) : undefined}

// OUTPUT:
"on:input": enabled.value
    ? qrl(i_hash1, "name_hash1", [input])
    : undefined
```

### Anti-Patterns to Avoid

- **Modifying hash algorithm:** Hash must match SWC exactly for snapshot parity
- **Changing display name format:** Format is `{filename}_{context_path}` exactly
- **Reordering scoped_idents:** Must be sorted for deterministic output
- **Skipping parameter name extraction:** param_names field required in segment metadata
- **Forgetting useLexicalScope injection:** Required when scoped_idents is non-empty

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Hash generation | Custom hashing | `Id::calculate_hash` in id.rs | Must match SWC output exactly |
| Display name construction | String concatenation | `Id::new` in id.rs | Handles edge cases (leading digits, sanitization) |
| Segment uniqueness | Counter per name | `SegmentBuilder` in segment.rs | Already handles IndexQrl vs NamedQrl |
| QRL call generation | Manual AST construction | `Qrl::into_call_expression` in qrl.rs | Already handles PrefixedQrl wrapping |
| Import statement creation | Manual AST | `Import::into_statement` in shared.rs | Handles specifier types correctly |

**Key insight:** Much of the infrastructure already exists in `/optimizer/src/component/`. The gap is the transformation traverse logic that connects these pieces.

## Common Pitfalls

### Pitfall 1: Hash Mismatch with SWC
**What goes wrong:** Generated hashes don't match snapshot expectations
**Why it happens:** Different hash input ordering, missing scope prefix, wrong base64 encoding
**How to avoid:** Use `Id::calculate_hash` which matches SWC's `register_context_name`
**Warning signs:** Snapshot test failures with different hash suffixes

### Pitfall 2: Missing Parent Segment Link
**What goes wrong:** Nested QRLs don't have correct parent reference
**Why it happens:** `segment_stack` not maintained during recursive transformation
**How to avoid:** Push to segment_stack before folding first argument, pop after
**Warning signs:** `parent` field null in segment metadata when it should reference outer QRL

### Pitfall 3: Scoped Idents Not Sorted
**What goes wrong:** `useLexicalScope()` destructuring order differs from SWC
**Why it happens:** HashSet iteration order is non-deterministic
**How to avoid:** Sort scoped_idents after collection (SWC does `output.sort()`)
**Warning signs:** Capture array has variables in different order than SWC

### Pitfall 4: ctx_name vs ctx_kind Confusion
**What goes wrong:** Segment metadata has wrong context type
**Why it happens:** ctx_name is the marker function name (`onClick$`), ctx_kind is the category
**How to avoid:** ctx_kind is `EventHandler` for `on*$`, `JSXProp` for other JSX attributes, `Function` otherwise
**Warning signs:** ctxKind field shows "function" when should show "eventHandler"

### Pitfall 5: PrefixedQrl Not Wrapping Correctly
**What goes wrong:** `component$` output is `qrl(...)` instead of `componentQrl(qrl(...))`
**Why it happens:** QrlType::PrefixedQrl requires additional wrapping call
**How to avoid:** `Qrl::into_call_expression` already handles this - verify QrlType is correct
**Warning signs:** Missing outer `componentQrl()` wrapper in output

### Pitfall 6: Expression vs Concise Arrow Body
**What goes wrong:** `useLexicalScope` injection breaks when arrow has expression body
**Why it happens:** Expression body `() => expr` must become block `() => { const [...] = useLexicalScope(); return expr; }`
**How to avoid:** `transform_function_expr` in code_move.rs handles both cases
**Warning signs:** "Expected block statement" errors or missing return

## Code Examples

### QRL Detection (SWC Pattern to Port)

```rust
// Source: SWC transform.rs:163-177 - marker function detection
for (id, import) in options.global_collect.imports.iter() {
    if import.kind == ImportKind::Named && import.specifier.ends_with(QRL_SUFFIX) {
        marker_functions.insert(id.clone(), import.specifier.clone());
    }
}
// Also check exports for locally defined markers
for id in options.global_collect.exports.keys() {
    if id.0.ends_with(QRL_SUFFIX) {
        marker_functions.insert(id.clone(), id.0.clone());
    }
}
```

### OXC Segment Detection (Already Working)

```rust
// Source: OXC /optimizer/src/segment.rs:114-131
impl SegmentName {
    fn new(name0: String) -> Self {
        let name = name0.strip_suffix(MARKER_SUFFIX);  // Strips '$'
        match name {
            None => SegmentName::Name(name0),  // Not a QRL marker
            Some(name) if name.is_empty() => SegmentName::UnanchoredQrl,  // Just '$'
            Some(name) => SegmentName::AnchoredQrl(name.to_string()),  // 'component$' etc
        }
    }
}
```

### Identifier Collection (SWC Pattern to Port)

```rust
// Source: SWC collector.rs:327-380 - IdentCollector
impl Visit for IdentCollector {
    fn visit_expr(&mut self, node: &ast::Expr) {
        self.expr_ctxt.push(ExprOrSkip::Expr);
        node.visit_children_with(self);
        self.expr_ctxt.pop();
    }

    fn visit_ident(&mut self, node: &ast::Ident) {
        // Only collect identifiers in expression context
        if matches!(self.expr_ctxt.last(), Some(ExprOrSkip::Expr))
            && node.ctxt != SyntaxContext::empty()
            && !is_builtin(node.sym)  // undefined, NaN, Infinity, null
        {
            self.local_idents.insert(id!(node));
        }
    }

    // Skip property keys, member expression properties
    fn visit_key_value_prop(&mut self, node: &ast::KeyValueProp) {
        self.expr_ctxt.push(ExprOrSkip::Skip);
        node.visit_children_with(self);
        self.expr_ctxt.pop();
    }
}
```

### QRL Call Creation (OXC Already Working)

```rust
// Source: OXC /optimizer/src/component/qrl.rs:192-244
pub fn into_call_expression<'a>(&self, ctx: &mut TraverseCtx<'a, ()>, ...) -> CallExpression<'a> {
    let args = self.into_arguments(&ast_builder);  // [lazy_import_arrow, "symbol_name"]

    match qrl_type {
        QrlType::Qrl | QrlType::IndexedQrl(_) => qrl_call_expr,  // qrl(...)
        QrlType::PrefixedQrl(prefix) => {
            // Wrap: componentQrl(qrl(...))
            let ident = format!("{}Qrl", prefix);
            ast_builder.call_expression(
                SPAN,
                Expression::Identifier(ident),
                None,
                ast_builder.vec1(Argument::CallExpression(qrl_call_expr)),
                false,
            )
        }
    }
}
```

### useLexicalScope Injection (SWC Pattern to Port)

```rust
// Source: SWC code_move.rs:260-290
fn create_use_lexical_scope(use_lexical_scope: &Id, scoped_idents: &[Id]) -> ast::Stmt {
    // Creates: const [a, b, c] = useLexicalScope();
    ast::Stmt::Decl(ast::Decl::Var(Box::new(ast::VarDecl {
        kind: ast::VarDeclKind::Const,
        decls: vec![ast::VarDeclarator {
            init: Some(Box::new(ast::Expr::Call(ast::CallExpr {
                callee: useLexicalScope_ident,
                args: vec![],  // No arguments
                ..Default::default()
            }))),
            name: ast::Pat::Array(ast::ArrayPat {
                elems: scoped_idents.iter()
                    .map(|id| Some(ast::Pat::Ident(new_ident_from_id(id))))
                    .collect(),
                ..Default::default()
            }),
            ..Default::default()
        }],
        ..Default::default()
    })))
}
```

## State of the Art

| SWC Component | OXC Status | What's Needed |
|---------------|------------|---------------|
| marker_functions detection | Partial (segment.rs) | Integrate with import tracking |
| register_context_name | Implemented (id.rs) | Verify hash parity |
| compute_scoped_idents | Not ported | Port IdentCollector + partition logic |
| transform_function_expr | Not ported | Port useLexicalScope injection |
| create_segment | Partial (transform.rs) | Complete with segment metadata |
| handle_jsx_value | Not ported | Port event handler QRL creation |
| _create_synthetic_qsegment | Not ported | Core transformation logic |

## Open Questions

1. **Debug output in OXC transform.rs**
   - What we know: Current OXC transform.rs has `DEBUG: bool = true` and many `println!` statements
   - What's unclear: Whether to remove these or gate behind feature flag
   - Recommendation: Keep for development, remove or feature-gate before production

2. **JSX transformation coupling**
   - What we know: SWC handles QRL and JSX in same transform pass
   - What's unclear: Whether OXC should separate or combine these
   - Recommendation: Keep combined per SWC for output parity, can refactor later

3. **Error handling parity**
   - What we know: SWC uses HANDLER.with for error emission
   - What's unclear: Exact error messages and detection conditions
   - Recommendation: Match detection behavior, error message wording is Claude's discretion

## Sources

### Primary (HIGH confidence)
- `/optimizer/src/component/qrl.rs` - OXC Qrl implementation (verified)
- `/optimizer/src/component/id.rs` - OXC hash generation (verified)
- `/optimizer/src/segment.rs` - OXC segment builder (verified)
- `/qwik-core/src/optimizer/core/src/transform.rs` - SWC reference implementation (verified)
- `/qwik-core/src/optimizer/core/src/code_move.rs` - SWC useLexicalScope handling (verified)
- `/qwik-core/src/optimizer/core/src/collector.rs` - SWC identifier collection (verified)
- `/qwik-core/src/optimizer/core/src/snapshots/*.snap` - 90+ ground truth tests (verified)

### Secondary (MEDIUM confidence)
- `/qwik-core/src/optimizer/core/src/test.rs` - Test input patterns (verified)
- `/qwik-core/src/optimizer/core/src/words.rs` - Constant definitions (verified)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Codebase fully examined
- Architecture patterns: HIGH - SWC implementation is definitive
- Pitfalls: HIGH - Derived from SWC code analysis and snapshot examination
- Code examples: HIGH - Direct excerpts from codebase

**Research date:** 2026-01-29
**Valid until:** 2026-03-01 (SWC implementation is stable reference)

## QRL Requirements Mapping

| Requirement | SWC Implementation Location | OXC Status |
|-------------|----------------------------|------------|
| QRL-01: Arrow functions | transform.rs:667-668 `ast::Expr::Arrow` | Segment detection works |
| QRL-02: Function declarations | transform.rs:667-668 `ast::Expr::Fn` | Segment detection works |
| QRL-03: Component$ transformation | transform.rs:163-177 marker detection | Partial |
| QRL-04: Nested QRL handling | transform.rs:688-691 recursive fold | Not ported |
| QRL-05: Ternary expressions | handle_jsx_value passes through | Not ported |
| QRL-06: Multiple QRLs per file | segment_names HashMap for uniqueness | SegmentBuilder handles |
| QRL-07: Captured variables | transform.rs:3582-3596 compute_scoped_idents | Not ported |
| QRL-08: Display name generation | transform.rs:323-375 register_context_name | Id::new handles |
| QRL-09: Hash generation | transform.rs:353-360 hasher logic | Id::calculate_hash handles |
| QRL-10: Normal function transformation | code_move.rs:226-251 transform_fn | Not ported |

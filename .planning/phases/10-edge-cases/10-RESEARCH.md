# Phase 10: Edge Cases - Research

**Researched:** 2026-01-29
**Domain:** QRL edge case handling, loop semantics, error reporting
**Confidence:** HIGH

## Summary

Phase 10 focuses on edge cases that require special handling in the Qwik optimizer: nested loops in QRL, skip transform markers, illegal code detection (classes/functions in QRL), empty components, unicode identifiers, comments handling, async/await in QRL, and regression fixes for 6 documented issues.

The SWC reference implementation uses `loop_depth` and `iteration_var_stack` to track loop context and iteration variables for QRL hoisting. Illegal code detection reports errors but allows transformation to proceed. The OXC implementation already has `illegal_code.rs` with basic detection, but needs integration with diagnostic reporting.

**Primary recommendation:** Implement loop tracking infrastructure (loop_depth, iteration_var_stack) first, then add illegal code diagnostic reporting, then handle remaining edge cases (empty components, unicode, comments) through targeted test cases.

## Standard Stack

### Core (Already Present)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| OXC | 0.111.0 | AST parsing and transformation | Already adopted |
| oxc_ast | 0.111.0 | AST node types including ForStatement, WhileStatement | Required for loop handling |
| oxc_semantic | 0.111.0 | Scope and symbol tracking | Already used for variable capture |

### Supporting (Already Present)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| oxc_allocator | 0.111.0 | Arena allocation | All AST operations |
| thiserror | - | Error definition | Already used in error.rs |

## Architecture Patterns

### Recommended Pattern: Loop Tracking

The SWC implementation tracks loops using two fields:

```rust
// SWC struct fields
loop_depth: u32,
iteration_var_stack: Vec<Vec<ast::Ident>>,
```

**OXC Equivalent:**
```rust
// Add to TransformGenerator struct
loop_depth: u32,
iteration_var_stack: Vec<Vec<Id>>,  // Vec<(String, ScopeId)>
```

### Pattern 1: For Statement Handling

**What:** Track loop depth and iteration variables when entering for loops
**When to use:** All for/for-in/for-of/while statements
**Example (from SWC transform.rs:2786-2817):**
```rust
fn fold_for_stmt(&mut self, node: ast::ForStmt) -> ast::ForStmt {
    self.decl_stack.push(vec![]);
    self.loop_depth += 1;

    // Track the loop variable from the initialization
    let iteration_vars = if let Some(ast::VarDeclOrExpr::VarDecl(ref var_decl)) = node.init {
        var_decl.decls.first()
            .and_then(|decl| {
                if let ast::Pat::Ident(ident) = &decl.name {
                    Some(vec![ident.id.clone()])
                } else {
                    None
                }
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    self.iteration_var_stack.push(iteration_vars);

    let o = node.fold_children_with(self);

    self.iteration_var_stack.pop();
    self.loop_depth -= 1;
    self.decl_stack.pop();
    o
}
```

### Pattern 2: Illegal Code Detection and Reporting

**What:** Detect class/function declarations inside QRL and report as diagnostics
**When to use:** Any QRL ($, component$, useTask$) body that references a local class or function
**Example (from SWC snapshot qwik_core__test__example_capturing_fn_class.snap):**
```json
// DIAGNOSTICS section shows errors
{
  "category": "error",
  "code": "C02",
  "file": "test.tsx",
  "message": "Reference to identifier 'Thing' can not be used inside a Qrl($) scope because it's a function",
  "scope": "optimizer"
}
```

The OXC implementation already has `IllegalCodeType` enum in `illegal_code.rs`:
```rust
pub enum IllegalCodeType {
    Class(SymbolId, Option<String>),
    Function(SymbolId, Option<String>),
}
```

### Pattern 3: QRL Hoisting in Loops

**What:** Hoist QRL definitions outside loop bodies to avoid recreating them on each iteration
**When to use:** Event handlers inside map/forEach callbacks or traditional loops
**Key insight from SWC (transform.rs:990):**
```rust
let should_hoist = self.loop_depth > 0 && !is_fn && !self.hoisted_qrls.is_empty();
```

**Output pattern (from snapshot example_component_with_event_listeners_inside_loop.snap):**
```javascript
function loopArrowFn(results) {
    // QRL hoisted OUTSIDE the map callback
    const App_component_loopArrowFn_span_on_click_LKE0yw7iEx4 = /*#__PURE__*/ qrl(
        i_LKE0yw7iEx4,
        "App_component_loopArrowFn_span_on_click_LKE0yw7iEx4",
        [cart]
    );
    // Loop variable passed via q:p prop instead of capture
    return results.map((item) => /*#__PURE__*/ _jsxSorted("span", {
        "on:click": App_component_loopArrowFn_span_on_click_LKE0yw7iEx4,
        "q:p": item  // Iteration variable passed here
    }, null, item, 0, "u6_0"));
}
```

### Pattern 4: Iteration Variable as QRL Parameter

**What:** Loop iteration variables become QRL parameters (via `q:p` prop) instead of captures
**When to use:** When event handler QRLs inside loops reference the iteration variable
**Example (from snapshot):**
```javascript
// Event handler receives iteration var as third parameter
export const App_component_loopArrowFn_span_on_click_LKE0yw7iEx4 = (_, _1, item) => {
    const [cart] = useLexicalScope();
    cart.push(item);
};
```

The signature `(_, _1, item)` shows:
- First param: event (ignored with `_`)
- Second param: element (ignored with `_1`)
- Third param: iteration variable passed via `q:p`

### Anti-Patterns to Avoid

- **Capturing iteration variables in QRL:** Would cause stale closures where all iterations share the same captured value
- **Creating QRLs inside loops:** Would generate new QRL on each iteration, inefficient
- **Ignoring loop context in capture analysis:** Would miss iteration variables that need special handling

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Error reporting | Custom error struct | Existing `ProcessingFailure` struct | Already has file, message, code fields |
| Loop variable tracking | Manual AST inspection | OXC's pattern matching on ForStatement variants | Correct handling of all loop types |
| Identifier detection | String matching | ScopeId-based lookup via semantic analysis | Avoids name collision issues |

**Key insight:** The existing `illegal_code.rs` module has the detection logic - just needs wiring to diagnostic output.

## Common Pitfalls

### Pitfall 1: Nested Loop Variable Shadowing
**What goes wrong:** Inner loop variable shadows outer, causing incorrect capture analysis
**Why it happens:** Not tracking variable scope properly across nested loops
**How to avoid:** Push/pop `iteration_var_stack` for each loop level, not just `loop_depth`
**Warning signs:** Test `should_transform_nested_loops` fails with wrong captures

### Pitfall 2: Map Callback vs Traditional Loop
**What goes wrong:** Map callbacks not recognized as "loop context"
**Why it happens:** Only tracking for/while statements, not CallExpression with `.map()` callee
**How to avoid:** SWC tracks `root_jsx_mode` flag and handles map callbacks in fold_call_expression
**Warning signs:** `example_component_with_event_listeners_inside_loop` fails for `loopArrowFn` case

### Pitfall 3: Illegal Code Detection vs Rejection
**What goes wrong:** Transformation fails instead of reporting diagnostic and continuing
**Why it happens:** Treating illegal code as fatal error
**How to avoid:** Add to `errors: Vec<ProcessingFailure>` but continue transformation
**Warning signs:** Test `example_capturing_fn_class` snapshot mismatch - should output code AND diagnostics

### Pitfall 4: async/await Stripping
**What goes wrong:** async functions lose async keyword after transformation
**Why it happens:** Forgetting to preserve async flag when creating QRL segment
**How to avoid:** Check `is_async` on ArrowFunctionExpression/Function and preserve in output
**Warning signs:** `example_use_server_mount` outputs sync function instead of `async () => {...}`

## Code Examples

### Example 1: Loop Depth Tracking (OXC Traverse pattern)

```rust
impl<'a, 'gen> Traverse<'a> for TransformGenerator<'gen> {
    fn enter_for_statement(&mut self, node: &ForStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.decl_stack.push(vec![]);
        self.loop_depth += 1;

        // Extract iteration variable from init
        if let Some(ForStatementInit::VariableDeclaration(var_decl)) = &node.init {
            if let Some(decl) = var_decl.declarations.first() {
                if let BindingPatternKind::BindingIdentifier(id) = &decl.id.kind {
                    let name = id.name.to_string();
                    let scope_id = id.symbol_id.get().map(|s|
                        ctx.scoping().symbol_scope_id(s)
                    ).unwrap_or(ScopeId::new(0));
                    self.iteration_var_stack.push(vec![(name, scope_id)]);
                }
            }
        }
    }

    fn exit_for_statement(&mut self, node: &ForStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.loop_depth > 0 {
            self.iteration_var_stack.pop();
            self.loop_depth -= 1;
        }
        self.decl_stack.pop();
    }
}
```

### Example 2: Illegal Code Diagnostic Reporting

```rust
// In transform.rs, during QRL body analysis
for stmt in &body.statements {
    if let Some(illegal_type) = stmt.is_illegal_code_in_qrl() {
        self.errors.push(ProcessingFailure {
            category: "error".to_string(),
            code: "C02".to_string(),  // SWC uses C02 for illegal code
            file: self.source_info.file_name.to_string(),
            message: format!(
                "Reference to identifier '{}' can not be used inside a Qrl($) scope because it's a {}",
                illegal_type.identifier(),
                illegal_type.expression_type()
            ),
            highlights: None,
            suggestions: None,
            scope: "optimizer".to_string(),
        });
    }
}
```

### Example 3: QRL Hoisting Check

```rust
// When creating QRL in loop context
fn should_hoist_qrl(&self, is_fn_qrl: bool) -> bool {
    self.loop_depth > 0 && !is_fn_qrl
}

// When hoisting, add q:p attribute for iteration variable
fn add_iteration_var_prop(&mut self, jsx_props: &mut Vec<ObjectPropertyKind>) {
    if let Some(iter_vars) = self.iteration_var_stack.last() {
        for (var_name, _) in iter_vars {
            // Add "q:p": var_name to props
            jsx_props.push(/* ... property construction ... */);
        }
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Capture iteration vars | Pass via q:p prop | Qwik 1.x | Prevents stale closure bugs |
| Fatal error on illegal code | Diagnostic + continue | SWC optimizer | Better developer experience |

**Deprecated/outdated:**
- None identified - SWC patterns remain authoritative for this phase

## Issue Regression Tests Summary

From SWC test.rs analysis, here are the 6 issue regression tests to implement:

| Issue | Description | Key Pattern |
|-------|-------------|-------------|
| issue_117 | Empty/pass-through without QRL | No component$, just exports |
| issue_150 | Ternary in class object | `class={{ foo: true, bar: stuff.condition }}` |
| issue_476 | JSX without transpile | `transpile_jsx: false` preserves JSX |
| issue_964 | Generator function inside component | `function*(lo, t) { yield... }` |
| issue_5008 | Map with function expression | `.map(function (v, idx) {...})` |
| issue_7216 | (Additional issue mentioned in ROADMAP) | TBD - need to locate test |

## Open Questions

1. **Map callback detection**
   - What we know: SWC tracks loop context for traditional loops and `.map()` callbacks
   - What's unclear: Exact mechanism for detecting map callback as "loop" context
   - Recommendation: Search SWC for `fold_call_expression` handling of map

2. **q:p prop format**
   - What we know: Iteration variable passed via `q:p` prop on JSX elements
   - What's unclear: Exact serialization format for multiple iteration variables
   - Recommendation: Check nested loop snapshot more carefully

3. **Comments preservation (EDG-07)**
   - What we know: Comments appear in some snapshots
   - What's unclear: Which comments are preserved vs stripped
   - Recommendation: Add targeted test case to determine behavior

## Sources

### Primary (HIGH confidence)
- SWC reference implementation: `qwik-core/src/optimizer/core/src/transform.rs` - loop handling at lines 2786-2910
- SWC test cases: `qwik-core/src/optimizer/core/src/test.rs` - issue regression tests
- SWC snapshots: `qwik-core/src/optimizer/core/src/snapshots/` - expected output

### Secondary (MEDIUM confidence)
- OXC illegal_code.rs: Already implemented basic detection
- OXC error.rs: Error types defined with IllegalCode variant

### Tertiary (LOW confidence)
- None - all findings verified against SWC reference implementation

## Metadata

**Confidence breakdown:**
- Loop tracking pattern: HIGH - directly from SWC source
- Illegal code reporting: HIGH - snapshots show exact format
- QRL hoisting: HIGH - snapshots show exact output structure
- Async/await handling: HIGH - snapshots confirm async preservation
- Unicode/comments: MEDIUM - limited test coverage visible

**Research date:** 2026-01-29
**Valid until:** 2026-02-28 (stable domain, patterns well-established)

# Phase 11: Research & Code Cleanup - Research

**Researched:** 2026-01-29
**Domain:** Rust code modularization, OXC API utilization, AST transformer architecture
**Confidence:** MEDIUM (primary patterns verified through OXC docs; some specific API discoveries are LOW confidence)

## Summary

This phase focuses on reducing transform.rs from 7571 lines through OXC API adoption, modularization, and Rust ecosystem library integration. Research covered three key areas: (1) OXC APIs that could replace custom implementations, (2) modularization patterns for large Rust AST transformers, and (3) the OXC ecosystem's approach to composable transforms.

The primary finding is that OXC's design philosophy strongly favors single-pass traversal with composable sub-transforms. The `oxc_traverse` crate provides powerful parent context access that could simplify several workarounds currently in transform.rs. However, splitting a `Traverse` trait implementation across files requires careful design since trait impl methods must remain in one block.

**Primary recommendation:** Split transform.rs into domain-specific modules (jsx/, qrl/, scope/, imports/) with helper methods factored out, keeping the single `impl Traverse` block as a thin dispatcher that delegates to module-specific logic.

## Standard Stack

The established libraries/tools for this domain:

### Core (Already in Use)
| Library | Version | Purpose | Optimization Opportunity |
|---------|---------|---------|--------------------------|
| oxc_traverse | 0.111.0 | AST traversal with parent context | TraverseCtx has underutilized APIs for scope/parent access |
| oxc_ast | 0.111.0 | AST types and AstBuilder | AstBuilder has many convenience methods not being used |
| oxc_semantic | 0.111.0 | Semantic analysis, symbols, scopes | Scoping struct has rich symbol query APIs |
| oxc_codegen | 0.111.0 | Code generation | Already well-utilized |

### Supporting (Potential Additions)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| indexmap | 2.x | Order-preserving HashMap | Replace HashMap where order matters for deterministic output |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Custom scope tracking (decl_stack) | oxc_semantic Scoping APIs | More idiomatic but requires SemanticBuilder pass |
| Manual import deduplication | HashSet with custom Ord | Already using BTreeSet, which is optimal |
| Custom hash generation | oxc utilities | Current implementation is correct, no change needed |

## Architecture Patterns

### Recommended Module Structure
```
optimizer/src/
├── transform/
│   ├── mod.rs              # TransformGenerator struct + impl Traverse (dispatcher only)
│   ├── state.rs            # All state structs (JsxState, TransformOptions, etc.)
│   ├── jsx/
│   │   ├── mod.rs          # JSX transformation logic
│   │   ├── element.rs      # enter/exit_jsx_element helpers
│   │   ├── attribute.rs    # enter/exit_jsx_attribute helpers
│   │   ├── fragment.rs     # Fragment handling
│   │   └── event.rs        # Event handler transformation utilities
│   ├── qrl/
│   │   ├── mod.rs          # QRL extraction logic
│   │   ├── capture.rs      # Scope capture computation (compute_scoped_idents)
│   │   ├── hash.rs         # Hash and display name generation
│   │   └── segment.rs      # Segment stack management
│   ├── scope/
│   │   ├── mod.rs          # Declaration tracking
│   │   ├── decl_stack.rs   # decl_stack management
│   │   └── import_tracker.rs # ImportTracker struct
│   └── imports/
│       ├── mod.rs          # Import/export handling
│       ├── synthesized.rs  # Synthesized imports management
│       └── cleanup.rs      # Move ImportCleanUp here
├── transform.rs            # DEPRECATED - keep as re-export for backward compat
└── ...
```

### Pattern 1: Dispatcher Pattern for Traverse Impl
**What:** Keep `impl Traverse for TransformGenerator` minimal - each method just delegates to module-specific handler functions.
**When to use:** When you have a single trait impl that must stay in one block but want logic split across files.
**Example:**
```rust
// Source: OXC transformer organization pattern
// transform/mod.rs
impl<'a> Traverse<'a, ()> for TransformGenerator<'a> {
    fn enter_jsx_element(&mut self, node: &mut JSXElement<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        jsx::enter_element(self, node, ctx);
    }

    fn exit_jsx_element(&mut self, node: &mut JSXElement<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        jsx::exit_element(self, node, ctx);
    }
    // ... other methods delegate similarly
}

// transform/jsx/element.rs
pub fn enter_element<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &mut JSXElement<'a>,
    ctx: &mut TraverseCtx<'a, ()>
) {
    // All the actual logic lives here
}
```

### Pattern 2: State Extraction
**What:** Move state structs and their impls to dedicated files.
**When to use:** When state management logic is interleaved with transformation logic.
**Example:**
```rust
// transform/state.rs
pub struct JsxState<'gen> {
    pub is_fn: bool,
    pub is_text_only: bool,
    // ... all fields
}

impl<'gen> JsxState<'gen> {
    pub fn new(builder: &AstBuilder<'gen>, /* params */) -> Self { ... }
    pub fn finalize_props(&mut self) { ... }
}
```

### Anti-Patterns to Avoid
- **Splitting Traverse impl methods across files:** Rust doesn't allow this; keep the impl in one file as a dispatcher
- **Deep nesting in module handlers:** Keep handler functions flat, extract complex logic to helper functions
- **Circular dependencies between modules:** Use clear ownership hierarchy (jsx uses qrl utilities, not vice versa)

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Symbol resolution | Custom symbol_by_name HashMap | oxc_semantic Scoping::symbol_name() | Already has optimized lookup |
| Scope parent access | Manual parent tracking | TraverseCtx::ancestor() | Built-in, memory-safe |
| Import deduplication | Custom HashMap logic | BTreeSet (already used) | Current approach is optimal |
| AST node creation | Manual Expression construction | AstBuilder convenience methods | Many exist but aren't used |
| Identifier type checking | Pattern matching chains | AST match macros (match_expression!) | Cleaner, maintained |

**Key insight:** The OXC project invests heavily in making the builder and context APIs ergonomic. Many convenience methods exist but aren't documented well - the docs.rs coverage is only 39.53%. Exploring the source directly reveals more options.

## Common Pitfalls

### Pitfall 1: Breaking the Single Trait Impl Rule
**What goes wrong:** Attempting to split `impl Traverse for TransformGenerator` across multiple files causes compile errors.
**Why it happens:** Rust requires trait implementations to be contiguous.
**How to avoid:** Use the dispatcher pattern - keep impl in one file, delegate to module functions.
**Warning signs:** "conflicting implementations" or "impl already exists" errors.

### Pitfall 2: Lifetime Pollution Across Modules
**What goes wrong:** Module boundaries make lifetime annotations complex; you end up with `<'a, 'b, 'c>` everywhere.
**Why it happens:** TransformGenerator holds references with lifetime `'gen` that must propagate.
**How to avoid:** Pass `&mut TransformGenerator<'a>` to module functions rather than extracting individual fields.
**Warning signs:** Functions requiring many lifetime parameters.

### Pitfall 3: State Mutation Order Dependencies
**What goes wrong:** Extracting logic to modules can break subtle ordering dependencies (e.g., jsx_stack.push before segment_stack.push).
**Why it happens:** The original code relies on implicit ordering that isn't documented.
**How to avoid:** Document state machine transitions before refactoring; add assertion tests for critical invariants.
**Warning signs:** Tests pass individually but fail when run together.

### Pitfall 4: Over-Modularization
**What goes wrong:** Creating too many small modules (e.g., one file per enter_ method) increases cognitive load.
**Why it happens:** Mechanically applying "one function per file" rule.
**How to avoid:** Group by logical domain (jsx, qrl, scope), not by AST node type.
**Warning signs:** More than 20 files in transform/ directory; files under 50 lines.

### Pitfall 5: SemanticBuilder Pass Overhead
**What goes wrong:** Using oxc_semantic APIs requires running SemanticBuilder, adding a full AST pass.
**Why it happens:** Semantic analysis needs its own traversal to build symbol tables.
**How to avoid:** Current manual tracking may be more efficient for this use case; don't switch to semantic APIs without benchmarking.
**Warning signs:** Noticeable performance regression in transform time.

## Code Examples

Verified patterns from OXC sources:

### TraverseCtx Ancestor Access
```rust
// Source: https://docs.rs/oxc_traverse - TraverseCtx API
// Access parent nodes during traversal
fn exit_call_expression(&mut self, node: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a, ()>) {
    // Get immediate parent
    let parent = ctx.parent();

    // Check ancestor at specific depth
    if let Ancestor::ExportNamedDeclarationDeclaration(_) = ctx.ancestor(1) {
        // Inside an export declaration
    }

    // Iterate all ancestors
    for ancestor in ctx.ancestors() {
        // ...
    }
}
```

### AstBuilder Convenience Methods
```rust
// Source: https://docs.rs/oxc_ast - AstBuilder
// Many convenience methods exist for common patterns

// Instead of manual construction:
let expr = builder.expression_call(
    SPAN,
    builder.expression_identifier(SPAN, "foo"),
    None::<OxcBox<TSTypeParameterInstantiation>>,
    builder.vec_from_array([arg]),
    false,
);

// AstBuilder has many vec_from_* helpers:
builder.vec1(single_item)           // Creates Vec with one element
builder.vec_from_array([a, b, c])   // From array
builder.vec_from_iter(iterator)     // From iterator
builder.vec_with_capacity(n)        // Pre-allocated
```

### Scoping Symbol Access
```rust
// Source: https://docs.rs/oxc_semantic - Scoping API
// If SemanticBuilder pass is run, these APIs are available:

let scoping = ctx.scoping();

// Get symbol by reference
if let Some(symbol_id) = scoping.get_reference(ref_id).symbol_id() {
    let name = scoping.symbol_name(symbol_id);
    let flags = scoping.symbol_flags(symbol_id);

    // Get all references to this symbol
    for ref_id in scoping.get_resolved_reference_ids(symbol_id) {
        // ...
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| SWC Fold trait | OXC Traverse trait | OXC 0.100+ | Different visitor pattern, enter/exit style |
| Manual parent tracking | TraverseCtx::ancestor() | OXC traverse crate | Built-in parent access |
| Separate semantic pass | Optional with Transformer | OXC 0.90+ | Can skip if not needed |

**Deprecated/outdated:**
- `Visit` and `VisitMut` traits: Still work but `Traverse` is preferred for transforms
- Direct AST construction without AstBuilder: Works but loses safety benefits

## Open Questions

Things that couldn't be fully resolved:

1. **Optimal module boundary granularity**
   - What we know: OXC transformer uses one file per "transform" (e.g., ReactJsx, ReactDisplayName)
   - What's unclear: For a single large transform like QwikTransform, whether to split by domain or by AST node type
   - Recommendation: Start with domain split (jsx/, qrl/, scope/), adjust based on actual code review

2. **Performance impact of modularization**
   - What we know: Rust inlines aggressively; function calls across modules should be zero-cost
   - What's unclear: Whether the current monolithic approach has any cache locality benefits
   - Recommendation: Benchmark before/after with `cargo bench` if significant perf concerns

3. **oxc_semantic API completeness for our use case**
   - What we know: SemanticBuilder provides rich symbol tables and scope trees
   - What's unclear: Whether it tracks all info currently in decl_stack (IdPlusType with Var/Fn/Class distinction)
   - Recommendation: Keep custom decl_stack for now; explore semantic APIs in dedicated research task

## Sources

### Primary (HIGH confidence)
- [oxc_traverse docs](https://docs.rs/oxc_traverse) - TraverseCtx API reference
- [oxc_ast docs](https://docs.rs/oxc_ast) - AstBuilder and AST types
- [oxc_semantic docs](https://docs.rs/oxc_semantic) - Scoping and Symbol APIs
- [OXC GitHub - oxc_transformer](https://github.com/oxc-project/oxc/tree/main/crates/oxc_transformer) - Transform organization patterns

### Secondary (MEDIUM confidence)
- [OXC Transformer Guide](https://oxc.rs/docs/guide/usage/transformer.html) - Official usage patterns
- [Rust book - Modules](https://doc.rust-lang.org/book/ch07-05-separating-modules-into-different-files.html) - File separation patterns
- [Rust forum - Big impls](https://users.rust-lang.org/t/code-structure-for-big-impl-s-distributed-over-several-files/7785) - Community patterns

### Tertiary (LOW confidence)
- WebSearch results on Rolldown/Vite OXC integration patterns - Useful but not directly applicable
- WebSearch results on AST-grep refactoring - General Rust patterns, not OXC-specific

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Verified through docs.rs and official OXC documentation
- Architecture patterns: MEDIUM - Derived from OXC transformer organization, not explicitly documented for external use
- Pitfalls: MEDIUM - Based on general Rust patterns and code analysis, not OXC-specific documentation
- OXC API opportunities: LOW to MEDIUM - docs.rs coverage is only 39.53%; some APIs may have undocumented capabilities

**Research date:** 2026-01-29
**Valid until:** 60 days (OXC evolves rapidly but core patterns are stable)

---

## Appendix: Current transform.rs Analysis

### Line Count Breakdown (Approximate)
| Section | Lines | % | Notes |
|---------|-------|---|-------|
| Imports and constants | 1-50 | <1% | Could be in prelude |
| Structs (ImportTracker, OptimizedApp, JsxState, TransformGenerator) | 50-370 | 4% | Extract to state.rs |
| TransformGenerator::new and helpers | 370-650 | 4% | Could be in separate impl block |
| impl Traverse (enter/exit methods) | 650-2745 | 28% | KEEP but make dispatcher |
| Utility functions (jsx_event_*, compute_scoped_idents) | 2745-2920 | 2% | Extract to utils modules |
| TransformOptions and transform() | 2920-3080 | 2% | Extract to options.rs |
| Tests | 3080-7571 | 60% | Move to tests/ directory |

### Modularization Priority
1. **High Impact:** Move tests to `tests/transform_tests.rs` (-4500 lines immediately)
2. **Medium Impact:** Extract JSX handling to `transform/jsx/` module (~800 lines)
3. **Medium Impact:** Extract QRL/segment handling to `transform/qrl/` module (~400 lines)
4. **Low Impact:** Extract utility functions to dedicated modules (~200 lines)

### Dependencies Between Sections
```
TransformGenerator fields
    ├── jsx_stack (JsxState) ──────────> JSX enter/exit methods
    ├── segment_stack ─────────────────> QRL extraction, display name
    ├── decl_stack (IdPlusType) ───────> compute_scoped_idents
    ├── import_stack ──────────────────> Import tracking, finalization
    ├── props_identifiers ─────────────> Props destructuring, _wrapProp
    └── stack_ctxt ────────────────────> Entry strategy
```

This dependency map shows that fields are accessed across multiple enter/exit methods, confirming the dispatcher pattern is appropriate (single struct, delegated logic).

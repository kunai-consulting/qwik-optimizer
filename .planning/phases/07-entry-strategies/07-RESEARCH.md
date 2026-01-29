# Phase 7: Entry Strategies - Research

**Researched:** 2026-01-29
**Domain:** Qwik optimizer code-splitting strategies
**Confidence:** HIGH

## Summary

This phase implements the remaining entry strategies for the OXC port of the Qwik optimizer. Entry strategies control how QRL functions (lazy-loadable code segments) are grouped and bundled. The SWC reference implementation (`qwik-core/src/optimizer/core/src/entry_strategy.rs`) provides a complete working model that the OXC port must match exactly.

The OXC port already has the entry strategy infrastructure (enums, trait, basic implementations) but two strategies have `panic!("Not implemented")`: `PerComponentStrategy` and `SmartStrategy`. Additionally, the `stack_ctxt` (context stack) tracking that these strategies depend on is missing from the OXC port's `TransformGenerator`.

**Primary recommendation:** Implement `stack_ctxt` context tracking in TransformGenerator, then port the SWC implementations of PerComponentStrategy and SmartStrategy verbatim.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| oxc | 0.111.0 | AST traversal and transformation | Already used throughout the codebase |
| serde | current | Serialization for entry strategy enum | Already used for EntryStrategy serde |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| lazy_static | (SWC only) | Static ENTRY_SEGMENTS atom | OXC port uses const &str instead - simpler |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| &str ENTRY_SEGMENTS | lazy_static Atom | SWC uses Atom for interning; OXC port already uses String - keep consistent |

**No new dependencies needed** - all infrastructure exists.

## Architecture Patterns

### Existing Code Structure
```
optimizer/src/
├── entry_strategy.rs      # EntryStrategy enum, EntryPolicy trait, strategy implementations
├── segment.rs             # Segment enum (Named, NamedQrl, IndexQrl)
├── transform.rs           # TransformGenerator - NEEDS stack_ctxt
├── component/
│   ├── segment_data.rs    # SegmentData struct with origin, ctx_kind, scoped_idents
│   ├── qrl.rs             # Qrl struct
│   └── component.rs       # QrlComponent
└── js_lib_interface.rs    # SegmentAnalysis with entry field
```

### Pattern 1: Context Stack (stack_ctxt)
**What:** A `Vec<String>` that tracks the hierarchical context as the AST is traversed (file name, function names, component names, JSX element names, attribute names).
**When to use:** Always - PerComponentStrategy and SmartStrategy depend on this for determining the "root" component.
**Example from SWC:**
```rust
// Source: qwik-core/src/optimizer/core/src/transform.rs
struct QwikTransform {
    stack_ctxt: Vec<String>,  // Context stack
    // ...
}

// Push on entry to named scope:
// - fold_var_declarator: push(ident.id.sym.to_string())
// - fold_fn_decl: push(node.ident.sym.to_string())
// - fold_class_decl: push(node.ident.sym.to_string())
// - fold_jsx_element_name: push(ident.sym.to_string())
// - fold_jsx_attr: push(key_word.to_string())
// - fold_call_expr: push(ident.sym.to_string()) for marker functions
// - fold_module: push(filename)

// Pop on exit from named scope
```

### Pattern 2: EntryPolicy Trait
**What:** Trait that defines `get_entry_for_sym(&self, context: &[String], segment: &SegmentData) -> Option<String>`
**When to use:** Called during QRL creation to determine which entry file a segment belongs to.
**Example:**
```rust
// Source: qwik-core/src/optimizer/core/src/entry_strategy.rs
pub trait EntryPolicy: Send + Sync {
    fn get_entry_for_sym(&self, context: &[String], segment: &SegmentData) -> Option<String>;
}
```

### Pattern 3: Strategy Return Values
| Strategy | Return Value | Effect |
|----------|--------------|--------|
| InlineStrategy | `Some("entry_segments")` | All segments in one file |
| SingleStrategy | `Some("entry_segments")` | All segments in one file |
| PerSegmentStrategy | `None` | Each segment in its own file |
| PerComponentStrategy | `Some("{origin}_entry_{root}")` or `Some("entry_segments")` if no context | Groups by root component |
| SmartStrategy | Complex logic (see below) | Optimizes based on captures and segment type |

### Anti-Patterns to Avoid
- **Using PathBuf for origin in entry string:** The SWC uses `&segment.origin` (an Atom/string). The OXC SegmentData has `origin: PathBuf` - must convert to string using `.display().to_string()` or `.to_string_lossy()`.
- **Missing stack_ctxt updates:** Every named scope entry/exit must update the context stack, or PerComponentStrategy/SmartStrategy will return incorrect groupings.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Context tracking | Manual name tracking | stack_ctxt Vec<String> pattern | SWC pattern is proven, matches output exactly |
| Entry file naming | Custom naming scheme | `[origin, "_entry_", root].concat()` pattern | Must match SWC exactly for bundler compatibility |
| Smart strategy logic | Simplify the algorithm | Copy SWC implementation exactly | The logic considers multiple factors (scoped_idents, ctx_kind, ctx_name) |

**Key insight:** Entry strategies affect bundler output. Any deviation from SWC behavior will cause mismatched bundles in production builds.

## Common Pitfalls

### Pitfall 1: Missing PathBuf to String Conversion
**What goes wrong:** OXC SegmentData.origin is PathBuf, but entry string concatenation expects &str.
**Why it happens:** SWC uses Atom for all strings; OXC uses mixed types.
**How to avoid:** Always convert: `segment.origin.display().to_string()` or add a helper method.
**Warning signs:** Type errors during concatenation `[&segment.origin, "_entry_", root].concat()`.

### Pitfall 2: Incomplete stack_ctxt Tracking
**What goes wrong:** PerComponentStrategy returns wrong groupings; segments end up in wrong files.
**Why it happens:** Missing push/pop in some AST visitor methods.
**How to avoid:** Match all SWC fold_* methods that modify stack_ctxt:
- `fold_module` (filename)
- `fold_var_declarator` (variable name)
- `fold_fn_decl` (function name)
- `fold_class_decl` (class name)
- `fold_jsx_element_name` (element name)
- `fold_jsx_attr` (attribute name)
- `fold_call_expr` (callee name for marker functions)
**Warning signs:** Context stack is empty when it shouldn't be, or contains wrong items.

### Pitfall 3: SmartStrategy Logic Differences
**What goes wrong:** Event handlers grouped incorrectly; wrong segments separated.
**Why it happens:** Subtle logic differences in the scoped_idents/ctx_kind checks.
**How to avoid:** Copy SWC implementation exactly:
```rust
// Event handlers without scope variables go to separate file
if segment.scoped_idents.is_empty()
    && (segment.ctx_kind != SegmentKind::Function || &segment.ctx_name == "event$")
{
    return None;
}
// Everything else grouped by component
context.first().map_or(|| None, |root| Some(...))
```
**Warning signs:** Test failures on SmartStrategy-specific tests.

### Pitfall 4: ScopeId Type Mismatch
**What goes wrong:** The OXC Segment enum (Named, NamedQrl, IndexQrl) doesn't have SegmentData fields.
**Why it happens:** The entry strategy needs SegmentData, but the Segment enum is used for name tracking.
**How to avoid:** The entry_strategy.rs already takes `&Segment` but the commented code shows it should take `&SegmentData`. Need to verify the trait signature and update implementations to use SegmentData fields.
**Warning signs:** Strategy can't access scoped_idents, ctx_kind, origin.

## Code Examples

Verified patterns from the SWC reference implementation:

### PerComponentStrategy Implementation
```rust
// Source: qwik-core/src/optimizer/core/src/entry_strategy.rs:76-82
impl EntryPolicy for PerComponentStrategy {
    fn get_entry_for_sym(&self, context: &[String], segment: &SegmentData) -> Option<String> {
        context.first().map_or_else(
            || Some(ENTRY_SEGMENTS.to_string()),
            |root| Some([segment.origin.display().to_string().as_str(), "_entry_", root].concat()),
        )
    }
}
```

### SmartStrategy Implementation
```rust
// Source: qwik-core/src/optimizer/core/src/entry_strategy.rs:94-112
impl EntryPolicy for SmartStrategy {
    fn get_entry_for_sym(&self, context: &[String], segment: &SegmentData) -> Option<String> {
        // Event handlers without scope variables are put into a separate file
        if segment.scoped_idents.is_empty()
            && (segment.ctx_kind != SegmentKind::Function || &segment.ctx_name == "event$")
        {
            return None;
        }

        // Everything else is put into a single file per component
        // This means that all QRLs for a component are loaded together
        // if one is used
        context.first().map_or_else(
            // Top-level QRLs are put into a separate file
            || None,
            // Other QRLs are put into a file named after the original file + the root component
            |root| Some([segment.origin.display().to_string().as_str(), "_entry_", root].concat()),
        )
    }
}
```

### Context Stack Updates (Visitor Methods)
```rust
// Source: qwik-core/src/optimizer/core/src/transform.rs (various fold_* methods)

// Variable declarator - push identifier name
fn fold_var_declarator(&mut self, node: ast::VarDeclarator) -> ast::VarDeclarator {
    let stacked = if let ast::Pat::Ident(ref ident) = node.name {
        self.stack_ctxt.push(ident.id.sym.to_string());
        true
    } else {
        false
    };
    let o = node.fold_children_with(self);
    if stacked {
        self.stack_ctxt.pop();
    }
    o
}

// Function declaration - push function name
fn fold_fn_decl(&mut self, node: ast::FnDecl) -> ast::FnDecl {
    self.stack_ctxt.push(node.ident.sym.to_string());
    let o = node.fold_children_with(self);
    self.stack_ctxt.pop();
    o
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hook strategy | Segment strategy alias | Early Qwik | EntryStrategy::Hook maps to PerSegmentStrategy |
| Hoist strategy | Inline strategy alias | Early Qwik | EntryStrategy::Hoist maps to InlineStrategy |

**Deprecated/outdated:**
- `EntryStrategy::Hook`: Still in enum for backwards compatibility, maps to PerSegmentStrategy
- `EntryStrategy::Hoist`: Still in enum for backwards compatibility, maps to InlineStrategy

## Implementation Checklist

Based on analysis, the implementation requires:

1. **Add stack_ctxt to TransformGenerator**
   - New field: `stack_ctxt: Vec<String>`
   - Initialize in constructor: `stack_ctxt: Vec::with_capacity(16)`

2. **Add stack_ctxt updates to visitor methods**
   - `enter_variable_declarator` / `exit_variable_declarator` for variable names
   - `enter_function` / `exit_function` for function names (from fn_decl)
   - `enter_class` / `exit_class` for class names
   - `enter_jsx_element` / `exit_jsx_element` for element names
   - `enter_jsx_attribute` / `exit_jsx_attribute` for attribute names
   - `enter_call_expression` / `exit_call_expression` for marker function names
   - Module-level: push filename at start

3. **Update EntryPolicy trait signature**
   - Current: `fn get_entry_for_sym(&self, context: &[String], segment: &Segment)`
   - Should be: `fn get_entry_for_sym(&self, context: &[String], segment: &SegmentData)` (matches SWC)

4. **Implement PerComponentStrategy**
   - Remove panic!, implement using context.first() and segment.origin

5. **Implement SmartStrategy**
   - Remove panic!, implement logic checking scoped_idents and ctx_kind

6. **Wire entry strategy to SegmentAnalysis**
   - Call entry_policy.get_entry_for_sym() when creating segments
   - Populate entry field in SegmentAnalysis

## Open Questions

Things that couldn't be fully resolved:

1. **TransformOptions vs js_lib_interface entry_strategy**
   - What we know: TransformOptions doesn't have entry_strategy field; js_lib_interface.TransformFsModuleInput does
   - What's unclear: How entry_strategy flows from input to TransformGenerator
   - Recommendation: Add entry_strategy to TransformOptions, or pass entry_policy as separate parameter

2. **PathBuf origin conversion**
   - What we know: SWC uses Atom, OXC uses PathBuf for origin
   - What's unclear: Best conversion method (to_string_lossy vs display)
   - Recommendation: Use `.display().to_string()` for consistent cross-platform behavior

## Sources

### Primary (HIGH confidence)
- `qwik-core/src/optimizer/core/src/entry_strategy.rs` - Complete SWC reference implementation
- `qwik-core/src/optimizer/core/src/transform.rs` - stack_ctxt usage patterns (lines 105, 237, 332, 391, 827, 861, etc.)
- `optimizer/src/entry_strategy.rs` - Current OXC port (with panic! placeholders)
- `optimizer/src/component/segment_data.rs` - SegmentData struct with all needed fields

### Secondary (MEDIUM confidence)
- `qwik-core/src/optimizer/core/src/test.rs` - Test patterns for entry strategies

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - existing codebase, no new dependencies
- Architecture: HIGH - complete SWC reference available
- Pitfalls: HIGH - based on code analysis and type system differences

**Research date:** 2026-01-29
**Valid until:** 60 days (stable implementation, no upstream changes expected)

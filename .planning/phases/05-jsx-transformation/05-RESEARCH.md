# Phase 5: JSX Transformation - Research

**Researched:** 2026-01-29
**Domain:** JSX to _jsxSorted transformation with OXC AST
**Confidence:** HIGH

## Summary

This phase completes the JSX transformation from React-style JSX syntax to Qwik's optimized `_jsxSorted` (and `_jsxSplit`) output format. The OXC implementation already has substantial JSX infrastructure in place from Phases 3-4, including `enter_jsx_element`, `exit_jsx_element`, `JsxState` tracking, and child handling. The research identified that the core work remaining is:

1. Ensuring proper prop categorization (var_props vs const_props)
2. Implementing the `_jsxSplit` path for spread props with runtime sorting
3. Proper Fragment handling with `_Fragment` import
4. Children array construction matching SWC's exact format
5. Flags calculation for static_subtree and static_listeners

The SWC reference implementation in `qwik-core/src/optimizer/core/src/transform.rs` provides the exact logic to port. The output format is well-defined: `_jsxSorted(type, varProps, constProps, children, flags, key)`.

**Primary recommendation:** Port the SWC `internal_handle_jsx_props_obj` logic for prop categorization and extend the existing OXC `exit_jsx_element` to generate the complete `_jsxSorted` call with proper arguments.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| OXC | 0.111.0 | AST parsing, traversal, codegen | Already used in project |
| oxc_ast | 0.111.0 | JSX AST types (JSXElement, JSXAttribute, etc.) | Provides JSX node types |
| oxc_traverse | 0.111.0 | Visitor pattern for AST transformation | Used for enter/exit hooks |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| oxc_allocator | 0.111.0 | Arena allocation for AST nodes | When creating new nodes |
| oxc_span | 0.111.0 | Source spans for nodes | For error reporting |

### Not Applicable
This phase does not introduce new dependencies - it extends the existing JSX handling already implemented in transform.rs.

## Architecture Patterns

### Current OXC JSX Infrastructure

The OXC implementation already has:

```rust
struct JsxState<'gen> {
    is_fn: bool,              // Component vs native element
    is_text_only: bool,       // For input, textarea, etc.
    is_segment: bool,         // Whether segment tracking is active
    should_runtime_sort: bool, // Triggers _jsxSplit vs _jsxSorted
    static_listeners: bool,    // Part of flags calculation
    static_subtree: bool,      // Part of flags calculation
    key_prop: Option<Expression<'gen>>,
    var_props: OxcVec<'gen, ObjectPropertyKind<'gen>>,
    const_props: OxcVec<'gen, ObjectPropertyKind<'gen>>,
    children: OxcVec<'gen, ArrayExpressionElement<'gen>>,
}
```

### _jsxSorted Output Format

The `_jsxSorted` function signature:
```javascript
_jsxSorted(type, varProps, constProps, children, flags, key)
```

Arguments:
1. **type**: String literal for native elements ("div"), identifier reference for components (Foo)
2. **varProps**: Object with dynamic props, or `null` if empty
3. **constProps**: Object with static props, or `null` if empty (or `_getConstProps(x)` for spread)
4. **children**: Array of children, single child expression, or `null`
5. **flags**: Number combining `static_subtree (bit 0)` and `static_listeners (bit 1)`
6. **key**: String key, or `null`

### Prop Categorization Pattern

From SWC `internal_handle_jsx_props_obj`:

```
Props are categorized as CONST if:
1. No spread props before them
2. AND they are const expressions (no function calls, member access, or local vars)

Props are categorized as VAR if:
1. Any spread prop appears before them
2. OR they access local/mutable variables
3. OR they call functions
4. OR they access object members
```

When spread props exist:
- All props before the spread go to var_props
- `should_runtime_sort = true` triggers `_jsxSplit` instead of `_jsxSorted`
- Spread becomes `_getVarProps(x)` in varProps and `_getConstProps(x)` in constProps

### Children Handling Pattern

Children are processed in `exit_jsx_child`:
- Text nodes: Trimmed, skip if empty
- Element children: Use `replace_expr` from nested JSX processing
- Expression containers: May need `_wrapProp` for signal values
- Fragments: Result in array of children

Single child optimization:
- If one child and it's not dynamic, pass directly instead of array

### Fragment Handling

Two fragment types:
1. **Explicit Fragment**: `<Fragment>` component - requires import, uses `_jsxSorted(Fragment, ...)`
2. **Implicit Fragment**: `<></>` syntax - uses `_jsxSorted(_Fragment, ...)` with jsx-runtime import

Fragment children become a flat array when used as a child of another element.

### Recommended Processing Order

1. **enter_jsx_element**: Initialize JsxState, push to jsx_stack
2. **enter_jsx_attribute**: Track attribute for later processing
3. **exit_jsx_attribute**: Categorize into var_props or const_props
4. **exit_jsx_child**: Build children array
5. **exit_jsx_element**: Generate `_jsxSorted` call, set replace_expr

### Anti-Patterns to Avoid

- **Processing props in enter_jsx_attribute:** Must wait for full prop value evaluation before categorization
- **Modifying node in-place prematurely:** Use replace_expr pattern, let exit_expression handle replacement
- **Forgetting key prop special handling:** key is extracted separately, not part of props
- **Children in both props AND JSX children:** If spread contains children, it must go to var_props

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Prop constness detection | Custom visitor | Port SWC's is_const_expr | Edge cases with imports, exports, const stack |
| Flags bit manipulation | Inline bit ops | Named constants and clear formula | Easy to get bits wrong |
| Key generation | Random keys | Hash-based `{hash}_{counter}` format | Must match SWC exactly for parity |
| Pure annotation | Custom comments | OXC's `expression_call_with_pure` | Handles span correctly |

**Key insight:** The SWC implementation is the authoritative source. Any deviation from its logic risks output parity failures.

## Common Pitfalls

### Pitfall 1: Spread Props Position Matters
**What goes wrong:** All props get categorized as const regardless of spread position
**Why it happens:** Not tracking spread count during prop iteration
**How to avoid:** Count spreads first, iterate with decrementing counter
**Warning signs:** `_jsxSorted` instead of `_jsxSplit` when spread present

### Pitfall 2: Children Array vs Single Child
**What goes wrong:** Single child wrapped in array unnecessarily, or array child passed as single
**Why it happens:** Inconsistent handling in convert_children
**How to avoid:** Match SWC's exact logic for array vs single detection
**Warning signs:** Snapshot test failures with extra array brackets

### Pitfall 3: Fragment vs _Fragment Import Mismatch
**What goes wrong:** Using wrong Fragment identifier or missing import
**Why it happens:** Explicit `Fragment` from @qwik.dev/core vs implicit `_Fragment` from jsx-runtime
**How to avoid:** Check JSXElement name, use correct import source
**Warning signs:** "Fragment is not defined" runtime errors

### Pitfall 4: Event Handler Props Going to const_props
**What goes wrong:** Event handlers (onClick$) categorized as const when they're QRL calls
**Why it happens:** Not detecting QRL calls as "dynamic"
**How to avoid:** Transform event handlers first, then categorize
**Warning signs:** Static listeners flag set incorrectly

### Pitfall 5: Key Prop Handling
**What goes wrong:** Key appears in both props object and as 6th argument
**Why it happens:** Not filtering key from props during iteration
**How to avoid:** Extract key separately, skip it in prop iteration
**Warning signs:** Duplicate key prop in output

### Pitfall 6: Immutable Props Check Missing
**What goes wrong:** Props that should be const are marked as var
**Why it happens:** Using different const_idents than SWC
**How to avoid:** Port decl_stack tracking, match IdentType::Var(true) logic
**Warning signs:** More var_props than expected, different flags

## Code Examples

### _jsxSorted Call Generation (from SWC)

```rust
// Source: qwik-core transform.rs lines 901-924
let (jsx_func, mut args) = if should_sort {
    (
        self.ensure_core_import(&_JSX_SPLIT),
        vec![node_type, var_props, const_props, children, flags, key],
    )
} else {
    (
        self.ensure_core_import(&_JSX_SORTED),
        vec![node_type, var_props, const_props, children, flags, key],
    )
};

ast::CallExpr {
    callee: ast::Callee::Expr(Box::new(ast::Expr::Ident(new_ident_from_id(&jsx_func)))),
    args,
    ..node
}
```

### Prop Categorization Logic (from SWC)

```rust
// Source: qwik-core transform.rs lines 1563-1570
// Do we have spread arguments?
let mut spread_props_count = props
    .iter()
    .filter(|prop| !matches!(prop, ast::PropOrSpread::Prop(_)))
    .count();

let has_spread_props = spread_props_count > 0;
let should_runtime_sort = has_spread_props;
```

### Flags Calculation

```rust
// Flags format: bit 0 = static_subtree, bit 1 = static_listeners
let flags = (if jsx.static_subtree { 0b1 } else { 0 })
          | (if jsx.static_listeners { 0b10 } else { 0 });
// Common values: 3 = both static, 1 = subtree only, 0 = neither
```

### Children Building

```javascript
// Single child (not array)
_jsxSorted("div", null, { class: "x" }, "Hello", 3, null)

// Multiple children (array)
_jsxSorted("div", null, null, [
    _jsxSorted("span", null, null, null, 3, null),
    "text"
], 1, null)

// Dynamic child (loses static flags)
_jsxSorted("div", null, null, someVar, 1, null)
```

### Fragment Output

```javascript
// Implicit fragment <></>
import { Fragment as _Fragment } from "@qwik.dev/core/jsx-runtime";
_jsxSorted(_Fragment, null, null, [child1, child2], 1, "u6_0")

// Explicit Fragment
import { Fragment } from '@qwik.dev/core';
_jsxSorted(Fragment, null, null, [child1, child2], 1, "u6_0")
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| createElement/h | _jsxSorted/_jsxSplit | Qwik 2.x | Optimized for Qwik runtime |
| Single props object | var_props + const_props | Qwik 2.x | Enables static analysis |
| jsx/jsxs | _jsxSorted | Qwik 2.x | Qwik-specific optimizations |

**Deprecated/outdated:**
- `jsx`, `jsxs`, `jsxDEV` from React: Replaced by `_jsxSorted`/_jsxSplit`
- `h` function: Legacy, use `_jsxSorted`

## Open Questions

None - the SWC implementation provides complete reference for all JSX patterns.

## Sources

### Primary (HIGH confidence)
- `/Users/jackshelton/dev/open-source/qwik-optimizer/qwik-core/src/optimizer/core/src/transform.rs` - Complete SWC reference implementation
- `/Users/jackshelton/dev/open-source/qwik-optimizer/optimizer/src/transform.rs` - Current OXC implementation with JsxState
- `/Users/jackshelton/dev/open-source/qwik-optimizer/qwik-core/src/optimizer/core/src/words.rs` - Qwik constant definitions
- `/Users/jackshelton/dev/open-source/qwik-optimizer/optimizer/src/component/shared.rs` - OXC shared constants

### Snapshot Tests (HIGH confidence - define exact expected output)
- `qwik_core__test__example_jsx.snap` - Basic JSX with fragments, spread
- `qwik_core__test__example_jsx_listeners.snap` - Event handlers in JSX
- `qwik_core__test__example_spread_jsx.snap` - Spread props with _getVarProps/_getConstProps
- `qwik_core__test__example_mutable_children.snap` - Conditional rendering, ternary

## Requirements Mapping

| Requirement | SWC Reference | OXC Status | Notes |
|-------------|---------------|------------|-------|
| JSX-01: Basic JSX element | `fold_jsx_element` | Partial | Structure exists, needs completion |
| JSX-02: Dynamic children | `convert_children` | Partial | exit_jsx_child exists |
| JSX-03: Fragment handling | `fragment_fn` | Missing | Need _Fragment import |
| JSX-04: Spread attributes | `exit_jsx_spread_attribute` | Exists | Needs _getVarProps/_getConstProps |
| JSX-05: Conditional rendering | `convert_children` | Missing | Need ternary handling |
| JSX-06: List rendering | Child array | Partial | Map expressions need handling |
| JSX-07: _jsxSorted format | `exit_jsx_element` | Partial | 6 args structure exists |
| JSX-08: Immutable props | `is_const_expr` | Missing | Need ConstCollector port |

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Using existing OXC 0.111.0 patterns
- Architecture: HIGH - SWC reference is authoritative
- Pitfalls: HIGH - Identified from SWC code and snapshot tests

**Research date:** 2026-01-29
**Valid until:** Indefinite - Qwik 2.x API is stable

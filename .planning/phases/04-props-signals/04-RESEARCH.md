# Phase 4: Props & Signals - Research

**Researched:** 2026-01-29
**Domain:** Component props handling and signal/store reactivity transformation
**Confidence:** HIGH

## Summary

Phase 4 implements two interconnected features: (1) component props handling with destructuring reconstruction, and (2) signal/store access transformation for reactive data flow. Both features build on Phase 2/3 infrastructure but require significant new transformation logic.

The SWC reference implementation handles props in `props_destructuring.rs` (a dedicated module) and signals in `transform.rs` via `_fnSignal` and `_wrapProp` helpers. The key insight is that props destructuring must be **reconstructed** back to property access on a `_rawProps` parameter, while signals require **wrapping** in `_wrapProp` or `_fnSignal` based on their usage pattern.

The current OXC implementation has partial JSX transformation but lacks props destructuring reconstruction and signal wrapping. Phase 4 adds: (1) props destructuring detection and reconstruction to `_rawProps.propName` access, (2) `_restProps` for rest patterns, (3) `_wrapProp` for direct signal access, (4) `_fnSignal` for computed signal expressions, and (5) bind:value/bind:checked directive transformation to value/checked props + on:input handlers.

**Primary recommendation:** Implement props destructuring as a pre-pass visitor (similar to SWC's `PropsDestructuring` struct), then integrate signal wrapping into the existing JSX attribute transformation. The bind directive transformation should extend the existing event handler infrastructure from Phase 3.

## Standard Stack

The established components for this domain:

### Core Components (From Phases 2-3 - Already Implemented)
| Component | File | Purpose | Status |
|-----------|------|---------|--------|
| TransformGenerator | `/optimizer/src/transform.rs` | Main AST traversal | Implemented |
| Qrl | `/optimizer/src/component/qrl.rs` | QRL expression generation | Implemented |
| compute_scoped_idents | `/optimizer/src/transform.rs` | Variable capture detection | Implemented |
| Import | `/optimizer/src/component/shared.rs` | Import management | Implemented |
| jsx_element_is_native | `/optimizer/src/transform.rs:193` | Native element detection stack | Implemented |

### New Components Needed (Phase 4)
| Component | Purpose | SWC Reference |
|-----------|---------|---------------|
| PropsDestructuring | Reconstruct destructured props to property access | props_destructuring.rs:13-31 |
| transform_component_props | Convert `({prop1, prop2})` to `(_rawProps)` | props_destructuring.rs:56-82 |
| convert_inlined_fn | Generate `_fnSignal` for computed expressions | inlined_fn.rs:24-115 |
| is_bind_prop | Detect bind:value/bind:checked directives | transform.rs:1251-1253 |
| transform_jsx_prop (bind:) | Transform bind directives to value+on:input | transform.rs:1162-1243 |

### SWC Reference Locations
| Function | File:Line | Purpose |
|----------|-----------|---------|
| transform_props_destructuring | props_destructuring.rs:20-31 | Entry point for props transformation |
| transform_component_props | props_destructuring.rs:56-82 | Handle arrow function params |
| transform_component_body | props_destructuring.rs:83-247 | Transform variable declarations |
| transform_pat | props_destructuring.rs:312-430 | Handle destructuring patterns |
| transform_rest | props_destructuring.rs:432-474 | Handle rest props |
| convert_inlined_fn | inlined_fn.rs:24-115 | Generate _fnSignal calls |
| is_bind_prop | transform.rs:1251-1253 | Detect bind: directives |
| transform_jsx_prop | transform.rs:1131-1249 | Handle bind:value/checked |

## Architecture Patterns

### Pattern 1: Props Destructuring Reconstruction

**What:** Transform destructured component parameters back to property access

**When to use:** Every component$ arrow function with destructured props

**Transformation Rules (from SWC props_destructuring.rs:56-82):**

```
INPUT:  component$(({ message, id, count: c, ...rest }) => { ... })
OUTPUT: component$((_rawProps) => {
          const rest = _restProps(_rawProps, ["message", "id", "count"]);
          // Uses of 'message' become _rawProps.message (via _wrapProp in JSX)
          // Uses of 'id' become _rawProps.id
          // Uses of 'c' become _rawProps.count
          ...
        })
```

**Key insight:** The transformation does NOT simply rename variables. It:
1. Replaces destructured param with `_rawProps` identifier
2. Generates `_restProps` call if rest pattern exists
3. Tracks identifiers for later replacement with `_wrapProp` or `_fnSignal`

**OXC Implementation Approach:**
```rust
// Pre-pass visitor that runs before main transformation
struct PropsDestructuring {
    component_ident: Option<Id>,  // Track component$ identifier
    identifiers: HashMap<Id, Expression>,  // Map old idents to new access
    raw_props_param: Option<String>,  // Name of the _rawProps parameter
}

impl PropsDestructuring {
    fn transform_component_props(&mut self, arrow: &mut ArrowFunctionExpression) {
        // 1. Check if first param is ObjectPattern
        // 2. Create _rawProps identifier
        // 3. Build identifiers map: old_id -> MemberExpression(_rawProps, "old_name")
        // 4. Handle rest pattern if present
        // 5. Replace param with _rawProps BindingIdentifier
    }
}
```

### Pattern 2: Signal Wrapping with _wrapProp

**What:** Wrap signal/store access in JSX with `_wrapProp` for reactive updates

**When to use:** When a signal/store is used directly as a JSX prop value

**Transformation Rules (from example_props_wrapping.snap):**

```
INPUT:  <div props-wrap={fromProps} />
OUTPUT: <div props-wrap={_wrapProp(_rawProps, "fromProps")} />

INPUT:  <div local={fromLocal} />  // fromLocal is a signal
OUTPUT: <div local={fromLocal} />  // No wrapping - direct signal reference
```

**When _wrapProp is used:**
- Accessing a prop from `_rawProps` in JSX context
- The value is read (not computed/transformed)

**Example (from destructure_args_colon_props.snap):**
```javascript
// INPUT:
const { 'bind:value': bindValue } = props;
return <>{bindValue}</>

// OUTPUT:
return _jsxSorted(_Fragment, null, null, _wrapProp(props, "bind:value"), 1, "u6_0");
```

### Pattern 3: Computed Signal Expressions with _fnSignal

**What:** Wrap computed expressions involving signals in `_fnSignal` for reactive evaluation

**When to use:** When signal/store values are used in computations

**Transformation Rules (from inlined_fn.rs:24-115):**

```
INPUT:  <div computed={fromLocal + fromProps} />
OUTPUT: <div computed={_fnSignal(_hf0, [_rawProps, fromLocal], _hf0_str)} />
        // Where _hf0 = (p0, p1) => p1 + p0.fromProps
        // And _hf0_str = "p1+p0.fromProps"
```

**_fnSignal Generation Rules:**
1. Create a hoisted arrow function with positional params (p0, p1, ...)
2. Replace scoped identifiers with their positional param
3. Generate stringified version of the expression
4. Create _fnSignal call with: function, capture array, string representation

**Skip _fnSignal when:**
- Expression is an arrow function (return None, is_const)
- Expression calls a function (can't serialize)
- Rendered expression > 150 chars (too complex)
- No scoped identifiers (just return None, true for const)

### Pattern 4: bind:value and bind:checked Directives

**What:** Transform bind directives into value/checked prop + on:input handler

**When to use:** JSX elements with `bind:value={signal}` or `bind:checked={signal}`

**Transformation Rules (from transform.rs:1162-1243):**

```
INPUT:  <input bind:value={localValue} />
OUTPUT: <input value={localValue} on:input={inlinedQrl(_val, "_val", [localValue])} />

INPUT:  <input bind:checked={localValue} />
OUTPUT: <input checked={localValue} on:input={inlinedQrl(_chk, "_chk", [localValue])} />
```

**Key implementation details:**
1. Detect bind:value or bind:checked attribute
2. Add value/checked prop with the signal value
3. Create `inlinedQrl` call with `_val` or `_chk` helper
4. Add/merge on:input handler
5. Skip the bind: prop itself (it's been transformed)

**Merging with existing onInput$:**
```javascript
// INPUT:
<input onInput$={() => console.log("test")} bind:value={localValue} />

// OUTPUT:
<input
  value={localValue}
  on:input={[
    qrl(i_hash, "handler_name"),  // Original handler
    inlinedQrl(_val, "_val", [localValue])  // bind:value handler
  ]}
/>
```

### Pattern 5: Rest Props with _restProps

**What:** Handle `...rest` patterns in component props

**When to use:** Components with rest pattern: `({ id, ...rest }) => ...`

**Transformation Rules (from props_destructuring.rs:432-474):**

```
INPUT:  component$(({ message, id, ...rest }) => <span {...rest}>{message}</span>)
OUTPUT: component$((_rawProps) => {
          const rest = _restProps(_rawProps, ["message", "id"]);
          return <span {...rest}>{_wrapProp(_rawProps, "message")}</span>
        })
```

**Implementation:**
1. Detect RestElement in ObjectPattern
2. Collect all other property names as "omit" list
3. Generate: `const rest = _restProps(_rawProps, ["prop1", "prop2", ...])`
4. Insert statement at beginning of function body

### Pattern 6: Props with Default Values

**What:** Handle destructured props with default values

**When to use:** Props like `{ count = 0 }` or `{ label: name = "default" }`

**Transformation Rules (from props_destructuring.rs:323-346):**

```
INPUT:  ({ count = 0 }) => ...
OUTPUT: (_rawProps) => ...
        // Where count access becomes: _rawProps.count ?? 0
```

**Implementation:**
- Generate binary expression with NullishCoalescing operator
- Left side: property access (_rawProps.count)
- Right side: default value

### Anti-Patterns to Avoid

- **Wrapping non-prop signals:** `useSignal()` return values are passed directly, not wrapped
- **Double-wrapping computed values:** Check if already in _fnSignal before transforming
- **Incorrect bind: prop detection:** Must match exactly "bind:value" or "bind:checked"
- **Forgetting to skip bind: prop:** After transformation, the original bind: attribute must not be added

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Props reconstruction | Custom identifier replacement | Follow SWC's PropsDestructuring pattern | Handles edge cases (rest, defaults, aliasing) |
| Signal detection | Simple identifier check | Check usage context (member access = needs wrap) | Usage determines wrapping, not type |
| Expression stringification | Manual string building | OXC Codegen with minify options | Handles all expression types correctly |
| Bind handler creation | Custom QRL | inlinedQrl pattern | Matches existing runtime expectations |

**Key insight:** Props and signals transformation is **context-dependent**. A variable `count` might need `_wrapProp` in one place and `_fnSignal` in another depending on how it's used.

## Common Pitfalls

### Pitfall 1: Mixing Var and Const Props Incorrectly
**What goes wrong:** Props that should be in varProps end up in constProps (or vice versa)
**Why it happens:** Not correctly detecting signal/store reactivity
**How to avoid:** Follow SWC's is_const_expr checks; reactive values go to varProps
**Warning signs:** Runtime errors about undefined or stale values

### Pitfall 2: Not Handling Aliased Props
**What goes wrong:** `{ count: c }` doesn't get transformed correctly
**Why it happens:** Using the local name `c` instead of the prop name `count` for access
**How to avoid:** Track both local and prop names in identifiers map
**Warning signs:** Props aliasing fails in output

### Pitfall 3: Forgetting _restProps Import
**What goes wrong:** Rest props transformation generates code but import is missing
**Why it happens:** Not adding import when _restProps is used
**How to avoid:** Always add import when generating _restProps call
**Warning signs:** Runtime error "_restProps is not defined"

### Pitfall 4: Double Handler Array for bind:
**What goes wrong:** `on:input` becomes nested array `[[handler1, handler2]]`
**Why it happens:** Wrapping already-array handler in another array
**How to avoid:** Use merge_or_add_event_handler pattern from SWC
**Warning signs:** Event handlers don't fire correctly

### Pitfall 5: Incorrect _fnSignal Param Order
**What goes wrong:** Generated function references wrong parameter indices
**Why it happens:** Mismatch between scoped_idents order and param generation
**How to avoid:** Enumerate scoped_idents consistently; p0 = first ident, p1 = second, etc.
**Warning signs:** Wrong values passed to computed expressions

### Pitfall 6: Transforming Non-Component Arrow Functions
**What goes wrong:** Regular arrow functions get props destructuring treatment
**Why it happens:** Not checking if arrow is component$ argument
**How to avoid:** Track component$ identifier and only transform its direct argument
**Warning signs:** Non-component functions have wrong parameter names

## Code Examples

### Props Destructuring Detection (OXC Pattern)

```rust
// In enter_call_expression or similar:
fn detect_component_call(&self, call_expr: &CallExpression) -> bool {
    match &call_expr.callee {
        Expression::Identifier(ident) => {
            // Check if this is component$ (with possible alias)
            ident.name.ends_with('$') &&
            ident.name.starts_with("component")
        }
        _ => false
    }
}

fn should_transform_props(&self, arrow: &ArrowFunctionExpression) -> bool {
    // Check if first param is ObjectPattern
    matches!(
        arrow.params.items.first(),
        Some(FormalParameter { pattern: BindingPattern::ObjectPattern(_), .. })
    )
}
```

### _wrapProp Generation (OXC Pattern)

```rust
fn create_wrap_prop_call<'a>(
    &self,
    builder: &AstBuilder<'a>,
    props_ident: &str,
    prop_name: &str,
) -> Expression<'a> {
    // _wrapProp(_rawProps, "propName")
    builder.expression_call(
        SPAN,
        builder.expression_identifier_reference(SPAN, "_wrapProp"),
        NONE,
        builder.vec_from_array([
            Argument::SpreadElement(builder.alloc(SpreadElement {
                span: SPAN,
                argument: builder.expression_identifier_reference(SPAN, props_ident),
            })),
            Argument::SpreadElement(builder.alloc(SpreadElement {
                span: SPAN,
                argument: builder.expression_string_literal(SPAN, prop_name),
            })),
        ]),
        false,
    )
}
```

### bind:value Transformation (from SWC)

```rust
// Source: SWC transform.rs:1162-1243
fn transform_bind_directive(
    &mut self,
    is_checked: bool,
    signal_expr: Expression,
    props: &mut Vec<ObjectPropertyKind>,
) {
    // 1. Add value/checked prop
    let value_key = if is_checked { "checked" } else { "value" };
    let value_prop = ObjectPropertyKind::ObjectProperty(/* ... */);
    props.push(value_prop);

    // 2. Create inlinedQrl for handler
    let handler_fn = if is_checked { "_chk" } else { "_val" };
    let handler_qrl = self.create_inlined_qrl(
        handler_fn,
        handler_fn,
        vec![signal_expr.clone()],
    );

    // 3. Merge or add on:input handler
    self.merge_or_add_event_handler(props, "on:input", handler_qrl);
}
```

### _fnSignal Generation Pattern

```rust
// Generate: _fnSignal((p0, p1) => expr, [ident1, ident2], "expr_str")
fn create_fn_signal_call<'a>(
    &self,
    builder: &AstBuilder<'a>,
    hoisted_fn_name: &str,
    scoped_idents: Vec<Id>,
    expr_string: &str,
) -> Expression<'a> {
    // Build array of captured identifiers
    let captures: Vec<_> = scoped_idents.iter()
        .map(|id| builder.expression_identifier_reference(SPAN, &id.0))
        .map(|e| ArrayExpressionElement::SpreadElement(/* ... */))
        .collect();

    builder.expression_call(
        SPAN,
        builder.expression_identifier_reference(SPAN, "_fnSignal"),
        NONE,
        builder.vec_from_array([
            Argument::from(builder.expression_identifier_reference(SPAN, hoisted_fn_name)),
            Argument::from(builder.expression_array(SPAN, captures, None)),
            Argument::from(builder.expression_string_literal(SPAN, expr_string)),
        ]),
        false,
    )
}
```

## State of the Art

| Capability | OXC Status | What's Needed |
|------------|------------|---------------|
| Props destructuring detection | Not implemented | New: PropsDestructuring visitor |
| _rawProps parameter | Not implemented | Param replacement in arrow functions |
| _restProps call | Not implemented | Rest pattern handling |
| _wrapProp call | Not implemented | JSX prop wrapping for signals |
| _fnSignal call | Not implemented | Computed expression wrapping |
| bind:value/checked | Not implemented | Directive transformation |
| Signal detection | Partial (via is_const) | Extend to member access patterns |
| Hoisted functions | Not implemented | Top-level function generation |

## Open Questions

1. **Hoisted function naming convention**
   - What we know: SWC uses `_hf0`, `_hf1`, etc. for hoisted functions
   - What's unclear: Whether index should reset per file or be globally unique
   - Recommendation: Per-file counter matching SWC behavior

2. **Expression length limit for _fnSignal**
   - What we know: SWC skips _fnSignal if rendered expression > 150 chars
   - What's unclear: Whether this is a hard limit or configurable
   - Recommendation: Match SWC's 150 char limit exactly

3. **Props destructuring ordering**
   - What we know: SWC runs props_destructuring as separate visitor pass
   - What's unclear: Exact ordering relative to other transformations
   - Recommendation: Run before main QRL transformation pass

4. **Const vs. var props classification edge cases**
   - What we know: SWC uses complex is_const_expr checks
   - What's unclear: All edge cases for const classification
   - Recommendation: Start with clear cases, add edge cases based on test failures

## Sources

### Primary (HIGH confidence)
- `/qwik-core/src/optimizer/core/src/props_destructuring.rs` - Complete props reconstruction logic (verified)
- `/qwik-core/src/optimizer/core/src/inlined_fn.rs` - _fnSignal generation (verified)
- `/qwik-core/src/optimizer/core/src/transform.rs:1131-1253` - bind directive handling (verified)
- `/qwik-core/src/optimizer/core/src/words.rs` - All constant definitions (verified)
- Snapshot tests: `should_destructure_args`, `example_props_wrapping`, `should_convert_rest_props`, `example_input_bind`, `should_merge_bind_value_and_on_input` (verified)

### Secondary (MEDIUM confidence)
- `/optimizer/src/transform.rs` - Current OXC implementation structure (verified)
- Phase 3 research - Event handler infrastructure patterns (verified)

## Metadata

**Confidence breakdown:**
- Props destructuring: HIGH - Complete SWC implementation examined
- Signal wrapping: HIGH - _fnSignal and _wrapProp patterns verified
- bind directives: HIGH - Transformation logic verified with snapshots
- Integration approach: MEDIUM - Need to determine exact OXC visitor pattern

**Research date:** 2026-01-29
**Valid until:** 2026-03-01 (SWC implementation is stable reference)

## Requirements Mapping

| Requirement | SWC Implementation | OXC Status | Notes |
|-------------|-------------------|------------|-------|
| PRP-01: Props destructuring | transform_component_props | Not implemented | Core destructuring transformation |
| PRP-02: Props spread | _jsxSplit with _getVarProps | Not implemented | Spread props in JSX |
| PRP-03: Immutable props | is_const_expr checks | Partial (jsx const tracking) | Const vs var classification |
| PRP-04: bind:value | transform_jsx_prop | Not implemented | Value + on:input generation |
| PRP-05: bind:checked | transform_jsx_prop | Not implemented | Checked + on:input generation |
| PRP-06: Props in vars | transform_component_body | Not implemented | Local const with prop value |
| PRP-07: Destructured args | transform_pat | Not implemented | Pattern reconstruction |
| PRP-08: Props with defaults | NullishCoalescing generation | Not implemented | Default value handling |
| PRP-09: Rest props | transform_rest + _restProps | Not implemented | Rest pattern handling |
| PRP-10: Props aliasing | transform_pat with key/value | Not implemented | Aliased prop names |
| SIG-01: useSignal$ | No special handling needed | N/A | Just capture detection |
| SIG-02: useStore$ | No special handling needed | N/A | Just capture detection |
| SIG-03: useComputed$ | No special handling needed | N/A | Just capture detection |
| SIG-04: Signal in JSX | _wrapProp + _fnSignal | Not implemented | Context-dependent wrapping |
| SIG-05: Store mutations | No special handling | N/A | Pass-through |
| SIG-06: Derived signals | _fnSignal for computed | Not implemented | Computed expression wrapping |

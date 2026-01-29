# Phase 3: Event Handlers - Research

**Researched:** 2026-01-29
**Domain:** JSX event handler transformation (onClick$, onInput$, etc.)
**Confidence:** HIGH

## Summary

Phase 3 implements event handler transformation, building directly on Phase 2's QRL core. The key difference from Phase 2 is that event handlers (1) appear as JSX attributes rather than standalone call expressions, (2) require attribute name transformation (`onClick$` -> `on:click`), and (3) must detect whether they're on native elements vs. component elements (different handling rules).

The SWC reference implementation in `qwik-core/src/optimizer/core/src/transform.rs` provides complete specification for event handler transformation via `jsx_event_to_html_attribute()` (line 3347), `handle_jsx_value()` (line 927), and `fold_jsx_attr()` (line 3037). The existing snapshot tests (`should_convert_jsx_events.snap`, `should_not_transform_events_on_non_elements.snap`, etc.) serve as ground truth.

Phase 2 already established the QRL transformation infrastructure including `Qrl`, `SegmentData`, `SegmentKind::EventHandler`, and capture detection. This phase extends that infrastructure to handle JSX attributes specifically, with the event name transformation and native vs. component element distinction.

**Primary recommendation:** Extend JSX attribute handling in `exit_jsx_attribute` to detect event handler patterns (`onClick$`, `onInput$`, etc.), apply QRL transformation to the handler function, and transform the attribute name to `on:*` format on native elements only.

## Standard Stack

The established components for this domain (extends Phase 2):

### Core Components (Phase 2 - Already Implemented)
| Component | File | Purpose | Status |
|-----------|------|---------|--------|
| Qrl | `/optimizer/src/component/qrl.rs` | QRL expression generation | Implemented |
| SegmentKind::EventHandler | `/optimizer/src/component/segment_data.rs:24` | Event handler type marker | Implemented |
| SegmentData | `/optimizer/src/component/segment_data.rs` | Complete segment metadata | Implemented |
| IdentCollector | `/optimizer/src/collector.rs` | Variable usage collection | Implemented |
| compute_scoped_idents | `/optimizer/src/transform.rs:1392-1413` | Lexical scope capture | Implemented |
| TransformGenerator | `/optimizer/src/transform.rs` | Main traverse implementation | Partial (JSX needs extension) |

### New Components Needed (Phase 3)
| Component | Purpose | SWC Reference |
|-----------|---------|---------------|
| jsx_event_to_html_attribute | Convert `onClick$` -> `on:click` | transform.rs:3347-3372 |
| get_event_scope_data | Extract prefix (on:, on-document:, on-window:) | transform.rs:3375-3385 |
| is_native_element detection | Distinguish `<div>` from `<CustomComponent>` | fold_jsx_attr:3043 |
| handle_jsx_value | Transform event handler to QRL in JSX context | transform.rs:927-959 |

### SWC Reference Locations
| Function | File:Line | Purpose |
|----------|-----------|---------|
| jsx_event_to_html_attribute | transform.rs:3347-3372 | Event name transformation |
| get_event_scope_data_from_jsx_event | transform.rs:3375-3385 | Extract scope prefix |
| create_event_name | transform.rs:3389-3402 | camelCase to kebab-case |
| handle_jsx_value | transform.rs:927-959 | QRL wrapping for JSX values |
| fold_jsx_attr | transform.rs:3037-3098 | JSX attribute transformation |
| jsx_element_is_native | transform.rs:124 | Native element tracking stack |

## Architecture Patterns

### Pattern 1: Event Name Transformation

**What:** Transform Qwik event names to HTML attribute format

**When to use:** Every event handler attribute on native elements

**Transformation Rules (from SWC transform.rs:3347-3402):**

```
onClick$          -> on:click
onDblClick$       -> on:dblclick
onInput$          -> on:input
onKeyDown$        -> on:keydown
document:onFocus$ -> on-document:focus
window:onClick$   -> on-window:click
on-cLick$         -> on:c-lick        (- prefix preserves case)
```

**Algorithm:**
1. Check if attribute ends with `$`
2. Extract prefix: `window:on` -> `on-window:`, `document:on` -> `on-document:`, `on` -> `on:`
3. Extract event name after prefix
4. Convert camelCase to kebab-case (unless `-` prefix indicates preserve-case)
5. Combine prefix + transformed name

**Example (SWC transform.rs:3347-3372):**
```rust
fn jsx_event_to_html_attribute(jsx_event: &str) -> Option<Atom> {
    if !jsx_event.ends_with('$') {
        return None;
    }
    let (prefix, idx) = get_event_scope_data_from_jsx_event(jsx_event);
    if idx == usize::MAX {
        return None;
    }
    let name = &jsx_event[idx..jsx_event.len() - 1]; // Strip '$'

    // Handle case-sensitive marker '-'
    let processed_name = if let Some(stripped) = name.strip_prefix('-') {
        stripped.to_string()  // Preserve case
    } else {
        name.to_lowercase()   // Convert to lowercase
    };

    Some(create_event_name(&processed_name, prefix))
}
```

### Pattern 2: Native vs. Component Element Detection

**What:** Determine if a JSX element is a native HTML element or a component

**When to use:** Event name transformation only applies to native elements

**Rule (from SWC fold_jsx_attr:3043):**
- Native elements: First character is lowercase (`<div>`, `<button>`)
- Components: First character is uppercase (`<CustomButton>`, `<Header>`)

**Example:**
```jsx
// Native element - TRANSFORM attribute name
<button onClick$={() => {}}/>
// Output: <button on:click={qrl(...)}/>

// Component - DON'T transform attribute name
<CustomButton onClick$={() => {}}/>
// Output: <CustomButton onClick$={qrl(...)}/>
```

**Implementation (track via stack during traversal):**
```rust
// In enter_jsx_element:
let is_native = matches!(node.opening_element.name,
    JSXElementName::Identifier(id) if id.name.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false)
);
self.jsx_element_is_native.push(is_native);

// In exit_jsx_element:
self.jsx_element_is_native.pop();
```

### Pattern 3: Event Handler QRL Transformation

**What:** Transform function expression in event attribute to QRL call

**When to use:** When JSX attribute value is a function (`() => ...`) and attribute name ends with `$`

**Flow (from SWC handle_jsx_value:927-959):**
```
INPUT:  <button onClick$={() => count.value++}>
OUTPUT: <button on:click={qrl(() => import("./file_hash"), "name", [count])}>
```

**Key difference from Phase 2:** The QRL transformation happens in JSX attribute context, not as a standalone call expression.

**Implementation pattern:**
```rust
// In exit_jsx_attribute:
if let Some(JSXAttributeValue::ExpressionContainer(container)) = &mut node.value {
    if let JSXExpression::Expr(expr) = &container.expr {
        let is_fn = matches!(**expr, Expression::ArrowFunctionExpression(_) | Expression::Function(_));
        let is_event = node.name.get_identifier_name()
            .map(|n| n.ends_with('$'))
            .unwrap_or(false);

        if is_fn && is_event {
            // Apply QRL transformation using existing infrastructure
            // Same as exit_call_expression but for JSX context
        }
    }
}
```

### Pattern 4: Multiple Event Handlers on Same Element

**What:** Handle elements with multiple event handler attributes

**When to use:** Elements like `<button onClick$={...} onMouseOver$={...}>`

**Each handler is independent:**
- Each gets its own QRL segment
- Each gets unique display name (suffixed with `_1`, `_2` if same event type)
- Order preserved in output

**Example (from should_transform_multiple_event_handlers.snap):**
```jsx
// INPUT:
<button
    onClick$={() => {}}
    onMouseOver$={() => {}}
>

// OUTPUT:
<button
    on:click={qrl(i_hash1, "Foo_component_div_button_on_click_hash1")}
    on:mouseover={qrl(i_hash2, "Foo_component_div_button_on_mouseover_hash2")}
>
```

### Pattern 5: Event Handlers with Captured State

**What:** Event handlers that reference variables from enclosing scope

**When to use:** Handlers that access component state, props, or other outer variables

**This is already handled by Phase 2's capture detection:**
```jsx
// INPUT:
const count = useSignal(0);
return <button onClick$={() => count.value++}>

// OUTPUT (segment file):
import { useLexicalScope } from "@qwik.dev/core";
export const Counter_onClick_hash = () => {
    const [count] = useLexicalScope();
    count.value++;
};

// OUTPUT (qrl call):
qrl(() => import("./file_hash"), "name_hash", [count])
```

### Anti-Patterns to Avoid

- **Transforming event names on components:** Only native elements get `on:*` transformation
- **Missing the `$` suffix check:** Only `onClick$` transforms, not `onClick`
- **Forgetting scope prefixes:** `document:onClick$` and `window:onClick$` need special handling
- **Case sensitivity issues:** `-onClick$` preserves case, `onClick$` lowercases

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| QRL generation | New logic | Existing `Qrl::into_call_expression` | Already handles all QRL types |
| Capture detection | New collector | Existing `IdentCollector` + `compute_scoped_idents` | Phase 2 already works |
| Segment metadata | New struct | Existing `SegmentData::new` | Already has `SegmentKind::EventHandler` |
| Display name | Manual concatenation | Existing `current_display_name()` | Segment stack already tracked |
| Hash generation | Custom hasher | Existing `current_hash()` | Matches SWC output |

**Key insight:** The event handler transformation is essentially "QRL transformation triggered by JSX attribute instead of call expression". All the heavy lifting was done in Phase 2.

## Common Pitfalls

### Pitfall 1: Transforming Events on Components
**What goes wrong:** `<MyButton onClick$={...}>` gets `on:click` instead of `onClick$`
**Why it happens:** Not checking if element is native before transforming
**How to avoid:** Track `jsx_element_is_native` stack during traversal
**Warning signs:** Component event handlers have wrong attribute names in output

### Pitfall 2: Document/Window Prefix Handling
**What goes wrong:** `document:onFocus$` becomes `on:focus` instead of `on-document:focus`
**Why it happens:** Missing prefix extraction logic
**How to avoid:** Use `get_event_scope_data_from_jsx_event` pattern
**Warning signs:** Scope-prefixed events lose their scope in output

### Pitfall 3: Case-Sensitive Events
**What goes wrong:** `on-cLick$` becomes `on:click` instead of `on:c-lick`
**Why it happens:** Always lowercasing without checking `-` prefix
**How to avoid:** Check for leading `-` marker before case conversion
**Warning signs:** Case-sensitive events lose their casing

### Pitfall 4: Non-Function Values
**What goes wrong:** Trying to QRL-transform `onClick$={handler}` (identifier reference)
**Why it happens:** Only inline functions need transformation
**How to avoid:** Check if value is arrow/function expression before transforming
**Warning signs:** Identifier references get wrapped incorrectly

### Pitfall 5: JSX Without Transpilation Mode
**What goes wrong:** Event names transform but JSX doesn't become `_jsxSorted`
**Why it happens:** Event transform must work both with and without `transpile_jsx`
**How to avoid:** Event name transform is independent of JSX transpilation
**Warning signs:** Half-transformed output when `transpile_jsx: false`

### Pitfall 6: Non-Element Nodes
**What goes wrong:** Attempting to transform events on `<>{...}</>` or text nodes
**Why it happens:** Not checking node type before transformation
**How to avoid:** Only JSXElement nodes can have event attributes
**Warning signs:** Crashes or incorrect output on fragments

## Code Examples

### Event Name Transformation (from SWC)

```rust
// Source: SWC transform.rs:3375-3385
fn get_event_scope_data_from_jsx_event(jsx_event: &str) -> (&str, usize) {
    if jsx_event.starts_with("window:on") {
        ("on-window:", 9)
    } else if jsx_event.starts_with("document:on") {
        ("on-document:", 11)
    } else if jsx_event.starts_with("on") {
        ("on:", 2)
    } else {
        ("", usize::MAX)  // Not an event
    }
}

// Source: SWC transform.rs:3389-3402
fn create_event_name(name: &str, prefix: &str) -> Atom {
    let mut result = String::from(prefix);
    for c in name.chars() {
        if c.is_ascii_uppercase() || c == '-' {
            result.push('-');
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    Atom::from(result)
}
```

### JSX Attribute Transformation (from SWC)

```rust
// Source: SWC transform.rs:3037-3098 (simplified)
fn fold_jsx_attr(&mut self, node: ast::JSXAttr) -> ast::JSXAttr {
    match node.name {
        ast::JSXAttrName::Ident(ref ident) => {
            let new_word = convert_qrl_word(&ident.sym);  // component$ -> componentQrl

            // Transform event names only on native HTML elements
            let is_native_element = self.jsx_element_is_native.last().copied().unwrap_or(false);
            let transformed_name = if is_native_element {
                jsx_event_to_html_attribute(&ident.sym)
            } else {
                None
            };

            // Push to context stack for display name
            self.stack_ctxt.push(
                transformed_name.as_ref()
                    .unwrap_or(&ident.sym)
                    .to_string()
            );

            if new_word.is_some() {
                ast::JSXAttr {
                    value: self.handle_jsx_value(ident.sym.clone(), node.value),
                    name: transformed_name
                        .map(|n| ast::JSXAttrName::Ident(ast::IdentName { sym: n, span: ident.span }))
                        .unwrap_or(node.name.clone()),
                    ..node
                }
            } else if let Some(name) = transformed_name {
                ast::JSXAttr {
                    name: ast::JSXAttrName::Ident(ast::IdentName { sym: name, span: ident.span }),
                    ..node
                }
            } else {
                node
            }
        }
        // Handle namespaced names (host:onClick$, etc.)
        ast::JSXAttrName::JSXNamespacedName(ref namespaced) => {
            // Similar logic for namespaced attributes
        }
    }
}
```

### Handle JSX Value (from SWC)

```rust
// Source: SWC transform.rs:927-959
fn handle_jsx_value(
    &mut self,
    ctx_name: Atom,
    value: Option<ast::JSXAttrValue>,
) -> Option<ast::JSXAttrValue> {
    if let Some(ast::JSXAttrValue::JSXExprContainer(container)) = value {
        if let ast::JSXExpr::Expr(expr) = container.expr {
            let is_fn = matches!(*expr, ast::Expr::Arrow(_) | ast::Expr::Fn(_));
            if is_fn {
                // Determine segment kind
                let segment_kind = if jsx_event_to_html_attribute(&ctx_name).is_some() {
                    SegmentKind::EventHandler
                } else {
                    SegmentKind::JSXProp
                };

                Some(ast::JSXAttrValue::JSXExprContainer(ast::JSXExprContainer {
                    span: DUMMY_SP,
                    expr: ast::JSXExpr::Expr(Box::new(ast::Expr::Call(
                        self.create_synthetic_qsegment(*expr, segment_kind, ctx_name, None),
                    ))),
                }))
            } else {
                // Non-function value - pass through
                Some(ast::JSXAttrValue::JSXExprContainer(container))
            }
        } else {
            Some(ast::JSXAttrValue::JSXExprContainer(container))
        }
    } else {
        value
    }
}
```

### Expected Output Format

```javascript
// INPUT:
import { component$, useSignal } from '@qwik.dev/core';

export const Counter = component$(() => {
    const count = useSignal(0);
    return (
        <button onClick$={() => count.value++}>
            {count.value}
        </button>
    );
});

// OUTPUT (main file):
import { componentQrl, qrl } from "@qwik.dev/core";
const i_hash1 = () => import("./file_Counter_component_hash1");
export const Counter = componentQrl(qrl(i_hash1, "Counter_component_hash1"));

// OUTPUT (component segment):
import { _jsxSorted, useSignal, qrl } from "@qwik.dev/core";
const i_hash2 = () => import("./file_Counter_component_button_on_click_hash2");
export const Counter_component_hash1 = () => {
    const count = useSignal(0);
    return _jsxSorted("button", {
        "on:click": qrl(i_hash2, "Counter_component_button_on_click_hash2", [count])
    }, {}, [count.value], 3, null);
};

// OUTPUT (event handler segment):
import { useLexicalScope } from "@qwik.dev/core";
export const Counter_component_button_on_click_hash2 = () => {
    const [count] = useLexicalScope();
    count.value++;
};
```

## State of the Art

| Capability | OXC Status | What's Needed |
|------------|------------|---------------|
| QRL transformation | Complete (Phase 2) | Extend to JSX context |
| Capture detection | Complete (Phase 2) | No changes needed |
| useLexicalScope injection | Complete (Phase 2) | No changes needed |
| Event name transformation | Not implemented | New: `jsx_event_to_html_attribute()` |
| Native element detection | Not implemented | New: `jsx_element_is_native` stack |
| JSX attribute QRL wrapping | Partial | Extend `exit_jsx_attribute` |
| JSX transpilation | Implemented | Already has `_jsxSorted` output |
| Multiple handlers per element | Not tested | Should work with existing infra |

## Open Questions

1. **Host prefix handling (`host:onClick$`)**
   - What we know: SWC preserves `host:` prefix (becomes `host:onClick$`)
   - What's unclear: When exactly host: prefix is used
   - Recommendation: Preserve as-is per SWC behavior

2. **Custom event names (not starting with `on`)**
   - What we know: `custom$={...}` doesn't transform name (example_jsx_listeners.snap)
   - What's unclear: Whether these should still generate QRLs
   - Recommendation: QRL transformation yes, name transformation no (per SWC)

3. **Iteration variable injection in event handlers**
   - What we know: SWC has `transform_event_handler_with_iter_var` (transform.rs:3480)
   - What's unclear: Whether this is critical for Phase 3
   - Recommendation: Defer to later phase (complex iteration handling)

## Sources

### Primary (HIGH confidence)
- `/qwik-core/src/optimizer/core/src/transform.rs:3037-3098` - fold_jsx_attr (verified)
- `/qwik-core/src/optimizer/core/src/transform.rs:927-959` - handle_jsx_value (verified)
- `/qwik-core/src/optimizer/core/src/transform.rs:3347-3402` - Event name transformation (verified)
- `/qwik-core/src/optimizer/core/src/snapshots/qwik_core__test__should_convert_jsx_events.snap` (verified)
- `/qwik-core/src/optimizer/core/src/snapshots/qwik_core__test__should_not_transform_events_on_non_elements.snap` (verified)
- `/qwik-core/src/optimizer/core/src/snapshots/qwik_core__test__should_transform_event_names_without_jsx_transpile.snap` (verified)
- `/qwik-core/src/optimizer/core/src/snapshots/qwik_core__test__should_transform_multiple_event_handlers.snap` (verified)
- `/qwik-core/src/optimizer/core/src/snapshots/qwik_core__test__example_jsx_listeners.snap` (verified)

### Secondary (MEDIUM confidence)
- `/optimizer/src/transform.rs` - Current OXC implementation (verified)
- `/optimizer/src/component/segment_data.rs` - SegmentKind::EventHandler (verified)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Phase 2 established, SWC fully examined
- Architecture patterns: HIGH - SWC implementation is definitive
- Pitfalls: HIGH - Derived from SWC code analysis and snapshot examination
- Code examples: HIGH - Direct excerpts from SWC codebase

**Research date:** 2026-01-29
**Valid until:** 2026-03-01 (SWC implementation is stable reference)

## Event Handler Requirements Mapping

| Requirement | SWC Implementation | OXC Status | Notes |
|-------------|-------------------|------------|-------|
| EVT-01: onClick$ transformation | handle_jsx_value + jsx_event_to_html_attribute | Not implemented | Core event transformation |
| EVT-02: onInput$ transformation | Same as EVT-01 | Not implemented | Same pattern, different event |
| EVT-03: Multiple handlers | fold_jsx_attr handles each independently | Should work | Each attr gets own QRL |
| EVT-04: Captured state | compute_scoped_idents (Phase 2) | Complete | Phase 2 already handles |
| EVT-05: Without JSX transpile | fold_jsx_attr independent of transpile | Needs verification | Name transform + QRL only |
| EVT-06: Non-element skip | jsx_element_is_native check | Not implemented | Needs native element detection |
| EVT-07: Prevent default | Part of event handler body | Phase 2 handles | No special transformation needed |
| EVT-08: Custom events | handle_jsx_value for `custom$` | Not implemented | QRL yes, name transform no |

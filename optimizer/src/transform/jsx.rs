//! JSX transformation helpers for Qwik optimizer.
//!
//! This module contains JSX-specific transformation logic extracted from
//! generator.rs following the dispatcher pattern. The `impl Traverse` in
//! generator.rs delegates to these functions for JSX nodes.
//!
//! # Functions
//!
//! - Event name transformation: `jsx_event_to_html_attribute`, `create_event_name`
//! - Traverse dispatchers: `enter_jsx_element`, `exit_jsx_element`, etc.

use oxc_allocator::{Box as OxcBox, CloneIn, Vec as OxcVec};
use oxc_ast::ast::*;
use oxc_ast::NONE;
use oxc_ast_visit::Visit;
use oxc_span::{GetSpan, SPAN};
use oxc_traverse::TraverseCtx;
use std::collections::HashSet;

use crate::collector::{ExportInfo, Id, IdentCollector};
use crate::component::{Import, ImportId, Qrl, QrlType, MARKER_SUFFIX, QWIK_CORE_SOURCE};
use crate::is_const::is_const_expr;

use super::generator::{compute_scoped_idents, IdentType, IdPlusType, TransformGenerator};
use super::state::JsxState;

// JSX constants
const JSX_SORTED_NAME: &str = "_jsxSorted";
const JSX_SPLIT_NAME: &str = "_jsxSplit";
const JSX_RUNTIME_SOURCE: &str = "@qwik.dev/core/jsx-runtime";
const _FRAGMENT: &str = "_Fragment";
const _GET_VAR_PROPS: &str = "_getVarProps";
const _GET_CONST_PROPS: &str = "_getConstProps";

// =============================================================================
// Event Handler Transformation Utilities
// =============================================================================

/// Extracts scope prefix and event name start index from a JSX event attribute name.
///
/// # Returns
/// A tuple of (prefix, start_index) where:
/// - prefix: "on:", "on-document:", or "on-window:"
/// - start_index: index where the event name begins (after "on", "document:on", or "window:on")
/// - If not an event, returns ("", usize::MAX)
///
/// # Examples
/// - "onClick$" -> ("on:", 2)
/// - "document:onFocus$" -> ("on-document:", 11)
/// - "window:onClick$" -> ("on-window:", 9)
/// - "custom$" -> ("", usize::MAX)
pub(crate) fn get_event_scope_data_from_jsx_event(jsx_event: &str) -> (&'static str, usize) {
    if jsx_event.starts_with("window:on") {
        ("on-window:", 9)
    } else if jsx_event.starts_with("document:on") {
        ("on-document:", 11)
    } else if jsx_event.starts_with("on") {
        ("on:", 2)
    } else {
        ("", usize::MAX)
    }
}

/// Creates an HTML event attribute name from an event name and prefix.
///
/// Converts camelCase to kebab-case (e.g., "Click" -> "click", "DblClick" -> "dblclick").
/// The `-` prefix in the original name preserves case (e.g., "-cLick" -> "c-lick").
///
/// # Examples
/// - ("Click", "on:") -> "on:click"
/// - ("DblClick", "on:") -> "on:dblclick"
/// - ("Focus", "on-document:") -> "on-document:focus"
fn create_event_name(name: &str, prefix: &str) -> String {
    let mut result = String::from(prefix);

    // Check if name starts with '-' (case-preserving marker)
    let name = if let Some(stripped) = name.strip_prefix('-') {
        // Case-preserving: don't lowercase, but still convert camelCase humps to dashes
        for c in stripped.chars() {
            if c.is_ascii_uppercase() {
                result.push('-');
                result.push(c.to_ascii_lowercase());
            } else {
                result.push(c);
            }
        }
        return result;
    } else {
        name
    };

    // Standard camelCase to kebab-case: lowercase everything
    for c in name.chars() {
        if c.is_ascii_uppercase() {
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }

    result
}

/// Transforms a Qwik JSX event attribute name to HTML attribute format.
///
/// Returns `None` if the attribute is not a valid event (doesn't end with '$'
/// or doesn't start with a valid event prefix).
///
/// # Examples
/// - "onClick$" -> Some("on:click")
/// - "onDblClick$" -> Some("on:dblclick")
/// - "document:onFocus$" -> Some("on-document:focus")
/// - "window:onClick$" -> Some("on-window:click")
/// - "on-cLick$" -> Some("on:c-lick") (case preserved due to '-' prefix)
/// - "onClick" -> None (no '$' suffix)
/// - "custom$" -> None (not an event)
pub(crate) fn jsx_event_to_html_attribute(jsx_event: &str) -> Option<String> {
    // Must end with '$' to be a Qwik event handler
    if !jsx_event.ends_with('$') {
        return None;
    }

    let (prefix, idx) = get_event_scope_data_from_jsx_event(jsx_event);
    if idx == usize::MAX {
        return None;
    }

    // Extract event name: strip '$' suffix and take from idx
    // e.g., "onClick$" with idx=2 -> "Click"
    let name = &jsx_event[idx..jsx_event.len() - 1];

    Some(create_event_name(name, prefix))
}

/// Gets the full attribute name from a JSXAttributeName, including namespace if present.
///
/// # Returns
/// The full attribute name string (e.g., "onClick$", "document:onFocus$")
fn get_jsx_attribute_full_name(name: &JSXAttributeName) -> String {
    match name {
        JSXAttributeName::Identifier(id) => id.name.to_string(),
        JSXAttributeName::NamespacedName(ns) => {
            format!("{}:{}", ns.namespace.name, ns.name.name)
        }
    }
}

/// Checks if an element name is text-only (textarea, title, etc.)
fn is_text_only(node: &str) -> bool {
    matches!(
        node,
        "text" | "textarea" | "title" | "option" | "script" | "style" | "noscript"
    )
}

// =============================================================================
// JSX Element Traverse Helpers
// =============================================================================

/// Enter handler for JSXElement nodes.
///
/// Sets up JSX state tracking, determines if element is native or component,
/// pushes to stack_ctxt for entry strategy, and creates segment for the element.
pub fn enter_jsx_element<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &mut JSXElement<'a>,
    ctx: &mut TraverseCtx<'a, ()>,
) {
    // Determine if this is a native element (lowercase first char)
    let is_native = match &node.opening_element.name {
        JSXElementName::Identifier(_) => true, // lowercase native HTML
        JSXElementName::IdentifierReference(id) => {
            id.name.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false)
        }
        JSXElementName::MemberExpression(_) => false, // Foo.Bar = component
        JSXElementName::NamespacedName(_) => true,    // svg:rect = native
        JSXElementName::ThisExpression(_) => false,   // this = component
    };
    gen.jsx_element_is_native.push(is_native);

    // Push JSX element name to stack_ctxt for entry strategy (SWC fold_jsx_element)
    // Only push if it's an identifier (not member expression or other complex form)
    let jsx_element_name = match &node.opening_element.name {
        JSXElementName::Identifier(id) => Some(id.name.to_string()),
        JSXElementName::IdentifierReference(id) => Some(id.name.to_string()),
        _ => None,
    };
    if let Some(name) = &jsx_element_name {
        gen.stack_ctxt.push(name.clone());
    }

    let (segment, is_fn, is_text_only_elem) =
        if let Some(id) = node.opening_element.name.get_identifier() {
            (Some(gen.new_segment(id.name)), true, false)
        } else if let Some(name) = node.opening_element.name.get_identifier_name() {
            (
                Some(gen.new_segment(name)),
                false,
                is_text_only(name.into()),
            )
        } else {
            (None, true, false)
        };
    gen.jsx_stack.push(JsxState {
        is_fn,
        is_text_only: is_text_only_elem,
        is_segment: segment.is_some(),
        should_runtime_sort: false,
        static_listeners: true,
        static_subtree: true,
        key_prop: None,
        var_props: OxcVec::new_in(gen.builder.allocator),
        const_props: OxcVec::new_in(gen.builder.allocator),
        children: OxcVec::new_in(gen.builder.allocator),
        spread_expr: None,
        // Track whether we pushed to stack_ctxt
        stacked_ctxt: jsx_element_name.is_some(),
    });
    if let Some(segment) = segment {
        gen.debug(format!("ENTER: JSXElementName {segment}"), ctx);
        println!("push segment: {segment}");
        gen.segment_stack.push(segment);
    }
}

/// Exit handler for JSXElement nodes.
///
/// Generates the _jsxSorted or _jsxSplit call, handles props sorting,
/// children processing, and flags calculation.
pub fn exit_jsx_element<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &mut JSXElement<'a>,
    ctx: &mut TraverseCtx<'a, ()>,
) {
    if let Some(mut jsx) = gen.jsx_stack.pop() {
        if gen.options.transpile_jsx {
            if !jsx.should_runtime_sort {
                jsx.var_props.sort_by_key(|prop| match prop {
                    ObjectPropertyKind::ObjectProperty(b) => match &(*b).key {
                        PropertyKey::StringLiteral(b) => (*b).to_string(),
                        _ => "".to_string(),
                    },
                    _ => "".to_string(),
                });
            }
            let name = &node.opening_element.name;
            let (jsx_type, pure) = match name {
                JSXElementName::Identifier(b) => (
                    gen.builder.expression_string_literal(
                        (*b).span,
                        (*b).name,
                        Some((*b).name),
                    ),
                    true,
                ),
                JSXElementName::IdentifierReference(b) => (
                    gen.builder.expression_identifier((*b).span, (*b).name),
                    false,
                ),
                JSXElementName::NamespacedName(_b) => {
                    panic!("namespaced names in JSX not implemented")
                }
                JSXElementName::MemberExpression(b) => {
                    fn process_member_expr<'b>(
                        builder: &oxc_ast::AstBuilder<'b>,
                        expr: &JSXMemberExpressionObject<'b>,
                    ) -> Expression<'b> {
                        match expr {
                            JSXMemberExpressionObject::ThisExpression(b) => {
                                builder.expression_this((*b).span)
                            }
                            JSXMemberExpressionObject::IdentifierReference(b) => {
                                builder.expression_identifier((*b).span, (*b).name)
                            }
                            JSXMemberExpressionObject::MemberExpression(b) => builder
                                .member_expression_static(
                                    (*b).span,
                                    process_member_expr(builder, &(*b).object),
                                    builder.identifier_name(
                                        (*b).property.span(),
                                        (*b).property.name,
                                    ),
                                    false,
                                )
                                .into(),
                        }
                    }
                    (
                        gen.builder
                            .member_expression_static(
                                (*b).span(),
                                process_member_expr(&gen.builder, &((*b).object)),
                                gen.builder
                                    .identifier_name((*b).property.span(), (*b).property.name),
                                false,
                            )
                            .into(),
                        false,
                    )
                }
                JSXElementName::ThisExpression(b) => {
                    (gen.builder.expression_this((*b).span), false)
                }
            };
            // Output null instead of empty object for varProps/constProps
            let var_props_arg: Expression<'a> = if jsx.var_props.is_empty() {
                gen.builder.expression_null_literal(node.span())
            } else {
                gen.builder.expression_object(node.span(), jsx.var_props)
            };
            // When spread exists, constProps is _getConstProps(spread_expr) call directly
            let const_props_arg: Expression<'a> = if let Some(spread_expr) = jsx.spread_expr.take() {
                // Generate _getConstProps(spread_expr) - call directly, not wrapped in object
                gen.builder.expression_call(
                    node.span(),
                    gen.builder.expression_identifier(node.span(), _GET_CONST_PROPS),
                    None::<OxcBox<TSTypeParameterInstantiation<'a>>>,
                    gen.builder.vec1(Argument::from(spread_expr)),
                    false,
                )
            } else if jsx.const_props.is_empty() {
                gen.builder.expression_null_literal(node.span())
            } else {
                gen.builder.expression_object(node.span(), jsx.const_props)
            };
            // Children argument: null for empty, direct for single, array for multiple
            let children_arg: Expression<'a> = if jsx.children.is_empty() {
                gen.builder.expression_null_literal(node.span())
            } else if jsx.children.len() == 1 {
                // Single child - pass directly (unwrap from ArrayExpressionElement)
                let child = jsx.children.pop().unwrap();
                if let Some(expr) = child.as_expression() {
                    expr.clone_in(gen.builder.allocator)
                } else if let ArrayExpressionElement::SpreadElement(spread) = child {
                    // Wrap spread in array (spread must be in array context)
                    let mut children = OxcVec::new_in(gen.builder.allocator);
                    children.push(ArrayExpressionElement::SpreadElement(spread));
                    gen.builder.expression_array(node.span(), children)
                } else {
                    // Elision case
                    gen.builder.expression_null_literal(node.span())
                }
            } else {
                gen.builder.expression_array(node.span(), jsx.children)
            };
            let args: OxcVec<Argument<'a>> = OxcVec::from_array_in(
                [
                    // type
                    jsx_type.into(),
                    // varProps
                    var_props_arg.into(),
                    // constProps
                    const_props_arg.into(),
                    // children
                    children_arg.into(),
                    // flags: bit 0 = static_listeners, bit 1 = static_subtree (per SWC reference)
                    // Values: 3 = both static, 2 = static_subtree only, 1 = static_listeners only, 0 = neither
                    gen.builder
                        .expression_numeric_literal(
                            node.span(),
                            ((if jsx.static_listeners { 0b1 } else { 0 })
                                | (if jsx.static_subtree { 0b10 } else { 0 }))
                            .into(),
                            None,
                            NumberBase::Decimal,
                        )
                        .into(),
                    // key
                    jsx.key_prop
                        .unwrap_or_else(|| -> Expression<'a> {
                            // TODO: Figure out how to replicate root_jsx_mode from old optimizer
                            // (this conditional should be is_fn || root_jsx_mode)
                            if jsx.is_fn {
                                if let Some(cmp) = gen.component_stack.last() {
                                    let new_key = format!(
                                        "{}_{}",
                                        cmp.id.hash.chars().take(2).collect::<String>(),
                                        gen.jsx_key_counter
                                    );
                                    gen.jsx_key_counter += 1;
                                    return gen.builder.expression_string_literal(
                                        oxc_span::Span::default(),
                                        gen.builder.atom(&new_key),
                                        None,
                                    );
                                }
                            }
                            gen.builder.expression_null_literal(oxc_span::Span::default())
                        })
                        .into(),
                ],
                gen.builder.allocator,
            );
            let callee = if jsx.should_runtime_sort {
                JSX_SPLIT_NAME
            } else {
                JSX_SORTED_NAME
            };
            gen.replace_expr = Some(gen.builder.expression_call_with_pure(
                node.span,
                gen.builder.expression_identifier(name.span(), callee),
                None::<OxcBox<TSTypeParameterInstantiation<'a>>>,
                args,
                false,
                pure,
            ));
            if let Some(imports) = gen.import_stack.last_mut() {
                imports.insert(Import::new(vec![callee.into()], QWIK_CORE_SOURCE));
                // Add spread helper imports when _jsxSplit is used
                if jsx.should_runtime_sort {
                    imports.insert(Import::new(vec![_GET_VAR_PROPS.into()], QWIK_CORE_SOURCE));
                    imports.insert(Import::new(vec![_GET_CONST_PROPS.into()], QWIK_CORE_SOURCE));
                }
            }
        }
        if jsx.is_segment {
            let _popped = gen.segment_stack.pop();
        }
        // Pop stack_ctxt if we pushed for this JSX element (SWC fold_jsx_element)
        if jsx.stacked_ctxt {
            gen.stack_ctxt.pop();
        }
    }

    // Pop native element tracking
    gen.jsx_element_is_native.pop();

    gen.debug("EXIT: JSXElementName", ctx);
    gen.descend();
}

// =============================================================================
// JSX Fragment Traverse Helpers
// =============================================================================

/// Enter handler for JSXFragment nodes.
pub fn enter_jsx_fragment<'a>(
    gen: &mut TransformGenerator<'a>,
    _node: &mut JSXFragment<'a>,
    ctx: &mut TraverseCtx<'a, ()>,
) {
    gen.jsx_stack.push(JsxState {
        is_fn: true, // Fragments generate keys like component elements
        is_text_only: false,
        is_segment: false,
        should_runtime_sort: false,
        static_listeners: true,
        static_subtree: true,
        key_prop: None,
        var_props: OxcVec::new_in(gen.builder.allocator),
        const_props: OxcVec::new_in(gen.builder.allocator),
        children: OxcVec::new_in(gen.builder.allocator),
        spread_expr: None,
        stacked_ctxt: false, // Fragments don't push to stack_ctxt
    });
    gen.debug("ENTER: JSXFragment", ctx);
}

/// Exit handler for JSXFragment nodes.
pub fn exit_jsx_fragment<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &mut JSXFragment<'a>,
    ctx: &mut TraverseCtx<'a, ()>,
) {
    if let Some(mut jsx) = gen.jsx_stack.pop() {
        if gen.options.transpile_jsx {
            // Generate _jsxSorted(_Fragment, null, null, children, flags, key)
            // Prepare children argument - single child or array
            let children_arg: Expression<'a> = if jsx.children.len() == 1 {
                // Single child - pass directly (unwrap from ArrayExpressionElement)
                let child = jsx.children.pop().unwrap();
                if let Some(expr) = child.as_expression() {
                    expr.clone_in(gen.builder.allocator)
                } else if let ArrayExpressionElement::SpreadElement(spread) = child {
                    // Wrap spread in array
                    let mut children = OxcVec::new_in(gen.builder.allocator);
                    children.push(ArrayExpressionElement::SpreadElement(spread));
                    gen.builder.expression_array(node.span, children)
                } else {
                    // Elision case
                    gen.builder.expression_null_literal(node.span)
                }
            } else if jsx.children.is_empty() {
                gen.builder.expression_null_literal(node.span)
            } else {
                gen.builder.expression_array(node.span, jsx.children)
            };

            // Generate key for fragment inside component
            let key_arg: Expression<'a> = jsx.key_prop.unwrap_or_else(|| {
                if let Some(cmp) = gen.component_stack.last() {
                    let new_key = format!(
                        "{}_{}",
                        cmp.id.hash.chars().take(2).collect::<String>(),
                        gen.jsx_key_counter
                    );
                    gen.jsx_key_counter += 1;
                    gen.builder.expression_string_literal(
                        oxc_span::Span::default(),
                        gen.builder.atom(&new_key),
                        None,
                    )
                } else {
                    gen.builder.expression_null_literal(oxc_span::Span::default())
                }
            });

            // Calculate flags: bit 0 = static_listeners, bit 1 = static_subtree (per SWC reference)
            let flags = ((if jsx.static_listeners { 0b1 } else { 0 })
                | (if jsx.static_subtree { 0b10 } else { 0 })) as f64;

            let args: OxcVec<Argument<'a>> = OxcVec::from_array_in(
                [
                    // type: _Fragment identifier
                    gen.builder
                        .expression_identifier(node.span, _FRAGMENT)
                        .into(),
                    // varProps: null (fragments have no props)
                    gen.builder.expression_null_literal(node.span).into(),
                    // constProps: null (fragments have no props)
                    gen.builder.expression_null_literal(node.span).into(),
                    // children
                    children_arg.into(),
                    // flags
                    gen.builder
                        .expression_numeric_literal(node.span, flags, None, NumberBase::Decimal)
                        .into(),
                    // key
                    key_arg.into(),
                ],
                gen.builder.allocator,
            );

            gen.replace_expr = Some(gen.builder.expression_call_with_pure(
                node.span,
                gen.builder.expression_identifier(node.span, JSX_SORTED_NAME),
                None::<OxcBox<TSTypeParameterInstantiation<'a>>>,
                args,
                false,
                true, // pure annotation
            ));

            // Add imports: _jsxSorted from @qwik.dev/core, Fragment as _Fragment from jsx-runtime
            if let Some(imports) = gen.import_stack.last_mut() {
                imports.insert(Import::new(vec![JSX_SORTED_NAME.into()], QWIK_CORE_SOURCE));
                imports.insert(Import::new(
                    vec![ImportId::NamedWithAlias("Fragment".to_string(), _FRAGMENT.to_string())],
                    JSX_RUNTIME_SOURCE,
                ));
            }
        }
    }
    gen.debug("EXIT: JSXFragment", ctx);
}

// =============================================================================
// JSX Attribute Traverse Helpers
// =============================================================================

/// Enter handler for JSXAttribute nodes.
pub fn enter_jsx_attribute<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &mut JSXAttribute<'a>,
    ctx: &mut TraverseCtx<'a, ()>,
) {
    if gen.options.transpile_jsx {
        gen.expr_is_const_stack.push(
            gen.jsx_stack
                .last()
                .map_or(false, |jsx| !jsx.should_runtime_sort),
        );
    }
    gen.ascend();
    gen.debug("ENTER: JSXAttribute", ctx);
    // JSX Attributes should be treated as part of the segment scope.
    // Use the last part of the name for segment naming (e.g., "onFocus$" from "document:onFocus$")
    let segment_name = match &node.name {
        JSXAttributeName::Identifier(id) => id.name.to_string(),
        JSXAttributeName::NamespacedName(ns) => ns.name.name.to_string(),
    };
    let segment = gen.new_segment(segment_name);
    gen.segment_stack.push(segment);

    // Check if this is an event handler attribute with a function value
    let attr_name = get_jsx_attribute_full_name(&node.name);

    // Push attribute name to stack_ctxt for entry strategy (SWC fold_jsx_attr)
    // For native elements with event handlers, push the transformed name (on:click);
    // otherwise push the original attribute name
    let is_native = gen.jsx_element_is_native.last().copied().unwrap_or(false);
    let stack_ctxt_name = if is_native {
        // Try to transform event name for native elements
        if let Some(html_attr) = jsx_event_to_html_attribute(&attr_name) {
            html_attr.to_string()
        } else {
            attr_name.clone()
        }
    } else {
        attr_name.clone()
    };
    gen.stack_ctxt.push(stack_ctxt_name);

    // Check for bind directive (bind:value or bind:checked)
    // Only process on native elements
    let is_native = gen.jsx_element_is_native.last().copied().unwrap_or(false);
    if is_native {
        if let Some(is_checked) = TransformGenerator::is_bind_directive(&attr_name) {
            // Extract signal expression from value
            if let Some(JSXAttributeValue::ExpressionContainer(container)) = &node.value {
                if let Some(expr) = container.expression.as_expression() {
                    gen.pending_bind_directives.push((
                        is_checked,
                        expr.clone_in(ctx.ast.allocator),
                    ));
                    // Mark import needs
                    if is_checked {
                        gen.needs_chk_import = true;
                    } else {
                        gen.needs_val_import = true;
                    }
                    gen.needs_inlined_qrl_import = true;
                }
            }
        }
    }

    if attr_name.ends_with(MARKER_SUFFIX) {
        if let Some(JSXAttributeValue::ExpressionContainer(container)) = &node.value {
            if let Some(expr) = container.expression.as_expression() {
                let is_fn = matches!(
                    expr,
                    Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
                );

                if is_fn {
                    // Push new import stack frame for this QRL (mirrors enter_call_expression)
                    gen.import_stack.push(std::collections::BTreeSet::new());
                }
            }
        }
    }
}

/// Exit handler for JSXAttribute nodes.
pub fn exit_jsx_attribute<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &mut JSXAttribute<'a>,
    ctx: &mut TraverseCtx<'a, ()>,
) {
    // Transform event handler attribute names on native elements
    let attr_name = get_jsx_attribute_full_name(&node.name);
    let is_native = gen.jsx_element_is_native.last().copied().unwrap_or(false);

    // Check for bind directive transformation (bind:value or bind:checked)
    // Only transform on native elements
    if is_native && gen.options.transpile_jsx {
        if let Some(is_checked) = TransformGenerator::is_bind_directive(&attr_name) {
            // This is bind:value or bind:checked - transform it
            if let Some(JSXAttributeValue::ExpressionContainer(container)) = &node.value {
                if let Some(expr) = container.expression.as_expression() {
                    let signal_expr = expr.clone_in(ctx.ast.allocator);
                    let prop_name = if is_checked { "checked" } else { "value" };

                    // Create the bind handler: inlinedQrl(_val/_chk, "_val"/"_chk", [signal])
                    let bind_handler = TransformGenerator::create_bind_handler(
                        &ctx.ast,
                        is_checked,
                        signal_expr.clone_in(ctx.ast.allocator),
                    );

                    // Pop the is_const from stack since we're handling this manually
                    gen.expr_is_const_stack.pop();

                    if let Some(jsx) = gen.jsx_stack.last_mut() {
                        // Add value/checked prop with signal to const_props
                        let prop_name_atom = gen.builder.atom(prop_name);
                        jsx.const_props.push(gen.builder.object_property_kind_object_property(
                            node.span,
                            PropertyKind::Init,
                            gen.builder.property_key_static_identifier(SPAN, prop_name_atom),
                            signal_expr,
                            false,
                            false,
                            false,
                        ));

                        // Check if there's an existing on:input handler to merge with
                        // Look in both const_props and var_props for "on:input"
                        let existing_on_input_idx = jsx.const_props.iter().position(|prop| {
                            if let ObjectPropertyKind::ObjectProperty(obj_prop) = prop {
                                if let PropertyKey::StaticIdentifier(id) = &obj_prop.key {
                                    return id.name == "on:input";
                                }
                            }
                            false
                        });

                        if let Some(idx) = existing_on_input_idx {
                            // Merge with existing on:input handler
                            if let ObjectPropertyKind::ObjectProperty(obj_prop) = &jsx.const_props[idx]
                            {
                                let existing_handler = obj_prop.value.clone_in(ctx.ast.allocator);
                                let merged = TransformGenerator::merge_event_handlers(
                                    &ctx.ast,
                                    existing_handler,
                                    bind_handler,
                                );

                                // Replace the existing prop with merged handler
                                let on_input_atom = gen.builder.atom("on:input");
                                jsx.const_props[idx] =
                                    gen.builder.object_property_kind_object_property(
                                        node.span,
                                        PropertyKind::Init,
                                        gen.builder.property_key_static_identifier(SPAN, on_input_atom),
                                        merged,
                                        false,
                                        false,
                                        false,
                                    );
                            }
                        } else {
                            // No existing on:input, add the bind handler as-is
                            let on_input_atom = gen.builder.atom("on:input");
                            jsx.const_props.push(gen.builder.object_property_kind_object_property(
                                node.span,
                                PropertyKind::Init,
                                gen.builder.property_key_static_identifier(SPAN, on_input_atom),
                                bind_handler,
                                false,
                                false,
                                false,
                            ));
                        }
                    }

                    // Skip the normal prop processing - pop segment/stack_ctxt and return
                    gen.segment_stack.pop();
                    gen.stack_ctxt.pop();
                    gen.debug("EXIT: JSXAttribute (bind directive)", ctx);
                    gen.descend();
                    return;
                }
            }
        }
    }

    if attr_name.ends_with(MARKER_SUFFIX) {
        if is_native {
            if let Some(html_attr) = jsx_event_to_html_attribute(&attr_name) {
                let new_name = gen.builder.atom(&html_attr);
                node.name = JSXAttributeName::Identifier(gen.builder.alloc(JSXIdentifier {
                    span: node.name.span(),
                    name: new_name,
                }));
            }
        }
    }

    // Handle QRL transformation for event handler function values
    if attr_name.ends_with(MARKER_SUFFIX) {
        if let Some(JSXAttributeValue::ExpressionContainer(container)) = &mut node.value {
            if let Some(expr) = container.expression.as_expression() {
                let is_fn = matches!(
                    expr,
                    Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
                );

                if is_fn {
                    // Create QRL using existing infrastructure (mirrors exit_call_expression)
                    // 1. Collect identifiers
                    let descendent_idents = {
                        let mut collector = IdentCollector::new();
                        use oxc_ast_visit::Visit;
                        collector.visit_expression(expr);
                        collector.get_words()
                    };

                    // 2. Get declarations and compute captures
                    let all_decl: Vec<IdPlusType> = gen
                        .decl_stack
                        .iter()
                        .flat_map(|v| v.iter())
                        .cloned()
                        .collect();
                    let (decl_collect, _): (Vec<_>, Vec<_>) = all_decl
                        .into_iter()
                        .partition(|(_, t)| matches!(t, IdentType::Var(_)));
                    let (scoped_idents, _) =
                        compute_scoped_idents(&descendent_idents, &decl_collect);

                    // 3. Filter imported identifiers
                    let imports = gen.import_stack.pop().unwrap_or_default();
                    let imported_names: HashSet<String> = imports
                        .iter()
                        .flat_map(|import| import.names.iter())
                        .filter_map(|id| match id {
                            ImportId::Named(name) | ImportId::Default(name) => Some(name.clone()),
                            ImportId::NamedWithAlias(_, local) => Some(local.clone()),
                            ImportId::Namespace(_) => None,
                        })
                        .collect();
                    let scoped_idents: Vec<Id> = scoped_idents
                        .into_iter()
                        .filter(|(name, _)| !imported_names.contains(name))
                        .collect();

                    // Collect referenced exports for segment file imports
                    let referenced_exports: Vec<ExportInfo> = descendent_idents
                        .iter()
                        .filter_map(|(name, _)| {
                            if imported_names.contains(name) {
                                return None;
                            }
                            if scoped_idents.iter().any(|(n, _)| n == name) {
                                return None;
                            }
                            gen.export_by_name.get(name).cloned()
                        })
                        .collect();

                    // 4. Create Qrl and transform
                    let display_name = gen.current_display_name();
                    let qrl = Qrl::new_with_exports(
                        gen.source_info.rel_path.clone(),
                        &display_name,
                        QrlType::Qrl,
                        scoped_idents,
                        referenced_exports,
                    );

                    let call_expr = qrl.into_call_expression(
                        ctx,
                        &mut gen.symbol_by_name,
                        &mut gen.import_by_symbol,
                    );

                    // 5. Replace expression with QRL call
                    container.expression =
                        JSXExpression::from(Expression::CallExpression(ctx.ast.alloc(call_expr)));

                    // 6. Add qrl import
                    if let Some(import_set) = gen.import_stack.last_mut() {
                        import_set.insert(Import::qrl());
                    }
                }
            }
        }
    }

    if gen.options.transpile_jsx {
        // Pre-compute wrap info before mutable borrow of jsx_stack
        // Check for prop identifier that needs wrapping
        let prop_wrap_key: Option<String> =
            if let Some(JSXAttributeValue::ExpressionContainer(container)) = &node.value {
                if let Some(expr) = container.expression.as_expression() {
                    gen.should_wrap_prop(expr).map(|(_, key)| key)
                } else {
                    None
                }
            } else {
                None
            };

        // Check for signal.value wrapping
        let needs_signal_wrap: bool = if prop_wrap_key.is_none() {
            if let Some(JSXAttributeValue::ExpressionContainer(container)) = &node.value {
                if let Some(expr) = container.expression.as_expression() {
                    gen.should_wrap_signal_value(expr)
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        // Pre-compute is_const using is_const_expr before mutable borrow of jsx_stack
        // Pop the stack value (maintains stack balance) but use is_const_expr for accuracy
        let stack_is_const = gen.expr_is_const_stack.pop().unwrap_or_default();
        let is_const = if let Some(JSXAttributeValue::ExpressionContainer(container)) = &node.value
        {
            if let Some(value_expr) = container.expression.as_expression() {
                // Only check is_const_expr if the stack says it could be const
                // (handles should_runtime_sort case where all props are var)
                if stack_is_const {
                    let import_names = gen.get_imported_names();
                    is_const_expr(value_expr, &import_names, &gen.decl_stack)
                } else {
                    false
                }
            } else {
                stack_is_const
            }
        } else {
            // String literals and boolean attributes are always const
            stack_is_const
        };

        if let Some(jsx) = gen.jsx_stack.last_mut() {
            let expr: Expression<'a> = {
                let v = &mut node.value;
                match v {
                    None => gen.builder.expression_boolean_literal(node.span, true),
                    Some(JSXAttributeValue::Element(_)) => {
                        println!("Replacing JSX attribute element on exit");
                        gen.replace_expr.take().unwrap()
                    }
                    Some(JSXAttributeValue::Fragment(_)) => {
                        println!("Replacing JSX attribute fragment on exit");
                        gen.replace_expr.take().unwrap()
                    }
                    Some(JSXAttributeValue::StringLiteral(b)) => gen
                        .builder
                        .expression_string_literal((*b).span, (*b).value, Some((*b).value)),
                    Some(JSXAttributeValue::ExpressionContainer(b)) => {
                        let inner_expr = (*b).expression.to_expression_mut();
                        let span = inner_expr.span();

                        // Check for prop that needs _wrapProp
                        if let Some(prop_key) = &prop_wrap_key {
                            gen.needs_wrap_prop_import = true;
                            // Build _wrapProp(_rawProps, "propKey") inline
                            let prop_key_str: &'a str = ctx.ast.allocator.alloc_str(prop_key);
                            ctx.ast.expression_call(
                                span,
                                ctx.ast.expression_identifier(SPAN, "_wrapProp"),
                                NONE,
                                ctx.ast.vec_from_array([
                                    Argument::from(ctx.ast.expression_identifier(SPAN, "_rawProps")),
                                    Argument::from(ctx.ast.expression_string_literal(
                                        SPAN,
                                        prop_key_str,
                                        None,
                                    )),
                                ]),
                                false,
                            )
                        }
                        // Check for signal.value that needs _wrapProp
                        else if needs_signal_wrap {
                            gen.needs_wrap_prop_import = true;
                            if let Expression::StaticMemberExpression(static_member) = inner_expr {
                                let signal_expr = static_member.object.clone_in(ctx.ast.allocator);
                                // Build _wrapProp(signal) inline
                                ctx.ast.expression_call(
                                    span,
                                    ctx.ast.expression_identifier(SPAN, "_wrapProp"),
                                    NONE,
                                    ctx.ast.vec1(Argument::from(signal_expr)),
                                    false,
                                )
                            } else {
                                move_expression(&gen.builder, inner_expr)
                            }
                        } else {
                            move_expression(&gen.builder, inner_expr)
                        }
                    }
                }
            };
            if node.is_key() {
                jsx.key_prop = Some(expr);
            } else {
                // Use the transformed name (or original if not transformed) for the property key
                let prop_name = get_jsx_attribute_full_name(&node.name);
                let prop_name_atom = gen.builder.atom(&prop_name);

                // Check if this is an on:input handler that needs to merge with existing bind handler
                if prop_name == "on:input" {
                    let existing_on_input_idx = jsx.const_props.iter().position(|prop| {
                        if let ObjectPropertyKind::ObjectProperty(obj_prop) = prop {
                            if let PropertyKey::StaticIdentifier(id) = &obj_prop.key {
                                return id.name == "on:input";
                            }
                        }
                        false
                    });

                    if let Some(idx) = existing_on_input_idx {
                        // Merge with existing on:input from bind directive
                        if let ObjectPropertyKind::ObjectProperty(obj_prop) = &jsx.const_props[idx] {
                            let existing_handler = obj_prop.value.clone_in(ctx.ast.allocator);
                            // For this case, the existing handler is from bind, new one is from onInput$
                            // So we want [onInput$_handler, bind_handler]
                            let merged = TransformGenerator::merge_event_handlers(
                                &ctx.ast,
                                expr,
                                existing_handler,
                            );

                            jsx.const_props[idx] = gen.builder.object_property_kind_object_property(
                                node.span,
                                PropertyKind::Init,
                                gen.builder.property_key_static_identifier(SPAN, prop_name_atom),
                                merged,
                                false,
                                false,
                                false,
                            );
                        }
                    } else {
                        // No existing on:input, add normally
                        let props = if is_const {
                            &mut jsx.const_props
                        } else {
                            &mut jsx.var_props
                        };
                        props.push(gen.builder.object_property_kind_object_property(
                            node.span,
                            PropertyKind::Init,
                            gen.builder.property_key_static_identifier(node.name.span(), prop_name_atom),
                            expr,
                            false,
                            false,
                            false,
                        ));
                    }
                } else {
                    let props = if is_const {
                        &mut jsx.const_props
                    } else {
                        &mut jsx.var_props
                    };
                    props.push(gen.builder.object_property_kind_object_property(
                        node.span,
                        PropertyKind::Init,
                        gen.builder.property_key_static_identifier(node.name.span(), prop_name_atom),
                        expr,
                        false,
                        false,
                        false,
                    ));
                }
            }
        }
    }
    let _popped = gen.segment_stack.pop();
    // Pop stack_ctxt for this attribute (SWC fold_jsx_attr)
    gen.stack_ctxt.pop();
    gen.debug("EXIT: JSXAttribute", ctx);
    gen.descend();
}

// =============================================================================
// JSX Spread Attribute Traverse Helpers
// =============================================================================

/// Exit handler for JSXSpreadAttribute nodes.
pub fn exit_jsx_spread_attribute<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &mut JSXSpreadAttribute<'a>,
    _ctx: &mut TraverseCtx<'a, ()>,
) {
    if !gen.options.transpile_jsx {
        return;
    }
    // Reference: qwik build/v2 internal_handle_jsx_props_obj
    // If we have spread props, all props that come before it are variable even if they're static
    if let Some(jsx) = gen.jsx_stack.last_mut() {
        let range = 0..jsx.const_props.len();
        jsx.const_props
            .drain(range)
            .for_each(|p| jsx.var_props.push(p));
        jsx.should_runtime_sort = true;
        jsx.static_subtree = false;
        jsx.static_listeners = false;

        // Store spread expression for _getConstProps generation in exit_jsx_element
        let spread_arg = move_expression(&gen.builder, &mut node.argument);
        jsx.spread_expr = Some(spread_arg.clone_in(gen.builder.allocator));

        // Generate _getVarProps(spread_arg) call and spread it into var_props
        // Output: { ..._getVarProps(props) }
        let get_var_props_call = gen.builder.expression_call(
            node.span(),
            gen.builder.expression_identifier(node.span(), _GET_VAR_PROPS),
            None::<OxcBox<TSTypeParameterInstantiation<'a>>>,
            gen.builder.vec1(Argument::from(spread_arg)),
            false,
        );
        jsx.var_props
            .push(gen.builder.object_property_kind_spread_property(
                node.span(),
                get_var_props_call,
            ))
    }
}

// =============================================================================
// JSX Attribute Value Traverse Helpers
// =============================================================================

/// Exit handler for JSXAttributeValue nodes.
pub fn exit_jsx_attribute_value<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &mut JSXAttributeValue<'a>,
    ctx: &mut TraverseCtx<'a, ()>,
) {
    if let JSXAttributeValue::ExpressionContainer(container) = node {
        let qrl = gen.qrl_stack.pop();

        if let Some(qrl) = qrl {
            container.expression = qrl.into_jsx_expression(
                ctx,
                &mut gen.symbol_by_name,
                &mut gen.import_by_symbol,
            )
        }
    }
}

// =============================================================================
// JSX Child Traverse Helpers
// =============================================================================

/// Exit handler for JSXChild nodes.
pub fn exit_jsx_child<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &mut JSXChild<'a>,
    ctx: &mut TraverseCtx<'a, ()>,
) {
    if !gen.options.transpile_jsx {
        return;
    }
    gen.debug("EXIT: JSX child", ctx);

    // Pre-compute wrap info before mutable borrow of jsx_stack
    let prop_wrap_key: Option<String> = if let JSXChild::ExpressionContainer(container) = node {
        if let Some(expr) = container.expression.as_expression() {
            gen.should_wrap_prop(expr).map(|(_, key)| key)
        } else {
            None
        }
    } else {
        None
    };

    let needs_signal_wrap: bool = if prop_wrap_key.is_none() {
        if let JSXChild::ExpressionContainer(container) = node {
            if let Some(expr) = container.expression.as_expression() {
                gen.should_wrap_signal_value(expr)
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    if let Some(jsx) = gen.jsx_stack.last_mut() {
        let maybe_child = match node {
            JSXChild::Text(b) => {
                let text: &'a str = gen.builder.allocator.alloc_str(b.value.trim());
                if text.is_empty() {
                    None
                } else {
                    Some(
                        gen.builder
                            .expression_string_literal((*b).span, text, Some(text.into()))
                            .into(),
                    )
                }
            }
            JSXChild::Element(_) => {
                println!("Replacing JSX child element on exit");
                Some(gen.replace_expr.take().unwrap().into())
            }
            JSXChild::Fragment(_) => {
                println!("Replacing JSX child fragment on exit");
                Some(gen.replace_expr.take().unwrap().into())
            }
            JSXChild::ExpressionContainer(b) => {
                jsx.static_subtree = false;
                let expr = (*b).expression.to_expression_mut();
                let span = expr.span();

                // Check for prop that needs _wrapProp
                if let Some(prop_key) = &prop_wrap_key {
                    gen.needs_wrap_prop_import = true;
                    // Build _wrapProp(_rawProps, "propKey") inline
                    let prop_key_str: &'a str = ctx.ast.allocator.alloc_str(prop_key);
                    Some(
                        ctx.ast
                            .expression_call(
                                span,
                                ctx.ast.expression_identifier(SPAN, "_wrapProp"),
                                NONE,
                                ctx.ast.vec_from_array([
                                    Argument::from(ctx.ast.expression_identifier(SPAN, "_rawProps")),
                                    Argument::from(ctx.ast.expression_string_literal(
                                        SPAN,
                                        prop_key_str,
                                        None,
                                    )),
                                ]),
                                false,
                            )
                            .into(),
                    )
                }
                // Check for signal.value that needs _wrapProp
                else if needs_signal_wrap {
                    gen.needs_wrap_prop_import = true;
                    if let Expression::StaticMemberExpression(static_member) = expr {
                        let signal_expr = static_member.object.clone_in(ctx.ast.allocator);
                        // Build _wrapProp(signal) inline
                        Some(
                            ctx.ast
                                .expression_call(
                                    span,
                                    ctx.ast.expression_identifier(SPAN, "_wrapProp"),
                                    NONE,
                                    ctx.ast.vec1(Argument::from(signal_expr)),
                                    false,
                                )
                                .into(),
                        )
                    } else {
                        Some(move_expression(&gen.builder, expr).into())
                    }
                } else {
                    Some(move_expression(&gen.builder, expr).into())
                }
            }
            JSXChild::Spread(b) => {
                jsx.static_subtree = false;
                let span = (*b).span.clone();
                Some(gen.builder.array_expression_element_spread_element(
                    span,
                    move_expression(&gen.builder, &mut (*b).expression),
                ))
            }
        };
        if let Some(child) = maybe_child {
            jsx.children.push(child);
        }
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

fn move_expression<'gen>(
    builder: &oxc_ast::AstBuilder<'gen>,
    expr: &mut Expression<'gen>,
) -> Expression<'gen> {
    let span = expr.span().clone();
    std::mem::replace(expr, builder.expression_null_literal(span))
}

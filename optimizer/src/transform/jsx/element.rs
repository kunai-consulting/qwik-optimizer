//! JSX Element traversal handlers.
//!
//! This module contains enter_jsx_element and exit_jsx_element handlers
//! for the Traverse impl dispatcher pattern.

use oxc_allocator::{Box as OxcBox, CloneIn, Vec as OxcVec};
use oxc_ast::ast::*;
use oxc_span::GetSpan;
use oxc_traverse::TraverseCtx;

use crate::component::{Import, QWIK_CORE_SOURCE};
use crate::transform::generator::TransformGenerator;
use crate::transform::state::JsxState;

use super::{is_text_only, JSX_SORTED_NAME, JSX_SPLIT_NAME, _GET_CONST_PROPS, _GET_VAR_PROPS};

/// Enter handler for JSXElement nodes.
///
/// Sets up JSX state tracking, determines if element is native or component,
/// pushes to stack_ctxt for entry strategy, and creates segment for the element.
pub fn enter_jsx_element<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &mut JSXElement<'a>,
    _ctx: &mut TraverseCtx<'a, ()>,
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

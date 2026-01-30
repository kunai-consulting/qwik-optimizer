//! JSX Child traversal handlers.
//!
//! This module contains exit_jsx_child handler for the Traverse impl dispatcher pattern.

use oxc_allocator::CloneIn;
use oxc_ast::ast::*;
use oxc_ast::NONE;
use oxc_span::{GetSpan, SPAN};
use oxc_traverse::TraverseCtx;

use crate::transform::generator::TransformGenerator;

use super::move_expression;

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
            JSXChild::Element(_) => Some(gen.replace_expr.take().unwrap().into()),
            JSXChild::Fragment(_) => Some(gen.replace_expr.take().unwrap().into()),
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

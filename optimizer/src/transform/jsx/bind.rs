//! Bind directive helpers for JSX transformation.
//!
//! This module handles bind:value and bind:checked directive transformation.

use oxc_allocator::{Box as OxcBox, Vec as OxcVec};
use oxc_ast::ast::*;
use oxc_span::SPAN;

/// Check if attribute name is a bind directive.
/// Returns Some(true) for bind:checked, Some(false) for bind:value, None otherwise.
pub(crate) fn is_bind_directive(name: &str) -> Option<bool> {
    if name == "bind:value" {
        Some(false) // is_checked = false
    } else if name == "bind:checked" {
        Some(true) // is_checked = true
    } else {
        None
    }
}

/// Create inlinedQrl for bind handler.
/// inlinedQrl(_val, "_val", [signal]) for bind:value
/// inlinedQrl(_chk, "_chk", [signal]) for bind:checked
pub(crate) fn create_bind_handler<'b>(
    builder: &oxc_ast::AstBuilder<'b>,
    is_checked: bool,
    signal_expr: Expression<'b>,
) -> Expression<'b> {
    let helper = if is_checked { "_chk" } else { "_val" };

    builder.expression_call(
        SPAN,
        builder.expression_identifier(SPAN, "inlinedQrl"),
        None::<OxcBox<TSTypeParameterInstantiation<'b>>>,
        builder.vec_from_array([
            Argument::from(builder.expression_identifier(SPAN, helper)),
            Argument::from(builder.expression_string_literal(SPAN, helper, None)),
            Argument::from(builder.expression_array(
                SPAN,
                builder.vec1(ArrayExpressionElement::from(signal_expr)),
            )),
        ]),
        false,
    )
}

/// Merge bind handler with existing on:input handler.
/// Returns array expression: [existingHandler, bindHandler]
pub(crate) fn merge_event_handlers<'b>(
    builder: &oxc_ast::AstBuilder<'b>,
    existing: Expression<'b>,
    bind_handler: Expression<'b>,
) -> Expression<'b> {
    // If existing is already an array, add bind_handler to it
    match existing {
        Expression::ArrayExpression(arr) => {
            // Clone and add bind_handler
            use oxc_allocator::CloneIn;
            let mut elements: OxcVec<'b, ArrayExpressionElement<'b>> =
                builder.vec_with_capacity(arr.elements.len() + 1);
            for elem in arr.elements.iter() {
                elements.push(elem.clone_in(builder.allocator));
            }
            elements.push(ArrayExpressionElement::from(bind_handler));
            builder.expression_array(SPAN, elements)
        }
        _ => {
            // Create new array with both handlers
            builder.expression_array(
                SPAN,
                builder.vec_from_array([
                    ArrayExpressionElement::from(existing),
                    ArrayExpressionElement::from(bind_handler),
                ]),
            )
        }
    }
}

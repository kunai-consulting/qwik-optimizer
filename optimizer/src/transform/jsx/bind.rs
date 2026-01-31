use oxc_allocator::Vec as OxcVec;
use oxc_ast::ast::*;
use oxc_ast::NONE;
use oxc_span::SPAN;

pub(crate) fn is_bind_directive(name: &str) -> Option<bool> {
    if name == "bind:value" {
        Some(false)
    } else if name == "bind:checked" {
        Some(true)
    } else {
        None
    }
}

pub(crate) fn create_bind_handler<'b>(
    builder: &oxc_ast::AstBuilder<'b>,
    is_checked: bool,
    signal_expr: Expression<'b>,
) -> Expression<'b> {
    let helper = if is_checked { "_chk" } else { "_val" };

    builder.expression_call(
        SPAN,
        builder.expression_identifier(SPAN, "inlinedQrl"),
        NONE,
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

pub(crate) fn merge_event_handlers<'b>(
    builder: &oxc_ast::AstBuilder<'b>,
    existing: Expression<'b>,
    bind_handler: Expression<'b>,
) -> Expression<'b> {
    match existing {
        Expression::ArrayExpression(arr) => {
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

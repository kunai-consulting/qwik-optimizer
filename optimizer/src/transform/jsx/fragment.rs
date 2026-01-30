use oxc_allocator::{CloneIn, Vec as OxcVec};
use oxc_ast::ast::*;
use oxc_ast::NONE;
use oxc_span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::component::{Import, ImportId, QWIK_CORE_SOURCE};
use crate::transform::generator::TransformGenerator;
use crate::transform::state::JsxState;

use super::{JSX_RUNTIME_SOURCE, JSX_SORTED_NAME, _FRAGMENT};

pub fn enter_jsx_fragment<'a>(
    gen: &mut TransformGenerator<'a>,
    _node: &mut JSXFragment<'a>,
    ctx: &mut TraverseCtx<'a, ()>,
) {
    gen.jsx_stack.push(JsxState {
        is_fn: true,
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
        stacked_ctxt: false,
    });
    gen.debug("ENTER: JSXFragment", ctx);
}

pub fn exit_jsx_fragment<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &mut JSXFragment<'a>,
    ctx: &mut TraverseCtx<'a, ()>,
) {
    if let Some(mut jsx) = gen.jsx_stack.pop() {
        if gen.options.transpile_jsx {
            let children_arg: Expression<'a> = if jsx.children.len() == 1 {
                let child = jsx.children.pop().unwrap();
                if let Some(expr) = child.as_expression() {
                    expr.clone_in(gen.builder.allocator)
                } else if let ArrayExpressionElement::SpreadElement(spread) = child {
                    let mut children = OxcVec::new_in(gen.builder.allocator);
                    children.push(ArrayExpressionElement::SpreadElement(spread));
                    gen.builder.expression_array(node.span, children)
                } else {
                    gen.builder.expression_null_literal(node.span)
                }
            } else if jsx.children.is_empty() {
                gen.builder.expression_null_literal(node.span)
            } else {
                gen.builder.expression_array(node.span, jsx.children)
            };

            let key_arg: Expression<'a> = jsx.key_prop.unwrap_or_else(|| {
                if let Some(cmp) = gen.component_stack.last() {
                    let new_key = format!(
                        "{}_{}",
                        cmp.id.hash.chars().take(2).collect::<String>(),
                        gen.jsx_key_counter
                    );
                    gen.jsx_key_counter += 1;
                    gen.builder.expression_string_literal(
                        SPAN,
                        gen.builder.atom(&new_key),
                        None,
                    )
                } else {
                    gen.builder.expression_null_literal(SPAN)
                }
            });

            let flags = ((if jsx.static_listeners { 0b1 } else { 0 })
                | (if jsx.static_subtree { 0b10 } else { 0 })) as f64;

            let args: OxcVec<Argument<'a>> = OxcVec::from_array_in(
                [
                    gen.builder
                        .expression_identifier(node.span, _FRAGMENT)
                        .into(),
                    gen.builder.expression_null_literal(node.span).into(),
                    gen.builder.expression_null_literal(node.span).into(),
                    children_arg.into(),
                    gen.builder
                        .expression_numeric_literal(node.span, flags, None, NumberBase::Decimal)
                        .into(),
                    key_arg.into(),
                ],
                gen.builder.allocator,
            );

            gen.replace_expr = Some(gen.builder.expression_call_with_pure(
                node.span,
                gen.builder.expression_identifier(node.span, JSX_SORTED_NAME),
                NONE,
                args,
                false,
                true,
            ));

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

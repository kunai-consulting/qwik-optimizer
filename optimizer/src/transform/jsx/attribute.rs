use oxc_allocator::CloneIn;
use oxc_ast::ast::*;
use oxc_ast::NONE;
use oxc_ast_visit::Visit;
use oxc_span::{GetSpan, SPAN};
use oxc_traverse::TraverseCtx;

use crate::collector::IdentCollector;
use crate::component::{Import, Qrl, QrlType, MARKER_SUFFIX};
use crate::is_const::is_const_expr;
use crate::transform::generator::{IdentType, IdPlusType, TransformGenerator};
use crate::transform::qrl as qrl_module;

use super::bind::{create_bind_handler, is_bind_directive, merge_event_handlers};
use super::event::jsx_event_to_html_attribute;
use super::{get_jsx_attribute_full_name, move_expression, _GET_VAR_PROPS};

pub fn enter_jsx_attribute<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &mut JSXAttribute<'a>,
    ctx: &mut TraverseCtx<'a, ()>,
) {
    if gen.options.transpile_jsx {
        gen.expr_is_const_stack.push(
            gen.jsx_stack
                .last()
                .is_some_and(|jsx| !jsx.should_runtime_sort),
        );
    }
    gen.ascend();
    gen.debug("ENTER: JSXAttribute", ctx);
    let segment_name = match &node.name {
        JSXAttributeName::Identifier(id) => id.name.to_string(),
        JSXAttributeName::NamespacedName(ns) => ns.name.name.to_string(),
    };
    let segment = gen.new_segment(segment_name);
    gen.segment_stack.push(segment);

    let attr_name = get_jsx_attribute_full_name(&node.name);

    let is_native = gen.jsx_element_is_native.last().copied().unwrap_or(false);
    let stack_ctxt_name = if is_native {
        if let Some(html_attr) = jsx_event_to_html_attribute(&attr_name) {
            html_attr.to_string()
        } else {
            attr_name.clone()
        }
    } else {
        attr_name.clone()
    };
    gen.stack_ctxt.push(stack_ctxt_name);

    let is_native = gen.jsx_element_is_native.last().copied().unwrap_or(false);
    if is_native {
        if let Some(is_checked) = is_bind_directive(&attr_name) {
            if let Some(JSXAttributeValue::ExpressionContainer(container)) = &node.value {
                if let Some(expr) = container.expression.as_expression() {
                    gen.pending_bind_directives.push((
                        is_checked,
                        expr.clone_in(ctx.ast.allocator),
                    ));
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
                    gen.import_stack.push(std::collections::BTreeSet::new());
                }
            }
        }
    }
}

pub fn exit_jsx_attribute<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &mut JSXAttribute<'a>,
    ctx: &mut TraverseCtx<'a, ()>,
) {
    let attr_name = get_jsx_attribute_full_name(&node.name);
    let is_native = gen.jsx_element_is_native.last().copied().unwrap_or(false);

    if is_native && gen.options.transpile_jsx {
        if let Some(is_checked) = is_bind_directive(&attr_name) {
            if let Some(JSXAttributeValue::ExpressionContainer(container)) = &node.value {
                if let Some(expr) = container.expression.as_expression() {
                    let signal_expr = expr.clone_in(ctx.ast.allocator);
                    let prop_name = if is_checked { "checked" } else { "value" };

                    let bind_handler = create_bind_handler(
                        &ctx.ast,
                        is_checked,
                        signal_expr.clone_in(ctx.ast.allocator),
                    );

                    gen.expr_is_const_stack.pop();

                    if let Some(jsx) = gen.jsx_stack.last_mut() {
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

                        let existing_on_input_idx = jsx.const_props.iter().position(|prop| {
                            if let ObjectPropertyKind::ObjectProperty(obj_prop) = prop {
                                if let PropertyKey::StaticIdentifier(id) = &obj_prop.key {
                                    return id.name == "on:input";
                                }
                            }
                            false
                        });

                        if let Some(idx) = existing_on_input_idx {
                            if let ObjectPropertyKind::ObjectProperty(obj_prop) = &jsx.const_props[idx]
                            {
                                let existing_handler = obj_prop.value.clone_in(ctx.ast.allocator);
                                let merged = merge_event_handlers(
                                    &ctx.ast,
                                    existing_handler,
                                    bind_handler,
                                );

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

                    gen.segment_stack.pop();
                    gen.stack_ctxt.pop();
                    gen.debug("EXIT: JSXAttribute (bind directive)", ctx);
                    gen.descend();
                    return;
                }
            }
        }
    }

    if attr_name.ends_with(MARKER_SUFFIX) && is_native {
        if let Some(html_attr) = jsx_event_to_html_attribute(&attr_name) {
            let new_name = gen.builder.atom(&html_attr);
            node.name = JSXAttributeName::Identifier(gen.builder.alloc(JSXIdentifier {
                span: node.name.span(),
                name: new_name,
            }));
        }
    }

    if attr_name.ends_with(MARKER_SUFFIX) {
        if let Some(JSXAttributeValue::ExpressionContainer(container)) = &mut node.value {
            if let Some(expr) = container.expression.as_expression() {
                let is_fn = matches!(
                    expr,
                    Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
                );

                if is_fn {
                    let descendent_idents = {
                        let mut collector = IdentCollector::new();
                        collector.visit_expression(expr);
                        collector.get_words()
                    };

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
                        qrl_module::compute_scoped_idents(&descendent_idents, &decl_collect);

                    let imports = gen.import_stack.pop().unwrap_or_default();
                    let imports_vec: Vec<_> = imports.iter().cloned().collect();
                    let imported_names = qrl_module::collect_imported_names(&imports_vec);
                    let scoped_idents = qrl_module::filter_imported_from_scoped(scoped_idents, &imported_names);

                    let referenced_exports = qrl_module::collect_referenced_exports(
                        &descendent_idents,
                        &imported_names,
                        &scoped_idents,
                        &gen.export_by_name,
                    );

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

                    container.expression =
                        JSXExpression::from(Expression::CallExpression(ctx.ast.alloc(call_expr)));

                    if let Some(import_set) = gen.import_stack.last_mut() {
                        import_set.insert(Import::qrl());
                    }
                }
            }
        }
    }

    if gen.options.transpile_jsx {
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

        let stack_is_const = gen.expr_is_const_stack.pop().unwrap_or_default();
        let is_const = if let Some(JSXAttributeValue::ExpressionContainer(container)) = &node.value
        {
            if let Some(value_expr) = container.expression.as_expression() {
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
            stack_is_const
        };

        if let Some(jsx) = gen.jsx_stack.last_mut() {
            let expr: Expression<'a> = {
                let v = &mut node.value;
                match v {
                    None => gen.builder.expression_boolean_literal(node.span, true),
                    Some(JSXAttributeValue::Element(_)) => gen.replace_expr.take().unwrap(),
                    Some(JSXAttributeValue::Fragment(_)) => gen.replace_expr.take().unwrap(),
                    Some(JSXAttributeValue::StringLiteral(b)) => gen
                        .builder
                        .expression_string_literal((*b).span, (*b).value, Some((*b).value)),
                    Some(JSXAttributeValue::ExpressionContainer(b)) => {
                        let inner_expr = (*b).expression.to_expression_mut();
                        let span = inner_expr.span();

                        if let Some(prop_key) = &prop_wrap_key {
                            gen.needs_wrap_prop_import = true;
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
                        } else if needs_signal_wrap {
                            gen.needs_wrap_prop_import = true;
                            if let Expression::StaticMemberExpression(static_member) = inner_expr {
                                let signal_expr = static_member.object.clone_in(ctx.ast.allocator);
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
                let prop_name = get_jsx_attribute_full_name(&node.name);
                let prop_name_atom = gen.builder.atom(&prop_name);

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
                        if let ObjectPropertyKind::ObjectProperty(obj_prop) = &jsx.const_props[idx] {
                            let existing_handler = obj_prop.value.clone_in(ctx.ast.allocator);
                            let merged = merge_event_handlers(
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
    gen.stack_ctxt.pop();
    gen.debug("EXIT: JSXAttribute", ctx);
    gen.descend();
}

pub fn exit_jsx_spread_attribute<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &mut JSXSpreadAttribute<'a>,
    _ctx: &mut TraverseCtx<'a, ()>,
) {
    if !gen.options.transpile_jsx {
        return;
    }
    if let Some(jsx) = gen.jsx_stack.last_mut() {
        let range = 0..jsx.const_props.len();
        jsx.const_props
            .drain(range)
            .for_each(|p| jsx.var_props.push(p));
        jsx.should_runtime_sort = true;
        jsx.static_subtree = false;
        jsx.static_listeners = false;

        let spread_arg = move_expression(&gen.builder, &mut node.argument);
        jsx.spread_expr = Some(spread_arg.clone_in(gen.builder.allocator));

        let get_var_props_call = gen.builder.expression_call(
            node.span(),
            gen.builder.expression_identifier(node.span(), _GET_VAR_PROPS),
            NONE,
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

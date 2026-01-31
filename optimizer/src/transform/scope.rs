use oxc_ast::ast::*;
use oxc_traverse::TraverseCtx;

use super::generator::{IdentType, TransformGenerator};

pub fn enter_function<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &Function<'a>,
    ctx: &mut TraverseCtx<'a, ()>,
) {
    if let Some(name) = node.name() {
        if let Some(current_scope) = gen.decl_stack.last_mut() {
            let scope_id = ctx.current_scope_id();
            current_scope.push(((name.to_string(), scope_id), IdentType::Fn));
        }
        gen.stack_ctxt.push(name.to_string());
    }

    gen.decl_stack.push(Vec::new());

    if let Some(current_scope) = gen.decl_stack.last_mut() {
        for param in &node.params.items {
            if let Some(ident) = param.pattern.get_binding_identifier() {
                let scope_id = ctx.current_scope_id();
                current_scope.push(((ident.name.to_string(), scope_id), IdentType::Var(false)));
            }
        }
    }
}

pub fn exit_function<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &Function<'a>,
    _ctx: &mut TraverseCtx<'a, ()>,
) {
    gen.decl_stack.pop();

    if node.name().is_some() {
        gen.stack_ctxt.pop();
    }
}

pub fn enter_class<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &Class<'a>,
    ctx: &mut TraverseCtx<'a, ()>,
) {
    if let Some(ident) = &node.id {
        if let Some(current_scope) = gen.decl_stack.last_mut() {
            let scope_id = ctx.current_scope_id();
            current_scope.push(((ident.name.to_string(), scope_id), IdentType::Class));
        }
        gen.stack_ctxt.push(ident.name.to_string());
    }

    gen.decl_stack.push(Vec::new());
}

pub fn exit_class<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &Class<'a>,
    _ctx: &mut TraverseCtx<'a, ()>,
) {
    gen.decl_stack.pop();

    if node.id.is_some() {
        gen.stack_ctxt.pop();
    }
}

pub fn enter_arrow_function_expression<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &ArrowFunctionExpression<'a>,
    ctx: &mut TraverseCtx<'a, ()>,
) {
    gen.decl_stack.push(Vec::new());

    if let Some(current_scope) = gen.decl_stack.last_mut() {
        for param in &node.params.items {
            if let Some(ident) = param.pattern.get_binding_identifier() {
                let scope_id = ctx.current_scope_id();
                current_scope.push(((ident.name.to_string(), scope_id), IdentType::Var(false)));
            }
        }
    }
}

pub fn exit_arrow_function_expression<'a>(
    gen: &mut TransformGenerator<'a>,
    _node: &ArrowFunctionExpression<'a>,
    _ctx: &mut TraverseCtx<'a, ()>,
) {
    gen.decl_stack.pop();
}

pub fn track_variable_declaration<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &VariableDeclarator<'a>,
    ctx: &mut TraverseCtx<'a, ()>,
) {
    let is_const = node.kind == VariableDeclarationKind::Const;

    if let Some(current_scope) = gen.decl_stack.last_mut() {
        if let Some(ident) = node.id.get_binding_identifier() {
            let scope_id = ctx.current_scope_id();
            current_scope.push(((ident.name.to_string(), scope_id), IdentType::Var(is_const)));
        }
    }
}

pub fn check_map_iteration_vars(node: &oxc_ast::ast::CallExpression) -> Option<Vec<crate::collector::Id>> {
    use oxc_ast::ast::Expression;

    if let Some(member) = node.callee.as_member_expression() {
        if member.static_property_name() == Some("map") {
            if let Some(arg) = node.arguments.first() {
                if let Some(expr) = arg.as_expression() {
                    let iteration_vars = match expr {
                        Expression::ArrowFunctionExpression(arrow) => {
                            let mut vars = Vec::new();
                            for param in arrow.params.items.iter() {
                                if let Some(ident) = param.pattern.get_binding_identifier() {
                                    vars.push((
                                        ident.name.to_string(),
                                        oxc_semantic::ScopeId::new(0),
                                    ));
                                }
                            }
                            Some(vars)
                        }
                        Expression::FunctionExpression(func) => {
                            let mut vars = Vec::new();
                            for param in &func.params.items {
                                if let Some(ident) = param.pattern.get_binding_identifier() {
                                    vars.push((
                                        ident.name.to_string(),
                                        oxc_semantic::ScopeId::new(0),
                                    ));
                                }
                            }
                            Some(vars)
                        }
                        _ => None,
                    };
                    return iteration_vars;
                }
            }
        }
    }
    None
}

pub fn is_map_with_function_callback(node: &oxc_ast::ast::CallExpression) -> bool {
    use oxc_ast::ast::Expression;

    if let Some(member) = node.callee.as_member_expression() {
        if member.static_property_name() == Some("map") {
            if let Some(arg) = node.arguments.first() {
                if let Some(expr) = arg.as_expression() {
                    return matches!(
                        expr,
                        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
                    );
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
}

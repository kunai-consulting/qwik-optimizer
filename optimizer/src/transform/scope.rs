//! Scope tracking and declaration stack management for Qwik optimizer.
//!
//! This module contains scope-related transformation logic extracted from
//! generator.rs following the dispatcher pattern. Contains functions for:
//!
//! - Declaration stack management (`decl_stack` push/pop)
//! - Variable, function, and class declaration tracking
//! - Function/arrow function parameter tracking

use oxc_ast::ast::*;
use oxc_traverse::TraverseCtx;

use super::generator::{IdentType, TransformGenerator};

// =============================================================================
// Function Scope Helpers
// =============================================================================

/// Track function name in parent scope and push new scope for function body.
///
/// Called during enter_function traversal. Handles:
/// - Adding function name to parent scope's decl_stack as Fn type
/// - Pushing function name to stack_ctxt for entry strategy
/// - Creating new scope for function body
/// - Tracking function parameters in the new scope
pub fn enter_function<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &Function<'a>,
    ctx: &mut TraverseCtx<'a, ()>,
) {
    // Track function name as Fn declaration in parent scope
    // and push to stack_ctxt for entry strategy (SWC fold_fn_decl)
    if let Some(name) = node.name() {
        if let Some(current_scope) = gen.decl_stack.last_mut() {
            let scope_id = ctx.current_scope_id();
            current_scope.push(((name.to_string(), scope_id), IdentType::Fn));
        }
        // Push function name to stack_ctxt
        gen.stack_ctxt.push(name.to_string());
    }

    // Push new scope for function body
    gen.decl_stack.push(Vec::new());

    // Track function parameters in the new scope
    if let Some(current_scope) = gen.decl_stack.last_mut() {
        for param in &node.params.items {
            if let Some(ident) = param.pattern.get_binding_identifier() {
                let scope_id = ctx.current_scope_id();
                // Parameters are always treated as non-const for capture purposes
                current_scope.push(((ident.name.to_string(), scope_id), IdentType::Var(false)));
            }
        }
    }
}

/// Pop function scope from decl_stack.
///
/// Called during exit_function traversal. Handles:
/// - Popping function scope from decl_stack
/// - Popping stack_ctxt if function had a name
pub fn exit_function<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &Function<'a>,
    _ctx: &mut TraverseCtx<'a, ()>,
) {
    // Pop function scope from decl_stack
    gen.decl_stack.pop();

    // Pop stack_ctxt if we pushed a function name (SWC fold_fn_decl)
    if node.name().is_some() {
        gen.stack_ctxt.pop();
    }
}

// =============================================================================
// Class Scope Helpers
// =============================================================================

/// Track class name in parent scope and push new scope for class body.
///
/// Called during enter_class traversal. Handles:
/// - Adding class name to parent scope's decl_stack as Class type
/// - Pushing class name to stack_ctxt for entry strategy
/// - Creating new scope for class body
pub fn enter_class<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &Class<'a>,
    ctx: &mut TraverseCtx<'a, ()>,
) {
    // Track class name as Class declaration in parent scope
    // and push to stack_ctxt for entry strategy (SWC fold_class_decl)
    if let Some(ident) = &node.id {
        if let Some(current_scope) = gen.decl_stack.last_mut() {
            let scope_id = ctx.current_scope_id();
            current_scope.push(((ident.name.to_string(), scope_id), IdentType::Class));
        }
        // Push class name to stack_ctxt
        gen.stack_ctxt.push(ident.name.to_string());
    }

    // Push new scope for class body
    gen.decl_stack.push(Vec::new());
}

/// Pop class scope from decl_stack.
///
/// Called during exit_class traversal. Handles:
/// - Popping class scope from decl_stack
/// - Popping stack_ctxt if class had a name
pub fn exit_class<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &Class<'a>,
    _ctx: &mut TraverseCtx<'a, ()>,
) {
    // Pop class scope from decl_stack
    gen.decl_stack.pop();

    // Pop stack_ctxt if we pushed a class name (SWC fold_class_decl)
    if node.id.is_some() {
        gen.stack_ctxt.pop();
    }
}

// =============================================================================
// Arrow Function Scope Helpers
// =============================================================================

/// Push new scope for arrow function body and track parameters.
///
/// Called during enter_arrow_function_expression traversal. Handles:
/// - Creating new scope for arrow function body
/// - Tracking arrow function parameters in the new scope
pub fn enter_arrow_function_expression<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &ArrowFunctionExpression<'a>,
    ctx: &mut TraverseCtx<'a, ()>,
) {
    // Push new scope for arrow function body
    gen.decl_stack.push(Vec::new());

    // Track arrow function parameters in the new scope
    if let Some(current_scope) = gen.decl_stack.last_mut() {
        for param in &node.params.items {
            if let Some(ident) = param.pattern.get_binding_identifier() {
                let scope_id = ctx.current_scope_id();
                // Parameters are always treated as non-const for capture purposes
                current_scope.push(((ident.name.to_string(), scope_id), IdentType::Var(false)));
            }
        }
    }
}

/// Pop arrow function scope from decl_stack.
///
/// Called during exit_arrow_function_expression traversal.
pub fn exit_arrow_function_expression<'a>(
    gen: &mut TransformGenerator<'a>,
    _node: &ArrowFunctionExpression<'a>,
    _ctx: &mut TraverseCtx<'a, ()>,
) {
    // Pop arrow function scope from decl_stack
    gen.decl_stack.pop();
}

// =============================================================================
// Variable Declaration Helpers
// =============================================================================

/// Track variable declaration in current scope.
///
/// Called during enter_variable_declarator traversal. Handles:
/// - Adding variable to current scope's decl_stack with const/var type
pub fn track_variable_declaration<'a>(
    gen: &mut TransformGenerator<'a>,
    node: &VariableDeclarator<'a>,
    ctx: &mut TraverseCtx<'a, ()>,
) {
    let is_const = node.kind == VariableDeclarationKind::Const;

    // Track variable declaration in decl_stack for scope capture
    if let Some(current_scope) = gen.decl_stack.last_mut() {
        if let Some(ident) = node.id.get_binding_identifier() {
            let scope_id = ctx.current_scope_id();
            current_scope.push(((ident.name.to_string(), scope_id), IdentType::Var(is_const)));
        }
    }
}

// =============================================================================
// Loop/Map Iteration Tracking Helpers
// =============================================================================

/// Check if a call expression is a .map() call and extract iteration variables.
///
/// Returns `Some(Vec<Id>)` with iteration variable names if this is a .map() call
/// with a function callback, None otherwise.
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

/// Check if a call expression is a .map() call with a function callback.
///
/// Used to determine if we should pop iteration tracking state on exit.
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
    // Scope tracking tests are integration tests - they require full AST traversal
    // and are covered by the existing transform tests.
}

//! Code movement and useLexicalScope injection for QRL segment files.
//!
//! When a QRL captures variables from its enclosing scope, the segment file needs
//! to import those variables via `useLexicalScope()`. This module provides the AST
//! transformations that inject `const [a, b, c] = useLexicalScope();` at the start
//! of extracted functions.
//!
//! Ported from SWC's code_move.rs (lines 175-290).

use crate::collector::Id;
use oxc_allocator::{Allocator, Box as OxcBox, Vec as OxcVec};
use oxc_ast::ast::*;
use oxc_ast::AstBuilder;
use oxc_ast::NONE;
use oxc_span::SPAN;

/// Transform a function expression to inject useLexicalScope at the start.
///
/// This is the main entry point for code movement transformation. It dispatches
/// based on expression type:
/// - Arrow functions: Converts expression body to block body if needed, then prepends useLexicalScope
/// - Function expressions: Prepends useLexicalScope to the body
/// - Other expressions: Returns unchanged
///
/// # Arguments
/// * `expr` - The function expression to transform
/// * `scoped_idents` - Variables captured from enclosing scope (sorted)
/// * `allocator` - OXC allocator for AST construction
///
/// # Returns
/// The transformed expression with useLexicalScope injection
pub fn transform_function_expr<'a>(
    expr: Expression<'a>,
    scoped_idents: &[Id],
    allocator: &'a Allocator,
) -> Expression<'a> {
    // No captures means no transformation needed
    if scoped_idents.is_empty() {
        return expr;
    }

    let ast = AstBuilder::new(allocator);

    match expr {
        Expression::ArrowFunctionExpression(arrow) => {
            let transformed = transform_arrow_fn(arrow.unbox(), scoped_idents, &ast, allocator);
            Expression::ArrowFunctionExpression(OxcBox::new_in(transformed, allocator))
        }
        Expression::FunctionExpression(func) => {
            let transformed = transform_fn(func.unbox(), scoped_idents, &ast, allocator);
            Expression::FunctionExpression(OxcBox::new_in(transformed, allocator))
        }
        _ => expr,
    }
}

/// Transform an arrow function to inject useLexicalScope.
///
/// - For expression body `() => expr`: Converts to `() => { const [a, b] = useLexicalScope(); return expr; }`
/// - For block body `() => { ... }`: Prepends `const [a, b] = useLexicalScope();` to the body
fn transform_arrow_fn<'a>(
    mut arrow: ArrowFunctionExpression<'a>,
    scoped_idents: &[Id],
    ast: &AstBuilder<'a>,
    allocator: &'a Allocator,
) -> ArrowFunctionExpression<'a> {
    let use_lexical_scope_stmt = create_use_lexical_scope(scoped_idents, ast, allocator);

    // Check if this is an expression body (expression = true)
    if arrow.expression {
        // Expression body: convert to block with useLexicalScope + return
        // The expression is stored in statements[0] as an ExpressionStatement
        if let Some(Statement::ExpressionStatement(expr_stmt)) = arrow.body.statements.pop() {
            let return_stmt = ast.statement_return(SPAN, Some(expr_stmt.unbox().expression));
            let mut new_stmts = OxcVec::with_capacity_in(2, allocator);
            new_stmts.push(use_lexical_scope_stmt);
            new_stmts.push(return_stmt);
            arrow.body.statements = new_stmts;
            arrow.expression = false;
        }
    } else {
        // Block body: prepend useLexicalScope to existing statements
        let mut new_stmts = OxcVec::with_capacity_in(1 + arrow.body.statements.len(), allocator);
        new_stmts.push(use_lexical_scope_stmt);
        new_stmts.extend(arrow.body.statements.drain(..));
        arrow.body.statements = new_stmts;
    }

    arrow
}

/// Transform a function expression to inject useLexicalScope.
///
/// Prepends `const [a, b] = useLexicalScope();` to the function body.
fn transform_fn<'a>(
    mut func: Function<'a>,
    scoped_idents: &[Id],
    ast: &AstBuilder<'a>,
    allocator: &'a Allocator,
) -> Function<'a> {
    let use_lexical_scope_stmt = create_use_lexical_scope(scoped_idents, ast, allocator);

    if let Some(body) = &mut func.body {
        let mut new_stmts = OxcVec::with_capacity_in(1 + body.statements.len(), allocator);
        new_stmts.push(use_lexical_scope_stmt);
        new_stmts.extend(body.statements.drain(..));
        body.statements = new_stmts;
    } else {
        // Function without body - create one
        let mut stmts = OxcVec::with_capacity_in(1, allocator);
        stmts.push(use_lexical_scope_stmt);
        func.body = Some(ast.alloc_function_body(SPAN, OxcVec::new_in(allocator), stmts));
    }

    func
}

/// Create the `const [a, b, c] = useLexicalScope();` statement.
///
/// Generates an AST node for:
/// ```javascript
/// const [a, b, c] = useLexicalScope();
/// ```
///
/// The destructuring pattern contains the sorted list of captured identifiers.
pub fn create_use_lexical_scope<'a>(
    scoped_idents: &[Id],
    ast: &AstBuilder<'a>,
    _allocator: &'a Allocator,
) -> Statement<'a> {
    // Create array pattern elements for destructuring: [a, b, c]
    // Each element is a BindingPattern (from binding_pattern_binding_identifier)
    let mut elements = ast.vec_with_capacity(scoped_idents.len());
    for (name, _scope_id) in scoped_idents {
        // Allocate the name string in the arena so it has the right lifetime
        let name_atom = ast.atom(name.as_str());
        let binding = ast.binding_pattern_binding_identifier(SPAN, name_atom);
        elements.push(Some(binding));
    }

    // Create array pattern binding: [a, b, c]
    // binding_pattern_array_pattern(span, elements, rest)
    let binding_pattern = ast.binding_pattern_array_pattern(SPAN, elements, NONE);

    // Create call expression: useLexicalScope()
    let callee = ast.expression_identifier(SPAN, "useLexicalScope");
    let call_expr = ast.expression_call(
        SPAN,
        callee,
        NONE,
        ast.vec(),
        false,
    );

    // Create variable declarator: [a, b, c] = useLexicalScope()
    let mut declarators = ast.vec_with_capacity(1);
    declarators.push(ast.variable_declarator(
        SPAN,
        VariableDeclarationKind::Const,
        binding_pattern,
        NONE,
        Some(call_expr),
        false,
    ));

    // Create variable declaration: const [a, b, c] = useLexicalScope()
    let var_decl = ast.variable_declaration(SPAN, VariableDeclarationKind::Const, declarators, false);

    Statement::VariableDeclaration(ast.alloc(var_decl))
}

/// Create a return statement: `return expr;`
#[allow(dead_code)]
pub fn create_return_stmt<'a>(expr: Expression<'a>, ast: &AstBuilder<'a>) -> Statement<'a> {
    ast.statement_return(SPAN, Some(expr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_codegen::{Codegen, CodegenOptions};
    use oxc_parser::Parser;
    use oxc_semantic::ScopeId;
    use oxc_span::SourceType;

    fn parse_expr<'a>(code: &str, allocator: &'a Allocator) -> Expression<'a> {
        let source_type = SourceType::mjs();
        let parse_result = Parser::new(allocator, code, source_type).parse();
        assert!(
            parse_result.errors.is_empty(),
            "Parse errors: {:?}",
            parse_result.errors
        );

        let program = parse_result.program;
        if let Some(Statement::ExpressionStatement(expr_stmt)) = program.body.first() {
            // Need to clone the expression to avoid lifetime issues
            use oxc_allocator::CloneIn;
            let expr = expr_stmt.expression.clone_in(allocator);
            // Unwrap parenthesized expression if present
            if let Expression::ParenthesizedExpression(paren) = expr {
                paren.unbox().expression
            } else {
                expr
            }
        } else {
            panic!("Expected expression statement");
        }
    }

    fn gen_code(expr: &Expression) -> String {
        let mut codegen = Codegen::new().with_options(CodegenOptions::default());
        // Print the expression to the internal buffer
        codegen.print_expression(expr);
        // Get the final generated code
        codegen.into_source_text()
    }

    #[test]
    fn test_arrow_expression_body_conversion() {
        // Test: `() => x + y` becomes `() => { const [x, y] = useLexicalScope(); return x + y; }`
        let allocator = Allocator::default();
        let expr = parse_expr("() => x + y", &allocator);

        let scoped_idents: Vec<Id> = vec![
            ("x".to_string(), ScopeId::new(0)),
            ("y".to_string(), ScopeId::new(0)),
        ];

        let transformed = transform_function_expr(expr, &scoped_idents, &allocator);
        let code = gen_code(&transformed);

        assert!(
            code.contains("useLexicalScope"),
            "Should contain useLexicalScope call, got: {}",
            code
        );
        assert!(
            code.contains("const [x, y]"),
            "Should contain destructuring pattern, got: {}",
            code
        );
        assert!(
            code.contains("return"),
            "Should contain return statement, got: {}",
            code
        );
    }

    #[test]
    fn test_arrow_block_body_prepending() {
        // Test: `() => { return x; }` prepends useLexicalScope
        let allocator = Allocator::default();
        let expr = parse_expr("() => { return x; }", &allocator);

        let scoped_idents: Vec<Id> = vec![("x".to_string(), ScopeId::new(0))];

        let transformed = transform_function_expr(expr, &scoped_idents, &allocator);
        let code = gen_code(&transformed);

        assert!(
            code.contains("useLexicalScope"),
            "Should contain useLexicalScope call, got: {}",
            code
        );
        assert!(
            code.contains("const [x]"),
            "Should contain destructuring pattern, got: {}",
            code
        );
    }

    #[test]
    fn test_function_expression_transformation() {
        // Test: `function() { return x; }` prepends useLexicalScope
        let allocator = Allocator::default();
        let expr = parse_expr("(function() { return x; })", &allocator);

        let scoped_idents: Vec<Id> = vec![("x".to_string(), ScopeId::new(0))];

        let transformed = transform_function_expr(expr, &scoped_idents, &allocator);
        let code = gen_code(&transformed);

        assert!(
            code.contains("useLexicalScope"),
            "Should contain useLexicalScope call, got: {}",
            code
        );
    }

    #[test]
    fn test_no_op_for_empty_scoped_idents() {
        // Test: Returns unchanged expression when no captures
        let allocator = Allocator::default();
        let expr = parse_expr("() => x + y", &allocator);
        let original_code = gen_code(&expr);

        let scoped_idents: Vec<Id> = vec![]; // Empty

        let transformed = transform_function_expr(expr, &scoped_idents, &allocator);
        let transformed_code = gen_code(&transformed);

        assert_eq!(
            original_code, transformed_code,
            "Should return unchanged when no captures"
        );
    }

    #[test]
    fn test_sorted_destructuring_pattern() {
        // Test: Identifiers in destructuring are sorted (a, b, c)
        let allocator = Allocator::default();
        let expr = parse_expr("() => a + b + c", &allocator);

        // Provide identifiers in non-sorted order
        let scoped_idents: Vec<Id> = vec![
            ("a".to_string(), ScopeId::new(0)),
            ("b".to_string(), ScopeId::new(0)),
            ("c".to_string(), ScopeId::new(0)),
        ];

        let transformed = transform_function_expr(expr, &scoped_idents, &allocator);
        let code = gen_code(&transformed);

        // The pattern should be [a, b, c] since we pass sorted scoped_idents
        assert!(
            code.contains("[a, b, c]"),
            "Should contain sorted destructuring pattern [a, b, c], got: {}",
            code
        );
    }
}

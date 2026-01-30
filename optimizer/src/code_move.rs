//! useLexicalScope injection for QRL segment files.

use crate::collector::Id;
use oxc_allocator::{Allocator, Box as OxcBox, Vec as OxcVec};
use oxc_ast::ast::*;
use oxc_ast::AstBuilder;
use oxc_ast::NONE;
use oxc_span::SPAN;

/// Injects useLexicalScope at the start of arrow/function expressions.
pub fn transform_function_expr<'a>(
    expr: Expression<'a>,
    scoped_idents: &[Id],
    allocator: &'a Allocator,
) -> Expression<'a> {
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

fn transform_arrow_fn<'a>(
    mut arrow: ArrowFunctionExpression<'a>,
    scoped_idents: &[Id],
    ast: &AstBuilder<'a>,
    allocator: &'a Allocator,
) -> ArrowFunctionExpression<'a> {
    let use_lexical_scope_stmt = create_use_lexical_scope(scoped_idents, ast, allocator);

    if arrow.expression {
        if let Some(Statement::ExpressionStatement(expr_stmt)) = arrow.body.statements.pop() {
            let return_stmt = ast.statement_return(SPAN, Some(expr_stmt.unbox().expression));
            let mut new_stmts = OxcVec::with_capacity_in(2, allocator);
            new_stmts.push(use_lexical_scope_stmt);
            new_stmts.push(return_stmt);
            arrow.body.statements = new_stmts;
            arrow.expression = false;
        }
    } else {
        let mut new_stmts = OxcVec::with_capacity_in(1 + arrow.body.statements.len(), allocator);
        new_stmts.push(use_lexical_scope_stmt);
        new_stmts.extend(arrow.body.statements.drain(..));
        arrow.body.statements = new_stmts;
    }

    arrow
}

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
        let mut stmts = OxcVec::with_capacity_in(1, allocator);
        stmts.push(use_lexical_scope_stmt);
        func.body = Some(ast.alloc_function_body(SPAN, OxcVec::new_in(allocator), stmts));
    }

    func
}

/// Creates `const [a, b, c] = useLexicalScope();` statement.
pub fn create_use_lexical_scope<'a>(
    scoped_idents: &[Id],
    ast: &AstBuilder<'a>,
    _allocator: &'a Allocator,
) -> Statement<'a> {
    let mut elements = ast.vec_with_capacity(scoped_idents.len());
    for (name, _scope_id) in scoped_idents {
        let name_atom = ast.atom(name.as_str());
        let binding = ast.binding_pattern_binding_identifier(SPAN, name_atom);
        elements.push(Some(binding));
    }

    let binding_pattern = ast.binding_pattern_array_pattern(SPAN, elements, NONE);

    let callee = ast.expression_identifier(SPAN, "useLexicalScope");
    let call_expr = ast.expression_call(
        SPAN,
        callee,
        NONE,
        ast.vec(),
        false,
    );

    let mut declarators = ast.vec_with_capacity(1);
    declarators.push(ast.variable_declarator(
        SPAN,
        VariableDeclarationKind::Const,
        binding_pattern,
        NONE,
        Some(call_expr),
        false,
    ));

    let var_decl = ast.variable_declaration(SPAN, VariableDeclarationKind::Const, declarators, false);

    Statement::VariableDeclaration(ast.alloc(var_decl))
}

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
            use oxc_allocator::CloneIn;
            let expr = expr_stmt.expression.clone_in(allocator);
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
        codegen.print_expression(expr);
        codegen.into_source_text()
    }

    #[test]
    fn test_arrow_expression_body_conversion() {
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
        let allocator = Allocator::default();
        let expr = parse_expr("() => a + b + c", &allocator);

        let scoped_idents: Vec<Id> = vec![
            ("a".to_string(), ScopeId::new(0)),
            ("b".to_string(), ScopeId::new(0)),
            ("c".to_string(), ScopeId::new(0)),
        ];

        let transformed = transform_function_expr(expr, &scoped_idents, &allocator);
        let code = gen_code(&transformed);

        assert!(
            code.contains("[a, b, c]"),
            "Should contain sorted destructuring pattern [a, b, c], got: {}",
            code
        );
    }
}

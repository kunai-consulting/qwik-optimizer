//! Prop constness detection for JSX transformation.

use crate::transform::{IdPlusType, IdentType};
use oxc_ast::ast;
use oxc_ast_visit::Visit;
use oxc_semantic::ScopeFlags;

/// Returns true if expression is constant and can be hoisted to constProps.
pub fn is_const_expr(
    expr: &ast::Expression<'_>,
    import_names: &std::collections::HashSet<String>,
    decl_stack: &[Vec<IdPlusType>],
) -> bool {
    let mut collector = ConstCollector::new(import_names, decl_stack);
    collector.visit_expression(expr);
    collector.is_const
}

struct ConstCollector<'a> {
    import_names: &'a std::collections::HashSet<String>,
    decl_stack: &'a [Vec<IdPlusType>],
    pub is_const: bool,
}

impl<'a> ConstCollector<'a> {
    fn new(
        import_names: &'a std::collections::HashSet<String>,
        decl_stack: &'a [Vec<IdPlusType>],
    ) -> Self {
        Self {
            import_names,
            decl_stack,
            is_const: true,
        }
    }

    fn is_const_var(&self, name: &str) -> bool {
        for scope in self.decl_stack.iter() {
            for (id, ident_type) in scope.iter() {
                if id.0 == name {
                    if let IdentType::Var(true) = ident_type {
                        return true;
                    }
                    return false;
                }
            }
        }
        false
    }
}

impl<'b> Visit<'b> for ConstCollector<'_> {
    fn visit_call_expression(&mut self, _node: &ast::CallExpression<'b>) {
        self.is_const = false;
    }

    fn visit_member_expression(&mut self, _node: &ast::MemberExpression<'b>) {
        self.is_const = false;
    }

    fn visit_arrow_function_expression(&mut self, _node: &ast::ArrowFunctionExpression<'b>) {
    }

    fn visit_function(&mut self, _node: &ast::Function<'b>, _flags: ScopeFlags) {
    }

    fn visit_identifier_reference(&mut self, node: &ast::IdentifierReference<'b>) {
        let name = node.name.as_str();

        if self.import_names.contains(name) {
            return;
        }

        if self.is_const_var(name) {
            return;
        }

        self.is_const = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_semantic::ScopeId;
    use oxc_span::SourceType;
    use std::collections::HashSet;

    fn check_const(code: &str, imports: &[&str], const_vars: &[&str]) -> bool {
        let allocator = Allocator::default();
        let source_type = SourceType::tsx();
        let wrapped = format!("({})", code);
        let parse_result = Parser::new(&allocator, &wrapped, source_type).parse();
        assert!(
            parse_result.errors.is_empty(),
            "Parse errors: {:?}",
            parse_result.errors
        );

        let stmt = &parse_result.program.body[0];
        let expr = match stmt {
            ast::Statement::ExpressionStatement(es) => &es.expression,
            _ => panic!("Expected expression statement"),
        };

        let import_names: HashSet<String> = imports.iter().map(|s| s.to_string()).collect();
        let decl_stack: Vec<Vec<IdPlusType>> = vec![const_vars
            .iter()
            .map(|name| ((name.to_string(), ScopeId::new(0)), IdentType::Var(true)))
            .collect()];

        is_const_expr(expr, &import_names, &decl_stack)
    }

    #[test]
    fn test_literal_is_const() {
        assert!(check_const("\"hello\"", &[], &[]));
        assert!(check_const("42", &[], &[]));
        assert!(check_const("true", &[], &[]));
    }

    #[test]
    fn test_call_expression_is_var() {
        assert!(!check_const("fn()", &[], &[]));
        assert!(!check_const("getData()", &["getData"], &[]));
    }

    #[test]
    fn test_member_expression_is_var() {
        assert!(!check_const("obj.prop", &[], &[]));
        assert!(!check_const("arr[0]", &[], &[]));
    }

    #[test]
    fn test_import_reference_is_const() {
        assert!(check_const("Component", &["Component"], &[]));
        assert!(check_const("importedValue", &["importedValue"], &[]));
    }

    #[test]
    fn test_const_var_reference_is_const() {
        assert!(check_const("localConst", &[], &["localConst"]));
    }

    #[test]
    fn test_unknown_var_is_not_const() {
        assert!(!check_const("unknownVar", &[], &[]));
    }

    #[test]
    fn test_arrow_function_is_const() {
        assert!(check_const("() => unknownVar", &[], &[]));
    }

    #[test]
    fn test_template_literal_with_var_is_not_const() {
        assert!(!check_const("`hello ${name}`", &[], &[]));
        assert!(check_const("`hello ${name}`", &[], &["name"]));
    }

    #[test]
    fn test_binary_expression_propagates() {
        assert!(check_const("a + b", &["a"], &["b"]));
        assert!(!check_const("a + b", &["a"], &[]));
    }
}

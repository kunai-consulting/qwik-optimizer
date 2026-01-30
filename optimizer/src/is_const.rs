//! Prop constness detection for JSX transformation.
//!
//! This module provides `is_const_expr` which determines whether a JSX prop
//! value expression is constant (can be hoisted) or variable (must be evaluated
//! at runtime). This follows the SWC optimizer's ConstCollector pattern.
//!
//! A prop is considered variable if it:
//! - Calls a function
//! - Accesses a member (obj.prop or obj\[key\])
//! - References a variable that is not an import, export, or const variable

use crate::transform::{IdPlusType, IdentType};
use oxc_ast::ast;
use oxc_ast_visit::Visit;
use oxc_semantic::ScopeFlags;

/// Check if an expression is constant (can be hoisted to constProps).
///
/// # Arguments
/// * `expr` - The expression to analyze
/// * `import_by_symbol` - Map of imported symbols for import checking
/// * `decl_stack` - Stack of declaration scopes for const variable checking
///
/// # Returns
/// `true` if the expression is constant, `false` if it's variable
pub fn is_const_expr(
    expr: &ast::Expression<'_>,
    import_names: &std::collections::HashSet<String>,
    decl_stack: &[Vec<IdPlusType>],
) -> bool {
    let mut collector = ConstCollector::new(import_names, decl_stack);
    collector.visit_expression(expr);
    collector.is_const
}

/// Collects constness information while visiting an expression AST.
struct ConstCollector<'a> {
    /// Set of imported symbol names
    import_names: &'a std::collections::HashSet<String>,
    /// Stack of declaration scopes for const variable checking
    decl_stack: &'a [Vec<IdPlusType>],
    /// Result: true if expression is const, false if variable
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

    /// Check if an identifier is a const variable in the declaration stack
    fn is_const_var(&self, name: &str) -> bool {
        for scope in self.decl_stack.iter() {
            for (id, ident_type) in scope.iter() {
                // Match by name only (consistent with compute_scoped_idents)
                if id.0 == name {
                    // Check if it's a const variable
                    if let IdentType::Var(true) = ident_type {
                        return true;
                    }
                    // Found but not const - still return false
                    return false;
                }
            }
        }
        false
    }
}

impl<'b> Visit<'b> for ConstCollector<'_> {
    fn visit_call_expression(&mut self, _node: &ast::CallExpression<'b>) {
        // Function calls make the expression variable
        self.is_const = false;
    }

    fn visit_member_expression(&mut self, _node: &ast::MemberExpression<'b>) {
        // Member access makes the expression variable
        self.is_const = false;
    }

    fn visit_arrow_function_expression(&mut self, _node: &ast::ArrowFunctionExpression<'b>) {
        // Don't recurse into arrow functions - they're self-contained
        // The arrow function itself is const if all its captured variables are const
    }

    fn visit_function(&mut self, _node: &ast::Function<'b>, _flags: ScopeFlags) {
        // Don't recurse into function expressions - they're self-contained
    }

    fn visit_identifier_reference(&mut self, node: &ast::IdentifierReference<'b>) {
        let name = node.name.as_str();

        // Check if it's an import - imports are const
        if self.import_names.contains(name) {
            return;
        }

        // Check if it's a const variable in scope
        if self.is_const_var(name) {
            return;
        }

        // Unknown identifier reference - not const
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
        // Wrap in expression statement for parsing
        let wrapped = format!("({})", code);
        let parse_result = Parser::new(&allocator, &wrapped, source_type).parse();
        assert!(
            parse_result.errors.is_empty(),
            "Parse errors: {:?}",
            parse_result.errors
        );

        // Extract the expression from the program
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
        // Arrow functions themselves are const (don't recurse into body)
        assert!(check_const("() => unknownVar", &[], &[]));
    }

    #[test]
    fn test_template_literal_with_var_is_not_const() {
        // Template with unknown identifier
        assert!(!check_const("`hello ${name}`", &[], &[]));
        // Template with const var
        assert!(check_const("`hello ${name}`", &[], &["name"]));
    }

    #[test]
    fn test_binary_expression_propagates() {
        // All parts const
        assert!(check_const("a + b", &["a"], &["b"]));
        // One part is unknown
        assert!(!check_const("a + b", &["a"], &[]));
    }
}

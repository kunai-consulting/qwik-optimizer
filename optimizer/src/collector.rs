//! Identifier collector for QRL variable usage analysis.
//!
//! This module provides `IdentCollector` which traverses AST to identify
//! which variables are referenced within QRL function bodies. This is
//! essential for generating the `[captures]` array in `qrl()` calls.

use std::collections::HashSet;

use oxc_ast::ast;
use oxc_ast_visit::Visit;
use oxc_semantic::ScopeId;

/// Identifier type for OXC - (name, scope_id)
/// Similar to SWC's `(Atom, SyntaxContext)` pattern
pub type Id = (String, ScopeId);

/// Context for tracking whether we're in an expression or should skip collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExprOrSkip {
    Expr,
    Skip,
}

/// Collects all identifiers while visiting the AST.
///
/// Used to determine which variables are referenced in QRL expressions,
/// enabling proper capture array generation.
#[derive(Debug)]
pub struct IdentCollector {
    /// Collected local identifiers (variable references)
    pub local_idents: HashSet<Id>,
    /// Whether JSX elements were encountered (need h import)
    pub use_h: bool,
    /// Whether JSX fragments were encountered (need Fragment import)
    pub use_fragment: bool,
    /// Context stack for tracking expression vs statement context
    expr_ctxt: Vec<ExprOrSkip>,
}

impl IdentCollector {
    /// Create a new IdentCollector
    pub fn new() -> Self {
        Self {
            local_idents: HashSet::new(),
            expr_ctxt: Vec::with_capacity(32),
            use_h: false,
            use_fragment: false,
        }
    }

    /// Get collected identifiers as a sorted vector.
    /// Sorting ensures deterministic output for capture arrays.
    pub fn get_words(self) -> Vec<Id> {
        let mut local_idents: Vec<Id> = self.local_idents.into_iter().collect();
        local_idents.sort();
        local_idents
    }
}

impl Default for IdentCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Builtin identifiers that should not be captured
const BUILTINS: &[&str] = &["undefined", "NaN", "Infinity", "null"];

impl<'a> Visit<'a> for IdentCollector {
    fn visit_expression(&mut self, node: &ast::Expression<'a>) {
        self.expr_ctxt.push(ExprOrSkip::Expr);
        // Visit children using the walk functions
        oxc_ast_visit::walk::walk_expression(self, node);
        self.expr_ctxt.pop();
    }

    fn visit_statement(&mut self, node: &ast::Statement<'a>) {
        self.expr_ctxt.push(ExprOrSkip::Skip);
        oxc_ast_visit::walk::walk_statement(self, node);
        self.expr_ctxt.pop();
    }

    fn visit_jsx_element(&mut self, node: &ast::JSXElement<'a>) {
        self.use_h = true;
        oxc_ast_visit::walk::walk_jsx_element(self, node);
    }

    fn visit_jsx_fragment(&mut self, node: &ast::JSXFragment<'a>) {
        self.use_h = true;
        self.use_fragment = true;
        oxc_ast_visit::walk::walk_jsx_fragment(self, node);
    }

    fn visit_jsx_element_name(&mut self, node: &ast::JSXElementName<'a>) {
        // Only visit children (collecting the identifier) if it starts with uppercase
        // Lowercase JSX elements are HTML tags and shouldn't be collected
        if let ast::JSXElementName::IdentifierReference(ref ident) = node {
            let ident_name = ident.name.chars().next();
            if let Some('A'..='Z') = ident_name {
                // Component reference - visit to collect it
            } else {
                // HTML tag - skip
                return;
            }
        }
        oxc_ast_visit::walk::walk_jsx_element_name(self, node);
    }

    fn visit_jsx_attribute(&mut self, node: &ast::JSXAttribute<'a>) {
        // Skip attribute names, but visit attribute values
        self.expr_ctxt.push(ExprOrSkip::Skip);
        oxc_ast_visit::walk::walk_jsx_attribute(self, node);
        self.expr_ctxt.pop();
    }

    fn visit_identifier_reference(&mut self, node: &ast::IdentifierReference<'a>) {
        // Only collect identifiers when in expression context
        if matches!(self.expr_ctxt.last(), Some(ExprOrSkip::Expr)) {
            let name = node.name.as_str();
            // Exclude builtins
            if !BUILTINS.contains(&name) {
                // Use a default scope for now - in actual use, we'd track scope properly
                // For simple identifier collection, we primarily care about the name
                self.local_idents.insert((name.to_string(), ScopeId::new(0)));
            }
        }
    }

    fn visit_object_property(&mut self, node: &ast::ObjectProperty<'a>) {
        // Skip property keys, only visit values
        self.expr_ctxt.push(ExprOrSkip::Skip);
        oxc_ast_visit::walk::walk_object_property(self, node);
        self.expr_ctxt.pop();
    }

    fn visit_member_expression(&mut self, member: &ast::MemberExpression<'a>) {
        // Skip property access names, only visit the object
        self.expr_ctxt.push(ExprOrSkip::Skip);
        oxc_ast_visit::walk::walk_member_expression(self, member);
        self.expr_ctxt.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    fn collect_idents(code: &str) -> IdentCollector {
        let allocator = Allocator::default();
        let source_type = SourceType::tsx();
        let parse_result = Parser::new(&allocator, code, source_type).parse();
        assert!(parse_result.errors.is_empty(), "Parse errors: {:?}", parse_result.errors);

        let mut collector = IdentCollector::new();
        collector.visit_program(&parse_result.program);
        collector
    }

    fn get_names(collector: IdentCollector) -> Vec<String> {
        collector.get_words().into_iter().map(|(name, _)| name).collect()
    }

    #[test]
    fn test_basic_identifier_collection() {
        let collector = collect_idents("const x = a + b;");
        let names = get_names(collector);
        assert!(names.contains(&"a".to_string()), "Should contain 'a', got: {:?}", names);
        assert!(names.contains(&"b".to_string()), "Should contain 'b', got: {:?}", names);
    }

    #[test]
    fn test_builtin_exclusion() {
        let collector = collect_idents("const x = undefined + NaN + Infinity + null;");
        let names = get_names(collector);
        assert!(names.is_empty(), "Should exclude builtins, got: {:?}", names);
    }

    #[test]
    fn test_property_key_skipping() {
        let collector = collect_idents("const x = { foo: bar };");
        let names = get_names(collector);
        assert!(names.contains(&"bar".to_string()), "Should contain 'bar'");
        assert!(!names.contains(&"foo".to_string()), "Should NOT contain 'foo' (property key)");
    }

    #[test]
    fn test_member_expression_property_skipping() {
        let collector = collect_idents("const x = obj.prop;");
        let names = get_names(collector);
        assert!(names.contains(&"obj".to_string()), "Should contain 'obj'");
        assert!(!names.contains(&"prop".to_string()), "Should NOT contain 'prop' (member property)");
    }

    #[test]
    fn test_jsx_element_tracking() {
        let collector = collect_idents("const x = <div />;");
        assert!(collector.use_h, "use_h should be true for JSX elements");
    }

    #[test]
    fn test_jsx_fragment_tracking() {
        let collector = collect_idents("const x = <></>;");
        assert!(collector.use_h, "use_h should be true for JSX fragments");
        assert!(collector.use_fragment, "use_fragment should be true for JSX fragments");
    }

    #[test]
    fn test_sorted_output() {
        let collector = collect_idents("const x = c + a + b;");
        let names = get_names(collector);
        assert_eq!(names, vec!["a".to_string(), "b".to_string(), "c".to_string()],
            "Output should be sorted alphabetically");
    }
}

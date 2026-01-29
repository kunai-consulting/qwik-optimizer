//! Inlined function generation for _fnSignal
//!
//! This module handles the conversion of computed expressions involving
//! signals/stores/props into _fnSignal calls with hoisted arrow functions.
//!
//! When an expression uses a signal/store/prop as an object in a member expression
//! (e.g., `signal.value`, `store.count`, `_rawProps.field`), it needs to be wrapped
//! in `_fnSignal` for reactive updates.
//!
//! # Example
//!
//! ```javascript
//! // Input
//! <div value={store.count + 1}>
//!
//! // Output (with hoisted function)
//! const _hf0 = (p0) => p0.count + 1;
//! const _hf0_str = "p0.count+1";
//! <div value={_fnSignal(_hf0, [store], _hf0_str)}>
//! ```

use crate::collector::Id;
use oxc_allocator::{Allocator, Box as OxcBox, CloneIn};
use oxc_ast::ast::*;
use oxc_ast::AstBuilder;
use oxc_ast_visit::Visit;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_span::SPAN;
use std::collections::HashMap;

/// Maximum expression length before skipping _fnSignal wrapping
pub const MAX_EXPR_LENGTH: usize = 150;

/// Result of converting an expression to _fnSignal
pub struct InlinedFnResult<'a> {
    /// The hoisted arrow function (e.g., (p0, p1) => p1 + p0.fromProps)
    pub hoisted_fn: ArrowFunctionExpression<'a>,
    /// Name for the hoisted function (e.g., "_hf0")
    pub hoisted_name: String,
    /// The string representation (e.g., "p1+p0.fromProps")
    pub hoisted_str: String,
    /// The capture array element identifiers (variable names to capture)
    pub captures: Vec<Id>,
    /// Whether expression is const
    pub is_const: bool,
}

/// Check if expression should be wrapped in _fnSignal
///
/// Returns true if:
/// - Expression is not an arrow function
/// - Expression has scoped identifiers
/// - Any scoped identifier is used as the object of a member expression
pub fn should_wrap_in_fn_signal(expr: &Expression, scoped_idents: &[Id]) -> bool {
    // Don't wrap arrow functions
    if matches!(expr, Expression::ArrowFunctionExpression(_)) {
        return false;
    }

    // Don't wrap if no scoped idents
    if scoped_idents.is_empty() {
        return false;
    }

    // Check if expression uses identifiers as object of member access
    let (used_as_object, used_as_call) = is_used_as_object_or_call(expr, scoped_idents);

    // If used in a call expression, we can't wrap it (can't serialize)
    if used_as_call {
        return false;
    }

    used_as_object
}

/// Check if any scoped identifier is used as object in member expression OR in a call
fn is_used_as_object_or_call(expr: &Expression, scoped_idents: &[Id]) -> (bool, bool) {
    let mut checker = ObjectUsageChecker {
        identifiers: scoped_idents,
        used_as_object: false,
        used_as_call: false,
    };

    checker.visit_expression(expr);

    (checker.used_as_object, checker.used_as_call)
}

/// Visitor to check if identifiers are used as objects in member expressions
struct ObjectUsageChecker<'b> {
    identifiers: &'b [Id],
    used_as_object: bool,
    used_as_call: bool,
}

impl<'b> ObjectUsageChecker<'b> {
    /// Helper function to recursively check if an expression contains one of the target identifiers.
    fn recursively_check_object_expr<'a>(&mut self, expr: &Expression<'a>) {
        if self.used_as_object {
            return; // Already found
        }
        match expr {
            Expression::Identifier(ident) => {
                // Check if this identifier is one of the target identifiers (by name)
                if self.identifiers.iter().any(|id| id.0 == ident.name.as_str()) {
                    self.used_as_object = true;
                }
            }
            Expression::LogicalExpression(log_expr) => {
                // For logical expressions (including ||), check both sides
                self.recursively_check_object_expr(&log_expr.left);
                if self.used_as_object {
                    return;
                }
                self.recursively_check_object_expr(&log_expr.right);
            }
            Expression::ParenthesizedExpression(paren_expr) => {
                // If it's a parenthesized expression, check the inner expression
                self.recursively_check_object_expr(&paren_expr.expression);
            }
            _ => {
                // For other expression types, traversal is handled by the Visit trait
            }
        }
    }
}

impl<'a, 'b> Visit<'a> for ObjectUsageChecker<'b> {
    fn visit_call_expression(&mut self, _: &CallExpression<'a>) {
        // If we're in a call expression, we can't wrap it in a signal
        // because it's a function call, and later we need to serialize it
        self.used_as_call = true;
    }

    fn visit_member_expression(&mut self, node: &MemberExpression<'a>) {
        if self.used_as_object {
            return;
        }

        // Check if the object of the member expression is one of our identifiers
        let obj = node.object();
        self.recursively_check_object_expr(obj);

        if self.used_as_object {
            return;
        }
        oxc_ast_visit::walk::walk_member_expression(self, node);
    }
}

/// Convert expression to _fnSignal-ready result with hoisted function
///
/// Returns None if:
/// - Expression is an arrow function
/// - Expression has no scoped identifiers
/// - No scoped identifier is used as object in member expression
/// - Expression contains a call expression (can't serialize)
/// - Rendered expression exceeds MAX_EXPR_LENGTH
pub fn convert_inlined_fn<'a>(
    expr: &Expression<'a>,
    scoped_idents: &[Id],
    hoisted_index: usize,
    builder: &AstBuilder<'a>,
    allocator: &'a Allocator,
) -> Option<InlinedFnResult<'a>> {
    // Don't wrap arrow functions
    if matches!(expr, Expression::ArrowFunctionExpression(_)) {
        return None;
    }

    // Don't wrap if no scoped idents
    if scoped_idents.is_empty() {
        return None;
    }

    // Check if used as object (needs wrapping)
    let (used_as_object, used_as_call) = is_used_as_object_or_call(expr, scoped_idents);

    if used_as_call {
        return None;
    }

    if !used_as_object {
        return None;
    }

    // Build identifier map: old_id name -> pN
    let mut ident_map: HashMap<String, String> = HashMap::new();
    for (i, id) in scoped_idents.iter().enumerate() {
        ident_map.insert(id.0.clone(), format!("p{}", i));
    }

    // Clone expression and replace identifiers
    let transformed_expr = replace_identifiers_in_expr(expr.clone_in(allocator), &ident_map, builder, allocator);

    // Generate string representation of transformed expression
    let expr_str = render_expression(&transformed_expr);
    if expr_str.len() > MAX_EXPR_LENGTH {
        return None;
    }

    // Build parameter list: p0, p1, p2, ... using direct struct creation like props_destructuring.rs
    let params: oxc_allocator::Vec<'a, FormalParameter<'a>> = builder.vec_from_iter(
        scoped_idents.iter().enumerate().map(|(i, _)| {
            FormalParameter {
                span: SPAN,
                decorators: builder.vec(),
                pattern: builder.binding_pattern_binding_identifier(SPAN, builder.atom(&format!("p{}", i))),
                accessibility: None,
                readonly: false,
                r#override: false,
                initializer: None,
                optional: false,
                type_annotation: None,
            }
        }),
    );

    // Build formal parameters with explicit None type for rest
    let formal_params = builder.formal_parameters(
        SPAN,
        FormalParameterKind::ArrowFormalParameters,
        params,
        None::<OxcBox<FormalParameterRest>>,
    );

    // Build hoisted arrow function: (p0, p1) => transformed_expr
    let hoisted_fn = builder.arrow_function_expression(
        SPAN,
        true,  // expression (body is expression, not block)
        false, // async
        None::<OxcBox<TSTypeParameterDeclaration>>,
        formal_params,
        None::<OxcBox<TSTypeAnnotation>>,
        builder.function_body(
            SPAN,
            builder.vec(), // directives
            builder.vec1(Statement::ExpressionStatement(builder.alloc(
                builder.expression_statement(SPAN, transformed_expr),
            ))),
        ),
    );

    // Generate hoisted name
    let hoisted_name = format!("_hf{}", hoisted_index);

    Some(InlinedFnResult {
        hoisted_fn,
        hoisted_name,
        hoisted_str: expr_str,
        captures: scoped_idents.to_vec(),
        is_const: true,
    })
}

/// Render expression to minified string using OXC codegen
fn render_expression(expr: &Expression) -> String {
    let codegen_options = CodegenOptions {
        minify: true,
        ..Default::default()
    };
    let mut codegen = Codegen::new().with_options(codegen_options);

    // Print the expression to the internal buffer
    codegen.print_expression(expr);

    // Get the final generated code
    codegen.into_source_text()
}

/// Replace identifiers in expression AST with their mapped positional params
fn replace_identifiers_in_expr<'a>(
    expr: Expression<'a>,
    ident_map: &HashMap<String, String>,
    builder: &AstBuilder<'a>,
    allocator: &'a Allocator,
) -> Expression<'a> {
    let mut replacer = IdentifierReplacer {
        ident_map,
        builder,
        allocator,
    };
    replacer.replace_expression(expr)
}

/// Visitor that replaces identifiers with their mapped names
struct IdentifierReplacer<'a, 'b> {
    ident_map: &'b HashMap<String, String>,
    builder: &'b AstBuilder<'a>,
    allocator: &'a Allocator,
}

impl<'a, 'b> IdentifierReplacer<'a, 'b> {
    fn replace_expression(&mut self, expr: Expression<'a>) -> Expression<'a> {
        match expr {
            Expression::Identifier(ident) => {
                // Check if this identifier should be replaced
                if let Some(new_name) = self.ident_map.get(ident.name.as_str()) {
                    self.builder.expression_identifier(ident.span, self.builder.atom(new_name))
                } else {
                    Expression::Identifier(ident)
                }
            }
            Expression::BinaryExpression(bin) => {
                let bin = bin.unbox();
                let left = self.replace_expression(bin.left);
                let right = self.replace_expression(bin.right);
                self.builder.expression_binary(SPAN, left, bin.operator, right)
            }
            Expression::LogicalExpression(log) => {
                let log = log.unbox();
                let left = self.replace_expression(log.left);
                let right = self.replace_expression(log.right);
                self.builder.expression_logical(SPAN, left, log.operator, right)
            }
            Expression::StaticMemberExpression(static_member) => {
                let static_member = static_member.unbox();
                let object = self.replace_expression(static_member.object);
                Expression::from(self.builder.member_expression_static(
                    static_member.span,
                    object,
                    static_member.property.clone_in(self.allocator),
                    false,
                ))
            }
            Expression::ComputedMemberExpression(computed) => {
                let computed = computed.unbox();
                let object = self.replace_expression(computed.object);
                let property = self.replace_expression(computed.expression);
                Expression::from(self.builder.member_expression_computed(
                    computed.span,
                    object,
                    property,
                    false,
                ))
            }
            Expression::PrivateFieldExpression(private) => {
                let private = private.unbox();
                let object = self.replace_expression(private.object);
                Expression::from(MemberExpression::PrivateFieldExpression(
                    self.builder.alloc(PrivateFieldExpression {
                        span: private.span,
                        object,
                        field: private.field.clone_in(self.allocator),
                        optional: private.optional,
                    }),
                ))
            }
            Expression::ConditionalExpression(cond) => {
                let cond = cond.unbox();
                let test = self.replace_expression(cond.test);
                let consequent = self.replace_expression(cond.consequent);
                let alternate = self.replace_expression(cond.alternate);
                self.builder.expression_conditional(SPAN, test, consequent, alternate)
            }
            Expression::ObjectExpression(obj) => {
                let obj = obj.unbox();
                let properties = self.builder.vec_from_iter(
                    obj.properties.into_iter().map(|prop| {
                        match prop {
                            ObjectPropertyKind::ObjectProperty(p) => {
                                let p = p.unbox();
                                let value = self.replace_expression(p.value);
                                ObjectPropertyKind::ObjectProperty(self.builder.alloc(ObjectProperty {
                                    span: p.span,
                                    kind: p.kind,
                                    key: p.key.clone_in(self.allocator),
                                    value,
                                    method: p.method,
                                    shorthand: false, // Can't be shorthand after replacing identifier
                                    computed: p.computed,
                                }))
                            }
                            other => other.clone_in(self.allocator),
                        }
                    }),
                );
                self.builder.expression_object(obj.span, properties)
            }
            Expression::ParenthesizedExpression(paren) => {
                let paren = paren.unbox();
                let inner = self.replace_expression(paren.expression);
                self.builder.expression_parenthesized(paren.span, inner)
            }
            Expression::UnaryExpression(unary) => {
                let unary = unary.unbox();
                let argument = self.replace_expression(unary.argument);
                self.builder.expression_unary(unary.span, unary.operator, argument)
            }
            // For other expression types, return as-is (we handle the most common cases)
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_semantic::ScopeId;
    use oxc_span::SourceType;

    fn parse_expr(code: &str) -> (Allocator, Expression<'static>) {
        let allocator = Allocator::default();
        let source_type = SourceType::tsx();
        // Wrap in a variable declaration to get a valid program
        let full_code = format!("const x = {};", code);
        let allocator_static: &'static Allocator = Box::leak(Box::new(allocator));
        let parse_result = Parser::new(allocator_static, &full_code, source_type).parse();
        assert!(
            parse_result.errors.is_empty(),
            "Parse errors: {:?}",
            parse_result.errors
        );

        // Extract the expression from the variable declaration
        if let Some(Statement::VariableDeclaration(decl)) = parse_result.program.body.first() {
            if let Some(declarator) = decl.declarations.first() {
                if let Some(init) = &declarator.init {
                    return (Allocator::default(), init.clone_in(allocator_static));
                }
            }
        }
        panic!("Could not extract expression from parsed code");
    }

    #[test]
    fn test_should_wrap_member_access() {
        let scoped_idents = vec![("store".to_string(), ScopeId::new(0))];

        // Simple member access should be wrapped
        let (_, expr) = parse_expr("store.count");
        assert!(
            should_wrap_in_fn_signal(&expr, &scoped_idents),
            "store.count should be wrapped"
        );
    }

    #[test]
    fn test_should_not_wrap_simple_identifier() {
        let scoped_idents = vec![("count".to_string(), ScopeId::new(0))];

        // Just an identifier without member access should NOT be wrapped
        let (_, expr) = parse_expr("count");
        assert!(
            !should_wrap_in_fn_signal(&expr, &scoped_idents),
            "simple identifier should NOT be wrapped"
        );
    }

    #[test]
    fn test_should_not_wrap_call_expression() {
        let scoped_idents = vec![("signal".to_string(), ScopeId::new(0))];

        // Expression with function call should NOT be wrapped
        let (_, expr) = parse_expr("signal.value()");
        assert!(
            !should_wrap_in_fn_signal(&expr, &scoped_idents),
            "call expression should NOT be wrapped"
        );
    }

    #[test]
    fn test_should_not_wrap_no_scoped_idents() {
        let scoped_idents: Vec<Id> = vec![];

        // No scoped idents should NOT be wrapped
        let (_, expr) = parse_expr("foo.bar");
        assert!(
            !should_wrap_in_fn_signal(&expr, &scoped_idents),
            "no scoped idents should NOT be wrapped"
        );
    }

    #[test]
    fn test_should_not_wrap_arrow_function() {
        let scoped_idents = vec![("store".to_string(), ScopeId::new(0))];

        // Arrow function should NOT be wrapped
        let (_, expr) = parse_expr("() => store.count");
        assert!(
            !should_wrap_in_fn_signal(&expr, &scoped_idents),
            "arrow function should NOT be wrapped"
        );
    }

    #[test]
    fn test_render_expression_simple() {
        let allocator = Allocator::default();
        let builder = AstBuilder::new(&allocator);
        let expr = builder.expression_numeric_literal(SPAN, 42.0, None, NumberBase::Decimal);
        let result = render_expression(&expr);
        assert_eq!(result, "42");
    }
}

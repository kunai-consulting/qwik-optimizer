//! Inlined function generation for _fnSignal reactive expressions.

use crate::collector::Id;
use oxc_allocator::{Allocator, Box as OxcBox, CloneIn};
use oxc_ast::ast::*;
use oxc_ast::AstBuilder;
use oxc_ast_visit::Visit;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_span::SPAN;
use std::collections::HashMap;

pub const MAX_EXPR_LENGTH: usize = 150;

pub struct InlinedFnResult<'a> {
    pub hoisted_fn: ArrowFunctionExpression<'a>,
    pub hoisted_name: String,
    pub hoisted_str: String,
    pub captures: Vec<Id>,
    pub is_const: bool,
}

/// Returns true if expression should be wrapped in _fnSignal.
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

fn is_used_as_object_or_call(expr: &Expression, scoped_idents: &[Id]) -> (bool, bool) {
    let mut checker = ObjectUsageChecker {
        identifiers: scoped_idents,
        used_as_object: false,
        used_as_call: false,
    };

    checker.visit_expression(expr);

    (checker.used_as_object, checker.used_as_call)
}

struct ObjectUsageChecker<'b> {
    identifiers: &'b [Id],
    used_as_object: bool,
    used_as_call: bool,
}

impl<'b> ObjectUsageChecker<'b> {
    fn recursively_check_object_expr<'a>(&mut self, expr: &Expression<'a>) {
        if self.used_as_object {
            return;
        }
        match expr {
            Expression::Identifier(ident) => {
                if self.identifiers.iter().any(|id| id.0 == ident.name.as_str()) {
                    self.used_as_object = true;
                }
            }
            Expression::LogicalExpression(log_expr) => {
                self.recursively_check_object_expr(&log_expr.left);
                if self.used_as_object {
                    return;
                }
                self.recursively_check_object_expr(&log_expr.right);
            }
            Expression::ParenthesizedExpression(paren_expr) => {
                self.recursively_check_object_expr(&paren_expr.expression);
            }
            _ => {
            }
        }
    }
}

impl<'a, 'b> Visit<'a> for ObjectUsageChecker<'b> {
    fn visit_call_expression(&mut self, _: &CallExpression<'a>) {
        self.used_as_call = true;
    }

    fn visit_member_expression(&mut self, node: &MemberExpression<'a>) {
        if self.used_as_object {
            return;
        }

        let obj = node.object();
        self.recursively_check_object_expr(obj);

        if self.used_as_object {
            return;
        }
        oxc_ast_visit::walk::walk_member_expression(self, node);
    }
}

/// Converts expression to _fnSignal-ready result with hoisted function, or None if not applicable.
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

    let mut ident_map: HashMap<String, String> = HashMap::new();
    for (i, id) in scoped_idents.iter().enumerate() {
        ident_map.insert(id.0.clone(), format!("p{}", i));
    }

    let transformed_expr = replace_identifiers_in_expr(expr.clone_in(allocator), &ident_map, builder, allocator);

    let expr_str = render_expression(&transformed_expr);
    if expr_str.len() > MAX_EXPR_LENGTH {
        return None;
    }

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

    let formal_params = builder.formal_parameters(
        SPAN,
        FormalParameterKind::ArrowFormalParameters,
        params,
        None::<OxcBox<FormalParameterRest>>,
    );

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

    let hoisted_name = format!("_hf{}", hoisted_index);

    Some(InlinedFnResult {
        hoisted_fn,
        hoisted_name,
        hoisted_str: expr_str,
        captures: scoped_idents.to_vec(),
        is_const: true,
    })
}

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

struct IdentifierReplacer<'a, 'b> {
    ident_map: &'b HashMap<String, String>,
    builder: &'b AstBuilder<'a>,
    allocator: &'a Allocator,
}

impl<'a, 'b> IdentifierReplacer<'a, 'b> {
    fn replace_expression(&mut self, expr: Expression<'a>) -> Expression<'a> {
        match expr {
            Expression::Identifier(ident) => {
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
                                    shorthand: false,
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
        let full_code = format!("const x = {};", code);
        let allocator_static: &'static Allocator = Box::leak(Box::new(allocator));
        let parse_result = Parser::new(allocator_static, &full_code, source_type).parse();
        assert!(
            parse_result.errors.is_empty(),
            "Parse errors: {:?}",
            parse_result.errors
        );

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

        let (_, expr) = parse_expr("store.count");
        assert!(
            should_wrap_in_fn_signal(&expr, &scoped_idents),
            "store.count should be wrapped"
        );
    }

    #[test]
    fn test_should_not_wrap_simple_identifier() {
        let scoped_idents = vec![("count".to_string(), ScopeId::new(0))];

        let (_, expr) = parse_expr("count");
        assert!(
            !should_wrap_in_fn_signal(&expr, &scoped_idents),
            "simple identifier should NOT be wrapped"
        );
    }

    #[test]
    fn test_should_not_wrap_call_expression() {
        let scoped_idents = vec![("signal".to_string(), ScopeId::new(0))];

        let (_, expr) = parse_expr("signal.value()");
        assert!(
            !should_wrap_in_fn_signal(&expr, &scoped_idents),
            "call expression should NOT be wrapped"
        );
    }

    #[test]
    fn test_should_not_wrap_no_scoped_idents() {
        let scoped_idents: Vec<Id> = vec![];

        let (_, expr) = parse_expr("foo.bar");
        assert!(
            !should_wrap_in_fn_signal(&expr, &scoped_idents),
            "no scoped idents should NOT be wrapped"
        );
    }

    #[test]
    fn test_should_not_wrap_arrow_function() {
        let scoped_idents = vec![("store".to_string(), ScopeId::new(0))];

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

    #[test]
    fn test_convert_inlined_fn_basic() {
        let scoped_idents = vec![("store".to_string(), ScopeId::new(0))];

        let (allocator, expr) = parse_expr("store.count");

        let builder = AstBuilder::new(&allocator);
        let result = convert_inlined_fn(&expr, &scoped_idents, 0, &builder, &allocator);

        assert!(result.is_some(), "convert_inlined_fn should succeed");
        let result = result.unwrap();

        assert_eq!(result.hoisted_name, "_hf0");

        assert_eq!(result.captures.len(), 1);
        assert_eq!(result.captures[0].0, "store");

        assert!(
            result.hoisted_str.contains("p0"),
            "Expected p0 in hoisted_str, got: {}",
            result.hoisted_str
        );
    }

    #[test]
    fn test_convert_inlined_fn_skips_call() {
        let scoped_idents = vec![("signal".to_string(), ScopeId::new(0))];

        let (allocator, expr) = parse_expr("signal.value()");

        let builder = AstBuilder::new(&allocator);
        let result = convert_inlined_fn(&expr, &scoped_idents, 0, &builder, &allocator);

        assert!(result.is_none(), "convert_inlined_fn should return None for call expressions");
    }

    #[test]
    fn test_convert_inlined_fn_skips_arrow() {
        let scoped_idents = vec![("store".to_string(), ScopeId::new(0))];

        let (allocator, expr) = parse_expr("() => store.count");

        let builder = AstBuilder::new(&allocator);
        let result = convert_inlined_fn(&expr, &scoped_idents, 0, &builder, &allocator);

        assert!(result.is_none(), "convert_inlined_fn should return None for arrow functions");
    }
}

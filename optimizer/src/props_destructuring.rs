//! Props destructuring transformation for component$ functions.

use oxc_allocator::{Allocator, CloneIn};
use oxc_ast::ast::*;
use oxc_ast::AstBuilder;
use oxc_span::SPAN;
use std::collections::HashMap;

use crate::collector::Id;

/// Transforms destructured props to _rawProps parameter with identifier mappings.
pub struct PropsDestructuring<'a> {
    component_ident: Option<Id>,
    pub identifiers: HashMap<Id, String>,
    #[allow(dead_code)]
    allocator: &'a Allocator,
    raw_props_name: &'static str,
    pub rest_id: Option<Id>,
    pub omit_keys: Vec<String>,
}

impl<'a> PropsDestructuring<'a> {
    pub fn new(allocator: &'a Allocator, component_ident: Option<Id>) -> Self {
        Self {
            component_ident,
            identifiers: HashMap::new(),
            allocator,
            raw_props_name: "_rawProps",
            rest_id: None,
            omit_keys: Vec::new(),
        }
    }

    #[allow(dead_code)]
    fn is_component_call(&self, call: &CallExpression) -> bool {
        if let Some(ref comp_id) = self.component_ident {
            if let Some(name) = call.callee_name() {
                return name == comp_id.0;
            }
        }
        false
    }

    fn has_object_pattern_param(&self, arrow: &ArrowFunctionExpression) -> bool {
        if let Some(first_param) = arrow.params.items.first() {
            matches!(first_param.pattern, BindingPattern::ObjectPattern(_))
        } else {
            false
        }
    }

    /// Returns true if transformation was applied.
    pub fn transform_component_props<'b>(
        &mut self,
        arrow: &mut ArrowFunctionExpression<'b>,
        builder: &AstBuilder<'b>,
    ) -> bool
    where
        'b: 'a,
    {
        if !self.has_object_pattern_param(arrow) {
            return false;
        }

        let first_param = &arrow.params.items[0];
        if let BindingPattern::ObjectPattern(obj_pat) = &first_param.pattern {
            for prop in &obj_pat.properties {
                let prop_key = self.extract_prop_key(&prop.key);
                let local_name = self.extract_binding_name(&prop.value);
                if let (Some(key), Some(local)) = (prop_key, local_name) {
                    self.identifiers.insert(local, key.clone());
                    self.omit_keys.push(key);
                }
            }

            if let Some(rest) = &obj_pat.rest {
                if let BindingPattern::BindingIdentifier(ident) = &rest.argument {
                    self.rest_id = Some((
                        ident.name.to_string(),
                        oxc_semantic::ScopeId::new(0),
                    ));
                }
            }

            let new_binding = builder.binding_pattern_binding_identifier(SPAN, self.raw_props_name);

            let mut new_items = builder.vec();

            let new_param = FormalParameter {
                span: SPAN,
                decorators: builder.vec(),
                pattern: new_binding,
                accessibility: None,
                readonly: false,
                r#override: false,
                initializer: None,
                optional: false,
                type_annotation: None,
            };
            new_items.push(new_param);

            for i in 1..arrow.params.items.len() {
                if let Some(param) = arrow.params.items.get(i) {
                    let cloned = FormalParameter {
                        span: param.span,
                        decorators: builder.vec(),
                        pattern: param.pattern.clone_in(builder.allocator),
                        accessibility: param.accessibility,
                        readonly: param.readonly,
                        r#override: param.r#override,
                        initializer: param.initializer.clone_in(builder.allocator),
                        optional: param.optional,
                        type_annotation: param.type_annotation.clone_in(builder.allocator),
                    };
                    new_items.push(cloned);
                }
            }

            let rest = arrow.params.rest.clone_in(builder.allocator);

            arrow.params = builder.alloc_formal_parameters(
                arrow.params.span,
                FormalParameterKind::ArrowFormalParameters,
                new_items,
                rest,
            );

            return true;
        }

        false
    }

    fn extract_prop_key(&self, key: &PropertyKey) -> Option<String> {
        match key {
            PropertyKey::StaticIdentifier(ident) => Some(ident.name.to_string()),
            PropertyKey::StringLiteral(lit) => Some(lit.value.to_string()),
            _ => None,
        }
    }

    fn extract_binding_name(&self, pattern: &BindingPattern) -> Option<Id> {
        match pattern {
            BindingPattern::BindingIdentifier(ident) => {
                // Use ScopeId::new(0) since we don't have semantic info here
                // The ScopeId will be matched by name later
                Some((ident.name.to_string(), oxc_semantic::ScopeId::new(0)))
            }
            _ => None,
        }
    }

    /// Generates rest props statement if rest pattern is present.
    pub fn generate_rest_stmt<'b>(&self, builder: &AstBuilder<'b>) -> Option<Statement<'b>> {
        let rest_id = self.rest_id.as_ref()?;

        let raw_props_arg = Argument::from(builder.expression_identifier(SPAN, self.raw_props_name));

        let args = if self.omit_keys.is_empty() {
            builder.vec1(raw_props_arg)
        } else {
            let omit_elements: oxc_allocator::Vec<'b, ArrayExpressionElement<'b>> = builder.vec_from_iter(
                self.omit_keys.iter().map(|key| {
                    ArrayExpressionElement::from(
                        builder.expression_string_literal(SPAN, builder.atom(key.as_str()), None)
                    )
                })
            );

            let omit_array = builder.expression_array(SPAN, omit_elements);

            builder.vec_from_array([
                raw_props_arg,
                Argument::from(omit_array),
            ])
        };

        let call_expr = builder.expression_call(
            SPAN,
            builder.expression_identifier(SPAN, "_restProps"),
            None::<oxc_allocator::Box<'b, TSTypeParameterInstantiation<'b>>>,
            args,
            false,
        );

        let decl = builder.variable_declaration(
            SPAN,
            VariableDeclarationKind::Const,
            builder.vec1(builder.variable_declarator(
                SPAN,
                VariableDeclarationKind::Const,
                builder.binding_pattern_binding_identifier(SPAN, builder.atom(&rest_id.0)),
                oxc_ast::NONE,
                Some(call_expr),
                false,
            )),
            false,
        );

        Some(Statement::VariableDeclaration(builder.alloc(decl)))
    }
}

#[cfg(test)]
mod tests {
    use crate::component::Language;
    use crate::source::Source;
    use crate::transform::{transform, TransformOptions};

    /// Test that simple destructure is detected and transformed.
    /// component$(({ message }) => ...) should become component$((_rawProps) => ...)
    #[test]
    fn test_props_destructuring_simple() {
        let source_code = r#"
import { component$ } from "@qwik.dev/core";
export const Cmp = component$(({ message }) => {
    return <div>{message}</div>;
});
"#;

        let source = Source::from_source(source_code, Language::Typescript, Some("test".into()))
            .expect("Source should parse");
        let options = TransformOptions::default().with_transpile_jsx(true);
        let result = transform(source, options).expect("Transform should succeed");

        let has_raw_props = result.optimized_app.body.contains("_rawProps")
            || result.optimized_app.components.iter().any(|c| c.code.contains("_rawProps"));

        assert!(
            has_raw_props,
            "Expected _rawProps parameter, got body: {}\ncomponents: {:?}",
            result.optimized_app.body,
            result.optimized_app.components.iter().map(|c| &c.code).collect::<Vec<_>>()
        );
    }

    /// Test that multiple destructured props are tracked.
    #[test]
    fn test_props_destructuring_multiple() {
        let source_code = r#"
import { component$ } from "@qwik.dev/core";
export const Cmp = component$(({ message, id, count }) => {
    return <div id={id}>{message} - {count}</div>;
});
"#;

        let source = Source::from_source(source_code, Language::Typescript, Some("test".into()))
            .expect("Source should parse");
        let options = TransformOptions::default().with_transpile_jsx(true);
        let result = transform(source, options).expect("Transform should succeed");

        let has_raw_props = result.optimized_app.body.contains("_rawProps")
            || result.optimized_app.components.iter().any(|c| c.code.contains("_rawProps"));

        assert!(
            has_raw_props,
            "Expected _rawProps parameter, got body: {}\ncomponents: {:?}",
            result.optimized_app.body,
            result.optimized_app.components.iter().map(|c| &c.code).collect::<Vec<_>>()
        );
    }

    /// Test that aliased destructure ({{ count: c }}) is handled.
    #[test]
    fn test_props_destructuring_aliased() {
        let source_code = r#"
import { component$ } from "@qwik.dev/core";
export const Cmp = component$(({ count: c, name: n }) => {
    return <div>{c} - {n}</div>;
});
"#;

        let source = Source::from_source(source_code, Language::Typescript, Some("test".into()))
            .expect("Source should parse");
        let options = TransformOptions::default().with_transpile_jsx(true);
        let result = transform(source, options).expect("Transform should succeed");

        let has_raw_props = result.optimized_app.body.contains("_rawProps")
            || result.optimized_app.components.iter().any(|c| c.code.contains("_rawProps"));

        assert!(
            has_raw_props,
            "Expected _rawProps parameter, got body: {}\ncomponents: {:?}",
            result.optimized_app.body,
            result.optimized_app.components.iter().map(|c| &c.code).collect::<Vec<_>>()
        );
    }

    /// Test that non-component arrow functions are not transformed.
    #[test]
    fn test_ignores_non_component() {
        let source_code = r#"
const regularFn = ({ x }) => x * 2;
const result = regularFn({ x: 5 });
"#;

        let source = Source::from_source(source_code, Language::Typescript, Some("test".into()))
            .expect("Source should parse");
        let options = TransformOptions::default();
        let result = transform(source, options).expect("Transform should succeed");

        // Should NOT contain _rawProps - this is a regular function
        assert!(
            !result.optimized_app.body.contains("_rawProps"),
            "Non-component function should not have _rawProps: {}",
            result.optimized_app.body
        );
    }

    /// Test that non-destructured params are not transformed.
    #[test]
    fn test_ignores_non_destructured() {
        let source_code = r#"
import { component$ } from "@qwik.dev/core";
export const Cmp = component$((props) => {
    return <div>{props.message}</div>;
});
"#;

        let source = Source::from_source(source_code, Language::Typescript, Some("test".into()))
            .expect("Source should parse");
        let options = TransformOptions::default().with_transpile_jsx(true);
        let result = transform(source, options).expect("Transform should succeed");

        // Should NOT contain _rawProps - props is already a simple param
        // The body should still contain 'props' as the parameter
        let has_raw_props = result.optimized_app.body.contains("_rawProps")
            || result.optimized_app.components.iter().any(|c| c.code.contains("_rawProps"));

        assert!(
            !has_raw_props,
            "Non-destructured param should not become _rawProps, body: {}\ncomponents: {:?}",
            result.optimized_app.body,
            result.optimized_app.components.iter().map(|c| &c.code).collect::<Vec<_>>()
        );
    }
}

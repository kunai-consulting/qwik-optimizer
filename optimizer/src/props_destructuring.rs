//! Props destructuring transformation for Qwik component$ functions.
//!
//! This module handles the transformation of destructured props parameters in component$ functions
//! from ObjectPattern (`({ message, id })`) to a simple `_rawProps` parameter. The original prop
//! names are tracked in an identifiers map for later replacement with property access expressions.
//!
//! Example transformation:
//! ```javascript
//! // Input
//! component$(({ message, id }) => { ... })
//!
//! // Output (parameter only - identifier replacement is a separate pass)
//! component$((_rawProps) => { ... })
//! ```

use oxc_allocator::{Allocator, CloneIn};
use oxc_ast::ast::*;
use oxc_ast::AstBuilder;
use oxc_span::SPAN;
use std::collections::HashMap;

use crate::collector::Id;

/// Props destructuring transformation state.
///
/// Tracks component$ identifier and stores mappings from local variable names
/// to their original property keys for later identifier replacement.
pub struct PropsDestructuring<'a> {
    /// Track component$ identifier from imports (optional, can be None if we already know it's component$)
    component_ident: Option<Id>,

    /// Map original identifier to property key string for _rawProps.key access.
    /// Key: (local_name, scope_id), Value: property key string
    pub identifiers: HashMap<Id, String>,

    /// Allocator reference for AST building
    allocator: &'a Allocator,

    /// Name of the _rawProps parameter (for uniqueness if needed)
    raw_props_name: &'static str,
}

impl<'a> PropsDestructuring<'a> {
    /// Create a new PropsDestructuring instance.
    ///
    /// # Arguments
    /// * `allocator` - OXC allocator for AST node creation
    /// * `component_ident` - Optional component$ identifier to check against
    pub fn new(allocator: &'a Allocator, component_ident: Option<Id>) -> Self {
        Self {
            component_ident,
            identifiers: HashMap::new(),
            allocator,
            raw_props_name: "_rawProps",
        }
    }

    /// Check if CallExpression is a component$ call.
    #[allow(dead_code)]
    fn is_component_call(&self, call: &CallExpression) -> bool {
        if let Some(ref comp_id) = self.component_ident {
            if let Some(name) = call.callee_name() {
                return name == comp_id.0;
            }
        }
        false
    }

    /// Check if arrow function has ObjectPattern as first param.
    fn has_object_pattern_param(&self, arrow: &ArrowFunctionExpression) -> bool {
        if let Some(first_param) = arrow.params.items.first() {
            // OXC 0.111.0: BindingPattern has get_binding_identifier() method
            // ObjectPattern is detected when get_binding_identifier() returns None
            // and we need to match on the pattern itself
            matches!(first_param.pattern, BindingPattern::ObjectPattern(_))
        } else {
            false
        }
    }

    /// Transform component props parameter from destructured to _rawProps.
    ///
    /// This method:
    /// 1. Checks if the first parameter is an ObjectPattern
    /// 2. Extracts property mappings (local_name -> prop_key)
    /// 3. Replaces the ObjectPattern with a simple _rawProps BindingIdentifier
    /// 4. Stores the mappings in self.identifiers for later use
    ///
    /// # Returns
    /// `true` if transformation was applied, `false` otherwise.
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

        // Extract ObjectPattern from first param
        let first_param = &arrow.params.items[0];
        if let BindingPattern::ObjectPattern(obj_pat) = &first_param.pattern {
            // Collect property mappings (prop_name -> local_name)
            // Will be used later for identifier replacement
            for prop in &obj_pat.properties {
                // Handle { propName } and { propName: localName }
                let prop_key = self.extract_prop_key(&prop.key);
                let local_name = self.extract_binding_name(&prop.value);
                if let (Some(key), Some(local)) = (prop_key, local_name) {
                    // Store: local_name -> prop_key for later lookup
                    self.identifiers.insert(local, key);
                }
            }

            // Replace ObjectPattern with _rawProps BindingIdentifier
            // Create new BindingPattern with BindingIdentifier using the simpler API
            let new_binding = builder.binding_pattern_binding_identifier(SPAN, self.raw_props_name);

            // Build new FormalParameters with a single _rawProps param
            let mut new_items = builder.vec();

            // Create a FormalParameter directly using the builder
            // OXC 0.111.0 formal_parameter has all these fields
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

            // Copy any remaining parameters (if there are more than 1)
            for i in 1..arrow.params.items.len() {
                if let Some(param) = arrow.params.items.get(i) {
                    // Clone the parameter
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

            // Clone the rest parameter if present
            let rest = arrow.params.rest.clone_in(builder.allocator);

            // Replace the entire params with new FormalParameters
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

    /// Extract property key as string from PropertyKey.
    fn extract_prop_key(&self, key: &PropertyKey) -> Option<String> {
        match key {
            PropertyKey::StaticIdentifier(ident) => Some(ident.name.to_string()),
            PropertyKey::StringLiteral(lit) => Some(lit.value.to_string()),
            _ => None,
        }
    }

    /// Extract binding name and scope from BindingPattern.
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transform::{default_options, do_transform};

    /// Test that simple destructure is detected and transformed.
    /// component$(({ message }) => ...) should become component$((_rawProps) => ...)
    #[test]
    fn test_props_destructuring_simple() {
        let input = r#"
            import { component$ } from "@qwik.dev/core";
            export const Cmp = component$(({ message }) => {
                return <div>{message}</div>;
            });
        "#;

        let (app, errors) = do_transform(input, &default_options(), None);
        assert!(errors.is_empty(), "Transform errors: {:?}", errors);

        // Should contain _rawProps in the component output
        assert!(
            app.body.contains("_rawProps"),
            "Expected _rawProps parameter, got: {}",
            app.body
        );
    }

    /// Test that multiple destructured props are tracked.
    #[test]
    fn test_props_destructuring_multiple() {
        let input = r#"
            import { component$ } from "@qwik.dev/core";
            export const Cmp = component$(({ message, id, count }) => {
                return <div id={id}>{message} - {count}</div>;
            });
        "#;

        let (app, errors) = do_transform(input, &default_options(), None);
        assert!(errors.is_empty(), "Transform errors: {:?}", errors);

        // Should contain _rawProps in the component output
        assert!(
            app.body.contains("_rawProps"),
            "Expected _rawProps parameter, got: {}",
            app.body
        );
    }

    /// Test that aliased destructure ({{ count: c }}) is handled.
    #[test]
    fn test_props_destructuring_aliased() {
        let input = r#"
            import { component$ } from "@qwik.dev/core";
            export const Cmp = component$(({ count: c, name: n }) => {
                return <div>{c} - {n}</div>;
            });
        "#;

        let (app, errors) = do_transform(input, &default_options(), None);
        assert!(errors.is_empty(), "Transform errors: {:?}", errors);

        // Should contain _rawProps in the component output
        assert!(
            app.body.contains("_rawProps"),
            "Expected _rawProps parameter, got: {}",
            app.body
        );
    }

    /// Test that non-component arrow functions are not transformed.
    #[test]
    fn test_ignores_non_component() {
        let input = r#"
            const regularFn = ({ x }) => x * 2;
            const result = regularFn({ x: 5 });
        "#;

        let (app, errors) = do_transform(input, &default_options(), None);
        assert!(errors.is_empty(), "Transform errors: {:?}", errors);

        // Should NOT contain _rawProps - this is a regular function
        assert!(
            !app.body.contains("_rawProps"),
            "Non-component function should not have _rawProps: {}",
            app.body
        );
    }

    /// Test that non-destructured params are not transformed.
    #[test]
    fn test_ignores_non_destructured() {
        let input = r#"
            import { component$ } from "@qwik.dev/core";
            export const Cmp = component$((props) => {
                return <div>{props.message}</div>;
            });
        "#;

        let (app, errors) = do_transform(input, &default_options(), None);
        assert!(errors.is_empty(), "Transform errors: {:?}", errors);

        // Should NOT contain _rawProps - props is already a simple param
        // The body should still contain 'props' as the parameter
        assert!(
            !app.body.contains("_rawProps"),
            "Non-destructured param should not become _rawProps: {}",
            app.body
        );
    }
}

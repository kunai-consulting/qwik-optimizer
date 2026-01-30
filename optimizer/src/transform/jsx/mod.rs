//! JSX transformation helpers for Qwik optimizer.
//!
//! This module contains JSX-specific transformation logic extracted from
//! generator.rs following the dispatcher pattern. The `impl Traverse` in
//! generator.rs delegates to these functions for JSX nodes.
//!
//! # Submodules
//!
//! - `event`: Event name transformation utilities
//! - `bind`: Bind directive helpers
//! - `element`: JSX element enter/exit handlers
//! - `fragment`: JSX fragment enter/exit handlers
//! - `attribute`: JSX attribute handlers
//! - `child`: JSX child handlers

mod attribute;
mod bind;
mod child;
mod element;
pub(crate) mod event;
mod fragment;

use oxc_ast::ast::*;
use oxc_span::GetSpan;

// JSX constants
pub(crate) const JSX_SORTED_NAME: &str = "_jsxSorted";
pub(crate) const JSX_SPLIT_NAME: &str = "_jsxSplit";
pub(crate) const JSX_RUNTIME_SOURCE: &str = "@qwik.dev/core/jsx-runtime";
pub(crate) const _FRAGMENT: &str = "_Fragment";
pub(crate) const _GET_VAR_PROPS: &str = "_getVarProps";
pub(crate) const _GET_CONST_PROPS: &str = "_getConstProps";

// Re-export public functions from submodules
pub use attribute::{enter_jsx_attribute, exit_jsx_attribute, exit_jsx_attribute_value, exit_jsx_spread_attribute};
pub use child::exit_jsx_child;
pub use element::{enter_jsx_element, exit_jsx_element};
pub use fragment::{enter_jsx_fragment, exit_jsx_fragment};

// Re-export bind helpers for use by transform_tests and mod.rs
pub(crate) use bind::is_bind_directive;

// =============================================================================
// Helper Functions
// =============================================================================

/// Gets the full attribute name from a JSXAttributeName, including namespace if present.
///
/// # Returns
/// The full attribute name string (e.g., "onClick$", "document:onFocus$")
pub(crate) fn get_jsx_attribute_full_name(name: &JSXAttributeName) -> String {
    match name {
        JSXAttributeName::Identifier(id) => id.name.to_string(),
        JSXAttributeName::NamespacedName(ns) => {
            format!("{}:{}", ns.namespace.name, ns.name.name)
        }
    }
}

/// Checks if an element name is text-only (textarea, title, etc.)
pub(super) fn is_text_only(node: &str) -> bool {
    matches!(
        node,
        "text" | "textarea" | "title" | "option" | "script" | "style" | "noscript"
    )
}

/// Move expression helper for replacing AST nodes.
pub(crate) fn move_expression<'gen>(
    builder: &oxc_ast::AstBuilder<'gen>,
    expr: &mut Expression<'gen>,
) -> Expression<'gen> {
    let span = expr.span().clone();
    std::mem::replace(expr, builder.expression_null_literal(span))
}

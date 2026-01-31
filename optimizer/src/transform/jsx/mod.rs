mod attribute;
mod bind;
mod child;
mod element;
pub(crate) mod event;
mod fragment;

use oxc_ast::ast::*;
use oxc_span::GetSpan;

pub(crate) const JSX_SORTED_NAME: &str = "_jsxSorted";
pub(crate) const JSX_SPLIT_NAME: &str = "_jsxSplit";
pub(crate) const JSX_RUNTIME_SOURCE: &str = "@qwik.dev/core/jsx-runtime";
pub(crate) const _FRAGMENT: &str = "_Fragment";
pub(crate) const _GET_VAR_PROPS: &str = "_getVarProps";
pub(crate) const _GET_CONST_PROPS: &str = "_getConstProps";

pub use attribute::{enter_jsx_attribute, exit_jsx_attribute, exit_jsx_attribute_value, exit_jsx_spread_attribute};
pub use child::exit_jsx_child;
pub use element::{enter_jsx_element, exit_jsx_element};
pub use fragment::{enter_jsx_fragment, exit_jsx_fragment};

pub(crate) use bind::is_bind_directive;

pub(crate) fn get_jsx_attribute_full_name(name: &JSXAttributeName) -> String {
    match name {
        JSXAttributeName::Identifier(id) => id.name.to_string(),
        JSXAttributeName::NamespacedName(ns) => {
            format!("{}:{}", ns.namespace.name, ns.name.name)
        }
    }
}

pub(super) fn is_text_only(node: &str) -> bool {
    matches!(
        node,
        "text" | "textarea" | "title" | "option" | "script" | "style" | "noscript"
    )
}

pub(crate) fn move_expression<'gen>(
    builder: &oxc_ast::AstBuilder<'gen>,
    expr: &mut Expression<'gen>,
) -> Expression<'gen> {
    let span = expr.span().clone();
    std::mem::replace(expr, builder.expression_null_literal(span))
}

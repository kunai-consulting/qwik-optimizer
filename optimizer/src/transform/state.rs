use oxc_allocator::Vec as OxcVec;
use oxc_ast::ast::{ArrayExpressionElement, Expression, ObjectPropertyKind};
use std::collections::HashMap;

pub struct JsxState<'gen> {
    pub is_fn: bool,
    pub is_text_only: bool,
    pub is_segment: bool,
    pub should_runtime_sort: bool,
    pub static_listeners: bool,
    pub static_subtree: bool,
    pub key_prop: Option<Expression<'gen>>,
    pub var_props: OxcVec<'gen, ObjectPropertyKind<'gen>>,
    pub const_props: OxcVec<'gen, ObjectPropertyKind<'gen>>,
    pub children: OxcVec<'gen, ArrayExpressionElement<'gen>>,
    pub spread_expr: Option<Expression<'gen>>,
    pub stacked_ctxt: bool,
    /// Track if q:p or q:ps has been added to this element (only add once per element)
    pub added_iter_var_prop: bool,
}

#[derive(Debug, Default)]
pub struct ImportTracker {
    imports: HashMap<(String, String), String>,
}

impl ImportTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_import(&mut self, source: &str, specifier: &str, local: &str) {
        self.imports.insert(
            (source.to_string(), specifier.to_string()),
            local.to_string(),
        );
    }

    pub fn get_imported_local(&self, specifier: &str, source: &str) -> Option<&String> {
        self.imports
            .get(&(source.to_string(), specifier.to_string()))
    }
}

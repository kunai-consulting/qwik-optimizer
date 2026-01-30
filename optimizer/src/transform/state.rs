//! State tracking types for JSX and import handling during transformation.
//!
//! This module contains:
//! - `JsxState`: Tracks JSX element state during traversal
//! - `ImportTracker`: Tracks imported identifiers by source module

use oxc_allocator::Vec as OxcVec;
use oxc_ast::ast::{ArrayExpressionElement, Expression, ObjectPropertyKind};
use std::collections::HashMap;

/// Tracks JSX element state during AST traversal.
///
/// Each JSX element in the tree gets its own `JsxState` pushed onto a stack.
/// This tracks props (var vs const), children, spread expressions, and
/// various optimization flags.
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
    /// Spread expression for _getVarProps/_getConstProps generation.
    /// Set when encountering spread attribute, used in exit_jsx_element.
    pub spread_expr: Option<Expression<'gen>>,
    /// Whether we pushed to stack_ctxt for this JSX element (for pop on exit).
    pub stacked_ctxt: bool,
}

/// Tracks imported identifiers by source module.
/// Maps (source, specifier) -> local_name for finding aliased imports.
///
/// This is used to look up local names for const replacement, determining
/// if identifiers like `isServer` are imported from `@qwik.dev/core/build`.
#[derive(Debug, Default)]
pub struct ImportTracker {
    /// Maps (source, imported_name) -> local_name
    /// e.g., ("@qwik.dev/core/build", "isServer") -> "s" for `import { isServer as s }`
    imports: HashMap<(String, String), String>,
}

impl ImportTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an import: import { specifier as local } from 'source'
    pub fn add_import(&mut self, source: &str, specifier: &str, local: &str) {
        self.imports.insert(
            (source.to_string(), specifier.to_string()),
            local.to_string(),
        );
    }

    /// Get the local name for an imported specifier from a source.
    /// Returns None if not imported.
    pub fn get_imported_local(&self, specifier: &str, source: &str) -> Option<&String> {
        self.imports
            .get(&(source.to_string(), specifier.to_string()))
    }
}

//! Core AST transformation generator for Qwik optimizer.
//!
//! This module contains the `TransformGenerator` struct which implements
//! the `Traverse` trait to walk the AST and transform QRL markers.

#![allow(unused)]

use crate::dead_code::DeadCode;
use crate::entry_strategy::*;
use crate::ext::*;
use crate::prelude::*;
use crate::ref_counter::RefCounter;
use crate::segment::{Segment, SegmentBuilder};
use oxc_allocator::{
    Allocator, Box as OxcBox, CloneIn, FromIn, GetAddress, HashMap as OxcHashMap, IntoIn,
    Vec as OxcVec,
};
use oxc_ast::ast::*;
use oxc_ast::{match_member_expression, AstBuilder, AstType, Comment, NONE};
use oxc_ast_visit::{Visit, VisitMut};
use oxc_codegen::{Codegen, CodegenOptions, Context, Gen};
use oxc_index::Idx;
use std::borrow::{Borrow, Cow};
use std::collections::{BTreeSet, HashMap, HashSet};

use crate::component::*;
use crate::import_clean_up::ImportCleanUp;
use crate::macros::*;
use oxc_semantic::{
    NodeId, ReferenceId, ScopeFlags, Scoping, SymbolFlags, SymbolId,
};
use oxc_span::*;
use oxc_traverse::{Ancestor, Traverse, TraverseCtx};
use serde::{Deserialize, Serialize};
use std::cell::{Cell, RefCell};
use std::fmt::{write, Display, Pointer};

use crate::collector::{ExportInfo, Id};
use crate::is_const::is_const_expr;

// Import types from sibling modules
use super::jsx;
use super::options::TransformOptions;
use super::qrl as qrl_module;
use super::state::{ImportTracker, JsxState};

// Re-export types needed by tests in transform_tests.rs
pub(crate) use crate::component::Target;

/// Type of declaration for tracking captured variables.
/// Used in compute_scoped_idents to determine if captured variables are const.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IdentType {
    /// Variable declaration - bool indicates is_const
    Var(bool),
    /// Function declaration
    Fn,
    /// Class declaration
    Class,
}

/// Identifier plus its type for scope tracking
pub type IdPlusType = (Id, IdentType);

use std::iter::Sum;
use std::ops::Deref;
use std::path::{Components, PathBuf};

use std::fs;
use std::path::Path;
use std::str;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Serialize)]
pub struct OptimizedApp {
    pub body: String,
    pub components: Vec<QrlComponent>,
}

use crate::ext::*;
use crate::illegal_code::{IllegalCode, IllegalCodeType};
use crate::processing_failure::ProcessingFailure;

impl OptimizedApp {
    fn get_component(&self, name: String) -> Option<&QrlComponent> {
        self.components
            .iter()
            .find(|comp| comp.id.symbol_name == name)
    }
}

impl Display for OptimizedApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let component_count = self.components.len();
        let comp_heading = format!(
            "------------------- COMPONENTS[{}] ------------------\n",
            component_count
        );

        let sep = format!("{}\n", "-".repeat(comp_heading.len()));
        let all_comps = self.components.iter().fold(String::new(), |acc, comp| {
            let heading = format!("-------- COMPONENT: {}", comp.id.symbol_name);

            let body = &comp.code;
            format!("{}\n{}\n{}\n{}", acc, heading, body, sep)
        });

        let body_heading = "------------------------ BODY -----------------------\n".to_string();

        write!(
            f,
            "{}{}{}{}",
            comp_heading, all_comps, body_heading, self.body
        )
    }
}

pub struct OptimizationResult {
    pub optimized_app: OptimizedApp,
    pub errors: Vec<ProcessingFailure>,
}

impl OptimizationResult {
    pub fn new(optimized_app: OptimizedApp, errors: Vec<ProcessingFailure>) -> Self {
        Self {
            optimized_app,
            errors,
        }
    }
}

// JsxState is defined in super::state

pub struct TransformGenerator<'gen> {
    pub options: TransformOptions,

    pub components: Vec<QrlComponent>,

    pub app: OptimizedApp,

    pub errors: Vec<ProcessingFailure>,

    pub(crate) builder: AstBuilder<'gen>,

    depth: usize,

    pub(crate) segment_stack: Vec<Segment>,

    segment_builder: SegmentBuilder,

    pub(crate) symbol_by_name: HashMap<String, SymbolId>,

    pub(crate) component_stack: Vec<QrlComponent>,

    pub(crate) qrl_stack: Vec<Qrl>,

    pub(crate) import_stack: Vec<BTreeSet<Import>>,

    const_stack: Vec<BTreeSet<SymbolId>>,

    pub(crate) import_by_symbol: HashMap<SymbolId, Import>,

    removed: HashMap<SymbolId, IllegalCodeType>,

    pub(crate) source_info: &'gen SourceInfo,

    scope: Option<String>,

    pub(crate) jsx_stack: Vec<JsxState<'gen>>,

    pub(crate) jsx_key_counter: u32,

    /// Marks whether each JSX attribute in the stack is var (false) or const (true).
    /// An attribute is considered var if it:
    /// - calls a function
    /// - accesses a member
    /// - is a variable that is not an import, an export, or in the const stack
    pub(crate) expr_is_const_stack: Vec<bool>,

    /// Used to replace the current expression in the AST. Should be set when exiting a specific
    /// type of expression (e.g., `exit_jsx_element`); this will be picked up in `exit_expression`,
    /// which will replace the entire expression with the contents of this field.
    pub(crate) replace_expr: Option<Expression<'gen>>,

    /// Stack of declaration scopes for tracking variable captures.
    /// Each scope level contains the identifiers declared at that level.
    /// Used by compute_scoped_idents to determine which variables need to be captured.
    pub(crate) decl_stack: Vec<Vec<IdPlusType>>,

    /// Stack tracking whether each JSX element is a native HTML element.
    /// Native elements (lowercase first char like `<div>`, `<button>`) get event name transformation.
    /// Component elements (uppercase first char like `<MyButton>`) keep original attribute names.
    pub(crate) jsx_element_is_native: Vec<bool>,

    /// Props destructuring state for current component.
    /// Maps local variable names to their original property keys for _rawProps.key access.
    /// Key: (local_name, scope_id), Value: property key string
    pub(crate) props_identifiers: HashMap<Id, String>,

    /// Flag indicating we're inside a component$ that needs props transformation.
    /// Set to true when entering a component$ with destructured props, cleared on exit.
    pub(crate) in_component_props: bool,

    /// Flag indicating _wrapProp import needs to be added.
    /// Set when any prop identifier or signal.value access is wrapped.
    pub(crate) needs_wrap_prop_import: bool,

    /// Hoisted functions for _fnSignal (hoisted_name, hoisted_fn_expr, hoisted_str).
    /// These are emitted at module top before the component code.
    hoisted_fns: Vec<(String, Expression<'gen>, String)>,

    /// Counter for hoisted function names (_hf0, _hf1, ...).
    hoisted_fn_counter: usize,

    /// Flag indicating _fnSignal import needs to be added.
    needs_fn_signal_import: bool,

    /// Pending bind directives for current element: (is_checked, signal_expr)
    /// Collected during attribute processing and applied at element exit.
    pub(crate) pending_bind_directives: Vec<(bool, Expression<'gen>)>,

    /// Pending on:input handlers for current element.
    /// Used to merge with bind handlers when both exist on same element.
    pending_on_input_handlers: Vec<Expression<'gen>>,

    /// Flag indicating _val import needs to be added (bind:value).
    pub(crate) needs_val_import: bool,

    /// Flag indicating _chk import needs to be added (bind:checked).
    pub(crate) needs_chk_import: bool,

    /// Flag indicating inlinedQrl import needs to be added.
    pub(crate) needs_inlined_qrl_import: bool,

    /// Tracks all module exports for segment file import generation.
    /// When QRL segment files reference symbols that are exports from the source file,
    /// those segments need to import from the source file (e.g., "./test").
    /// Key: local name of the exported symbol
    /// Value: ExportInfo with local_name, exported_name, is_default, source
    pub(crate) export_by_name: HashMap<String, ExportInfo>,

    /// Synthesized imports to be emitted at module top during finalization.
    /// Maps source path to set of import names for deduplication and merging.
    /// Key: source path (e.g., "@qwik.dev/core"), Value: set of ImportId
    synthesized_imports: HashMap<String, BTreeSet<ImportId>>,

    /// Context stack for entry strategy component grouping.
    /// Tracks names as AST is traversed (file name, function names, component names,
    /// JSX elements, attributes) for PerComponentStrategy and SmartStrategy.
    pub(crate) stack_ctxt: Vec<String>,

    /// Entry policy for determining how segments are grouped for bundling.
    /// Parsed from TransformOptions.entry_strategy at initialization.
    entry_policy: Box<dyn EntryPolicy>,

    /// Tracks imported identifiers for const replacement.
    /// Used to find aliased imports like `import { isServer as s }` from @qwik.dev/core/build.
    import_tracker: ImportTracker,

    /// Tracks nesting depth of loops (for/while/for-in/for-of/map callbacks).
    /// Used to determine if QRLs inside loops need special handling for iteration variables.
    loop_depth: u32,

    /// Stack of iteration variable names per loop level.
    /// Each loop level can have multiple iteration variables (e.g., `(v, idx)` in map callback).
    /// Used to pass iteration variables via `q:p` prop instead of capture.
    iteration_var_stack: Vec<Vec<Id>>,

    /// Tracks aliased $ marker functions that should skip QRL transformation.
    /// When `component$` or `$` is imported as a different name (e.g., `component$ as Component`),
    /// the aliased name is added here. Calls using the alias will NOT be transformed as QRLs.
    /// This matches SWC's skip_transform behavior.
    skip_transform_names: HashSet<String>,
}

impl<'gen> TransformGenerator<'gen> {
    pub(crate) fn new(
        source_info: &'gen SourceInfo,
        options: TransformOptions,
        scope: Option<String>,
        allocator: &'gen Allocator,
    ) -> Self {
        let qwik_core_import_path = PathBuf::from("@qwik/core");
        let builder = AstBuilder::new(allocator);
        let entry_policy = parse_entry_strategy(&options.entry_strategy);
        Self {
            options,
            components: Vec::new(),
            app: OptimizedApp::default(),
            errors: Vec::new(),
            builder,
            depth: 0,
            segment_stack: Vec::new(),
            segment_builder: SegmentBuilder::new(),
            symbol_by_name: Default::default(),
            component_stack: Vec::new(),
            qrl_stack: Vec::new(),
            import_stack: vec![BTreeSet::new()],
            const_stack: vec![BTreeSet::new()],
            import_by_symbol: HashMap::default(),
            removed: HashMap::new(),
            source_info,
            scope,
            jsx_stack: Vec::new(),
            jsx_key_counter: 0,
            expr_is_const_stack: Vec::new(),
            replace_expr: None,
            decl_stack: vec![Vec::new()],
            jsx_element_is_native: Vec::new(),
            props_identifiers: HashMap::new(),
            in_component_props: false,
            needs_wrap_prop_import: false,
            hoisted_fns: Vec::new(),
            hoisted_fn_counter: 0,
            needs_fn_signal_import: false,
            pending_bind_directives: Vec::new(),
            pending_on_input_handlers: Vec::new(),
            needs_val_import: false,
            needs_chk_import: false,
            needs_inlined_qrl_import: false,
            export_by_name: HashMap::new(),
            synthesized_imports: HashMap::new(),
            stack_ctxt: Vec::with_capacity(16),
            entry_policy,
            import_tracker: ImportTracker::new(),
            loop_depth: 0,
            iteration_var_stack: Vec::new(),
            skip_transform_names: HashSet::new(),
        }
    }

    /// Adds a synthesized import to be emitted at module finalization.
    /// Imports from the same source are automatically merged.
    fn add_synthesized_import(&mut self, name: ImportId, source: &str) {
        self.synthesized_imports
            .entry(source.to_string())
            .or_insert_with(BTreeSet::new)
            .insert(name);
    }

    /// Finalizes all synthesized imports and emits them at module top.
    /// Merges imports from the same source into single import statements.
    fn finalize_imports(&mut self) -> Vec<Import> {
        self.synthesized_imports
            .drain()
            .map(|(source, names)| Import::new(names.into_iter().collect(), &source))
            .collect()
    }

    /// Returns the current context stack for entry strategy tracking.
    /// Used primarily for testing to verify stack_ctxt tracking works correctly.
    #[cfg(test)]
    pub fn current_context(&self) -> &[String] {
        &self.stack_ctxt
    }

    fn is_recording(&self) -> bool {
        self.segment_stack
            .last()
            .map(|s| s.is_qrl())
            .unwrap_or(false)
    }

    pub(crate) fn render_segments(&self) -> String {
        let ss: Vec<String> = self
            .segment_stack
            .iter()
            // .filter(|s| !matches!(s, Segment::IndexQrl(0)))
            .map(|s| {
                let s: String = s.into();
                format!("/{}", s)
            })
            .collect();

        ss.concat()
    }

    pub(crate) fn descend(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        }
    }

    pub(crate) fn ascend(&mut self) {
        self.depth += 1;
    }

    pub(crate) fn debug<T: AsRef<str>>(&self, s: T, traverse_ctx: &TraverseCtx<'_, ()>) {
        if DEBUG {
            let scope_id = traverse_ctx.current_scope_id();
            let indent = "--".repeat(scope_id.index());
            let prefix = format!("|{}", indent);
            println!(
                "{prefix}[SCOPE {:?}, RECORDING: {}]{}. Segments: {}",
                scope_id,
                self.is_recording(),
                s.as_ref(),
                self.render_segments()
            );
        }
    }

    pub(crate) fn new_segment<T: AsRef<str>>(&mut self, input: T) -> Segment {
        self.segment_builder.new_segment(input, &self.segment_stack)
    }

    /// Builds the display name from the current segment stack.
    ///
    /// Joins segment names with underscores, handling special cases for named QRLs
    /// and indexed QRLs.
    pub(crate) fn current_display_name(&self) -> String {
        qrl_module::build_display_name(&self.segment_stack)
    }

    /// Calculates the hash for the current context.
    ///
    /// Uses the source file path, display name, and scope to generate a stable hash.
    fn current_hash(&self) -> String {
        let display_name = self.current_display_name();
        qrl_module::compute_hash(
            &self.source_info.rel_path,
            &display_name,
            self.scope.as_deref(),
        )
    }

    /// Create _fnSignal call: _fnSignal(_hfN, [captures], _hfN_str)
    fn create_fn_signal_call<'b>(
        &self,
        builder: &AstBuilder<'b>,
        hoisted_name: &'b str,
        captures: Vec<Expression<'b>>,
        str_name: &'b str,
    ) -> Expression<'b> {
        builder.expression_call(
            SPAN,
            builder.expression_identifier(SPAN, "_fnSignal"),
            None::<OxcBox<TSTypeParameterInstantiation<'b>>>,
            builder.vec_from_array([
                Argument::from(builder.expression_identifier(SPAN, hoisted_name)),
                Argument::from(builder.expression_array(
                    SPAN,
                    builder.vec_from_iter(
                        captures.into_iter().map(ArrayExpressionElement::from)
                    ),
                )),
                Argument::from(builder.expression_identifier(SPAN, str_name)),
            ]),
            false,
        )
    }

    /// Get all imported symbol names for is_const_expr checking.
    pub(crate) fn get_imported_names(&self) -> HashSet<String> {
        self.import_by_symbol
            .values()
            .flat_map(|import| import.names.iter())
            .filter_map(|id| match id {
                ImportId::Named(name) | ImportId::Default(name) => Some(name.clone()),
                ImportId::NamedWithAlias(_, local) => Some(local.clone()),
                ImportId::Namespace(_) => None,
            })
            .collect()
    }

    /// Check if this expression is a prop identifier that needs _wrapProp wrapping.
    /// Returns Some((raw_props_name, prop_key)) if wrapping is needed.
    pub(crate) fn should_wrap_prop(&self, expr: &Expression) -> Option<(String, String)> {
        if let Expression::Identifier(ident) = expr {
            // Match by name only since props_identifiers uses scope from declaration
            for ((name, _scope_id), prop_key) in &self.props_identifiers {
                if name == &ident.name.to_string() {
                    return Some(("_rawProps".to_string(), prop_key.clone()));
                }
            }
        }
        None
    }

    /// Check if this expression is a signal.value access that needs _wrapProp wrapping.
    pub(crate) fn should_wrap_signal_value(&self, expr: &Expression) -> bool {
        if let Expression::StaticMemberExpression(static_member) = expr {
            if static_member.property.name == "value" {
                // Wrap any .value access - runtime will determine if it's actually a signal
                return true;
            }
        }
        false
    }

    /// Check if attribute name is a bind directive.
    /// Returns Some(true) for bind:checked, Some(false) for bind:value, None otherwise.
    pub(crate) fn is_bind_directive(name: &str) -> Option<bool> {
        if name == "bind:value" {
            Some(false)  // is_checked = false
        } else if name == "bind:checked" {
            Some(true)   // is_checked = true
        } else {
            None
        }
    }

    /// Create inlinedQrl for bind handler.
    /// inlinedQrl(_val, "_val", [signal]) for bind:value
    /// inlinedQrl(_chk, "_chk", [signal]) for bind:checked
    pub(crate) fn create_bind_handler<'b>(
        builder: &AstBuilder<'b>,
        is_checked: bool,
        signal_expr: Expression<'b>,
    ) -> Expression<'b> {
        let helper = if is_checked { "_chk" } else { "_val" };

        builder.expression_call(
            SPAN,
            builder.expression_identifier(SPAN, "inlinedQrl"),
            None::<OxcBox<TSTypeParameterInstantiation<'b>>>,
            builder.vec_from_array([
                Argument::from(builder.expression_identifier(SPAN, helper)),
                Argument::from(builder.expression_string_literal(SPAN, helper, None)),
                Argument::from(builder.expression_array(
                    SPAN,
                    builder.vec1(ArrayExpressionElement::from(signal_expr)),
                )),
            ]),
            false,
        )
    }

    /// Merge bind handler with existing on:input handler.
    /// Returns array expression: [existingHandler, bindHandler]
    pub(crate) fn merge_event_handlers<'b>(
        builder: &AstBuilder<'b>,
        existing: Expression<'b>,
        bind_handler: Expression<'b>,
    ) -> Expression<'b> {
        // If existing is already an array, add bind_handler to it
        match existing {
            Expression::ArrayExpression(arr) => {
                // Clone and add bind_handler
                let mut elements: OxcVec<'b, ArrayExpressionElement<'b>> =
                    builder.vec_with_capacity(arr.elements.len() + 1);
                for elem in arr.elements.iter() {
                    elements.push(elem.clone_in(builder.allocator));
                }
                elements.push(ArrayExpressionElement::from(bind_handler));
                builder.expression_array(SPAN, elements)
            }
            _ => {
                // Create new array with both handlers
                builder.expression_array(
                    SPAN,
                    builder.vec_from_array([
                        ArrayExpressionElement::from(existing),
                        ArrayExpressionElement::from(bind_handler),
                    ]),
                )
            }
        }
    }
}

fn move_expression<'gen>(
    builder: &AstBuilder<'gen>,
    expr: &mut Expression<'gen>,
) -> Expression<'gen> {
    let span = expr.span().clone();
    std::mem::replace(expr, builder.expression_null_literal(span))
}

const DEBUG: bool = true;
const DUMP_FINAL_AST: bool = false;

impl<'a> Traverse<'a, ()> for TransformGenerator<'a> {
    fn enter_program(&mut self, node: &mut Program<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        println!("ENTERING PROGRAM {}", self.source_info.file_name);
    }

    fn exit_program(&mut self, node: &mut Program<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        println!("EXITING PROGRAM {}", self.source_info.file_name);

        // Collect synthesized imports based on transformation flags
        if self.needs_wrap_prop_import {
            self.add_synthesized_import(ImportId::Named("_wrapProp".into()), QWIK_CORE_SOURCE);
        }
        if self.needs_fn_signal_import {
            self.add_synthesized_import(ImportId::Named("_fnSignal".into()), QWIK_CORE_SOURCE);
        }
        if self.needs_val_import {
            self.add_synthesized_import(ImportId::Named("_val".into()), QWIK_CORE_SOURCE);
        }
        if self.needs_chk_import {
            self.add_synthesized_import(ImportId::Named("_chk".into()), QWIK_CORE_SOURCE);
        }
        if self.needs_inlined_qrl_import {
            self.add_synthesized_import(ImportId::Named("inlinedQrl".into()), QWIK_CORE_SOURCE);
        }

        // Finalize and emit synthesized imports
        // These are merged by source and emitted at module top
        let synthesized = self.finalize_imports();
        for import in synthesized {
            if let Some(imports) = self.import_stack.last_mut() {
                imports.insert(import);
            }
        }

        // Emit hoisted functions at top of module: const _hf0 = ...; const _hf0_str = "...";
        // Insert in reverse order so they appear in order at top
        for (name, fn_expr, str_val) in self.hoisted_fns.drain(..).rev() {
            // const _hfN = (p0, p1) => expr;
            let fn_stmt = Statement::VariableDeclaration(ctx.ast.alloc(ctx.ast.variable_declaration(
                SPAN,
                VariableDeclarationKind::Const,
                ctx.ast.vec1(ctx.ast.variable_declarator(
                    SPAN,
                    VariableDeclarationKind::Const,
                    ctx.ast.binding_pattern_binding_identifier(SPAN, ctx.ast.atom(&name)),
                    NONE,
                    Some(fn_expr),
                    false,
                )),
                false,
            )));
            node.body.insert(0, fn_stmt);

            // const _hfN_str = "expr";
            let str_name = format!("{}_str", name);
            let str_stmt = Statement::VariableDeclaration(ctx.ast.alloc(ctx.ast.variable_declaration(
                SPAN,
                VariableDeclarationKind::Const,
                ctx.ast.vec1(ctx.ast.variable_declarator(
                    SPAN,
                    VariableDeclarationKind::Const,
                    ctx.ast.binding_pattern_binding_identifier(SPAN, ctx.ast.atom(&str_name)),
                    NONE,
                    Some(ctx.ast.expression_string_literal(SPAN, ctx.ast.atom(&str_val), None)),
                    false,
                )),
                false,
            )));
            node.body.insert(1, str_stmt); // Insert after the function
        }

        if let Some(tree) = self.import_stack.pop() {
            tree.iter().for_each(|import| {
                node.body.insert(0, import.into_in(ctx.ast.allocator));
            });
        }

        ImportCleanUp::clean_up(node, ctx.ast.allocator);

        let codegen_options = CodegenOptions {
            minify: self.options.minify,
            ..Default::default()
        };
        let codegen = Codegen::new().with_options(codegen_options);

        let body = codegen.build(node).code;

        self.app = OptimizedApp {
            body,
            components: self.components.clone(),
        };

        if DEBUG && DUMP_FINAL_AST {
            println!(
                "-------------------FINAL AST DUMP--------------------\n{:#?}",
                node
            );
        }
    }

    fn enter_call_expression(
        &mut self,
        node: &mut CallExpression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        self.ascend();
        self.debug(format!("ENTER: CallExpression, {:?}", node), ctx);

        if let Some(mut is_const) = self.expr_is_const_stack.last_mut() {
            *is_const = false;
        }

        let name = node.callee_name().unwrap_or_default().to_string();

        // Check if this is an aliased $ marker function that should skip QRL transformation
        // e.g., `import { component$ as Component }` - calls to `Component(...)` skip transform
        if self.skip_transform_names.contains(&name) {
            if DEBUG {
                println!("Skipping QRL transform for aliased call: {}", name);
            }
            // Don't push to import_stack or stack_ctxt - this is not a QRL call
            // Just push a regular segment for tracking
            let segment: Segment = self.new_segment(&name);
            println!("push segment (skip transform): {segment}");
            self.segment_stack.push(segment);
            return;
        }

        if name.ends_with(MARKER_SUFFIX) {
            self.import_stack.push(BTreeSet::new());
            // Push marker function name to stack_ctxt for entry strategy (SWC fold_call_expr)
            self.stack_ctxt.push(name.clone());
        }

        // Check for component$ with destructured props
        // Populate props_identifiers EARLY so JSX processing can use them for _wrapProp
        if name.starts_with("component") && name.ends_with(MARKER_SUFFIX) {
            if let Some(arg) = node.arguments.first() {
                if let Some(expr) = arg.as_expression() {
                    if let Expression::ArrowFunctionExpression(arrow) = expr {
                        // Check if first param is ObjectPattern
                        if let Some(first_param) = arrow.params.items.first() {
                            if let BindingPattern::ObjectPattern(obj_pat) = &first_param.pattern {
                                self.in_component_props = true;
                                // Populate props_identifiers NOW before JSX processing
                                // This maps local var names to original property keys
                                for prop in &obj_pat.properties {
                                    use oxc_ast::ast::BindingProperty;
                                    let BindingProperty { key, value, .. } = prop;

                                    // Get the original property key
                                    let prop_key = match key {
                                        PropertyKey::StaticIdentifier(id) => id.name.to_string(),
                                        PropertyKey::StringLiteral(s) => s.value.to_string(),
                                        _ => continue,
                                    };

                                    // Get the local binding name
                                    let local_name = match value {
                                        BindingPattern::BindingIdentifier(id) => id.name.to_string(),
                                        _ => continue,
                                    };

                                    // Store mapping: (local_name, scope_id) -> prop_key
                                    let scope_id = ctx.current_scope_id();
                                    self.props_identifiers.insert((local_name.clone(), scope_id), prop_key.clone());
                                    if DEBUG {
                                        println!("Registered prop: {} -> key {:?}", local_name, prop_key);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Detect .map() callbacks as "loop context" for QRL hoisting
        // Map callbacks should be treated like loops - iteration variables should be passed via q:p
        // instead of captured, to avoid stale closure bugs.
        if let Some(member) = node.callee.as_member_expression() {
            if member.static_property_name() == Some("map") {
                // Check if first arg is a function (arrow or function expression)
                if let Some(arg) = node.arguments.first() {
                    if let Some(expr) = arg.as_expression() {
                        let iteration_vars = match expr {
                            Expression::ArrowFunctionExpression(arrow) => {
                                // Extract iteration vars from arrow function params
                                // First param is usually the item, second is index
                                let mut vars = Vec::new();
                                for param in arrow.params.items.iter() {
                                    if let Some(ident) = param.pattern.get_binding_identifier() {
                                        // Use ScopeId::new(0) as we match by name later
                                        vars.push((ident.name.to_string(), oxc_semantic::ScopeId::new(0)));
                                    }
                                }
                                Some(vars)
                            }
                            Expression::FunctionExpression(func) => {
                                // Extract iteration vars from function expression params
                                let mut vars = Vec::new();
                                for param in &func.params.items {
                                    if let Some(ident) = param.pattern.get_binding_identifier() {
                                        vars.push((ident.name.to_string(), oxc_semantic::ScopeId::new(0)));
                                    }
                                }
                                Some(vars)
                            }
                            _ => None,
                        };

                        if let Some(vars) = iteration_vars {
                            self.loop_depth += 1;
                            self.iteration_var_stack.push(vars);
                            if DEBUG {
                                println!(
                                    "Entered .map() loop context, depth: {}, iteration vars: {:?}",
                                    self.loop_depth,
                                    self.iteration_var_stack.last()
                                );
                            }
                        }
                    }
                }
            }
        }

        let segment: Segment = self.new_segment(name);
        println!("push segment: {segment}");
        self.segment_stack.push(segment);
    }

    fn exit_call_expression(
        &mut self,
        node: &mut CallExpression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        // Pop from iteration_var_stack if this was a .map() call we tracked
        // Check if we pushed on entry (match the same pattern)
        if let Some(member) = node.callee.as_member_expression() {
            if member.static_property_name() == Some("map") {
                if let Some(arg) = node.arguments.first() {
                    if let Some(expr) = arg.as_expression() {
                        let is_function = matches!(
                            expr,
                            Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
                        );
                        if is_function && self.loop_depth > 0 {
                            self.iteration_var_stack.pop();
                            self.loop_depth -= 1;
                            if DEBUG {
                                println!(
                                    "Exited .map() loop context, depth: {}",
                                    self.loop_depth
                                );
                            }
                        }
                    }
                }
            }
        }

        // Handle component$ props destructuring BEFORE QRL extraction
        // Transform ObjectPattern parameter to _rawProps and track prop mappings
        if self.in_component_props {
            if let Some(arg) = node.arguments.first_mut() {
                if let Some(expr) = arg.as_expression_mut() {
                    if let Expression::ArrowFunctionExpression(arrow) = expr {
                        use crate::props_destructuring::PropsDestructuring;
                        let mut props_trans = PropsDestructuring::new(
                            ctx.ast.allocator,
                            None, // component_ident not needed here, we already know it's component$
                        );
                        if props_trans.transform_component_props(arrow, &ctx.ast) {
                            // If rest pattern present, inject _restProps statement
                            // Do this BEFORE moving identifiers
                            if props_trans.rest_id.is_some() {
                                if let Some(rest_stmt) = props_trans.generate_rest_stmt(&ctx.ast) {
                                    // Inject at start of function body
                                    // arrow.body is a FunctionBody struct with statements field
                                    // arrow.expression indicates if body was originally an expression

                                    if arrow.expression {
                                        // Expression body: convert to block with rest stmt + return
                                        // The expression is stored in statements[0] as ExpressionStatement
                                        if let Some(Statement::ExpressionStatement(expr_stmt)) = arrow.body.statements.pop() {
                                            let return_stmt = ctx.ast.statement_return(SPAN, Some(expr_stmt.unbox().expression));
                                            let mut new_stmts = ctx.ast.vec_with_capacity(2);
                                            new_stmts.push(rest_stmt);
                                            new_stmts.push(return_stmt);
                                            arrow.body.statements = new_stmts;
                                            arrow.expression = false;
                                        }
                                    } else {
                                        // Block body: prepend _restProps statement
                                        arrow.body.statements.insert(0, rest_stmt);
                                    }
                                }

                                // Add _restProps import
                                if let Some(imports) = self.import_stack.last_mut() {
                                    imports.insert(Import::new(
                                        vec![ImportId::Named("_restProps".into())],
                                        QWIK_CORE_SOURCE,
                                    ));
                                }
                            }

                            // Store prop identifiers for later replacement
                            self.props_identifiers = props_trans.identifiers;
                        }
                    }
                }
            }
            self.in_component_props = false;
        }

        let segment = self.segment_stack.last();

        if let Some(segment) = segment {
            if segment.is_qrl() {
                let comp = node.arguments.first().map(|arg0| {
                    // Collect all identifiers referenced in the QRL body
                    let descendent_idents = {
                        use crate::collector::IdentCollector;
                        let mut collector = IdentCollector::new();
                        if let Some(expr) = arg0.as_expression() {
                            use oxc_ast_visit::Visit;
                            collector.visit_expression(expr);
                        }
                        collector.get_words()
                    };

                    // Get all declarations from parent scopes (flatten decl_stack)
                    let all_decl: Vec<IdPlusType> = self
                        .decl_stack
                        .iter()
                        .flat_map(|v| v.iter())
                        .cloned()
                        .collect();

                    // Partition into valid (Var) and invalid (Fn, Class)
                    let (decl_collect, invalid_decl): (Vec<_>, Vec<_>) = all_decl
                        .into_iter()
                        .partition(|(_, t)| matches!(t, IdentType::Var(_)));

                    // Check for invalid function/class references
                    for (id, ident_type) in &invalid_decl {
                        if descendent_idents.iter().any(|ident| ident == id) {
                            let type_name = match ident_type {
                                IdentType::Fn => "function",
                                IdentType::Class => "class",
                                IdentType::Var(_) => unreachable!(),
                            };
                            // Log warning for now - full error integration in later plan
                            if DEBUG {
                                println!(
                                    "Warning: Reference to {} '{}' cannot be used inside a QRL scope",
                                    type_name, id.0
                                );
                            }
                        }
                    }

                    // Compute captured variables (scoped_idents)
                    let (scoped_idents, _is_const) =
                        qrl_module::compute_scoped_idents(&descendent_idents, &decl_collect);

                    // Get imports collected for this QRL
                    // These are identifiers that will be imported, so we should exclude them from scoped_idents
                    let imports: Vec<Import> = self
                        .import_stack
                        .pop()
                        .unwrap_or_default()
                        .iter()
                        .cloned()
                        .collect();

                    // Collect imported identifier names to filter from scoped_idents
                    // Identifiers that are imported should not be captured via useLexicalScope
                    let imported_names: HashSet<String> = imports
                        .iter()
                        .flat_map(|import| import.names.iter())
                        .filter_map(|id| match id {
                            ImportId::Named(name) => Some(name.clone()),
                            ImportId::Default(name) => Some(name.clone()),
                            ImportId::NamedWithAlias(_, local) => Some(local.clone()),
                            ImportId::Namespace(_) => None, // Namespace imports are accessed via member expr
                        })
                        .collect();

                    // Filter out identifiers that will be imported
                    let scoped_idents: Vec<Id> = scoped_idents
                        .into_iter()
                        .filter(|(name, _)| !imported_names.contains(name))
                        .collect();

                    // Collect referenced exports - identifiers in QRL body that are source exports
                    // These need to be imported in segment files from the source file
                    let referenced_exports: Vec<ExportInfo> = descendent_idents
                        .iter()
                        .filter_map(|(name, _)| {
                            // Skip if it's an import (will be handled via imports)
                            if imported_names.contains(name) {
                                return None;
                            }
                            // Skip if it's a captured variable (handled via useLexicalScope)
                            if scoped_idents.iter().any(|(n, _)| n == name) {
                                return None;
                            }
                            // Check if it's a source export
                            self.export_by_name.get(name).cloned()
                        })
                        .collect();

                    // Log referenced exports for debugging
                    if DEBUG && !referenced_exports.is_empty() {
                        println!(
                            "QRL references exports: {:?}",
                            referenced_exports.iter().map(|e| &e.local_name).collect::<Vec<_>>()
                        );
                    }

                    // Log captured variables for debugging
                    if DEBUG && !scoped_idents.is_empty() {
                        println!(
                            "QRL captures: {:?}",
                            scoped_idents.iter().map(|(name, _)| name).collect::<Vec<_>>()
                        );
                    }

                    // Build ctx_name from the callee name (e.g., "component$", "onClick$", "$")
                    let ctx_name = node.callee_name().unwrap_or("$").to_string();

                    // Build display_name from segment stack
                    let display_name = self.current_display_name();

                    // Get hash from source_info and segments (calculated during Id::new)
                    let hash = self.current_hash();

                    // Determine parent segment (for nested QRLs)
                    // Look for the first QRL segment in the stack before the current one
                    let parent_segment = self.segment_stack.iter().rev().skip(1).find_map(|s| {
                        if s.is_qrl() {
                            Some(s.to_string())
                        } else {
                            None
                        }
                    });

                    // Create SegmentData with all collected metadata including referenced exports
                    let segment_data = SegmentData::new_with_exports(
                        &ctx_name,
                        display_name,
                        hash,
                        self.source_info.rel_path.clone(),
                        scoped_idents,
                        descendent_idents, // local_idents are all identifiers used in segment
                        parent_segment,
                        referenced_exports,
                    );

                    // Compute entry grouping using the entry policy and stack_ctxt
                    let entry = self.entry_policy.get_entry_for_sym(&self.stack_ctxt, &segment_data);

                    QrlComponent::from_call_expression_argument(
                        arg0,
                        imports,
                        &self.segment_stack,
                        &self.scope,
                        &self.options,
                        self.source_info,
                        Some(segment_data),
                        entry,
                        ctx.ast.allocator,
                    )
                });

                if let Some(comp) = &comp {
                    let qrl = &comp.qrl;
                    let qrl = qrl.clone();
                    *node = qrl.into_call_expression(
                        ctx,
                        &mut self.symbol_by_name,
                        &mut self.import_by_symbol,
                    );
                }

                if let Some(comp) = comp {
                    let import: Import = comp.qrl.qrl_type.clone().into();
                    self.qrl_stack.push(comp.qrl.clone());
                    self.components.push(comp);
                    let parent_scope = ctx
                        .ancestor_scopes()
                        .last()
                        .map(|s: oxc_syntax::scope::ScopeId| s.index())
                        .unwrap_or_default();
                    self.import_stack.last_mut().unwrap().insert(import);
                }
            }
        }

        self.segment_stack.pop();

        // Pop stack_ctxt if we pushed a marker function name (SWC fold_call_expr)
        let name = node.callee_name().unwrap_or_default().to_string();
        if name.ends_with(MARKER_SUFFIX) {
            self.stack_ctxt.pop();
        }
    }

    fn enter_member_expression(
        &mut self,
        node: &mut MemberExpression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        if let Some(mut is_const) = self.expr_is_const_stack.last_mut() {
            *is_const = false;
        }
    }

    fn enter_function(&mut self, node: &mut Function<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        let segment: Segment = node
            .name()
            .map(|n| self.new_segment(n))
            .unwrap_or(self.new_segment("$"));
        println!("push segment: {segment}");
        self.segment_stack.push(segment);

        // Track function name as Fn declaration in parent scope
        // and push to stack_ctxt for entry strategy (SWC fold_fn_decl)
        if let Some(name) = node.name() {
            if let Some(current_scope) = self.decl_stack.last_mut() {
                let scope_id = ctx.current_scope_id();
                current_scope.push(((name.to_string(), scope_id), IdentType::Fn));
            }
            // Push function name to stack_ctxt
            self.stack_ctxt.push(name.to_string());
        }

        // Push new scope for function body
        self.decl_stack.push(Vec::new());

        // Track function parameters in the new scope
        if let Some(current_scope) = self.decl_stack.last_mut() {
            for param in &node.params.items {
                if let Some(ident) = param.pattern.get_binding_identifier() {
                    let scope_id = ctx.current_scope_id();
                    // Parameters are always treated as non-const for capture purposes
                    current_scope.push(((ident.name.to_string(), scope_id), IdentType::Var(false)));
                }
            }
        }
    }

    fn exit_function(&mut self, node: &mut Function<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        let popped = self.segment_stack.pop();
        println!("pop segment: {popped:?}");

        // Pop function scope from decl_stack
        self.decl_stack.pop();

        // Pop stack_ctxt if we pushed a function name (SWC fold_fn_decl)
        if node.name().is_some() {
            self.stack_ctxt.pop();
        }
    }

    fn enter_class(&mut self, node: &mut Class<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        // Track class name as Class declaration in parent scope
        // and push to stack_ctxt for entry strategy (SWC fold_class_decl)
        if let Some(ident) = &node.id {
            if let Some(current_scope) = self.decl_stack.last_mut() {
                let scope_id = ctx.current_scope_id();
                current_scope.push(((ident.name.to_string(), scope_id), IdentType::Class));
            }
            // Push class name to stack_ctxt
            self.stack_ctxt.push(ident.name.to_string());
        }

        // Push new scope for class body
        self.decl_stack.push(Vec::new());
    }

    fn exit_class(&mut self, node: &mut Class<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        // Pop class scope from decl_stack
        self.decl_stack.pop();

        // Pop stack_ctxt if we pushed a class name (SWC fold_class_decl)
        if node.id.is_some() {
            self.stack_ctxt.pop();
        }
    }

    fn enter_export_named_declaration(
        &mut self,
        node: &mut ExportNamedDeclaration<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        // Track named exports for segment file import generation
        let source = node.source.as_ref().map(|s| s.value.to_string());

        // Export with declaration: `export const Foo = ...`, `export function bar() {}`
        if let Some(decl) = &node.declaration {
            match decl {
                Declaration::VariableDeclaration(var_decl) => {
                    for declarator in &var_decl.declarations {
                        if let Some(ident) = declarator.id.get_binding_identifier() {
                            let name = ident.name.to_string();
                            self.export_by_name.insert(name.clone(), ExportInfo {
                                local_name: name.clone(),
                                exported_name: name,
                                is_default: false,
                                source: source.clone(),
                            });
                        }
                    }
                }
                Declaration::FunctionDeclaration(fn_decl) => {
                    if let Some(ident) = &fn_decl.id {
                        let name = ident.name.to_string();
                        self.export_by_name.insert(name.clone(), ExportInfo {
                            local_name: name.clone(),
                            exported_name: name,
                            is_default: false,
                            source: source.clone(),
                        });
                    }
                }
                Declaration::ClassDeclaration(class_decl) => {
                    if let Some(ident) = &class_decl.id {
                        let name = ident.name.to_string();
                        self.export_by_name.insert(name.clone(), ExportInfo {
                            local_name: name.clone(),
                            exported_name: name,
                            is_default: false,
                            source: source.clone(),
                        });
                    }
                }
                _ => {}
            }
        }

        // Export specifiers: `export { foo, bar as baz }`
        for specifier in &node.specifiers {
            let local_name = specifier.local.name().to_string();
            let exported_name = specifier.exported.name().to_string();
            self.export_by_name.insert(local_name.clone(), ExportInfo {
                local_name,
                exported_name,
                is_default: false,
                source: source.clone(),
            });
        }
    }

    fn enter_export_default_declaration(
        &mut self,
        node: &mut ExportDefaultDeclaration<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        // Track default exports for segment file import generation
        let local_name = match &node.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(fn_decl) => {
                if let Some(ident) = &fn_decl.id {
                    ident.name.to_string()
                } else {
                    "_default".to_string()
                }
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class_decl) => {
                if let Some(ident) = &class_decl.id {
                    ident.name.to_string()
                } else {
                    "_default".to_string()
                }
            }
            ExportDefaultDeclarationKind::Identifier(ident) => {
                ident.name.to_string()
            }
            _ => "_default".to_string(),
        };

        self.export_by_name.insert(local_name.clone(), ExportInfo {
            local_name,
            exported_name: "default".to_string(),
            is_default: true,
            source: None,
        });
    }

    fn enter_arrow_function_expression(
        &mut self,
        node: &mut ArrowFunctionExpression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        // Push new scope for arrow function body
        self.decl_stack.push(Vec::new());

        // Track arrow function parameters in the new scope
        if let Some(current_scope) = self.decl_stack.last_mut() {
            for param in &node.params.items {
                if let Some(ident) = param.pattern.get_binding_identifier() {
                    let scope_id = ctx.current_scope_id();
                    // Parameters are always treated as non-const for capture purposes
                    current_scope.push(((ident.name.to_string(), scope_id), IdentType::Var(false)));
                }
            }
        }
    }

    fn exit_arrow_function_expression(
        &mut self,
        node: &mut ArrowFunctionExpression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        // Pop arrow function scope from decl_stack
        self.decl_stack.pop();
    }

    fn exit_argument(&mut self, node: &mut Argument<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        if let Argument::CallExpression(call_expr) = node {
            let qrl = self.qrl_stack.pop();

            if let Some(qrl) = qrl {
                let idr = qrl.into_identifier_reference(
                    ctx,
                    &mut self.symbol_by_name,
                    &mut self.import_by_symbol,
                );
                let args: OxcVec<'a, Argument<'a>> = qrl.into_in(ctx.ast.allocator);

                call_expr.callee = Expression::Identifier(OxcBox::new_in(idr, ctx.ast.allocator));
                call_expr.arguments = args
            }
        }
    }

    fn enter_variable_declarator(
        &mut self,
        node: &mut VariableDeclarator<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        self.ascend();
        let id = &node.id;

        let segment_name: String = id
            .get_identifier_name()
            .iter()
            .map(|s| s.to_string())
            .collect();

        let s: Segment = self.new_segment(segment_name.clone());
        self.segment_stack.push(s);

        // Push variable name to stack_ctxt for entry strategy tracking (SWC fold_var_declarator)
        if !segment_name.is_empty() {
            self.stack_ctxt.push(segment_name);
        }

        let is_const = node.kind == VariableDeclarationKind::Const;

        if self.options.transpile_jsx {
            self.expr_is_const_stack.push(is_const);
        }

        // Track variable declaration in decl_stack for scope capture
        if let Some(current_scope) = self.decl_stack.last_mut() {
            if let Some(ident) = id.get_binding_identifier() {
                let scope_id = ctx.current_scope_id();
                current_scope.push(((ident.name.to_string(), scope_id), IdentType::Var(is_const)));
            }
        }

        if let Some(name) = id.get_identifier_name() {
            /// Adds symbol and import information in the case this declaration ends up being referenced in
            /// an exported component.
            let grandparent = ctx.ancestor(1);
            if let Ancestor::ExportNamedDeclarationDeclaration(export) = grandparent {
                let symbol_id = id.get_binding_identifier().and_then(|b| b.symbol_id.get());
                if let Some(symbol_id) = symbol_id {
                    self.symbol_by_name.insert(name.to_string(), symbol_id);
                    let import_id = ImportId::Named(name.to_string());
                    self.import_by_symbol.insert(
                        symbol_id,
                        Import::new(
                            vec![import_id],
                            self.source_info.rel_import_path().to_string_lossy(),
                        ),
                    );
                }
            }
        }
    }

    fn exit_variable_declarator(
        &mut self,
        node: &mut VariableDeclarator<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        if let Some(init) = &mut node.init {
            let qrl = self.qrl_stack.pop();
            if let Some(qrl) = qrl {
                node.init = Some(qrl.into_expression(
                    ctx,
                    &mut self.symbol_by_name,
                    &mut self.import_by_symbol,
                ));
            }
        }

        // If this definition is constant, mark it as constant within the current scope
        if self.options.transpile_jsx && self.expr_is_const_stack.pop().unwrap_or_default() {
            if let Some(consts) = self.const_stack.last_mut() {
                let symbol_id = node
                    .id
                    .get_binding_identifier()
                    .and_then(|b| b.symbol_id.get());
                if let Some(symbol_id) = symbol_id {
                    consts.insert(symbol_id);
                }
            }
        }

        // Pop stack_ctxt if we pushed a variable name (SWC fold_var_declarator)
        if node.id.get_identifier_name().is_some() {
            self.stack_ctxt.pop();
        }

        let popped = self.segment_stack.pop();
        println!("pop segment: {popped:?}");
    }

    fn enter_block_statement(
        &mut self,
        node: &mut BlockStatement<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        if (self.options.transpile_jsx) {
            self.const_stack.push(BTreeSet::new());
        }
    }

    fn exit_block_statement(
        &mut self,
        node: &mut BlockStatement<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        if (self.options.transpile_jsx) {
            self.const_stack.pop();
        }
    }

    fn enter_expression_statement(
        &mut self,
        node: &mut ExpressionStatement<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        self.ascend();
        self.debug("ENTER: ExpressionStatement", ctx);
    }

    fn exit_expression_statement(
        &mut self,
        node: &mut ExpressionStatement<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        self.debug("EXIT: ExpressionStatement", ctx);
        self.descend();
    }

    fn exit_expression(&mut self, node: &mut Expression<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        if let Some(expr) = self.replace_expr.take() {
            println!("Replacing expression on exit");
            *node = expr;
        }
    }

    fn enter_jsx_element(&mut self, node: &mut JSXElement<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        jsx::enter_jsx_element(self, node, ctx);
    }

    fn exit_jsx_element(&mut self, node: &mut JSXElement<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        jsx::exit_jsx_element(self, node, ctx);
    }

    fn enter_jsx_fragment(&mut self, node: &mut JSXFragment<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        jsx::enter_jsx_fragment(self, node, ctx);
    }

    fn exit_jsx_fragment(&mut self, node: &mut JSXFragment<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        jsx::exit_jsx_fragment(self, node, ctx);
    }

    fn exit_jsx_spread_attribute(
        &mut self,
        node: &mut JSXSpreadAttribute<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        jsx::exit_jsx_spread_attribute(self, node, ctx);
    }

    fn enter_jsx_attribute(&mut self, node: &mut JSXAttribute<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        jsx::enter_jsx_attribute(self, node, ctx);
    }

    fn exit_jsx_attribute(&mut self, node: &mut JSXAttribute<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        jsx::exit_jsx_attribute(self, node, ctx);
    }

    fn exit_jsx_attribute_value(
        &mut self,
        node: &mut JSXAttributeValue<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        jsx::exit_jsx_attribute_value(self, node, ctx);
    }

    fn exit_jsx_child(&mut self, node: &mut JSXChild<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        jsx::exit_jsx_child(self, node, ctx);
    }

    fn exit_return_statement(
        &mut self,
        node: &mut ReturnStatement<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        if let Some(expr) = &node.argument {
            if expr.is_qrl_replaceable() {
                let qrl = self.qrl_stack.pop();
                if let Some(qrl) = qrl {
                    let expression = qrl.into_expression(
                        ctx,
                        &mut self.symbol_by_name,
                        &mut self.import_by_symbol,
                    );
                    node.argument = Some(expression);
                }
            }
        }
    }

    fn enter_statements(
        &mut self,
        node: &mut OxcVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        node.retain(|s| {
            let not_dead = !s.is_dead_code();
            let mut legal = true;
            if self.is_recording() {
                if let Some(e) = s.is_illegal_code_in_qrl() {
                    legal = false;
                    self.removed.insert(e.symbol_id(), e.clone());
                }
            }

            legal && not_dead
        });
    }

    fn exit_statements(
        &mut self,
        node: &mut OxcVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        for statement in node.iter_mut() {
            // This will determine whether the variable declaration can be replaced with just the call that is being used to initialize it.
            // e.g. `const x = componentQrl(...)` can be replaced with just `componentQrl(...)`,
            // `const Header = qrl(...)` can be replaced with qrl(...).
            // The semantics of this check are as follows: The declaration is not referenced, it is a `qrl`, and is not an export.
            if let Statement::VariableDeclaration(decl) = statement {
                if decl.declarations.len() == 1 {
                    if let Some(decl) = decl.declarations.first() {
                        let ref_count = decl.reference_count(ctx);
                        let grandparent = ctx.ancestor(1);
                        if ref_count < 1
                            && !matches!(
                                grandparent,
                                Ancestor::ExportNamedDeclarationDeclaration(_)
                            )
                        {
                            if let Some(Expression::CallExpression(expr)) = &decl.init {
                                let name = expr.callee_name().unwrap_or_default();
                                if name == QRL || name.ends_with(QRL_SUFFIX) {
                                    let ce = &**expr;
                                    let ce = ce.clone_in(ctx.ast.allocator);
                                    let ce = Expression::CallExpression(OxcBox::new_in(
                                        ce,
                                        ctx.ast.allocator,
                                    ));
                                    let ces = ctx.ast.expression_statement(SPAN, ce);
                                    let s = Statement::ExpressionStatement(OxcBox::new_in(
                                        ces,
                                        ctx.ast.allocator,
                                    ));
                                    *statement = s;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn enter_import_declaration(
        &mut self,
        node: &mut ImportDeclaration<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        self.debug(format!("{:?}", node), ctx);

        if let Some(specifiers) = &mut node.specifiers {
            let source_str = node.source.value.to_string();

            for specifier in specifiers.iter_mut() {
                // Track imports for const replacement (isServer, isBrowser, isDev from @qwik.dev/core/build)
                // and aliased $ marker imports for skip transform detection
                match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                        let imported = spec.imported.name().to_string();
                        let local = spec.local.name.to_string();
                        self.import_tracker
                            .add_import(&source_str, &imported, &local);

                        // Track aliased $ marker imports for skip transform
                        // If `component$` is imported as `Component`, add `Component` to skip_transform_names
                        // When we see a call to `Component(...)`, we won't transform it as QRL
                        if imported.ends_with(MARKER_SUFFIX) && imported != local {
                            self.skip_transform_names.insert(local.clone());
                            if DEBUG {
                                println!("Skip transform: {} (aliased from {})", local, imported);
                            }
                        }
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                        let local = spec.local.name.to_string();
                        self.import_tracker.add_import(&source_str, "default", &local);
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                        let local = spec.local.name.to_string();
                        self.import_tracker.add_import(&source_str, "*", &local);
                    }
                }

                // Recording each import by its SymbolId will allow CallExpressions within newly-created modules to
                // determine if they need to add this import to their import_stack.
                if let Some(symbol_id) = specifier.local().symbol_id.get() {
                    let source = node.source.value;

                    let local_name = specifier
                        .local()
                        .name
                        .strip_suffix(MARKER_SUFFIX)
                        .map(|s| format!("{}{}", s, QRL_SUFFIX));

                    let name = specifier
                        .name()
                        .strip_suffix(MARKER_SUFFIX)
                        .map(|s| format!("{}{}", s, QRL_SUFFIX))
                        .unwrap_or(specifier.name().to_string());

                    // We want to rename all marker imports to their QRL equivalent yet preserve the original symbol id.
                    if let Some(local_name) = local_name {
                        // ctx. symbols_mut().set_name(symbol_id, local_name.as_str());
                        let scope_id = ctx.current_scope_id();
                        ctx.scoping_mut().rename_symbol(
                            symbol_id,
                            scope_id,
                            local_name.as_str().into(),
                        );

                        let local_name = if local_name == QRL_SUFFIX {
                            QRL.to_string()
                        } else {
                            local_name
                        };

                        let name = if name == QRL_SUFFIX {
                            QRL.to_string()
                        } else {
                            name
                        };

                        self.symbol_by_name.insert(local_name.clone(), symbol_id);

                        match specifier {
                            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                                specifier.imported = ModuleExportName::IdentifierName(
                                    ctx.ast.identifier_name(SPAN, ctx.ast.atom(&name)),
                                );
                                specifier.local.name = local_name.into_in(ctx.ast.allocator);
                            }

                            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                                specifier.local.name = local_name.into_in(ctx.ast.allocator);
                            }

                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                                specifier.local.name = local_name.into_in(ctx.ast.allocator);
                            }
                        }
                    }

                    let specifier: &ImportDeclarationSpecifier = specifier;
                    self.import_by_symbol
                        .insert(symbol_id, Import::new(vec![specifier.into()], source));
                }

                // Rename qwik imports per https://github.com/QwikDev/qwik/blob/build/v2/packages/qwik/src/optimizer/core/src/rename_imports.rs
                let source = node.source.value;
                let source = ImportCleanUp::rename_qwik_imports(source);
                node.source.value = source.into_in(ctx.ast.allocator);
            }
        }
    }

    fn exit_identifier_reference(
        &mut self,
        id_ref: &mut IdentifierReference<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        if let Some(illegal_code_type) = id_ref
            .reference_id
            .get()
            // .and_then(|ref_id| ctx.symbols().references.get(ref_id))
            .map(|ref_id| ctx.scoping().get_reference(ref_id))
            .and_then(|refr| refr.symbol_id())
            .and_then(|symbol_id| self.removed.get(&symbol_id))
        {
            // Create ProcessingFailure with file path for proper C02 diagnostic
            let file_name = self.source_info.file_name.to_string();
            self.errors.push(ProcessingFailure::illegal_code(illegal_code_type, &file_name));
        }

        // Whilst visiting each identifier reference, we check if that references refers to an import.
        // If so, we store on the current import stack so that it can be used later in the `exit_expression`
        // logic that ends up creating a new module/component.
        let ref_id = id_ref.reference_id();
        if let Some(symbol_id) = ctx.scoping().get_reference(ref_id).symbol_id() {
            if let Some(import) = self.import_by_symbol.get(&symbol_id) {
                let import = import.clone();
                if !id_ref.name.ends_with(MARKER_SUFFIX) {
                    self.import_stack.last_mut().unwrap().insert(import);
                }
            }
        }
    }
}

// JSX helper functions moved to jsx.rs module (is_text_only, get_jsx_attribute_full_name,
// get_event_scope_data_from_jsx_event, create_event_name, jsx_event_to_html_attribute).
// Re-exported via crate::transform::jsx.

// QRL helper functions moved to qrl.rs module (compute_scoped_idents, build_display_name,
// compute_hash). Re-exported via crate::transform::qrl.

// TransformOptions and transform() are defined in super::options

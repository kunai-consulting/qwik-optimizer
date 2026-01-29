#![allow(unused)]

use crate::dead_code::DeadCode;
use crate::entry_strategy::*;
use crate::error::Error;
use crate::ext::*;
use crate::prelude::*;
use crate::ref_counter::RefCounter;
use crate::segment::{Segment, SegmentBuilder};
use oxc_allocator::{
    Allocator, Box as OxcBox, CloneIn, FromIn, GetAddress, HashMap as OxcHashMap, IntoIn,
    Vec as OxcVec,
};
use oxc_ast::ast::*;
use oxc_ast::{match_member_expression, AstBuilder, AstType, Comment};
use oxc_ast_visit::{Visit, VisitMut};
use oxc_codegen::{Codegen, CodegenOptions, Context, Gen};
use oxc_index::Idx;
use oxc_transformer::JsxOptions;
use std::borrow::{Borrow, Cow};
use std::collections::{BTreeSet, HashMap, HashSet};

use crate::component::*;
use crate::import_clean_up::ImportCleanUp;
use crate::macros::*;
use crate::source::Source;
use oxc_parser::Parser;
use oxc_semantic::{
    NodeId, ReferenceId, ScopeFlags, Scoping, SemanticBuilder, SemanticBuilderReturn, SymbolFlags,
    SymbolId,
};
use oxc_span::*;
use oxc_transformer::{TransformOptions as OxcTransformOptions, Transformer, TypeScriptOptions};
use oxc_traverse::{traverse_mut, Ancestor, Traverse, TraverseCtx};
use serde::{Deserialize, Serialize};
use std::cell::{Cell, RefCell};
use std::fmt::{write, Display, Pointer};

use crate::collector::Id;

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

struct JsxState<'gen> {
    is_fn: bool,
    is_text_only: bool,
    is_segment: bool,
    should_runtime_sort: bool,
    static_listeners: bool,
    static_subtree: bool,
    key_prop: Option<Expression<'gen>>,
    var_props: OxcVec<'gen, ObjectPropertyKind<'gen>>,
    const_props: OxcVec<'gen, ObjectPropertyKind<'gen>>,
    children: OxcVec<'gen, ArrayExpressionElement<'gen>>,
}

pub struct TransformGenerator<'gen> {
    pub options: TransformOptions,

    pub components: Vec<QrlComponent>,

    pub app: OptimizedApp,

    pub errors: Vec<ProcessingFailure>,

    builder: AstBuilder<'gen>,

    depth: usize,

    segment_stack: Vec<Segment>,

    segment_builder: SegmentBuilder,

    symbol_by_name: HashMap<String, SymbolId>,

    component_stack: Vec<QrlComponent>,

    qrl_stack: Vec<Qrl>,

    import_stack: Vec<BTreeSet<Import>>,

    const_stack: Vec<BTreeSet<SymbolId>>,

    import_by_symbol: HashMap<SymbolId, Import>,

    removed: HashMap<SymbolId, IllegalCodeType>,

    source_info: &'gen SourceInfo,

    scope: Option<String>,

    jsx_stack: Vec<JsxState<'gen>>,

    jsx_key_counter: u32,

    /// Marks whether each JSX attribute in the stack is var (false) or const (true).
    /// An attribute is considered var if it:
    /// - calls a function
    /// - accesses a member
    /// - is a variable that is not an import, an export, or in the const stack
    expr_is_const_stack: Vec<bool>,

    /// Used to replace the current expression in the AST. Should be set when exiting a specific
    /// type of expression (e.g., `exit_jsx_element`); this will be picked up in `exit_expression`,
    /// which will replace the entire expression with the contents of this field.
    replace_expr: Option<Expression<'gen>>,

    /// Stack of declaration scopes for tracking variable captures.
    /// Each scope level contains the identifiers declared at that level.
    /// Used by compute_scoped_idents to determine which variables need to be captured.
    decl_stack: Vec<Vec<IdPlusType>>,
}

impl<'gen> TransformGenerator<'gen> {
    fn new(
        source_info: &'gen SourceInfo,
        options: TransformOptions,
        scope: Option<String>,
        allocator: &'gen Allocator,
    ) -> Self {
        let qwik_core_import_path = PathBuf::from("@qwik/core");
        let builder = AstBuilder::new(allocator);
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
        }
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

    fn descend(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        }
    }

    fn ascend(&mut self) {
        self.depth += 1;
    }

    fn debug<T: AsRef<str>>(&self, s: T, traverse_ctx: &TraverseCtx<'_, ()>) {
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

    fn new_segment<T: AsRef<str>>(&mut self, input: T) -> Segment {
        self.segment_builder.new_segment(input, &self.segment_stack)
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
        if (name.ends_with(MARKER_SUFFIX)) {
            self.import_stack.push(BTreeSet::new());
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
                        compute_scoped_idents(&descendent_idents, &decl_collect);

                    // Store scoped_idents for later use in SegmentData (Plan 04)
                    // For now, just log them for debugging
                    if DEBUG && !scoped_idents.is_empty() {
                        println!(
                            "QRL captures: {:?}",
                            scoped_idents.iter().map(|(name, _)| name).collect::<Vec<_>>()
                        );
                    }

                    let imports = self
                        .import_stack
                        .pop()
                        .unwrap_or_default()
                        .iter()
                        .cloned()
                        .collect();

                    QrlComponent::from_call_expression_argument(
                        arg0,
                        imports,
                        &self.segment_stack,
                        &self.scope,
                        &self.options,
                        self.source_info,
                        None, // TODO: Create SegmentData in Plan 03/04
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
        if let Some(name) = node.name() {
            if let Some(current_scope) = self.decl_stack.last_mut() {
                let scope_id = ctx.current_scope_id();
                current_scope.push(((name.to_string(), scope_id), IdentType::Fn));
            }
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
    }

    fn enter_class(&mut self, node: &mut Class<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        // Track class name as Class declaration in parent scope
        if let Some(ident) = &node.id {
            if let Some(current_scope) = self.decl_stack.last_mut() {
                let scope_id = ctx.current_scope_id();
                current_scope.push(((ident.name.to_string(), scope_id), IdentType::Class));
            }
        }

        // Push new scope for class body
        self.decl_stack.push(Vec::new());
    }

    fn exit_class(&mut self, node: &mut Class<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        // Pop class scope from decl_stack
        self.decl_stack.pop();
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

        let s: Segment = self.new_segment(segment_name);
        self.segment_stack.push(s);

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
        let (segment, is_fn, is_text_only) =
            if let Some(id) = node.opening_element.name.get_identifier() {
                (Some(self.new_segment(id.name)), true, false)
            } else if let Some(name) = node.opening_element.name.get_identifier_name() {
                (
                    Some(self.new_segment(name)),
                    false,
                    is_text_only(name.into()),
                )
            } else {
                (None, true, false)
            };
        self.jsx_stack.push(JsxState {
            is_fn,
            is_text_only,
            is_segment: segment.is_some(),
            should_runtime_sort: false,
            static_listeners: true,
            static_subtree: true,
            key_prop: None,
            var_props: OxcVec::new_in(self.builder.allocator),
            const_props: OxcVec::new_in(self.builder.allocator),
            children: OxcVec::new_in(self.builder.allocator),
        });
        if let Some(segment) = segment {
            self.debug(format!("ENTER: JSXElementName {segment}"), ctx);
            println!("push segment: {segment}");
            self.segment_stack.push(segment);
        }
    }

    fn exit_jsx_element(&mut self, node: &mut JSXElement<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        if let Some(mut jsx) = self.jsx_stack.pop() {
            if (self.options.transpile_jsx) {
                if (!jsx.should_runtime_sort) {
                    jsx.var_props.sort_by_key(|prop| match prop {
                        ObjectPropertyKind::ObjectProperty(b) => match &(*b).key {
                            PropertyKey::StringLiteral(b) => (*b).to_string(),
                            _ => "".to_string(),
                        },
                        _ => "".to_string(),
                    });
                }
                let name = &node.opening_element.name;
                let (jsx_type, pure) = match name {
                    JSXElementName::Identifier(b) => (
                        self.builder.expression_string_literal(
                            (*b).span,
                            (*b).name,
                            Some((*b).name),
                        ),
                        true,
                    ),
                    JSXElementName::IdentifierReference(b) => (
                        self.builder.expression_identifier((*b).span, (*b).name),
                        false,
                    ),
                    JSXElementName::NamespacedName(b) => {
                        panic!("namespaced names in JSX not implemented")
                    }
                    JSXElementName::MemberExpression(b) => {
                        fn process_member_expr<'b>(
                            builder: &AstBuilder<'b>,
                            expr: &JSXMemberExpressionObject<'b>,
                        ) -> Expression<'b> {
                            match expr {
                                JSXMemberExpressionObject::ThisExpression(b) => {
                                    builder.expression_this((*b).span)
                                }
                                JSXMemberExpressionObject::IdentifierReference(b) => {
                                    builder.expression_identifier((*b).span, (*b).name)
                                }
                                JSXMemberExpressionObject::MemberExpression(b) => builder
                                    .member_expression_static(
                                        (*b).span,
                                        process_member_expr(builder, &(*b).object),
                                        builder.identifier_name(
                                            (*b).property.span(),
                                            (*b).property.name,
                                        ),
                                        false,
                                    )
                                    .into(),
                            }
                        }
                        (
                            self.builder
                                .member_expression_static(
                                    (*b).span(),
                                    process_member_expr(&self.builder, &((*b).object)),
                                    self.builder
                                        .identifier_name((*b).property.span(), (*b).property.name),
                                    false,
                                )
                                .into(),
                            false,
                        )
                    }
                    JSXElementName::ThisExpression(b) => {
                        (self.builder.expression_this((*b).span), false)
                    }
                };
                let args: OxcVec<Argument<'a>> = OxcVec::from_array_in(
                    [
                        // type
                        jsx_type.into(),
                        // varProps
                        self.builder
                            .expression_object(node.span(), jsx.var_props)
                            .into(),
                        // constProps
                        self.builder
                            .expression_object(node.span(), jsx.const_props)
                            .into(),
                        // children
                        self.builder
                            .expression_array(node.span(), jsx.children)
                            .into(),
                        // flags
                        self.builder
                            .expression_numeric_literal(
                                node.span(),
                                ((if jsx.static_subtree { 0b1 } else { 0 })
                                    | (if jsx.static_listeners { 0b01 } else { 0 }))
                                .into(),
                                None,
                                NumberBase::Binary,
                            )
                            .into(),
                        // key
                        jsx.key_prop
                            .unwrap_or_else(|| -> Expression<'a> {
                                // TODO: Figure out how to replicate root_jsx_mode from old optimizer
                                // (this conditional should be is_fn || root_jsx_mode)
                                if jsx.is_fn {
                                    if let Some(cmp) = self.component_stack.last() {
                                        let new_key = format!(
                                            "{}_{}",
                                            cmp.id.hash.chars().take(2).collect::<String>(),
                                            self.jsx_key_counter
                                        );
                                        self.jsx_key_counter += 1;
                                        return self.builder.expression_string_literal(
                                            Span::default(),
                                            self.builder.atom(&new_key),
                                            None,
                                        );
                                    }
                                }
                                self.builder.expression_null_literal(Span::default())
                            })
                            .into(),
                    ],
                    self.builder.allocator,
                );
                let callee = if (jsx.should_runtime_sort) {
                    JSX_SPLIT_NAME
                } else {
                    JSX_SORTED_NAME
                };
                self.replace_expr = Some(self.builder.expression_call_with_pure(
                    node.span,
                    self.builder.expression_identifier(name.span(), callee),
                    None::<OxcBox<TSTypeParameterInstantiation<'a>>>,
                    args,
                    false,
                    pure,
                ));
                if let Some(imports) = self.import_stack.last_mut() {
                    imports.insert(Import::new(vec![callee.into()], QWIK_CORE_SOURCE));
                }
            }
            if jsx.is_segment {
                let popped = self.segment_stack.pop();
            }
        }
        self.debug("EXIT: JSXElementName", ctx);
        self.descend();
    }

    fn enter_jsx_fragment(&mut self, node: &mut JSXFragment<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        self.jsx_stack.push(JsxState {
            is_fn: false,
            is_text_only: false,
            is_segment: false,
            should_runtime_sort: false,
            static_listeners: true,
            static_subtree: true,
            key_prop: None,
            var_props: OxcVec::new_in(self.builder.allocator),
            const_props: OxcVec::new_in(self.builder.allocator),
            children: OxcVec::new_in(self.builder.allocator),
        });
        self.debug("ENTER: JSXFragment", ctx);
    }

    fn exit_jsx_fragment(&mut self, node: &mut JSXFragment<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        if let Some(mut jsx) = self.jsx_stack.pop() {
            if (self.options.transpile_jsx) {
                self.replace_expr = Some(self.builder.expression_array(node.span(), jsx.children));
            }
        }
        self.debug("EXIT: JSXFragment", ctx);
    }

    fn exit_jsx_spread_attribute(
        &mut self,
        node: &mut JSXSpreadAttribute<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        if (!self.options.transpile_jsx) {
            return;
        }
        // Reference: qwik build/v2 internal_handle_jsx_props_obj
        // If we have spread props, all props that come before it are variable even if they're static
        if let Some(jsx) = self.jsx_stack.last_mut() {
            let range = 0..jsx.const_props.len();
            jsx.const_props
                .drain(range)
                .for_each(|p| jsx.var_props.push(p));
            jsx.should_runtime_sort = true;
            jsx.static_subtree = false;
            jsx.static_listeners = false;
            jsx.var_props
                .push(self.builder.object_property_kind_spread_property(
                    node.span(),
                    move_expression(&self.builder, &mut node.argument).into(),
                ))
        }
    }

    fn enter_jsx_attribute(&mut self, node: &mut JSXAttribute<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        if (self.options.transpile_jsx) {
            self.expr_is_const_stack.push(
                self.jsx_stack
                    .last()
                    .map_or(false, |jsx| !jsx.should_runtime_sort),
            );
        }
        self.ascend();
        self.debug("ENTER: JSXAttribute", ctx);
        // JSX Attributes should be treated as part of the segment scope.
        let segment: Segment = self.new_segment(node.name.get_identifier().name);
        self.segment_stack.push(segment);
    }

    fn exit_jsx_attribute(&mut self, node: &mut JSXAttribute<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        if (self.options.transpile_jsx) {
            if let Some(jsx) = self.jsx_stack.last_mut() {
                let expr: Expression<'a> = {
                    let v = &mut node.value;
                    match v {
                        None => self.builder.expression_boolean_literal(node.span, true),
                        Some(JSXAttributeValue::Element(_)) => {
                            println!("Replacing JSX attribute element on exit");
                            self.replace_expr.take().unwrap()
                        }
                        Some(JSXAttributeValue::Fragment(_)) => {
                            println!("Replacing JSX attribute fragment on exit");
                            self.replace_expr.take().unwrap()
                        }
                        Some(JSXAttributeValue::StringLiteral(b)) => self
                            .builder
                            .expression_string_literal((*b).span, (*b).value, Some((*b).value)),
                        Some(JSXAttributeValue::ExpressionContainer(b)) => {
                            move_expression(&self.builder, (*b).expression.to_expression_mut())
                        }
                    }
                };
                let is_const = self.expr_is_const_stack.pop().unwrap_or_default();
                if node.is_key() {
                    jsx.key_prop = Some(expr);
                } else {
                    let props = if is_const {
                        &mut jsx.const_props
                    } else {
                        &mut jsx.var_props
                    };
                    props.push(self.builder.object_property_kind_object_property(
                        node.span,
                        PropertyKind::Init,
                        self.builder.property_key_static_identifier(
                            node.name.span(),
                            node.name.get_identifier().name,
                        ),
                        expr,
                        false,
                        false,
                        false,
                    ));
                }
            }
        }
        let popped = self.segment_stack.pop();
        self.debug("EXIT: JSXAttribute", ctx);
        self.descend();
    }

    fn exit_jsx_attribute_value(
        &mut self,
        node: &mut JSXAttributeValue<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        if let JSXAttributeValue::ExpressionContainer(container) = node {
            let qrl = self.qrl_stack.pop();

            if let Some(qrl) = qrl {
                container.expression = qrl.into_jsx_expression(
                    ctx,
                    &mut self.symbol_by_name,
                    &mut self.import_by_symbol,
                )
            }
        }
    }

    fn exit_jsx_child(&mut self, node: &mut JSXChild<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        if (!self.options.transpile_jsx) {
            return;
        }
        self.debug("EXIT: JSX child", ctx);
        if let Some(jsx) = self.jsx_stack.last_mut() {
            let maybe_child = match node {
                JSXChild::Text(b) => {
                    let text: &'a str = self.builder.allocator.alloc_str(b.value.trim());
                    if (text.is_empty()) {
                        None
                    } else {
                        Some(
                            self.builder
                                .expression_string_literal((*b).span, text, Some(text.into()))
                                .into(),
                        )
                    }
                }
                JSXChild::Element(_) => {
                    println!("Replacing JSX child element on exit");
                    Some(self.replace_expr.take().unwrap().into())
                }
                JSXChild::Fragment(_) => {
                    println!("Replacing JSX child fragment on exit");
                    Some(self.replace_expr.take().unwrap().into())
                }
                JSXChild::ExpressionContainer(b) => {
                    jsx.static_subtree = false;
                    Some(move_expression(&self.builder, (*b).expression.to_expression_mut()).into())
                }
                JSXChild::Spread(b) => {
                    jsx.static_subtree = false;
                    let span = (*b).span.clone();
                    Some(self.builder.array_expression_element_spread_element(
                        span,
                        move_expression(&self.builder, &mut (*b).expression),
                    ))
                }
            };
            if let Some(child) = maybe_child {
                jsx.children.push(child);
            }
        }
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
            for specifier in specifiers.iter_mut() {
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
            self.errors.push(illegal_code_type.into());
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

fn is_text_only(node: &str) -> bool {
    matches!(
        node,
        "text" | "textarea" | "title" | "option" | "script" | "style" | "noscript"
    )
}

/// Compute which identifiers from parent scopes are captured by a QRL.
///
/// Takes all identifiers referenced in the QRL body and all declarations from parent scopes,
/// returning the intersection (sorted for deterministic output) and whether all captured
/// variables are const.
///
/// # Arguments
/// * `all_idents` - All identifiers referenced in the QRL body (from IdentCollector)
/// * `all_decl` - All declarations from parent scopes (flattened decl_stack)
///
/// # Returns
/// A tuple of:
/// * `Vec<Id>` - Sorted list of captured identifiers
/// * `bool` - True if all captured variables are const
fn compute_scoped_idents(all_idents: &[Id], all_decl: &[IdPlusType]) -> (Vec<Id>, bool) {
    let mut set: HashSet<Id> = HashSet::new();
    let mut is_const = true;

    for ident in all_idents {
        if let Some(item) = all_decl.iter().find(|item| item.0 == *ident) {
            set.insert(ident.clone());
            if !matches!(item.1, IdentType::Var(true)) {
                is_const = false;
            }
        }
    }

    let mut output: Vec<Id> = set.into_iter().collect();
    output.sort(); // Deterministic ordering for stable output
    (output, is_const)
}

pub struct TransformOptions {
    pub minify: bool,
    pub target: Target,
    pub transpile_ts: bool,
    pub transpile_jsx: bool,
}

impl TransformOptions {
    pub fn with_transpile_ts(mut self, transpile_ts: bool) -> Self {
        self.transpile_ts = transpile_ts;
        self
    }

    pub fn with_transpile_jsx(mut self, transpile_jsx: bool) -> Self {
        self.transpile_jsx = transpile_jsx;
        self
    }
}

impl Default for TransformOptions {
    fn default() -> Self {
        TransformOptions {
            minify: false,
            target: Target::Dev,
            transpile_ts: false,
            transpile_jsx: false,
        }
    }
}

pub fn transform(script_source: Source, options: TransformOptions) -> Result<OptimizationResult> {
    let allocator = Allocator::default();
    let source_text = script_source.source_code();
    let source_info = script_source.source_info();
    let source_type = script_source.source_info().try_into()?;

    let mut errors = Vec::new();

    let parse_return = Parser::new(&allocator, source_text, source_type).parse();
    errors.extend(parse_return.errors);

    let mut program = parse_return.program;

    if (options.transpile_ts) {
        let SemanticBuilderReturn {
            semantic,
            errors: semantic_errors,
        } = SemanticBuilder::new().build(&program);
        let scoping = semantic.into_scoping();
        Transformer::new(
            &allocator,
            source_info.rel_path.as_path(),
            &OxcTransformOptions {
                typescript: TypeScriptOptions::default(),
                jsx: JsxOptions::disable(),
                ..OxcTransformOptions::default()
            },
        )
        .build_with_scoping(scoping, &mut program);
    }

    let SemanticBuilderReturn {
        semantic,
        errors: semantic_errors,
    } = SemanticBuilder::new()
        .with_check_syntax_error(true) // Enable extra syntax error checking
        .with_cfg(true) // Build a Control Flow Graph
        .build(&program);

    let mut transform = TransformGenerator::new(source_info, options, None, &allocator);

    // let (symbols, scopes) = semantic.into_symbol_table_and_scope_tree();
    let scoping = semantic.into_scoping();

    traverse_mut(&mut transform, &allocator, &mut program, scoping, ());

    let TransformGenerator { app, errors, .. } = transform;
    Ok(OptimizationResult::new(app, errors))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_semantic::ScopeId;

    #[test]
    fn test_compute_scoped_idents_basic() {
        // Test basic intersection of idents with declarations
        let idents: Vec<Id> = vec![
            ("a".to_string(), ScopeId::new(0)),
            ("b".to_string(), ScopeId::new(0)),
            ("c".to_string(), ScopeId::new(0)),
        ];
        let decls: Vec<IdPlusType> = vec![
            (("a".to_string(), ScopeId::new(0)), IdentType::Var(true)),
            (("b".to_string(), ScopeId::new(0)), IdentType::Var(false)),
        ];

        let (scoped, is_const) = compute_scoped_idents(&idents, &decls);

        // Should only contain a and b (c is not declared in parent scope)
        assert_eq!(scoped.len(), 2);
        assert!(scoped.contains(&("a".to_string(), ScopeId::new(0))));
        assert!(scoped.contains(&("b".to_string(), ScopeId::new(0))));
        // is_const should be false because b is not const
        assert!(!is_const);
    }

    #[test]
    fn test_compute_scoped_idents_all_const() {
        // Test when all captured variables are const
        let idents: Vec<Id> = vec![
            ("x".to_string(), ScopeId::new(0)),
            ("y".to_string(), ScopeId::new(0)),
        ];
        let decls: Vec<IdPlusType> = vec![
            (("x".to_string(), ScopeId::new(0)), IdentType::Var(true)),
            (("y".to_string(), ScopeId::new(0)), IdentType::Var(true)),
        ];

        let (scoped, is_const) = compute_scoped_idents(&idents, &decls);

        assert_eq!(scoped.len(), 2);
        assert!(is_const);
    }

    #[test]
    fn test_compute_scoped_idents_fn_class_not_const() {
        // Function and class declarations are not considered const
        let idents: Vec<Id> = vec![
            ("myFn".to_string(), ScopeId::new(0)),
            ("MyClass".to_string(), ScopeId::new(0)),
        ];
        let decls: Vec<IdPlusType> = vec![
            (("myFn".to_string(), ScopeId::new(0)), IdentType::Fn),
            (("MyClass".to_string(), ScopeId::new(0)), IdentType::Class),
        ];

        let (scoped, is_const) = compute_scoped_idents(&idents, &decls);

        assert_eq!(scoped.len(), 2);
        assert!(!is_const); // Fn and Class are not const
    }

    #[test]
    fn test_compute_scoped_idents_sorted_output() {
        // Test that output is sorted for deterministic hashes
        let idents: Vec<Id> = vec![
            ("z".to_string(), ScopeId::new(0)),
            ("a".to_string(), ScopeId::new(0)),
            ("m".to_string(), ScopeId::new(0)),
        ];
        let decls: Vec<IdPlusType> = vec![
            (("z".to_string(), ScopeId::new(0)), IdentType::Var(true)),
            (("a".to_string(), ScopeId::new(0)), IdentType::Var(true)),
            (("m".to_string(), ScopeId::new(0)), IdentType::Var(true)),
        ];

        let (scoped, _) = compute_scoped_idents(&idents, &decls);

        // Verify output is sorted
        assert_eq!(
            scoped,
            vec![
                ("a".to_string(), ScopeId::new(0)),
                ("m".to_string(), ScopeId::new(0)),
                ("z".to_string(), ScopeId::new(0)),
            ]
        );
    }

    #[test]
    fn test_compute_scoped_idents_empty() {
        // Test with no matching declarations
        let idents: Vec<Id> = vec![("a".to_string(), ScopeId::new(0))];
        let decls: Vec<IdPlusType> = vec![];

        let (scoped, is_const) = compute_scoped_idents(&idents, &decls);

        assert!(scoped.is_empty());
        assert!(is_const); // Default is true when nothing captured
    }
}

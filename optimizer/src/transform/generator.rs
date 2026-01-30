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

use super::jsx;
use super::options::{OptimizationResult, OptimizedApp, TransformOptions};
use super::qrl as qrl_module;
use super::scope as scope_module;
use super::state::{ImportTracker, JsxState};

pub(crate) use crate::component::Target;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IdentType {
    Var(bool),
    Fn,
    Class,
}

pub type IdPlusType = (Id, IdentType);

use std::iter::Sum;
use std::ops::Deref;
use std::path::{Components, PathBuf};

use std::fs;
use std::path::Path;
use std::str;

use crate::ext::*;
use crate::illegal_code::{IllegalCode, IllegalCodeType};
use crate::processing_failure::ProcessingFailure;

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

    pub(crate) scope: Option<String>,

    pub(crate) jsx_stack: Vec<JsxState<'gen>>,

    pub(crate) jsx_key_counter: u32,

    pub(crate) expr_is_const_stack: Vec<bool>,

    pub(crate) replace_expr: Option<Expression<'gen>>,

    pub(crate) decl_stack: Vec<Vec<IdPlusType>>,

    pub(crate) jsx_element_is_native: Vec<bool>,

    pub(crate) props_identifiers: HashMap<Id, String>,

    pub(crate) in_component_props: bool,

    pub(crate) needs_wrap_prop_import: bool,

    pub(crate) hoisted_fns: Vec<(String, Expression<'gen>, String)>,

    pub(crate) hoisted_fn_counter: usize,

    pub(crate) needs_fn_signal_import: bool,

    /// Hoisted import functions for QRLs: (identifier_name, filename)
    pub(crate) hoisted_imports: Vec<(String, String)>,

    pub(crate) pending_bind_directives: Vec<(bool, Expression<'gen>)>,

    pending_on_input_handlers: Vec<Expression<'gen>>,

    pub(crate) needs_val_import: bool,

    pub(crate) needs_chk_import: bool,

    pub(crate) needs_inlined_qrl_import: bool,

    pub(crate) export_by_name: HashMap<String, ExportInfo>,

    synthesized_imports: HashMap<String, BTreeSet<ImportId>>,

    pub(crate) stack_ctxt: Vec<String>,

    entry_policy: Box<dyn EntryPolicy>,

    import_tracker: ImportTracker,

    pub(crate) loop_depth: u32,

    pub(crate) iteration_var_stack: Vec<Vec<Id>>,

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
            hoisted_imports: Vec::new(),
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

    fn add_synthesized_import(&mut self, name: ImportId, source: &str) {
        self.synthesized_imports
            .entry(source.to_string())
            .or_insert_with(BTreeSet::new)
            .insert(name);
    }

    fn finalize_imports(&mut self) -> Vec<Import> {
        self.synthesized_imports
            .drain()
            .map(|(source, names)| Import::new(names.into_iter().collect(), &source))
            .collect()
    }

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

    pub(crate) fn debug<T: AsRef<str>>(&self, _s: T, _traverse_ctx: &TraverseCtx<'_, ()>) {
    }

    pub(crate) fn new_segment<T: AsRef<str>>(&mut self, input: T) -> Segment {
        self.segment_builder.new_segment(input, &self.segment_stack)
    }

    pub(crate) fn current_display_name(&self) -> String {
        qrl_module::build_display_name(&self.segment_stack)
    }

    fn current_hash(&self) -> String {
        let display_name = self.current_display_name();
        qrl_module::compute_hash(
            &self.source_info.rel_path,
            &display_name,
            self.scope.as_deref(),
        )
    }

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

    pub(crate) fn should_wrap_prop(&self, expr: &Expression) -> Option<(String, String)> {
        if let Expression::Identifier(ident) = expr {
            for ((name, _scope_id), prop_key) in &self.props_identifiers {
                if name == &ident.name.to_string() {
                    return Some(("_rawProps".to_string(), prop_key.clone()));
                }
            }
        }
        None
    }

    pub(crate) fn should_wrap_signal_value(&self, expr: &Expression) -> bool {
        if let Expression::StaticMemberExpression(static_member) = expr {
            if static_member.property.name == "value" {
                return true;
            }
        }
        false
    }

    fn get_component_object_pattern<'b>(
        &self,
        node: &'b CallExpression<'gen>,
    ) -> Option<&'b ObjectPattern<'gen>> {
        let arg = node.arguments.first()?;
        let expr = arg.as_expression()?;
        let Expression::ArrowFunctionExpression(arrow) = expr else {
            return None;
        };
        let first_param = arrow.params.items.first()?;
        match &first_param.pattern {
            BindingPattern::ObjectPattern(obj_pat) => Some(obj_pat),
            _ => None,
        }
    }

    fn populate_props_identifiers(
        &mut self,
        obj_pat: &ObjectPattern<'gen>,
        scope_id: oxc_semantic::ScopeId,
    ) {
        use oxc_ast::ast::BindingProperty;
        for prop in &obj_pat.properties {
            let BindingProperty { key, value, .. } = prop;

            let prop_key = match key {
                PropertyKey::StaticIdentifier(id) => id.name.to_string(),
                PropertyKey::StringLiteral(s) => s.value.to_string(),
                _ => continue,
            };

            let local_name = match value {
                BindingPattern::BindingIdentifier(id) => id.name.to_string(),
                _ => continue,
            };

            self.props_identifiers.insert((local_name, scope_id), prop_key);
        }
    }

    fn transform_component_props(
        &mut self,
        node: &mut CallExpression<'gen>,
        ctx: &mut TraverseCtx<'gen, ()>,
    ) {
        let Some(arg) = node.arguments.first_mut() else {
            return;
        };
        let Some(expr) = arg.as_expression_mut() else {
            return;
        };
        let Expression::ArrowFunctionExpression(arrow) = expr else {
            return;
        };

        use crate::props_destructuring::PropsDestructuring;
        let mut props_trans = PropsDestructuring::new(ctx.ast.allocator, None);

        if !props_trans.transform_component_props(arrow, &ctx.ast) {
            return;
        }

        if props_trans.rest_id.is_some() {
            if let Some(rest_stmt) = props_trans.generate_rest_stmt(&ctx.ast) {
                self.inject_rest_stmt(arrow, rest_stmt, ctx);
                self.add_rest_props_import();
            }
        }

        self.props_identifiers = props_trans.identifiers;
    }

    fn inject_rest_stmt(
        &self,
        arrow: &mut ArrowFunctionExpression<'gen>,
        rest_stmt: Statement<'gen>,
        ctx: &mut TraverseCtx<'gen, ()>,
    ) {
        if arrow.expression {
            if let Some(Statement::ExpressionStatement(expr_stmt)) = arrow.body.statements.pop() {
                let return_stmt = ctx.ast.statement_return(SPAN, Some(expr_stmt.unbox().expression));
                let mut new_stmts = ctx.ast.vec_with_capacity(2);
                new_stmts.push(rest_stmt);
                new_stmts.push(return_stmt);
                arrow.body.statements = new_stmts;
                arrow.expression = false;
            }
        } else {
            arrow.body.statements.insert(0, rest_stmt);
        }
    }

    fn add_rest_props_import(&mut self) {
        if let Some(imports) = self.import_stack.last_mut() {
            imports.insert(Import::new(
                vec![ImportId::Named("_restProps".into())],
                QWIK_CORE_SOURCE,
            ));
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


impl<'a> Traverse<'a, ()> for TransformGenerator<'a> {
    fn enter_program(&mut self, _node: &mut Program<'a>, _ctx: &mut TraverseCtx<'a, ()>) {
    }

    fn exit_program(&mut self, node: &mut Program<'a>, ctx: &mut TraverseCtx<'a, ()>) {

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

        let synthesized = self.finalize_imports();
        for import in synthesized {
            if let Some(imports) = self.import_stack.last_mut() {
                imports.insert(import);
            }
        }

        for (name, fn_expr, str_val) in self.hoisted_fns.drain(..).rev() {
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
            node.body.insert(1, str_stmt);
        }

        // Emit hoisted import functions at module level
        // Format: const i_{name} = () => import("./file.js");
        for (name, filename) in self.hoisted_imports.drain(..).rev() {
            let import_stmt = Statement::VariableDeclaration(ctx.ast.alloc(ctx.ast.variable_declaration(
                SPAN,
                VariableDeclarationKind::Const,
                ctx.ast.vec1(ctx.ast.variable_declarator(
                    SPAN,
                    VariableDeclarationKind::Const,
                    ctx.ast.binding_pattern_binding_identifier(SPAN, ctx.ast.atom(&name)),
                    NONE,
                    Some(ctx.ast.expression_arrow_function(
                        SPAN,
                        true,
                        false,
                        NONE,
                        ctx.ast.formal_parameters(
                            SPAN,
                            FormalParameterKind::ArrowFormalParameters,
                            ctx.ast.vec(),
                            NONE,
                        ),
                        NONE,
                        ctx.ast.function_body(
                            SPAN,
                            ctx.ast.vec(),
                            ctx.ast.vec1(ctx.ast.statement_expression(
                                SPAN,
                                ctx.ast.expression_import(
                                    SPAN,
                                    ctx.ast.expression_string_literal(SPAN, ctx.ast.atom(&filename), None),
                                    None,
                                    None,
                                ),
                            )),
                        ),
                    )),
                    false,
                )),
                false,
            )));
            node.body.insert(0, import_stmt);
        }

        if let Some(tree) = self.import_stack.pop() {
            tree.iter().for_each(|import| {
                node.body.insert(0, import.into_in(ctx.ast.allocator));
            });
        }

        ImportCleanUp::clean_up(node, ctx.ast.allocator);

        // format_output overrides minify for whitespace purposes
        let should_minify = if self.options.format_output {
            false // Readable output when format_output is true
        } else {
            self.options.minify
        };

        let codegen_options = CodegenOptions {
            minify: should_minify,
            ..Default::default()
        };
        let codegen = Codegen::new().with_options(codegen_options);

        let body = codegen.build(node).code;
        // Post-process PURE annotations to match qwik-core format
        let body = body.replace("/* @__PURE__ */", "/*#__PURE__*/");

        self.app = OptimizedApp {
            body,
            components: self.components.clone(),
        };
    }

    fn enter_call_expression(
        &mut self,
        node: &mut CallExpression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        self.ascend();

        if let Some(mut is_const) = self.expr_is_const_stack.last_mut() {
            *is_const = false;
        }

        let name = node.callee_name().unwrap_or_default().to_string();

        if self.skip_transform_names.contains(&name) {
            let segment: Segment = self.new_segment(&name);
            self.segment_stack.push(segment);
            return;
        }

        if name.ends_with(MARKER_SUFFIX) {
            self.import_stack.push(BTreeSet::new());
            self.stack_ctxt.push(name.clone());
        }

        if name.starts_with("component") && name.ends_with(MARKER_SUFFIX) {
            if let Some(obj_pat) = self.get_component_object_pattern(node) {
                self.in_component_props = true;
                self.populate_props_identifiers(obj_pat, ctx.current_scope_id());
            }
        }

        if let Some(vars) = scope_module::check_map_iteration_vars(node) {
            self.loop_depth += 1;
            self.iteration_var_stack.push(vars);
        }

        let segment: Segment = self.new_segment(name);
        self.segment_stack.push(segment);
    }

    fn exit_call_expression(
        &mut self,
        node: &mut CallExpression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        if scope_module::is_map_with_function_callback(node) && self.loop_depth > 0 {
            self.iteration_var_stack.pop();
            self.loop_depth -= 1;
        }

        if self.in_component_props {
            self.transform_component_props(node, ctx);
            self.in_component_props = false;
        }

        let is_qrl = self.segment_stack.last().is_some_and(|s| s.is_qrl());
        if is_qrl {
            let comp = node.arguments.first().map(|arg0| {
                let descendent_idents = {
                    use crate::collector::IdentCollector;
                    let mut collector = IdentCollector::new();
                    if let Some(expr) = arg0.as_expression() {
                        use oxc_ast_visit::Visit;
                        collector.visit_expression(expr);
                    }
                    collector.get_words()
                };

                let all_decl: Vec<IdPlusType> = self
                    .decl_stack
                    .iter()
                    .flat_map(|v| v.iter())
                    .cloned()
                    .collect();

                let (decl_collect, _invalid_decl): (Vec<_>, Vec<_>) = all_decl
                    .into_iter()
                    .partition(|(_, t)| matches!(t, IdentType::Var(_)));

                let (scoped_idents, _is_const) =
                    qrl_module::compute_scoped_idents(&descendent_idents, &decl_collect);

                let imports: Vec<Import> = self
                    .import_stack
                    .pop()
                    .unwrap_or_default()
                    .iter()
                    .cloned()
                    .collect();

                let imported_names = qrl_module::collect_imported_names(&imports);
                let scoped_idents = qrl_module::filter_imported_from_scoped(scoped_idents, &imported_names);

                let referenced_exports = qrl_module::collect_referenced_exports(
                    &descendent_idents,
                    &imported_names,
                    &scoped_idents,
                    &self.export_by_name,
                );

                let ctx_name = node.callee_name().unwrap_or("$").to_string();

                let display_name = self.current_display_name();

                let hash = self.current_hash();

                let parent_segment = self.segment_stack.iter().rev().skip(1).find_map(|s| {
                    if s.is_qrl() {
                        Some(s.to_string())
                    } else {
                        None
                    }
                });

                let segment_data = SegmentData::new_with_exports(
                    &ctx_name,
                    display_name,
                    hash,
                    self.source_info.rel_path.clone(),
                    scoped_idents,
                    descendent_idents,
                    parent_segment,
                    referenced_exports,
                );

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
                self.import_stack.last_mut().unwrap().insert(import);
            }
        }

        self.segment_stack.pop();

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
        self.segment_stack.push(segment);

        scope_module::enter_function(self, node, ctx);
    }

    fn exit_function(&mut self, node: &mut Function<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        self.segment_stack.pop();

        scope_module::exit_function(self, node, ctx);
    }

    fn enter_class(&mut self, node: &mut Class<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        scope_module::enter_class(self, node, ctx);
    }

    fn exit_class(&mut self, node: &mut Class<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        scope_module::exit_class(self, node, ctx);
    }

    fn enter_export_named_declaration(
        &mut self,
        node: &mut ExportNamedDeclaration<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        let source = node.source.as_ref().map(|s| s.value.to_string());

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
        scope_module::enter_arrow_function_expression(self, node, ctx);
    }

    fn exit_arrow_function_expression(
        &mut self,
        node: &mut ArrowFunctionExpression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        scope_module::exit_arrow_function_expression(self, node, ctx);
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

        if !segment_name.is_empty() {
            self.stack_ctxt.push(segment_name);
        }

        let is_const = node.kind == VariableDeclarationKind::Const;

        if self.options.transpile_jsx {
            self.expr_is_const_stack.push(is_const);
        }

        scope_module::track_variable_declaration(self, node, ctx);

        if let Some(name) = id.get_identifier_name() {
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

        if node.id.get_identifier_name().is_some() {
            self.stack_ctxt.pop();
        }

        self.segment_stack.pop();
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

    fn exit_expression(&mut self, node: &mut Expression<'a>, _ctx: &mut TraverseCtx<'a, ()>) {
        if let Some(expr) = self.replace_expr.take() {
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
                match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                        let imported = spec.imported.name().to_string();
                        let local = spec.local.name.to_string();
                        self.import_tracker
                            .add_import(&source_str, &imported, &local);

                        if imported.ends_with(MARKER_SUFFIX) && imported != local {
                            self.skip_transform_names.insert(local.clone());
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

                    if let Some(local_name) = local_name {
                        let scope_id = ctx.current_scope_id();

                        let existing_binding = ctx.scoping().get_binding(scope_id, &local_name);
                        let can_rename = existing_binding.is_none()
                            || existing_binding == Some(symbol_id);

                        if can_rename {
                            ctx.scoping_mut().rename_symbol(
                                symbol_id,
                                scope_id,
                                local_name.as_str(),
                            );
                        }

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
            .map(|ref_id| ctx.scoping().get_reference(ref_id))
            .and_then(|refr| refr.symbol_id())
            .and_then(|symbol_id| self.removed.get(&symbol_id))
        {
            let file_name = self.source_info.file_name.to_string();
            self.errors.push(ProcessingFailure::illegal_code(illegal_code_type, &file_name));
        }

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

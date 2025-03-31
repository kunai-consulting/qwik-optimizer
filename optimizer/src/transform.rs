#![allow(unused)]

use crate::dead_code::DeadCode;
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
use oxc_ast::visit::walk_mut::*;
use oxc_ast::{
    match_member_expression, AstBuilder, AstType, Comment, CommentKind, Visit, VisitMut,
};
use oxc_codegen::{Codegen, CodegenOptions, Context, Gen};
use oxc_index::Idx;
use std::borrow::{Borrow, Cow};
use std::collections::{BTreeSet, HashMap, HashSet};

use crate::component::*;
use crate::import_clean_up::ImportCleanUp;
use crate::macros::*;
use crate::source::Source;
use oxc_parser::Parser;
use oxc_semantic::{
    NodeId, ReferenceId, ScopeFlags, ScopeId, SemanticBuilder, SemanticBuilderReturn, SymbolFlags,
    SymbolId,
};
use oxc_span::*;
use oxc_traverse::{traverse_mut, Ancestor, Traverse, TraverseCtx};
use serde::{Deserialize, Serialize};
use std::cell::{Cell, RefCell};
use std::fmt::{write, Display};
use std::ops::Deref;
use std::path::{Components, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Serialize)]
pub struct OptimizedApp {
    pub body: String,
    pub components: Vec<QrlComponent>,
}

use crate::ext::*;

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

pub struct TransformGenerator<'gen> {
    pub components: Vec<QrlComponent>,

    pub app: OptimizedApp,

    pub errors: Vec<Error>,

    depth: usize,

    segment_stack: Vec<Segment>,

    segment_builder: SegmentBuilder,

    symbol_by_name: HashMap<String, SymbolId>,

    component_stack: Vec<QrlComponent>,

    qrl_stack: Vec<Qrl>,

    import_stack: Vec<BTreeSet<Import>>,

    import_by_symbol: HashMap<SymbolId, Import>,

    source_info: &'gen SourceInfo,

    target: Target,

    scope: Option<String>,

    minify: bool,
}

impl<'gen> TransformGenerator<'gen> {
    fn new(
        source_info: &'gen SourceInfo,
        minify: bool,
        target: Target,
        scope: Option<String>,
    ) -> Self {
        Self {
            components: Vec::new(),
            app: OptimizedApp::default(),
            errors: Vec::new(),
            depth: 0,
            segment_stack: Vec::new(),
            segment_builder: SegmentBuilder::new(),
            symbol_by_name: Default::default(),
            component_stack: Vec::new(),
            qrl_stack: Vec::new(),
            import_stack: vec![BTreeSet::new()],
            import_by_symbol: Default::default(),
            source_info,
            target,
            scope,
            minify,
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

    fn debug<T: AsRef<str>>(&self, s: T, traverse_ctx: &TraverseCtx) {
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

const DEBUG: bool = true;
const DUMP_FINAL_AST: bool = false;

impl<'a> Traverse<'a> for TransformGenerator<'a> {
    fn exit_program(&mut self, node: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(tree) = self.import_stack.pop() {
            tree.iter().for_each(|import| {
                node.body.insert(0, import.into_in(ctx.ast.allocator));
            });
        }

        ImportCleanUp::clean_up(node, ctx.ast.allocator);

        let codegen_options = CodegenOptions {
            annotation_comments: true,
            minify: self.minify,
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

    fn enter_call_expression(&mut self, node: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.ascend();
        self.debug(format!("ENTER: CallExpression, {:?}", node), ctx);

        let name = node.callee_name().unwrap_or_default().to_string();
        if (name.ends_with(MARKER_SUFFIX)) {
            self.import_stack.push(BTreeSet::new());
        }

        let segment: Segment = self.new_segment(name);
        println!("push segment: {segment}");
        self.segment_stack.push(segment);
    }

    fn exit_call_expression(&mut self, node: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        let segment = self.segment_stack.last();

        if let Some(segment) = segment {
            // let callee_name = node.callee_name().unwrap_or_default();
            if segment.is_qrl() {
                let comp = node.arguments.first().map(|arg0| {
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
                        &self.target,
                        &self.scope,
                        self.source_info,
                        self.minify,
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
                        .map(|s| s.index())
                        .unwrap_or_default();
                    self.import_stack.last_mut().unwrap().insert(import);
                }
            }
        }
        &self.segment_stack.pop();
    }

    fn enter_function(&mut self, node: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        let segment: Segment = node
            .name()
            .map(|n| self.new_segment(n))
            .unwrap_or(self.new_segment("$"));
        println!("push segment: {segment}");
        self.segment_stack.push(segment);
    }

    fn exit_function(&mut self, node: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        let popped = self.segment_stack.pop();
        println!("pop segment: {popped:?}");
    }

    fn exit_argument(&mut self, node: &mut Argument<'a>, ctx: &mut TraverseCtx<'a>) {
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

    // fn enter_export_named_declaration(&mut self, node: &mut ExportNamedDeclaration<'a>, ctx: &mut TraverseCtx<'a>) {
    //
    //
    //     todo!()
    // }

    fn enter_variable_declarator(
        &mut self,
        node: &mut VariableDeclarator<'a>,
        ctx: &mut TraverseCtx<'a>,
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
        ctx: &mut TraverseCtx<'a>,
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

        let popped = self.segment_stack.pop();
        println!("pop segment: {popped:?}");
    }

    fn enter_expression_statement(
        &mut self,
        node: &mut ExpressionStatement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.ascend();
        self.debug("ENTER: ExpressionStatement", ctx);
    }

    fn exit_expression_statement(
        &mut self,
        node: &mut ExpressionStatement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.debug("EXIT: ExpressionStatement", ctx);
        self.descend();
    }

    fn enter_jsx_element(&mut self, node: &mut JSXElement<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(name) = node.opening_element.name.get_identifier_name() {
            let segment: Segment = self.new_segment(name);
            self.debug(format!("ENTER: JSXElementName {segment}"), ctx);
            println!("push segment: {segment}");
            self.segment_stack.push(segment);
            dbg!(node);
        }
    }

    fn exit_jsx_element(&mut self, node: &mut JSXElement<'a>, ctx: &mut TraverseCtx<'a>) {
        // JSX Elements should be treated as part of the segment scope.
        if let Some(name) = node.opening_element.name.get_identifier_name() {
            let popped = self.segment_stack.pop();
        }
        self.debug("EXIT: JSXElementName", ctx);
        self.descend();
    }

    fn enter_jsx_attribute(&mut self, node: &mut JSXAttribute<'a>, ctx: &mut TraverseCtx<'a>) {
        self.ascend();
        self.debug("ENTER: JSXAttribute", ctx);
        // JSX Attributes should be treated as part of the segment scope.
        let segment: Segment = self.new_segment(node.name.get_identifier().name);
        self.segment_stack.push(segment);
    }

    fn exit_jsx_attribute(&mut self, node: &mut JSXAttribute<'a>, ctx: &mut TraverseCtx<'a>) {
        let popped = self.segment_stack.pop();
        println!("pop segment: {popped:?}");
        self.debug("EXIT: JSXAttribute", ctx);
        self.descend();
    }

    fn exit_jsx_attribute_value(
        &mut self,
        node: &mut JSXAttributeValue<'a>,
        ctx: &mut TraverseCtx<'a>,
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

    fn exit_return_statement(&mut self, node: &mut ReturnStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(expr) = &node.argument {
            if expr.is_qrl_replaceable() {
                let qrl = self.qrl_stack.pop();
                if let Some(qrl) = qrl {
                    let expression = qrl.into_expression(
                        ctx,
                        &mut self.symbol_by_name,
                        &mut self.import_by_symbol,
                    );
                    dbg!(&expression);
                    node.argument = Some(expression);
                }
            }
        }
    }

    fn enter_statements(
        &mut self,
        node: &mut OxcVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.debug("ENTER: Statements", ctx);
        node.retain(|s| !s.is_dead_code());
    }

    fn exit_statements(&mut self, node: &mut OxcVec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
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
        ctx: &mut TraverseCtx<'a>,
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
                        ctx.symbols_mut().set_name(symbol_id, local_name.as_str());

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
                                    ctx.ast.identifier_name(SPAN, name),
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
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Whilst visiting each identifier reference, we check if that references refers to an import.
        // If so, we store on the current import stack so that it can be used later in the `exit_expression`
        // logic that ends up creating a new module/component.
        let ref_id = id_ref.reference_id();
        if let Some(symbol_id) = ctx.symbols().get_reference(ref_id).symbol_id() {
            if let Some(import) = self.import_by_symbol.get(&symbol_id) {
                let import = import.clone();
                if !id_ref.name.ends_with(MARKER_SUFFIX) {
                    self.import_stack.last_mut().unwrap().insert(import);
                }
            }
        }
    }
}

#[derive(Debug, Deserialize, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MinifyMode {
    Simplify,
    None,
}

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EntryStrategy {
    Inline,
    Hoist,
    Single,
    Hook,
    Segment,
    Component,
    Smart,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformFsOptions {
    pub src_dir: String,
    pub root_dir: Option<String>,
    pub vendor_roots: Vec<String>,
    pub glob: Option<String>,
    pub minify: MinifyMode,
    pub entry_strategy: EntryStrategy,
    pub source_maps: bool,
    pub transpile_ts: bool,
    pub transpile_jsx: bool,
    pub preserve_filenames: bool,
    pub explicit_extensions: bool,
    pub mode: Target,
    pub scope: Option<String>,

    pub core_module: Option<String>,
    pub strip_exports: Option<Vec<String>>,
    pub strip_ctx_name: Option<Vec<String>>,
    pub strip_event_handlers: bool,
    pub reg_ctx_name: Option<Vec<String>>,
    pub is_server: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformModuleInput {
    pub path: String,
    pub dev_path: Option<String>,
    pub code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformModulesOptions {
    pub src_dir: String,
    pub root_dir: Option<String>,
    pub input: Vec<TransformModuleInput>,
    pub source_maps: bool,
    pub minify: MinifyMode,
    pub transpile_ts: bool,
    pub transpile_jsx: bool,
    pub preserve_filenames: bool,
    pub entry_strategy: EntryStrategy,
    pub explicit_extensions: bool,
    pub mode: Target,
    pub scope: Option<String>,

    pub core_module: Option<String>,
    pub strip_exports: Option<Vec<String>>,
    pub strip_ctx_name: Option<Vec<String>>,
    pub strip_event_handlers: bool,
    pub reg_ctx_name: Option<Vec<String>>,
    pub is_server: Option<bool>,
}

pub fn transform_fs(config: TransformFsOptions) -> Result<(OptimizedApp)> {
    Err(Error::Generic("Not yet implemented".to_string()))
}

pub fn transform_modules(config: TransformModulesOptions) -> Result<(OptimizedApp)> {
    Err(Error::Generic("Not yet implemented".to_string()))
}

pub fn transform(script_source: Source) -> Result<(OptimizedApp)> {
    let allocator = Allocator::default();
    let source_text = script_source.source_code();
    let source_info = script_source.source_info();
    let source_type = script_source.source_info().try_into()?;

    let mut errors = Vec::new();

    let parse_return = Parser::new(&allocator, source_text, source_type).parse();
    errors.extend(parse_return.errors);

    let mut program = parse_return.program;

    let SemanticBuilderReturn {
        semantic,
        errors: semantic_errors,
    } = SemanticBuilder::new()
        .with_check_syntax_error(true) // Enable extra syntax error checking
        .with_build_jsdoc(true) // Enable JSDoc parsing
        .with_cfg(true) // Build a Control Flow Graph
        .build(&program);

    let mut transform = &mut TransformGenerator::new(source_info, false, Target::Dev, None);

    let (symbols, scopes) = semantic.into_symbol_table_and_scope_tree();

    traverse_mut(transform, &allocator, &mut program, symbols, scopes);

    Ok(transform.app.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;
    use std::path::PathBuf;

    #[test]
    fn test_example_1() {
        assert_valid_transform_debug!();
    }

    #[test]
    fn test_example_2() {
        assert_valid_transform!();
    }

    #[test]
    fn test_example_3() {
        assert_valid_transform!();
    }

    #[test]
    fn test_example_4() {
        assert_valid_transform!();
    }

    #[test]
    fn test_example_5() {
        assert_valid_transform!();
    }

    #[test]
    fn test_example_6() {
        assert_valid_transform!();
    }

    #[test]
    fn test_example_7() {
        assert_valid_transform_debug!();
    }

    #[test]
    fn test_example_8() {
        assert_valid_transform_debug!();
    }

    // #[test]
    fn test_example_9() {
        // Not removing:
        // const decl8 = 1, decl9;
        assert_valid_transform_debug!();
    }

    // #[test]
    fn test_example_10() {
        // Not converting:
        // const a = ident1 + ident3;
        // const b = ident1 + ident3;
        // to:
        // ident1, ident3;
        // ident1, ident3;
        assert_valid_transform_debug!();
    }

    #[test]
    fn test_example_11() {
        // Unused imports present in main app output.
        assert_valid_transform_debug!();
    }

    #[test]
    fn test_example_capture_imports() {
        assert_valid_transform_debug!();
    }
}

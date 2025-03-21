#![allow(unused)]

use crate::dead_code::DeadCode;
use crate::error::Error;
use crate::ext::*;
use crate::prelude::*;
use crate::ref_counter::RefCounter;
use crate::segment::Segment;
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
use std::collections::{HashMap, HashSet};

use crate::component::*;
use crate::import_clean_up::ImportCleanUp;
use crate::macros::*;
use crate::source::Source;
use oxc_parser::Parser;
use oxc_semantic::{
    ReferenceId, ScopeFlags, ScopeId, SemanticBuilder, SemanticBuilderReturn, SymbolId,
};
use oxc_span::*;
use oxc_traverse::{traverse_mut, Ancestor, Traverse, TraverseCtx};
use std::cell::{Cell, RefCell};
use std::fmt::{write, Display};
use std::ops::Deref;
use std::path::{Components, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct OptimizedApp {
    pub body: String,
    pub components: Vec<QrlComponent>,
}
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

struct TransformGenerator<'gen> {
    pub components: Vec<QrlComponent>,

    pub app: OptimizedApp,

    pub errors: Vec<Error>,

    depth: usize,

    segment_stack: Vec<Segment>,

    component_stack: Vec<QrlComponent>,

    jsx_qurl_stack: Vec<Qrl>,

    var_decl_stack: Vec<Qrl>,

    call_args_stack: Vec<Qrl>,

    import_stack: Vec<Vec<CommonImport>>,

    import_by_ref: HashMap<SymbolId, Import>,

    anonymous_return_depth: usize,

    return_arg_stack: Vec<Qrl>,

    scoped_references: HashMap<u32, Vec<Reference>>,

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
            component_stack: Vec::new(),
            jsx_qurl_stack: Vec::new(),
            var_decl_stack: Vec::new(),
            call_args_stack: Vec::new(),
            import_stack: vec![vec![]],
            import_by_ref: Default::default(),
            return_arg_stack: Vec::new(),
            scoped_references: HashMap::new(),
            anonymous_return_depth: 0,
            source_info,
            target,
            scope,
            minify,
        }
    }

    fn is_recording(&self) -> bool {
        self.segment_stack
            .last()
            .map(|s| s.is_qrl_extractable())
            .unwrap_or(false)
    }

    fn qrl_type(&self) -> QrlType {
        self.segment_stack
            .last()
            .map(|s| s.associated_qrl_type())
            .unwrap_or(QrlType::Qrl)
    }

    fn requires_handle_watch(&self) -> bool {
        self.segment_stack
            .last()
            .map(|s| s.requires_handle_watch())
            .unwrap_or(false)
    }

    fn render_segments(&self) -> String {
        let ss: Vec<String> = self
            .segment_stack
            .iter()
            .filter(|s| **s != Segment::AnonymousCaptured)
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

    // fn import_clean_up(pgm: &mut Program<'_>, ctx: &mut TraverseCtx) {
    //     let mut remove: Vec<usize> = Vec::new();
    //
    //     for (idx, statement) in pgm.body.iter_mut().enumerate() {
    //         if let Statement::ImportDeclaration(import) = statement {
    //             let source_value = import.source.value;
    //             let specifiers = &mut import.specifiers;
    //             if let Some(specifiers) = specifiers {
    //                 // specifiers.retain(|s| !QRL_MARKER_IMPORTS.contains(&s.name().deref()));
    //                 specifiers.retain(|s| {
    //                     let local = s.local();
    //                     ctx.symbols().symbol_is_used(local.symbol_id())
    //                         && !local.name.ends_with("$")
    //                 });
    //
    //                 // If all specifiers are removed, we will want to eventually remove that statement completely.
    //                 if specifiers.is_empty() {
    //                     remove.push(idx);
    //                 }
    //             }
    //         }
    //     }
    //
    //     for idx in remove.iter() {
    //         pgm.body.remove(*idx);
    //     }
    // }

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

    fn add_scoped_reference(&mut self, reference: Reference, scope_id: ScopeId) {
        let scope_id: u32 = scope_id.index() as u32;
        let mut scoped_references = &mut self.scoped_references;
        let refs_in_scope = scoped_references.get_mut(&scope_id);
        match refs_in_scope {
            None => {
                scoped_references.insert(scope_id, vec![reference]);
            }
            Some(refs) => {
                if !refs.contains(&reference) {
                    refs.push(reference);
                }
            }
        }
    }

    fn get_scoped_references_rec(&self, scope_id: u32, ref_name: String) -> Option<Reference> {
        let refs = self.scoped_references.get(&scope_id);
        match refs {
            Some(refs) if !refs.is_empty() => {
                let reference = refs.iter().find(|r| r.name().deref() == ref_name);
                match reference {
                    Some(r) => Some(r.clone()),
                    None if scope_id > 0 => self.get_scoped_references_rec(scope_id - 1, ref_name),
                    None => None,
                }
            }
            Some(_) => None,
            None if scope_id > 0 => self.get_scoped_references_rec(scope_id - 1, ref_name),
            None => None,
        }
    }

    fn add_scoped_reference_import(&mut self, ref_name: String, scope_id: ScopeId) {
        let reference = self
            .get_scoped_references_rec(scope_id.index() as u32, ref_name)
            .clone();
        if let Some(r) = reference {
            let imp = CommonImport::Import(r.into_import(PathBuf::from("./test")));
            self.push_import(imp)
        }
    }

    fn push_import(&mut self, import: CommonImport) {
        if let Some(mut imports) = self.import_stack.last_mut() {
            imports.push(import);
        }
    }
}

const DEBUG: bool = true;
const DUMP_FINAL_AST: bool = false;

impl<'a> Traverse<'a> for TransformGenerator<'a> {
    fn exit_program(&mut self, node: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        let flattened_imports = self
            .import_stack
            .iter()
            .flatten()
            .collect::<Vec<&CommonImport>>();

        let imports: HashSet<&CommonImport> = HashSet::from_iter(flattened_imports).clone();
        let imports: Vec<&&CommonImport> = imports.iter().clone().collect();

        for import in imports {
            let import = Statement::from_in(*import, ctx.ast.allocator);
            node.body.insert(0, import);
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

        let segment: Segment = name.into();
        let is_extractable = segment.is_qrl_extractable();
        self.segment_stack.push(segment);
    }

    fn exit_call_expression(&mut self, node: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        let segment = self.segment_stack.pop();

        if let Some(segment) = segment {
            if segment.is_qrl_extractable() {
                if let Some(comp) = self.component_stack.pop() {
                    let qrl = &comp.qurl;
                    let qrl = qrl.clone();
                    if DEBUG {
                        println!(
                            "CALLEE BEFORE: {:#?} PARENT {:?} GRANDPARENT {:?}",
                            node.callee,
                            ctx.parent(),
                            ctx.ancestor(1)
                        );
                    }

                    match ctx.parent() {
                        Ancestor::JSXExpressionContainerExpression(_) => {
                            self.jsx_qurl_stack.push(qrl)
                        }
                        Ancestor::VariableDeclaratorInit(_) => self.var_decl_stack.push(qrl),
                        Ancestor::CallExpressionArguments(_) => self.call_args_stack.push(qrl),
                        Ancestor::ReturnStatementArgument(r) => self.return_arg_stack.push(qrl),
                        ancestor => panic!(
                            "You need to properly implement a stack and logic for {:?}",
                            ancestor
                        ),
                    }

                    if DEBUG {
                        println!("CALLEE AFTER: {:#?}", node.callee);
                    }
                    self.components.push(comp);
                }
            }

            // Build up a list imports this extracted will need best one what it is currently referencing.
            for symbol_id in &node.callee.get_referenced_symbols(ctx) {
                if let Some(import) = self.import_by_ref.get(&symbol_id) {
                    if let Some(import_stack) = self.import_stack.last_mut() {
                        let import = CommonImport::Import(import.clone());
                        import_stack.push(import);
                    }
                }
            }

            self.debug(
                format!("EXIT: CallExpression. SCOPE[{:?}]", ctx.current_scope_id()),
                ctx,
            );
            self.descend();
        }
    }

    fn enter_function(&mut self, node: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        let segment: Segment = node
            .name()
            .map(|n| n.into())
            .unwrap_or(Segment::NamedCaptured("".to_string()));
        self.segment_stack.push(segment);
    }

    fn exit_function(&mut self, node: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        self.segment_stack.pop();
    }

    fn exit_argument(&mut self, node: &mut Argument<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Argument::CallExpression(call_expr) = node {
            let qrl = self.call_args_stack.pop();

            if let Some(qrl) = qrl {
                let idr = IdentifierReference::from_in(qrl.clone(), ctx.ast.allocator);
                let args: OxcVec<'a, Argument<'a>> = qrl.into_in(ctx.ast.allocator);

                call_expr.callee = Expression::Identifier(OxcBox::new_in(idr, ctx.ast.allocator));
                call_expr.arguments = args
            }
        }
    }

    fn enter_variable_declarator(
        &mut self,
        node: &mut VariableDeclarator<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.ascend();
        self.debug(
            format!(
                "ENTER: VariableDeclarator.  ID {:?}. [P: {:?}, GP: {:?}]]",
                node.id,
                ctx.parent(),
                ctx.ancestor(1)
            ),
            ctx,
        );
        let id = &node.id;
        let s: Segment = id.into_in(ctx.ast.allocator);
        self.segment_stack.push(s);

        if let Some(name) = id.get_identifier_name() {
            let reference = Reference::Variable(name.into());
            self.add_scoped_reference(reference, ctx.current_scope_id());
        }
    }

    fn exit_variable_declarator(
        &mut self,
        node: &mut VariableDeclarator<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Some(init) = &mut node.init {
            let qrl = self.var_decl_stack.pop();
            if let Some(qrl) = qrl {
                node.init = Some(qrl.into_in(ctx.ast.allocator))
            }
        }

        self.segment_stack.pop();

        self.debug("EXIT: VariableDeclarator", ctx);
        self.descend();
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

    fn enter_arrow_function_expression(
        &mut self,
        node: &mut ArrowFunctionExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.ascend();
        self.debug("ENTER: ArrowFunctionExpression", ctx);
        if self.is_recording() {
            self.import_stack.push(vec![]);
        }
    }

    fn exit_arrow_function_expression(
        &mut self,
        node: &mut ArrowFunctionExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.is_recording() {
            let name = self.render_segments();

            let segments: Vec<String> = self
                .segment_stack
                .iter()
                .filter(|s| **s != Segment::AnonymousCaptured)
                .map(|s| {
                    let string: String = s.into();
                    string
                })
                .collect();

            let qrl_type = self.qrl_type().clone();

            self.debug(
                format!("Calculating component imports, {:?}", self.import_stack),
                ctx,
            );

            let imports: Vec<CommonImport> = self.import_stack.pop().unwrap_or_default();

            let comp = QrlComponent::new(
                &self.source_info,
                &segments,
                node,
                imports,
                self.requires_handle_watch(),
                self.minify,
                qrl_type,
                &self.target,
                &self.scope,
            );
            match comp {
                Ok(comp) => {
                    let target_scope = ctx.current_scope_id().index() as u32;
                    self.push_import(self.qrl_type().into());
                    self.component_stack.push(comp);
                }
                Err(e) => {
                    self.errors.push(e);
                }
            }
        }
        self.debug("EXIT: ArrowFunctionExpression", ctx);
        self.descend();
    }

    fn enter_jsx_element(&mut self, node: &mut JSXElement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.ascend();
        if let Some(name) = node.opening_element.name.get_identifier_name() {
            let segment: Segment = name.into();
            self.debug(format!("ENTER: JSXElementName {segment}"), ctx);
            self.segment_stack.push(segment);
            self.add_scoped_reference_import(name.to_string(), ctx.current_scope_id());
        } else {
            self.debug("ENTER: JSXElementName", ctx);
        }
    }

    fn exit_jsx_element(&mut self, node: &mut JSXElement<'a>, ctx: &mut TraverseCtx<'a>) {
        // JSX Elements should be treated as part of the segment scope.
        if let Some(name) = node.opening_element.name.get_identifier_name() {
            self.segment_stack.pop();
        }
        self.debug("EXIT: JSXElementName", ctx);
        self.descend();
    }

    fn enter_jsx_attribute(&mut self, node: &mut JSXAttribute<'a>, ctx: &mut TraverseCtx<'a>) {
        self.ascend();
        self.debug("ENTER: JSXAttribute", ctx);
        // JSX Attributes should be treated as part of the segment scope.
        let segment: Segment = node.name.get_identifier().name.into();
        self.segment_stack.push(segment);
    }

    fn exit_jsx_attribute(&mut self, node: &mut JSXAttribute<'a>, ctx: &mut TraverseCtx<'a>) {
        self.segment_stack.pop();
        self.debug("EXIT: JSXAttribute", ctx);
        self.descend();
    }

    fn exit_jsx_attribute_value(
        &mut self,
        node: &mut JSXAttributeValue<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let JSXAttributeValue::ExpressionContainer(container) = node {
            let qrl = self.jsx_qurl_stack.pop();
            if let Some(qrl) = qrl {
                container.expression = qrl.into_in(ctx.ast.allocator)
            }
        }
    }

    fn enter_return_statement(
        &mut self,
        node: &mut ReturnStatement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.ascend();
        self.debug("ENTER: ReturnStatement", ctx);

        if let Some(expr) = &node.argument {
            if expr.is_qrl_replaceable() {
                self.anonymous_return_depth += 1;
                self.segment_stack.push(Segment::NamedCaptured(format!(
                    "{}",
                    self.anonymous_return_depth
                )))
            }
        }
    }

    fn exit_return_statement(&mut self, node: &mut ReturnStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.debug(
            format!("EXIT: ReturnStatement ARG {:?}", node.argument),
            ctx,
        );
        self.descend();

        if let Some(expr) = &node.argument {
            if expr.is_qrl_replaceable() {
                self.anonymous_return_depth -= 1;
                self.segment_stack.pop();
                let qrl = self.return_arg_stack.pop();
                if let Some(qrl) = qrl {
                    node.argument = Some(qrl.into_in(ctx.ast.allocator));
                }
            }
        }
    }

    fn enter_statements(
        &mut self,
        node: &mut OxcVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        for statement in node.iter_mut() {
            if let Statement::VariableDeclaration(decl) = statement {
                if let Some(decl) = decl.declarations.first() {
                    let name = decl.id.get_identifier_name();
                    let refs = decl.reference_count(ctx);
                    self.debug(
                        format!("REF COUNT `{:?}` has  {} reference(s)", name, refs),
                        ctx,
                    );
                }
            }
        }

        self.debug("ENTER: Statements", ctx);
        node.retain(|s| !s.is_dead_code());
    }

    fn exit_statements(&mut self, node: &mut OxcVec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        self.debug(format!("Scoped QRL Imports: {:?}", self.import_stack), ctx);

        for statement in node.iter_mut() {
            // This will determine whether the variable declaration can be replaced with just the call that is being used to initialize it.
            // e.g. `const x = componentQrl(...)` can be replaced with just `componentQrl(...)`.
            if let Statement::VariableDeclaration(decl) = statement {
                if let Some(decl) = decl.declarations.first() {
                    let ref_count = decl.reference_count(ctx);
                    if (ref_count < 1) {
                        if let Some(Expression::CallExpression(expr)) = &decl.init {
                            let name = expr.callee_name().unwrap_or_default();
                            if name == COMPONENT_QRL {
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
                    let source = PathBuf::from(source.as_ref());
                    self.import_by_ref
                        .insert(symbol_id, Import::new(vec![specifier.into()], source));
                }

                // Rename qwik imports per https://github.com/QwikDev/qwik/blob/build/v2/packages/qwik/src/optimizer/core/src/rename_imports.rs
                let source = node.source.value;
                let source = ImportCleanUp::rename_qwik_imports(source);
                node.source.value = source.into_in(ctx.ast.allocator);
            }
        }
    }
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
        assert_valid_transform!();
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
        assert_valid_transform!();
    }

    #[test]
    fn test_example_8() {
        assert_valid_transform!();
    }

    #[test]
    fn test_example_9() {
        // Not removing:
        // const decl8 = 1, decl9;
        assert_valid_transform_debug!();
    }

    #[test]
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
        assert_valid_transform_debug!();
    }
}

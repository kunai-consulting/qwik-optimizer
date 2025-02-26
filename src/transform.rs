#![allow(unused)]

use crate::error::Error;
use crate::prelude::*;
use crate::segment::Segment;
use crate::sources::*;
use oxc_allocator::{
    Allocator, Box as OxcBox, CloneIn, FromIn, GetAddress, HashMap as OxcHashMap, IntoIn,
    Vec as OxcVec,
};
use oxc_ast::ast::{
    Argument, ArrowFunctionExpression, BindingIdentifier, BindingPattern, BindingPatternKind,
    CallExpression, Expression, ExpressionStatement, Function, IdentifierName, IdentifierReference,
    JSXAttribute, JSXAttributeName, JSXAttributeValue, JSXClosingElement, JSXElement,
    JSXElementName, JSXExpression, JSXOpeningElement, Program, Statement, TSTypeAnnotation,
    VariableDeclaration, VariableDeclarator,
};
use oxc_ast::visit::walk_mut::*;
use oxc_ast::{match_member_expression, AstBuilder, AstType, Visit, VisitMut};
use oxc_codegen::{Codegen, Context, Gen};
use oxc_index::Idx;
use std::borrow::{Borrow, Cow};

use crate::component::*;
use oxc_parser::Parser;
use oxc_semantic::{ScopeFlags, ScopeId, SemanticBuilder, SemanticBuilderReturn};
use oxc_span::*;
use oxc_traverse::{traverse_mut, Ancestor, Traverse, TraverseCtx};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt::{write, Display};
use std::ops::Deref;
use std::path::Components;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct QwikApp {
    pub body: String,
    pub components: Vec<QwikComponent>,
}

impl Display for QwikApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let component_count = self.components.len();
        let comp_heading = format!(
            "------------------- COMPONENTS[{}] ------------------\n",
            component_count
        );
        let sep = format!("{}\n", "-".repeat(comp_heading.len()));
        let all_comps = self.components.iter().fold(String::new(), |acc, comp| {
            let mut code_gen0 = Codegen::default();
            let code_gen = &mut code_gen0;

            let body = &comp.code;
            format!("{}\n{}\n{}", acc, body, sep)
        });

        let body_heading = "------------------------ BODY -----------------------\n".to_string();

        write!(
            f,
            "{}{}{}{}",
            comp_heading, all_comps, body_heading, self.body
        )
    }
}

struct TransformGenerator {
    pub components: Vec<QwikComponent>,

    pub app: QwikApp,

    pub errors: Vec<Error>,

    depth: usize,

    segment_stack: Vec<Segment>,

    component_stack: Vec<QwikComponent>,

    jsx_qurl_stack: Vec<Qrl>,

    var_decl_stack: Vec<Qrl>,

    call_args_stack: Vec<Qrl>,

    qrl_import_stack: usize,

    mode: Mode,

    source_info: SourceInfo,

    target: Target,

    scope: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Mode {
    Scanning,
    Recording(usize),
}

impl TransformGenerator {
    fn new(source_info: SourceInfo, target: Target, scope: Option<String>) -> Self {
        Self {
            components: Vec::new(),
            app: QwikApp::default(),
            errors: Vec::new(),
            depth: 0,
            segment_stack: Vec::new(),
            component_stack: Vec::new(),
            jsx_qurl_stack: Vec::new(),
            var_decl_stack: Vec::new(),
            call_args_stack: Vec::new(),
            qrl_import_stack: 0,
            mode: Mode::Scanning,
            source_info,
            target,
            scope,
        }
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

    fn start_recording(&mut self) {
        self.mode = match self.mode {
            Mode::Scanning => Mode::Recording(1),
            Mode::Recording(count) => Mode::Recording(count + 1),
        }
    }

    fn stop_recording(&mut self) {
        self.mode = match self.mode {
            Mode::Scanning => Mode::Scanning,
            Mode::Recording(count) if count > 1 => Mode::Recording(count - 1),
            Mode::Recording(_) => Mode::Scanning,
        }
    }

    fn descend(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        }
    }

    fn ascend(&mut self) {
        self.depth += 1;
    }

    fn debug<T: AsRef<str>>(&self, s: T) {
        if DEBUG {
            let indent = "--".repeat(self.depth);
            let prefix = format!("|{}", indent);
            println!(
                "{}[{}|MODE: {:?}]{}. Segments: {}",
                prefix,
                self.depth,
                self.mode,
                s.as_ref(),
                self.render_segments()
            );
        }
    }
}

const DEBUG: bool = true;
const DUMP_FINAL_AST: bool = false;

impl<'a> Traverse<'a> for TransformGenerator {
    fn enter_program(&mut self, node: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        let statements = &mut node.body;
        statements.insert(0, CommonImport::qrl().into_in(ctx.ast.allocator));
    }
    fn exit_program(&mut self, node: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        let codegen = Codegen::new();
        let body = codegen.build(node).code;

        self.app = QwikApp {
            body,
            components: self.components.clone(),
        };

        if DUMP_FINAL_AST {
            println!("-------------------FINAL AST DUMP--------------------\n{:#?}", node);
        }
    }

    fn enter_call_expression(&mut self, node: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.ascend();
        self.debug(format!("ENTER: CallExpression, {:?}", node));

        let name = node.callee_name().unwrap_or_default().to_string();

        let segment: Segment = name.into();
        let is_qwik = segment.is_qwik();
        self.segment_stack.push(segment);

        if is_qwik {
            self.start_recording();
        }
    }

    fn exit_call_expression(&mut self, node: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        let segment = self.segment_stack.pop();

        if let Some(segment) = segment {
            if segment.is_qwik() {
                self.stop_recording();
                if let Some(comp) = self.component_stack.pop() {
                    let qrl = &comp.qurl;
                    let qrl = qrl.clone();
                    println!(
                        "CALLEE BEFORE: {:#?} PARENT {:?} GRANDPARENT {:?}",
                        node.callee,
                        ctx.parent(),
                        ctx.ancestor(1)
                    );

                    match ctx.parent() {
                        Ancestor::JSXExpressionContainerExpression(_) => {
                            self.jsx_qurl_stack.push(qrl)
                        }
                        Ancestor::VariableDeclaratorInit(_) => self.var_decl_stack.push(qrl),
                        Ancestor::CallExpressionArguments(_) => self.call_args_stack.push(qrl),
                        _ => node.callee = qrl.into_in(ctx.ast.allocator),
                    }

                    println!("CALLEE AFTER: {:#?}", node.callee);
                    self.components.push(comp);
                }
            }
        }

        self.debug(format!(
            "EXIT: CallExpression. SCOPE[{:?}]",
            ctx.current_scope_id()
        ));
        self.descend();
    }

    fn exit_argument(&mut self, node: &mut Argument<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Argument::CallExpression(call_expr) = node {
            let qrl = self.call_args_stack.pop();

            if let Some(qrl) = qrl {
                let idr = IdentifierReference::from_in(qrl.clone(), ctx.ast.allocator);
                let args: OxcVec<'a, Argument<'a>> = qrl.into_in(ctx.ast.allocator) ;

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
        self.debug(format!("ENTER: VariableDeclarator.  ID {:?}", node.id));
        let id = &node.id;
        let s: Segment = id.into_in(ctx.ast.allocator);
        self.segment_stack.push(s);
    }

    fn exit_variable_declarator(
        &mut self,
        node: &mut VariableDeclarator<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Some(init) = &mut node.init {
            let qrl = self.var_decl_stack.pop();
            if let Some(qrl) = qrl {
                node.init = Some(qrl.into_in(ctx.ast.allocator));
            }
        }

        self.segment_stack.pop();

        self.debug("EXIT: VariableDeclarator");
        self.descend();
    }

    fn enter_expression_statement(
        &mut self,
        node: &mut ExpressionStatement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.ascend();
        self.debug("ENTER: ExpressionStatement");
    }

    fn exit_expression_statement(
        &mut self,
        node: &mut ExpressionStatement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.debug("EXIT: ExpressionStatement");
        self.descend();
    }

    fn enter_arrow_function_expression(
        &mut self,
        node: &mut ArrowFunctionExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.ascend();
        self.debug("ENTER: ArrowFunctionExpression");
    }

    fn exit_arrow_function_expression(
        &mut self,
        node: &mut ArrowFunctionExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Mode::Recording(_) = self.mode {
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

            let t = &self.target;
            let scope = &self.scope;
            let imports = if self.qrl_import_stack > 0 {
                self.qrl_import_stack -= 1;
                let qrl_import = CommonImport::qrl();
                vec![qrl_import]
            } else {
                vec![]
            };

            let comp = QwikComponent::new(&self.source_info, &segments, node, imports, t, scope);
            match comp {
                Ok(comp) => {
                    self.qrl_import_stack += 1;
                    self.component_stack.push(comp);
                }
                Err(e) => {
                    self.errors.push(e);
                }
            }
        }
        self.debug("EXIT: ArrowFunctionExpression");
        self.descend();
    }

    fn enter_jsx_element(&mut self, node: &mut JSXElement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.ascend();
        self.debug("ENTER: JSXElementName");
        if let Some(name) = node.opening_element.name.get_identifier_name() {
            let segment: Segment = name.into();
            self.segment_stack.push(segment);
        }
    }

    fn exit_jsx_element(&mut self, node: &mut JSXElement<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(name) = node.opening_element.name.get_identifier_name() {
            self.segment_stack.pop();
        }
        self.debug("EXIT: JSXElementName");
        self.descend();
    }

    fn enter_jsx_attribute(&mut self, node: &mut JSXAttribute<'a>, ctx: &mut TraverseCtx<'a>) {
        self.ascend();
        self.debug("ENTER: JSXAttribute");
        println!("JSX_ATTRIBUTE BEFORE: {:#?}", node.value);
        let segment: Segment = node.name.get_identifier().name.into();
        self.segment_stack.push(segment);
    }

    fn exit_jsx_attribute(&mut self, node: &mut JSXAttribute<'a>, ctx: &mut TraverseCtx<'a>) {
        self.segment_stack.pop();
        self.debug("EXIT: JSXAttribute");
        self.descend();
        println!("JSX_ATTRIBUTE AFTER: {:#?}", node.value);
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
}

pub fn transform<S: ScriptSource>(script_source: S) -> Result<(QwikApp)> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path("foo.js")?;
    let source_text = script_source.scripts()?;
    let first_script = source_text
        .first()
        .ok_or_else(|| Error::Generic("No script found".to_string()))?;
    let mut errors = Vec::new();

    let parse_return = Parser::new(&allocator, first_script, source_type).parse();
    errors.extend(parse_return.errors);

    let mut program = parse_return.program;

    let SemanticBuilderReturn {
        semantic,
        errors: semantic_errors,
    } = SemanticBuilder::new()
        // .with_check_syntax_error(true) // Enable extra syntax error checking
        // .with_build_jsdoc(true)        // Enable JSDoc parsing
        // .with_cfg(true)                // Build a Control Flow Graph
        .build(&program);

    let source_info = SourceInfo::new("./test.tsx")?;
    let mut transform = &mut TransformGenerator::new(source_info, Target::Dev, None);

    let (symbols, scopes) = semantic.into_symbol_table_and_scope_tree();

    traverse_mut(transform, &allocator, &mut program, symbols, scopes);

    Ok(transform.app.clone())
}

#[cfg(test)]
mod tests {
    use super::*;


    const SCRIPT0: &str =r#"
    const renderHeader = component($(() => {
        console.log("mount");
     return render;
    }));
    "#;



    const SCRIPT1: &str = r#"
     import { $, component, onRender } from '@builder.io/qwik';

    export const renderHeader = $(() => {
       return (
          <div onClick={$((ctx) => console.log(ctx))}/>
       );
    });
    
    const renderHeader = component($(() => {
        console.log("mount");
     return render;
    }));
    "#;

    const EXPORT: &str = r#"export { _hW } from "@builder.io/qwik";"#;

    const QURL: &str = r#"qurl(() => import("./test.tsx_renderHeader_component_U6Kkv07sbpQ"), "renderHeader_component_U6Kkv07sbpQ")"#;

    #[test]
    fn test_transform() {
        let qwik_app = transform(Container::from_script(SCRIPT1)).unwrap();
        println!("{}", qwik_app);

        let components = &qwik_app.components;
        assert_eq!(components.len(), 3);

        let onclick = &components.get(0).unwrap().code.trim().to_string();
        let onclick_expected =
            r#"export const renderHeader_div_onClick_fV2uzAL99u4 = (ctx) => console.log(ctx);
export { _hW } from "@builder.io/qwik";"#
                .trim();

        let renderHeader = &components
            .get(1)
            .unwrap()
            .code
            .trim()
            .to_string()
            .replace("\t", "");
        let renderHeader_expected = r#"import { qrl } from "@builder.io/qwik";
export const renderHeader_zBbHWn4e8Cg = () => {
return <div onClick={qrl(() => import("./test.tsx_renderHeader_div_onClick_fV2uzAL99u4"), "renderHeader_div_onClick_fV2uzAL99u4")} />;
};
export { _hW } from "@builder.io/qwik";"#.trim();

        assert_eq!(onclick, onclick_expected);
        assert_eq!(renderHeader, renderHeader_expected);
    }
    #[test]
    fn test_script_0() {
        let qwik_app = transform(Container::from_script(SCRIPT0)).unwrap();
        println!("{}", qwik_app);
    }
}

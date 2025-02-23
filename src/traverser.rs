#![allow(unused)]

use crate::error::Error;
use crate::prelude::*;
use crate::sources::*;
use oxc_allocator::{
    Allocator, Box as OxcBox, CloneIn, FromIn, HashMap as OxcHashMap, IntoIn, Vec as OxcVec,
};
use oxc_ast::ast::{
    ArrowFunctionExpression, BindingIdentifier, BindingPattern, CallExpression, Expression,
    ExpressionStatement, Function, IdentifierName, IdentifierReference, JSXAttribute,
    JSXAttributeName, JSXClosingElement, JSXElement, JSXElementName, JSXOpeningElement, Program,
    Statement, VariableDeclaration, VariableDeclarator,
};
use oxc_ast::visit::walk_mut::*;
use oxc_ast::{match_member_expression, AstBuilder, AstType, Visit, VisitMut};
use oxc_codegen::{Codegen, Context, Gen};
use oxc_index::Idx;
use std::borrow::Cow;

use crate::component::{Qrl, QwikComponent, SourceInfo, Target};
use oxc_parser::Parser;
use oxc_semantic::{ScopeFlags, ScopeId, SemanticBuilder, SemanticBuilderReturn};
use oxc_span::*;
use oxc_traverse::{traverse_mut, Traverse, TraverseCtx};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Deref;
use std::path::Components;
use std::rc::Rc;

struct TransformGenerator<'a> {
    pub components: Vec<QwikComponent>,

    pub errors: Vec<Error>,

    allocator: &'a Allocator,

    depth: usize,

    segments: Vec<Segment>,

    mode: Mode,

    source_info: &'a SourceInfo,
    target: Target,
    scope: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Segment {
    Named(String),
    AnonymousCaptured,
    NamedCaptured(String),
}

impl Segment {
    fn new(input: String) -> Segment {
        if (input == "$") {
            Segment::AnonymousCaptured
        } else {
            match input.strip_suffix("$") {
                Some(name) => Segment::NamedCaptured(name.to_string()),
                None => Segment::Named(input),
            }
        }
    }

    fn is_qwik(&self) -> bool {
        match self {
            Segment::Named(_) => false,
            Segment::AnonymousCaptured => true,
            Segment::NamedCaptured(_) => true,
        }
    }
}

impl Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Segment::Named(name) => write!(f, "{}", name),
            Segment::AnonymousCaptured => write!(f, ""),
            Segment::NamedCaptured(name) => write!(f, "{}", name),
        }
    }
}

impl Into<String> for &Segment {
    fn into(self) -> String {
        match self {
            Segment::Named(name) => name.into(),
            Segment::AnonymousCaptured => "$".into(),
            Segment::NamedCaptured(name) => name.into(),
        }
    }
}

impl From<Atom<'_>> for Segment {
    fn from(input: Atom) -> Segment {
        input.to_string().into()
    }
}

impl From<String> for Segment {
    fn from(input: String) -> Segment {
        Segment::new(input)
    }
}

impl From<&BindingPattern<'_>> for Segment {
    fn from(input: &BindingPattern) -> Segment {
        input
            .get_identifier_name()
            .map(|s| s.to_string())
            .unwrap_or_default()
            .into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Mode {
    Scanning,
    Recording(usize),
}

impl<'a> TransformGenerator<'a> {
    fn new(
        allocator: &'a Allocator,
        source_info: &'a SourceInfo,
        target: Target,
        scope: Option<String>,
    ) -> Self {
        Self {
            components: Vec::new(),
            errors: Vec::new(),
            allocator,
            depth: 0,
            segments: Vec::new(),
            mode: Mode::Scanning,
            source_info,
            target,
            scope,
        }
    }

    fn print_segments(&self) {
        println!("{}", self.render_segments());
    }

    fn render_segments(&self) -> String {
        let ss: Vec<String> = self
            .segments
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

    fn debug(&self, s: &str) {
        if DEBUG {
            let indent = "--".repeat(self.depth);
            let prefix = format!("|{}", indent);
            println!(
                "{}[{}|MODE: {:?}]{}. Segments: {}",
                prefix, self.depth, self.mode, s, self.render_segments()
            );
        }
    }
}

const DEBUG: bool = true;

impl<'a> Traverse<'a> for TransformGenerator<'a> {
    fn enter_call_expression(&mut self, node: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.ascend();
        self.debug("ENTER: CallExpression");

        let name = node.callee_name().unwrap_or_default().to_string();

        let segment: Segment = name.into();
        let is_qwik = segment.is_qwik();
        self.segments.push(segment);

        if is_qwik {
            self.start_recording();
        }
    }

    fn exit_call_expression(&mut self, node: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        let segment = self.segments.pop();

        if let Some(segment) = segment {
            if segment.is_qwik() {
                self.stop_recording();
            }
        }

        self.debug("EXIT: CallExpression");
        self.descend();
    }

    fn enter_variable_declaration(
        &mut self,
        node: &mut VariableDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.ascend();
        self.debug("ENTER: VariableDeclaration");

        if let Some(vd) = node.declarations.first() {
            let id = &vd.id;
            let s: Segment = id.into();
            self.segments.push(s);
        }
    }

    fn exit_variable_declaration(
        &mut self,
        node: &mut VariableDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Some(vd) = node.declarations.first() {
            self.segments.pop();
        }
        self.debug("EXIT: VariableDeclaration");
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
                .segments
                .iter()
                .filter(|s| **s != Segment::AnonymousCaptured)
                .map(|s| {
                    let string: String = s.into();
                    string
                })
                .collect();

            let t = &self.target;
            let scope = &self.scope;

            let comp = QwikComponent::new(self.source_info, &segments, &node, t, scope);
            match comp {
                Ok(comp) => {
                    let qrl = comp.qurl.clone();
                    self.components.push(comp);
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
            self.segments.push(segment);
        }
    }

    fn exit_jsx_element(&mut self, node: &mut JSXElement<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(name) = node.opening_element.name.get_identifier_name() {
            self.segments.pop();
        }
        self.debug("EXIT: JSXElementName");
        self.descend();
    }

    fn enter_jsx_attribute(&mut self, node: &mut JSXAttribute<'a>, ctx: &mut TraverseCtx<'a>) {
        self.ascend();
        self.debug("ENTER: JSXAttribute");
        let segment: Segment = node.name.get_identifier().name.into();
        self.segments.push(segment);
    }

    fn exit_jsx_attribute(&mut self, node: &mut JSXAttribute<'a>, ctx: &mut TraverseCtx<'a>) {
        self.segments.pop();
        self.debug("EXIT: JSXAttributeName");
        self.descend();
    }
}

pub fn transform<'a, S: ScriptSource>(script_source: S) -> Result<(Vec<QwikComponent>)> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path("foo.js")?;
    let source_text = script_source.scripts()?;
    let first_script = source_text
        .first()
        .ok_or_else(|| Error::Generic("No script found".to_string()))?;
    let mut errors = Vec::new();

    // Step 1: Parsing
    // Parse the TSX file into an AST. The root AST node is a `Program` struct.
    // let ParserReturn { program, module_record, errors , irregular_whitespaces, panicked, is_flow_language } =
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

    let source_info = SourceInfo::new("test.tsx")?;
    let mut v = &mut TransformGenerator::new(&allocator, &source_info, Target::Dev, None);

    let (symbols, scopes) = semantic.into_symbol_table_and_scope_tree();

    traverse_mut(v, &allocator, &mut program, symbols, scopes);

    println!("-------------------------------------");
    println!("Arrow funcs {}", v.components.len());
    v.components.iter().for_each(|comp| {
        let mut code_gen0 = Codegen::default();
        let code_gen = &mut code_gen0;

        let body = &comp.code;
        println!("{}", body);
    });
    println!("-------------------------------------");

    Ok(v.components.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let components = transform(Container::from_script(SCRIPT1)).unwrap();
        assert_eq!(components.len(), 3);

        let onclick = &components.get(0).unwrap().code.trim().to_string();
        let onclick_expected =
            r#"export const renderHeader_div_onClick_fV2uzAL99u4 = (ctx) => console.log(ctx);
export { _hW } from "@builder.io/qwik";"#
                .trim();
        assert_eq!(onclick, onclick_expected);
    }
}

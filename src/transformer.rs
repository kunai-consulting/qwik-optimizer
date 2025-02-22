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
    JSXClosingElement, JSXElement, JSXOpeningElement, Program, Statement, VariableDeclaration,
    VariableDeclarator,
};
use oxc_ast::visit::walk_mut::*;
use oxc_ast::{match_member_expression, AstBuilder, AstType, Visit, VisitMut};
use oxc_codegen::{Codegen, Context, Gen};
use oxc_index::Idx;
use std::borrow::Cow;

use crate::component::{Qrl, QwikComponent, SourceInfo, Target};
use crate::transformer::Mode::Recording;
use oxc_semantic::{ScopeFlags, ScopeId, SemanticBuilder, SemanticBuilderReturn};
use oxc_span::*;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ops::Deref;
use std::path::Components;
use std::rc::Rc;
use oxc_parser::Parser;

struct TransformGenerator<'a> {
    pub components: Vec<QwikComponent>,

    pub errors: Vec<Error>,

    allocator: &'a Allocator,

    current_scope_id: usize,

    segments: Vec<String>,

    depth: usize,

    mode: Mode,

    source_info: &'a SourceInfo,
    target: Target,
    scope: Option<String>,
}

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
            current_scope_id: 0,
            segments: Vec::new(),
            depth: 0,
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
        self.segments.iter().fold("".to_string(), |acc, s| {
            format!("{}_{}", acc.to_string(), s.to_string())
        })
    }

    fn ascend(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        }
    }

    fn descend(&mut self) {
        self.depth += 1;
    }

    fn push_segment(&mut self, segment: String) -> bool {
        if (!segment.is_empty()) {
            self.segments.push(segment);
            true
        } else {
            false
        }
    }

    fn start_recording(&mut self) {
        self.mode = match self.mode {
            Mode::Scanning => Recording(1),
            Mode::Recording(count) => Recording(count + 1),
        }
    }

    fn stop_recording(&mut self) {
        self.mode = match self.mode {
            Mode::Scanning => Mode::Scanning,
            Mode::Recording(count) if count > 1 => Recording(count - 1),
            Mode::Recording(_) => Mode::Scanning,
        }
    }

    // pub fn component_aware_walk_expression_statement<'a, V: VisitMut<'a>>(
    //     visitor: &mut V,
    //     it: &mut ExpressionStatement<'a>,
    // ) {
    //     let kind = AstType::ExpressionStatement;
    //     visitor.enter_node(kind);
    //     visitor.visit_span(&mut it.span);
    //     Self::component_aware_walk_expression(visitor, it);
    //     visitor.leave_node(kind);
    // }
    //
    // pub fn component_aware_walk_expression<'a, V: VisitMut<'a>>(
    //     visitor: &mut V,
    //     es: &mut ExpressionStatement<'a>,
    // ) {
    //     // No `AstType` for this type
    //     let expr0 = &mut es.expression;
    //     match expr0 {
    //         Expression::CallExpression(it) => {
    //             Self::component_aware_walk_call_expression(visitor, es)
    //         }
    //
    //         // visitor.visit_call_expression(it),
    //         expr => walk_expression(visitor, expr),
    //     }
    // }
    //
    // pub fn component_aware_walk_call_expression<'a, V: VisitMut<'a>>(
    //     visitor: &mut V,
    //     es: &mut ExpressionStatement<'a>,
    // ) {
    //     let it = &mut es.expression;
    //
    //     if let Expression::CallExpression(it) = it {
    //         let kind = AstType::CallExpression;
    //         visitor.enter_node(kind);
    //         visitor.visit_span(&mut it.span);
    //         visitor.visit_expression(&mut it.callee);
    //         if let Some(type_parameters) = &mut it.type_parameters {
    //             visitor.visit_ts_type_parameter_instantiation(type_parameters);
    //         }
    //         visitor.visit_arguments(&mut it.arguments);
    //         visitor.leave_node(kind);
    //     }
    // }
}

const LOG_STATEMENTS: bool = false;
const LOG_NODE_WALKS: bool = true;

const LOG_ARROW_FUNCS: bool = true;

impl<'a> VisitMut<'a> for TransformGenerator<'a> {
    fn enter_node(&mut self, kind: AstType) {
        self.descend();
        if LOG_NODE_WALKS {
            let indent = "-".repeat(self.depth);
            println!(
                "{}-> [S:{}] Entering {:?}.  Segments: {}",
                indent,
                self.current_scope_id,
                kind,
                self.render_segments()
            );
        }
    }

    fn leave_node(&mut self, kind: AstType) {
        self.ascend();
        if LOG_NODE_WALKS {
            let indent = "-".repeat(self.depth);
            println!(
                "<-{} [s:{}]Leaving {:?}.  Segments: {}",
                indent,
                self.current_scope_id,
                kind,
                self.render_segments()
            );
        }
    }

    fn enter_scope(&mut self, flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {
        self.current_scope_id = scope_id.get().map(|s| s.index()).unwrap_or(0);
    }

    fn leave_scope(&mut self) {
        // println!("Exiting scope '{}'", self.current_scope_id);
        if self.current_scope_id > 0 {
            self.current_scope_id -= 1;
        }
    }

    fn visit_binding_identifier(&mut self, it: &mut BindingIdentifier<'a>) {
        // println!("Binding identifier {:?}", it);
        walk_binding_identifier(self, it);
    }

    fn visit_call_expression(&mut self, it: &mut CallExpression<'a>) {
        let mut recording = false;

        let name = it.normalize_name();
        let pushed = self.push_segment(name);
        if it.is_qwik() {
            recording = true;
            self.start_recording();
        }

        walk_call_expression(self, it);

        if pushed {
            self.segments.pop();
        }
        if recording {
            self.stop_recording();
        }
    }

    fn visit_statement(&mut self, it: &mut Statement<'a>) {
        if LOG_STATEMENTS {
            let indent = " ".repeat(self.current_scope_id * 4);
            println!("{}[{}]{:#?}", indent, self.current_scope_id, it);
        }
        walk_statement(self, it);
    }

    fn visit_variable_declaration(&mut self, it: &mut VariableDeclaration<'a>) {
        if let Some(vd) = it.declarations.first() {
            let name = &vd
                .id
                .get_identifier_name()
                .map(|s| s.to_string())
                .unwrap_or_default();

            let pushed = self.push_segment(name.clone());

            walk_variable_declaration(self, it);

            if pushed {
                let s = self.segments.pop();
                // if let Some(s) = s {
                //     println!("Popped segment {}", s);
                // }
            }
        }
    }

    fn visit_expression_statement(&mut self, expr_statement: &mut ExpressionStatement<'a>) {

        let kind = AstType::ExpressionStatement;
        self.enter_node(kind);
        self.visit_span(&mut expr_statement.span);

        //
        // let cell = RefCell::new(expr_statement);
        // let rc = Rc::new(cell);
        // self.expression_statements.push(rc);



        self.visit_expression(&mut expr_statement.expression);


        self.leave_node(kind);



        // self.expression_statements.push(expr_statement);

        // let expression = &mut expr_statement.expression;
        // // if let Expression::CallExpression(it) = expr {
        // let kind = AstType::ExpressionStatement;
        // self.enter_node(kind);
        // self.visit_span(&mut expr_statement.span);
        // match expression {
        //     Expression::CallExpression(call_expr) => {
        //
        //         let callee = &mut call_expr.callee;
        //
        //         match callee {
        //             Expression::ArrowFunctionExpression(arrow_func_expr) => {
        //                 if let Mode::Recording(_) = self.mode {
        //                     let name = self.render_segments();
        //                     let segments: &Vec<&str> =
        //                         &self.segments.iter().map(|s| s.as_str()).collect();
        //                     let comp = QwikComponent::new(
        //                         self.source_info,
        //                         segments,
        //                         &arrow_func_expr,
        //                         self.target,
        //                         self.scope,
        //                     );
        //                     match comp {
        //                         Ok(comp) => {
        //                             let qrl = comp.qurl.clone();
        //                             expr_statement.expression = qrl.into_in(self.allocator);
        //                             self.components.push(comp);
        //                         }
        //                         Err(e) => {
        //                             self.errors.push(e);
        //                         }
        //                     }
        //                 }
        //                 // walk_arrow_function_expression(self, arrow_func_expr);
        //             }
        //             // callee => walk_expression(self, callee)
        //         }
        //     },
        //     expr => self.visit_expression(&mut expr_statement.expression)
        // }
        // self.leave_node(kind);
    }

    fn visit_arrow_function_expression(&mut self, it: &mut ArrowFunctionExpression<'a>) {
        walk_arrow_function_expression(self, it);

        if LOG_ARROW_FUNCS {
            println!("BEGIN: Recording function expression {:?}", it);
        }

        if let Mode::Recording(_) = self.mode {
            let name = self.render_segments();
            let segments: &Vec<&str> = &self.segments.iter().map(|s| s.as_str()).collect();

            let t = &self.target;
            let scope = &self.scope;

            let comp = QwikComponent::new(self.source_info, segments, &it, t, scope);
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

        if LOG_ARROW_FUNCS {
            println!("END: Recording Arrow function expression {:?}", it);
        }
    }

    fn visit_jsx_element(&mut self, it: &mut JSXElement<'a>) {
        if let Some(name) = it.opening_element.name.get_identifier_name() {
            let pushed = self.push_segment(name.to_string());
            walk_jsx_element(self, it);
            if pushed {
                self.segments.pop();
            }
        }
    }

    fn visit_jsx_attribute(&mut self, it: &mut JSXAttribute<'a>) {
        let indent = " ".repeat(self.current_scope_id * 4);

        let pushed = self.push_segment(it.normalize_name());
        walk_jsx_attribute(self, it);
        if pushed {
            self.segments.pop();
        }
    }
}

pub fn transform<'a, S: ScriptSource>(script_source: S) -> Result<Vec<QwikComponent>> {
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
    let mut v = TransformGenerator::new(&allocator, &source_info, Target::Dev, None);

    v.visit_program(&mut program);

    // let app = Codegen::new().build(&v.app).code;
    println!("-------------------------------------");
    // println!("Application\n{}", app);
    println!("-------------------------------------");

    println!("-------------------------------------");
    println!("Arrow funcs {}", v.components.len());
    v.components.iter().for_each(|comp| {
        let mut code_gen0 = Codegen::default();
        let code_gen = &mut code_gen0;

        let body = &comp.code;
        println!("{}", body);
    });
    println!("-------------------------------------");

    Ok(v.components)
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

        let onclick = &components.get(1).unwrap().code.trim().to_string();
        let onclick_expected = r#"export const renderHeader_div_onClick_fV2uzAL99u4 = (ctx) => console.log(ctx);
export { _hW } from "@builder.io/qwik";"#;
        assert_eq!(onclick, onclick_expected);
    }
}

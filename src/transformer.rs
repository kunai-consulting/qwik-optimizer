#![allow(unused)]

use crate::error::Error;
use crate::prelude::*;
use crate::sources::*;
use oxc::allocator::{
    Allocator, Box as OxcBox, CloneIn, FromIn, HashMap as OxcHashMap, IntoIn, Vec as OxcVec,
};
use oxc::ast::ast::{
    ArrowFunctionExpression, BindingIdentifier, BindingPattern, CallExpression, Expression,
    Function, IdentifierName, IdentifierReference, JSXAttribute, JSXClosingElement, JSXElement,
    JSXOpeningElement, Program, Statement, VariableDeclaration, VariableDeclarator,
};
use oxc::ast::visit::walk_mut::*;
use oxc::ast::{AstType, Visit, VisitMut};
use oxc::codegen::{Codegen, Context, Gen};
use oxc_index::Idx;
use std::borrow::Cow;

use crate::component::{QwikComponent, SourceInfo, Target};
use crate::transformer::Mode::Recording;
use oxc::parser::Parser;
use oxc::semantic::{ScopeFlags, ScopeId, SemanticBuilder, SemanticBuilderReturn};
use oxc::span::*;
use std::cell::Cell;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::Components;

struct TransformGenerator<'a> {
    pub components: Vec<QwikComponent<'a>>,

    pub exported_components: OxcVec<'a, Program<'a>>,

    allocator: &'a Allocator,

    current_scope_id: usize,

    segments: Vec<String>,

    depth: usize,

    mode: Mode,

    source_info: &'a SourceInfo,
    target: &'a Target,
    scope: &'a Option<String>,
}

enum Mode {
    Scanning,
    Recording(usize),
}

impl<'a> TransformGenerator<'a> {
    fn new(
        allocator: &'a Allocator,
        source_info: &'a SourceInfo,
        target: &'a Target,
        scope: &'a Option<String>,
    ) -> Self {
        let source_type: SourceType /* Type */ = source_info.try_into().unwrap();

        Self {
            exported_components: OxcVec::new_in(allocator),
            allocator,
            current_scope_id: 0,
            segments: Vec::new(),
            depth: 0,
            components: Vec::new(),
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
}

const LOG_STATEMENTS: bool = false;
const LOG_NODE_WALKS: bool = true;

const LOG_ARROW_FUNCS: bool = true;

impl<'a> VisitMut<'a> for TransformGenerator<'a> {
    fn enter_scope(&mut self, flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {
        self.current_scope_id = scope_id.get().map(|s| s.index()).unwrap_or(0);
        // println!("Entering scope: {}", self.current_scope_id);
    }

    fn leave_scope(&mut self) {
        // println!("Exiting scope '{}'", self.current_scope_id);
        if self.current_scope_id > 0 {
            self.current_scope_id -= 1;
        }
    }

    fn visit_call_expression(&mut self, it: &mut CallExpression<'a>) {
        let indent = " ".repeat(self.current_scope_id * 4);
        println!("{}Call expression {:?}", indent, it);
        let mut recording = false;

        let name = it.normalize_name();
        self.push_segment(name);
        if it.is_qwik() {
            recording = true;
            self.start_recording();
        }

        walk_call_expression(self, it);

        self.segments.pop();
        if recording {
            self.stop_recording();
        }
    }

    fn visit_binding_identifier(&mut self, it: &mut BindingIdentifier<'a>) {
        // println!("Binding identifier {:?}", it);
        walk_binding_identifier(self, it);
    }

    fn visit_binding_pattern(&mut self, it: &mut BindingPattern<'a>) {
        // println!("Binding pattern {:?}", it);
        walk_binding_pattern(self, it);
    }

    fn visit_statement(&mut self, it: &mut Statement<'a>) {
        if LOG_STATEMENTS {
            let indent = " ".repeat(self.current_scope_id * 4);
            println!("{}[{}]{:#?}", indent, self.current_scope_id, it);
        }
        walk_statement(self, it);
    }

    fn visit_variable_declaration(&mut self, it: &mut VariableDeclaration<'a>) {
        let vd = it.declarations.first().unwrap();
        let name = &vd
            .id
            .get_identifier_name()
            .map(|s| s.to_string())
            .unwrap_or_default();

        // self.current_decl_name = name.clone();
        let pushed = self.push_segment(name.clone());
        walk_variable_declaration(self, it);
        if pushed {
            self.segments.pop();
        }
    }

    fn visit_variable_declarator(&mut self, it: &mut VariableDeclarator<'a>) {
        // println!("Variable declarator {:?}", it);
        walk_variable_declarator(self, it);
    }

    fn visit_jsx_element(&mut self, it: &mut JSXElement<'a>) {
        let indent = " ".repeat(self.current_scope_id * 4);
        let name = it.opening_element.name.get_identifier_name().unwrap();
        let pushed = self.push_segment(name.to_string());
        walk_jsx_element(self, it);
        if pushed {
            self.segments.pop();
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

    fn visit_arrow_function_expression(&mut self, it: &mut ArrowFunctionExpression<'a>) {
        if LOG_ARROW_FUNCS {
            println!("BEGIN: Arrow function expression {:?}", it);
        }
        match self.mode {
            Mode::Recording(_) => {
                let name = self.render_segments();
                // self.comps.insert(name.clone(), it.clone_in(self.allocator));
                let segments: &Vec<&str> = &self.segments.iter().map(|s| s.as_str()).collect();
                let comp = QwikComponent::new(
                    &self.source_info,
                    segments,
                    it.clone_in(self.allocator),
                    &self.target,
                    &self.scope,
                );
                self.components.push(comp);
            }
            Mode::Scanning => (),
        }

        walk_arrow_function_expression(self, it);

        if LOG_ARROW_FUNCS {
            println!("END: Arrow function expression {:?}", it);
        }
    }

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

    // fn visit_program(&mut self, pgm: &mut Program<'a>) {
    //     // self.app.comments = pgm.comments.clone_in(self.allocator);
    //     walk_program(self, pgm);
    // }

    // fn visit_function(&mut self, func: &mut Function<'a>, flags: ScopeFlags) {
    //     let name = func.name().map(|n| n.to_string());
    //
    //     let source_type = self.source_type;
    //     let f0 = func.clone_in(self.allocator);
    //     let f: Box<Function<'a>> = Box::new_in(f0, self.allocator);
    //     let s: Statement<'a> = Statement::FunctionDeclaration(f);
    //
    //     match name {
    //         Some(name) if name.contains("$") => {
    //             println!("Extracting Function name: {}", name);
    //             // self.exported_functions.push(s);
    //
    //             let mut body = OxcVec::new_in(self.allocator);
    //             body.push(s);
    //
    //             let new_pgm = Program {
    //                 span: SPAN,
    //                 source_type,
    //                 source_text: "",
    //                 comments: OxcVec::new_in(self.allocator),
    //                 hashbang: None,
    //                 directives: OxcVec::new_in(self.allocator),
    //                 body,
    //                 scope_id: Cell::new(None),
    //             };
    //
    //             self.exported_components.push(new_pgm);
    //         }
    //         Some(name) => {
    //             println!("Keeping Function name: {}", name);
    //             self.app.body.push(s);
    //         }
    //         None => {
    //             println!("Anonymous function");
    //             self.app.body.push(s);
    //         }
    //     }
    //     // println!("Function name: {:?}", func.name());
    //     walk_function(self, func, flags);
    // }
}

pub fn transform<'a, S: ScriptSource>(script_source: S) -> Result<()> {
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

    // let mut v = SimpleVisitor {
    //     exported_functions: Vec::new(),
    //     kept_functions: Vec::new(),
    //     allocator: &allocator,
    // };

    let source_info = SourceInfo::new("test.tsx")?;
    let mut v = TransformGenerator::new(&allocator, &source_info, &Target::Dev, &None);

    v.visit_program(&mut program);

    v.exported_components.iter().for_each(|pgm| {
        println!("-------------------------------------");
        println!("Exported component\n{}", Codegen::new().build(pgm).code);
        println!("-------------------------------------")
    });

    // let app = Codegen::new().build(&v.app).code;
    println!("-------------------------------------");
    // println!("Application\n{}", app);
    println!("-------------------------------------");

    println!("-------------------------------------");
    println!("Arrow funcs {}", v.components.len());
    v.components.iter().for_each(|comp| {
        let mut code_gen0 = Codegen::default();
        let code_gen = &mut code_gen0;

        let body = comp.gen(&allocator);
        println!("{}", body);
    });
    println!("-------------------------------------");

    //
    // let s1 = Statement::ExpressionStatement(
    //     Box::new_in (ExpressionStatement {
    //         span: SPAN,
    //         expression:
    //         Expression::StringLiteral(Box::new_in(StringLiteral { span: SPAN, value: Atom::from_in("hi!", &allocator), raw: None }, &allocator)),
    //     }, &allocator),
    // );
    //
    // let mut body = OxcVec::new_in(&allocator);
    // body.push(s1);
    //
    // let new_pgm = Program {
    //     span: SPAN,
    //     source_type,
    //     source_text: "",
    //     comments: OxcVec::new_in(&allocator),
    //     hashbang: None,
    //     directives: OxcVec::new_in(&allocator),
    //     body,
    //     scope_id: Cell::new(None),
    // };
    //
    // let ns = Codegen::new().build(&new_pgm).code;
    // println!("NEW JS!!!\n{}", ns);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCRIPT0: &str = r#"
        function $greet(name) {
            return "Hello, " + name + "!";
        }


        /**
         *
         * @param name
         * @returns {string}
         */
        function more_complex(name) {
            if(name === "Alice") {
                return "Hello, Alice!";
            } else if(name === "Bob") {
                return "Hello, Bob!";
            } else {
                return "Hello, " + name + "!";
            }
            return "Hello, " + name + "!";
        }

        function another_to_export(name) {
            return "Goodbye, " + name + "!";
        }

        $greet("Alice");
    "#;

    const SCRIPT1: &str = r#"
    export const Counter = component$(() => {
      const count = useSignal(0);
 
      return <button onClick$={() => count.value++}>{count.value}</button>;
    });
    
     export const Greeter = component$(() => {
 
      return <p>hello!</p>;
    });
    "#;

    const SCRIPT2: &str = r#"
    import { component$, useStore } from '@qwik.dev/core';

    export default component$(() => {
    const store = useStore({ count: 0 });

    return (
      <main>
        <p>Count: {store.count}</p>
        <p>
          <button onClick$={() => store.count++}>Click</button>
        </p>
      </main>
    );
 });
    "#;

    const SCRIPT3: &str = r#"
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

    #[test]
    fn test_transform() {
        transform(Container::from_script(SCRIPT3)).unwrap();
    }
}

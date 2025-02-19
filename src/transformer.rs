#![allow(unused)]

use crate::error::Error;
use crate::prelude::*;
use crate::sources::*;
use oxc::allocator::{Allocator, Box, CloneIn, FromIn, Vec as OxcVec};
use oxc::ast::ast::{Function, Program, Statement};
use oxc::ast::visit::walk_mut::{walk_function, walk_program, walk_statement};
use oxc::ast::{Visit, VisitMut};
use oxc::codegen::Codegen;
use oxc_index::Idx;

use oxc::parser::Parser;
use oxc::semantic::{ScopeFlags, ScopeId, SemanticBuilder, SemanticBuilderReturn};
use oxc::span::*;
use std::cell::Cell;

struct TransformGenerator<'a> {
    pub exported_components: OxcVec<'a, Program<'a>>,

    pub app: Program<'a>,

    pub source_type: SourceType,

    // pub kept_functions: Vec<Statement<'a>>,
    pub allocator: &'a Allocator,
    
    current_scope_id: usize
}

impl<'a> TransformGenerator<'a> {
    fn new(allocator: &'a Allocator, source_type: SourceType) -> Self {
        let app = Program {
            span: SPAN,
            source_type,
            source_text: "",
            comments: OxcVec::new_in(allocator),
            hashbang: None,
            directives: OxcVec::new_in(allocator),
            body: OxcVec::new_in(allocator),
            scope_id: Cell::new(None),
        };

        Self {
            exported_components: OxcVec::new_in(allocator),
            app,
            source_type,
            allocator,
            current_scope_id: 0,
        }
    }
}

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
   
    fn visit_statement(&mut self, it: &mut Statement<'a>) {
    
        // match it {
        //     Statement::FunctionDeclaration(func) => self.visit_function(func, ScopeFlags::empty()),
        //     
        //     statement => self.app.body.push(statement.clone_in(self.allocator)),
        // }
        // it.
        let indent = " ".repeat(self.current_scope_id * 4);
        println!("{}[{}]{:?}", indent,self.current_scope_id , it);
        walk_statement(self, it);
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

    let mut v = TransformGenerator::new(&allocator, source_type);

    v.visit_program(&mut program);

    v.exported_components.iter().for_each(|pgm| {
        println!("-------------------------------------");
        println!("Exported component\n{}", Codegen::new().build(pgm).code);
        println!("-------------------------------------")
    });


    let app = Codegen::new().build(&v.app).code;
    println!("-------------------------------------");
    println!("Application\n{}", app);
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
    "#;

    #[test]
    fn test_transform() {
        transform(Container::from_script(SCRIPT1)).unwrap();
    }
}

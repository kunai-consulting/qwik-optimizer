use std::fs;
use std::ops::Deref;
use std::rc::Rc;
use oxc::allocator::{Allocator, Box, CloneIn};
use oxc::ast::ast::{ExpressionStatement, Function, Statement};
use oxc::ast::visit::walk;
use oxc::ast::visit::walk_mut::walk_function;
use oxc::ast::{AstBuilder, Visit, VisitMut};
use oxc::ast::ast::FunctionType::FunctionExpression;
use oxc::codegen::Gen;
use oxc::parser::{Parser, ParserReturn};
use oxc::semantic::{ScopeFlags, SemanticBuilder, SemanticBuilderReturn};
use oxc::span::{SourceType, SPAN};
use oxc::transformer::Transformer;
use crate::component::SourceInfo;
use crate::error::Error;
use crate::prelude::*;
use crate::sources::*;

struct TransformGenerator<'a> {
    pub  ast_builder: AstBuilder<'a>,
    
    pub exported_functions: Vec<Statement<'a>>,
    
    pub kept_functions: Vec<Statement<'a>>,
    
    pub allocator: &'a Allocator,
}

impl<'a> TransformGenerator<'a> {
    fn new(allocator: &'a Allocator) -> Self {
        Self {
            ast_builder: AstBuilder::new(allocator),
            kept_functions: Vec::new(),
            exported_functions: Vec::new(),
            allocator,
        }
    }
}

impl<'a> Visit<'a> for TransformGenerator<'a> {
    fn leave_scope(&mut self) {
        println!("Leaving scope!")
    }
    
    
    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        let name = func.name().map(|n| n.to_string());
        
        match name {
            Some(name) if name.contains("$") => {
                println!("Extracting Function name: {}", name);
                let f0 =  func.clone_in(self.allocator);
                let f: Box<Function<'a>> = Box::new_in(f0, self.allocator);
                let s: Statement<'a> = Statement::FunctionDeclaration(f);
                self.exported_functions.push(s);
     
            }
            Some(name)  => {
                println!("Keeping Function name: {}", name);
                
            }
            None => {
                println!("Anonymous function");
            }
        }

        // walk_function(self, func, flags);
        
    }

}

// impl<'a> TransformGenerator<'a> {
//     pub fn export_program(&self) {
//         self.ast_builder.program(SPAN, )
//         
//     }
// }
// 


pub fn transform<'a, S: ScriptSource>(script_source: S) -> Result<()> {
    

    let allocator = Allocator::default();
    let source_type = SourceType::from_path("foo.js")?;
    let source_text = script_source.scripts()?;
    let first_script = source_text.first().ok_or_else(|| Error::Generic("No script found".to_string()))?;
    let mut errors = Vec::new();

    // Step 1: Parsing
    // Parse the TSX file into an AST. The root AST node is a `Program` struct.
    // let ParserReturn { program, module_record, errors , irregular_whitespaces, panicked, is_flow_language } =
    let parse_return =   Parser::new(&allocator, first_script, source_type).parse();
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
    
    let mut v = TransformGenerator::new(&allocator);
    
    v.visit_program(&mut program);
  
    // v.exported_functions.program(SPAN, SourceType::from_path("foo.js").unwrap(),  );
    

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const  script0: &str = r#"
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

        // split
        function another_to_export(name) {
            return "Goodbye, " + name + "!";
        }

        $greet("Alice");
    "#;


    #[test]
    fn test_transform() {

         transform(Container::from_script(script0)).unwrap();

    }
}
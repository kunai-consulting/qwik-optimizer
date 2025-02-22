use oxc_allocator::Allocator;
// use oxc_allocator::Allocator;
// use oxc_ast::ast::Program;
// use oxc::parser::Parser;
// use oxc_semantic::{SemanticBuilder, SemanticBuilderReturn};
// use oxc_span::SourceType;
use oxc_traverse::*;
use oxc_ast::ast::*;
use oxc_parser::*;
use oxc_semantic::*;
use crate::component::SourceInfo;
use crate::error::Error;
use crate::sources::ScriptSource;
use crate::prelude::*;


struct QwikTraverse {}

impl Traverse<'_> for QwikTraverse {
    fn enter_program(&mut self, node: &mut Program<'_>, ctx: &mut TraverseCtx<'_>) {
        todo!()
    }


}

impl QwikTraverse {
    fn new() -> Self {
        Self {}
    }

}

pub fn traverse<S: ScriptSource>(script_source: S) -> Result<()> { let allocator = Allocator::default();
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

    let mut traverse = QwikTraverse::new();

    let (symbols, scopes) = semantic.into_symbol_table_and_scope_tree();

    traverse_mut(&mut traverse, &allocator, &mut program, symbols, scopes);

    // traverse_mut()
    Ok(())

}
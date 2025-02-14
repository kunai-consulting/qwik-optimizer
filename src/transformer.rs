use std::fs;
use oxc::allocator::Allocator;
use oxc::parser::{Parser, ParserReturn};
use oxc::span::SourceType;
use crate::component::SourceInfo;
use crate::prelude::*;

pub fn transform(source_info: SourceInfo) -> Result<()> {
    
    let source_text = fs::read_to_string(source_info.abs_path)?;
    
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(source_info.rel_path)?;
    let mut errors = Vec::new();

    // Step 1: Parsing
    // Parse the TSX file into an AST. The root AST node is a `Program` struct.
    let ParserReturn { program, trivias, errors: parser_errors, panicked, .. } =
        Parser::new(&allocator, source_text.as_str(), source_type).parse();
    errors.extend(parser_errors);
    
    Ok(())
}
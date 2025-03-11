use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_codegen::*;
use oxc_minifier::*;
use oxc_parser::Parser;
use oxc_span::SourceType;

pub struct Minifier<'a> {
    allocator: &'a Allocator,
    source_type: SourceType,
}

impl<'a> Minifier<'a> {
    pub fn new(allocator: &'a Allocator, source_type: SourceType) -> Self {
        Self { allocator, source_type }
    }

    pub fn minify_source(&self, source_text: &str) -> String {
        let ret = Parser::new(self.allocator, source_text, self.source_type).parse();
        let errors = ret.errors;
        if !errors.is_empty() {
            panic!("{:?}", errors);
        }
        let mut program = ret.program;
        self.minify(&mut program)
    }

    pub fn minify(&self, program: &'a mut Program<'a>) -> String {
        CodeGenerator::new()
            .with_options(CodegenOptions {
                minify: true,
                annotation_comments: true,
                ..CodegenOptions::default()
            })
            .build(program)
            .code
    }
}

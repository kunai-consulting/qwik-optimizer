use crate::component::SourceInfo;
use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_semantic::SemanticBuilder;
use oxc_transformer::{TransformOptions, Transformer};

pub struct Transpiler;

impl Transpiler {
    pub fn transpile<'a>(
        allocator: &'a Allocator,
        pgm: &mut Program<'a>,
        source_info: &SourceInfo,
    ) {
        // TODO Correctly handle errors.
        let ret = SemanticBuilder::new().with_excess_capacity(2.0).build(&pgm);
        let scoping = ret.semantic.into_scoping();
        let path = &source_info.rel_path;
        // TODO Correctly handle errors.
        let ret = Transformer::new(allocator, path, &TransformOptions::default())
            .build_with_scoping(scoping, pgm);
    }
}

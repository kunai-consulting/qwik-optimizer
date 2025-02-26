use std::borrow::Borrow;
use crate::ext::AstBuilderExt;
use oxc_allocator::{Allocator, FromIn, IntoIn};
use oxc_ast::ast::{Expression, Statement};
use oxc_ast::AstBuilder;

const BUILDER_IO_QWIK: &str = "@builder.io/qwik";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommonImport {
    BuilderIoQwik(String),
}

impl CommonImport {

    pub fn qrl() -> CommonImport {
        CommonImport::BuilderIoQwik("qrl".to_string())
    }
}

impl<'a> FromIn<'a, CommonImport> for Statement<'a> {
    fn from_in(value: CommonImport, allocator: &'a Allocator) -> Self {
        let ast_builder = AstBuilder::new(allocator);
        match value {
            CommonImport::BuilderIoQwik(name) => {
                ast_builder.qwik_import(name.as_str(), BUILDER_IO_QWIK)
            }
        }
    }
    
}



#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommonExport {
    BuilderIoQwik(String),
}

impl<'a> FromIn<'a, CommonExport> for Statement<'a> {
    fn from_in(value: CommonExport, allocator: &'a Allocator) -> Self {
        let ast_builder = AstBuilder::new(allocator);
        match value {
            CommonExport::BuilderIoQwik(name) => {
                ast_builder.qwik_export(name.as_str(), BUILDER_IO_QWIK)
            }
        }
    }
}

/// Renamed from `EmitMode` in V 1.0.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Target {
    Prod,
    Lib,
    Dev,
    Test,
}

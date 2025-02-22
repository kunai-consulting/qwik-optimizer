use crate::ext::AstBuilderExt;
use oxc_allocator::{Allocator, IntoIn};
use oxc_ast::ast::{Expression, Statement};
use oxc_ast::AstBuilder;

const BUILDER_IO_QWIK: &str = "@builder.io/qwik";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum CommonImport {
    BuilderIoQwik(String),
}

impl<'a> IntoIn<'a, Statement<'a>> for CommonImport {
    fn into_in(self, allocator: &'a Allocator) -> Statement<'a> {
        let ast_builder = AstBuilder::new(allocator);
        match self {
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

impl<'a> IntoIn<'a, Statement<'a>> for CommonExport {
    fn into_in(self, allocator: &'a Allocator) -> Statement<'a> {
        let ast_builder = AstBuilder::new(allocator);
        match self {
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

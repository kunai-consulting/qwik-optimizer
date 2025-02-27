use std::convert::Into;
use crate::ext::AstBuilderExt;
use oxc_allocator::{Allocator, FromIn};
use oxc_ast::ast::Statement;
use oxc_ast::AstBuilder;

pub const BUILDER_IO_QWIK: &str = "@builder.io/qwik";
pub const COMPONENT_SUFFIX: &str = "$";
pub const COMPONENT: &str = "component";
pub const COMPONENT_QRL: &str = "componentQrl";
pub const QRL: &str = "qrl";

pub const QRL_MARKER: &str = "$";
pub const QRL_COMPONENT_MARKER: &str = "component$";
pub const QRL_MARKER_IMPORTS: [&str; 2] = [QRL_MARKER, QRL_COMPONENT_MARKER];
pub const PURE_ANNOTATION: &str = "/*#__PURE__*/";
pub const PURE_ANNOTATION_LENGTH: u32 = PURE_ANNOTATION.len() as u32;


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommonImport {
    BuilderIoQwik(Vec<String>)
}


impl CommonImport {
    pub fn qrl() -> CommonImport {
        CommonImport::BuilderIoQwik(vec![QRL.to_string()])
    }
    pub fn component_qrl() -> CommonImport {
        CommonImport::BuilderIoQwik(vec![COMPONENT_QRL.to_string()])
    }
    
}

impl<'a> FromIn<'a, CommonImport> for Statement<'a> {
    fn from_in(value: CommonImport, allocator: &'a Allocator) -> Self {
        let ast_builder = AstBuilder::new(allocator);
        match value {
            CommonImport::BuilderIoQwik(names) => ast_builder.qwik_import(names, BUILDER_IO_QWIK),
        }
    }
}

impl<'a> FromIn<'a, &CommonImport> for Statement<'a> {
    fn from_in(value: &CommonImport, allocator: &'a Allocator) -> Self {
        let ast_builder = AstBuilder::new(allocator);
        match value {
            CommonImport::BuilderIoQwik(names) => {
                let names = names.clone();
                ast_builder.qwik_import(names, BUILDER_IO_QWIK)
            },
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommonExport {
    BuilderIoQwik(String),
}

impl CommonExport {
    pub fn handle_watch() -> CommonExport {
        CommonExport::BuilderIoQwik("_hW".into())
    }
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





pub fn normalize_test_output<T: AsRef<str>>(input: T) -> String {
    input
        .as_ref()
        .trim()
        .replace("\t", "    ")
        .replace("()=>{", "() => {")
        .replace("()=>i", "() => i")
        .replace("/*#__PURE__*/ ", "") // TODO: Remove this after tree shaking is implemented/
        
}

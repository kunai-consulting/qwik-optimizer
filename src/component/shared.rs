use crate::ext::AstBuilderExt;
use oxc_allocator::{Allocator, FromIn, IntoIn};
use oxc_ast::ast::{Statement, VariableDeclarator};
use oxc_ast::AstBuilder;
use std::convert::Into;
use std::path::PathBuf;

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
    BuilderIoQwik(Vec<String>),
    Import(Import)
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
            CommonImport::Import(import) =>import.into_statement(allocator) 
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
            }
            CommonImport::Import(import) => import.into_in(allocator)
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Import {
    names: Vec<String>,
    source: PathBuf,
}
impl Import {
    pub fn new<T: Into<PathBuf>>(names: Vec<String>, source: T) -> Self {
        Self {
            names,
            source: source.into(),
        }
    }

    pub fn into_statement<'a>(&self, allocator: &'a Allocator) -> Statement<'a> {
        let ast_builder = AstBuilder::new(allocator);
        ast_builder.qwik_import(self.names.clone(), self.source.to_string_lossy())
    }
}

impl<'a> FromIn<'a, &Import> for Statement<'a> {
    fn from_in(value: &Import, allocator: &'a Allocator) -> Self {
        value.into_statement(allocator)
    }
}

impl<'a> FromIn<'a, Import> for Statement<'a> {
    fn from_in(value: Import, allocator: &'a Allocator) -> Self {
        value.into_statement(allocator)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Reference {
    Variable(String),
}

impl Reference {
    
    pub fn name(&self) -> String {
        match self {
            Reference::Variable(name) => name.clone(),
        }
    }
    pub fn into_import(&self, source: PathBuf) -> Import {
        match self {
            Reference::Variable(name) => Import::new(vec![name.clone()], source),
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

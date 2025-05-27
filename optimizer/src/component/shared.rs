use crate::ext::AstBuilderExt;
use crate::import_clean_up::ImportCleanUp;
use oxc_allocator::{Allocator, FromIn};
use oxc_ast::ast::{ImportDeclarationSpecifier, ImportOrExportKind, Statement};
use oxc_ast::AstBuilder;
use oxc_span::{Atom, SPAN};
use serde::{Deserialize, Serialize};
use std::convert::Into;
use std::path::PathBuf;

pub const QWIK_CORE_SOURCE: &str = "@qwik.dev/core";
pub const MARKER_SUFFIX: &str = "$";
pub const QRL: &str = "qrl";
pub const QRL_SUFFIX: &str = "Qrl";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ImportId {
    Named(String),
    NamedWithAlias(String, String),
    Default(String),
    Namespace(String),
}

impl From<&str> for ImportId {
    fn from(value: &str) -> Self {
        ImportId::Named(value.to_string())
    }
}

fn replace_marker_with_qrl(name: Atom<'_>) -> String {
    let name = name.to_string();
    if let Some(qrl_call) = name.strip_suffix(MARKER_SUFFIX) {
        format!("{}{}", qrl_call, QRL_SUFFIX)
    } else {
        name
    }
}

impl From<&ImportDeclarationSpecifier<'_>> for ImportId {
    fn from(value: &ImportDeclarationSpecifier<'_>) -> Self {
        match value {
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                let imported = replace_marker_with_qrl(specifier.imported.name());
                let local_name = replace_marker_with_qrl(specifier.local.name);

                if imported == local_name {
                    ImportId::Named(imported)
                } else {
                    ImportId::NamedWithAlias(imported, local_name)
                }
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                let local_name = specifier.local.name.to_string();
                ImportId::Default(local_name)
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                let local_name = specifier.local.name.to_string();
                ImportId::Namespace(local_name)
            }
        }
    }
}

impl<'a> FromIn<'a, ImportId> for ImportDeclarationSpecifier<'a> {
    fn from_in(value: ImportId, allocator: &'a Allocator) -> Self {
        let ast = AstBuilder::new(allocator);
        match value {
            ImportId::Named(name) => {
                let imported = ast.module_export_name_identifier_name(SPAN, &name);
                let local_name = ast.binding_identifier(SPAN, &name);
                ast.import_declaration_specifier_import_specifier(
                    SPAN,
                    imported,
                    local_name,
                    ImportOrExportKind::Value,
                )
            }

            ImportId::NamedWithAlias(name, local_name) => {
                let imported = ast.module_export_name_identifier_name(SPAN, &name);
                let local_name = ast.binding_identifier(SPAN, &local_name);
                ast.import_declaration_specifier_import_specifier(
                    SPAN,
                    imported,
                    local_name,
                    ImportOrExportKind::Value,
                )
            }
            ImportId::Namespace(local_name) => {
                let local_name = ast.binding_identifier(SPAN, &local_name);
                ast.import_declaration_specifier_import_namespace_specifier(SPAN, local_name)
            }
            ImportId::Default(name) => {
                let local_name = ast.binding_identifier(SPAN, &name);
                ast.import_declaration_specifier_import_default_specifier(SPAN, local_name)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Import {
    names: Vec<ImportId>,
    source: PathBuf,
}

impl Import {
    pub fn new<T: AsRef<str>>(names: Vec<ImportId>, source: T) -> Self {
        let source = ImportCleanUp::rename_qwik_imports(source);
        Self {
            names,
            source: source.into(),
        }
    }

    pub fn into_statement<'a>(&self, allocator: &'a Allocator) -> Statement<'a> {
        let ast_builder = AstBuilder::new(allocator);
        ast_builder.create_import_statement(self.names.clone(), self.source.to_string_lossy())
    }

    pub fn from_import_declaration_specifier<T: AsRef<str>>(
        import: &ImportDeclarationSpecifier<'_>,
        source: T,
    ) -> Self {
        let names = vec![import.into()];
        Self::new(names, source)
    }

    pub fn qrl() -> Self {
        let names = vec![QRL.into()];
        Self::new(names, QWIK_CORE_SOURCE)
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

/// Renamed from `EmitMode` in V 1.0.
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Target {
    #[serde(alias = "prod", alias = "PROD")]
    Prod,
    #[serde(alias = "lib", alias = "LIB")]
    Lib,
    #[serde(alias = "dev", alias = "DEV")]
    Dev,
    #[serde(alias = "test", alias = "TEST")]
    Test,
}

use crate::component::{Import, QWIK_CORE_SOURCE};
use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclaration, ImportOrExportKind, Program, Statement};
use oxc_semantic::{SemanticBuilder, SemanticBuilderReturn};
use oxc_traverse::{traverse_mut, Traverse, TraverseCtx};
use std::collections::BTreeSet;

/// This struct is used to clean up unused imports in the AST.
pub(crate) struct ImportCleanUp;

impl ImportCleanUp {
    pub fn new() -> Self {
        ImportCleanUp
    }

    pub fn clean_up<'a>(program: &mut Program<'a>, allocator: &'a Allocator) {
        let SemanticBuilderReturn {
            semantic,
            errors: semantic_errors,
        } = SemanticBuilder::new()
            .with_check_syntax_error(true) // Enable extra syntax error checking
            .with_build_jsdoc(true) // Enable JSDoc parsing
            .with_cfg(true) // Build a Control Flow Graph
            .build(program);

        let scoping = semantic.into_scoping();

        let transform = &mut ImportCleanUp::new();

        traverse_mut(transform, allocator, program, scoping);
    }

    /// This function renames the Qwik imports to the new qwik.dev imports.
    ///
    /// The following import sources are renamed:
    /// - `@builder.io/qwik-city/...` -> `@qwik.dev/router/...`
    /// - `@builder.io/qwik-react/...` -> `@qwik.dev/react/...`
    /// - `@builder.io/qwik/...` -> `@qwik.dev/core/...`
    ///
    /// Otherwise, it returns the original import source string.
    pub fn rename_qwik_imports<T: AsRef<str>>(source: T) -> String {
        let source = source.as_ref();
        const BUILDER_QWIK_CITY: &str = "@builder.io/qwik-city";
        const BUILDER_QWIK_REACT_SOURCE: &str = "@builder.io/qwik-react";
        const BUILDER_QWIK_SOURCE: &str = "@builder.io/qwik";
        const QWIK_ROUTER_SOURCE: &str = "@qwik.dev/router";
        const QWIK_REACT_SOURCE: &str = "@qwik.dev/react";

        if let Some(base) = source.strip_prefix(BUILDER_QWIK_CITY) {
            format!("{}{}", QWIK_ROUTER_SOURCE, base)
        } else if let Some(base) = source.strip_prefix(BUILDER_QWIK_REACT_SOURCE) {
            format!("{}{}", QWIK_REACT_SOURCE, base)
        } else if let Some(base) = source.strip_prefix(BUILDER_QWIK_SOURCE) {
            format!("{}{}", QWIK_CORE_SOURCE, base)
        } else {
            source.into()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Key(String);

impl From<&ImportDeclaration<'_>> for Key {
    fn from(import: &ImportDeclaration) -> Self {
        let mut key = String::new();
        if let Some(specifiers) = &import.specifiers {
            for specifier in specifiers {
                let local = specifier.local();
                let local_name = local.name;
                let name = specifier.name();
                key.push_str(&name.to_string());
                key.push('|');
                key.push_str(&local_name.to_string());
                key.push('|');
            }
        }

        key.push_str(import.source.value.as_ref());
        key.push('|');
        let kind = match import.import_kind {
            ImportOrExportKind::Value => "0",
            ImportOrExportKind::Type => "1",
        };
        key.push_str(kind);

        Key(key)
    }
}

impl<'a> Traverse<'a> for ImportCleanUp {
    fn enter_statements(
        &mut self,
        node: &mut oxc_allocator::Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let mut imports: BTreeSet<Import> = BTreeSet::new();

        node.retain_mut(|node| match node {
            Statement::ImportDeclaration(import) => {
                let source = import.source.clone();
                let specifiers = &mut import.specifiers;
                if let Some(specifiers) = specifiers {
                    for specifier in specifiers {
                        if ctx.scoping().symbol_is_used(specifier.local().symbol_id()) {
                            imports.insert(Import::from_import_declaration_specifier(
                                specifier, &source,
                            ));
                        }
                    }
                }
                false
            }
            _ => true,
        });

        imports.iter().for_each(|import| {
            node.insert(0, import.into_statement(ctx.ast.allocator));
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_allocator::Allocator;
    use oxc_codegen::Codegen;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    #[test]
    fn test_import_clean_up() {
        let allocator = Allocator::new();
        let source = r#"
            import { a } from '@builder.io/qwik-city';
            import { b } from '@builder.io/qwik-react';
            import { c } from '@builder.io/qwik';
            import { d } from '@qwik.dev/router';
            
            b.foo();
        "#;

        let parse_return = Parser::new(&allocator, source, SourceType::tsx()).parse();
        let mut program = parse_return.program;
        ImportCleanUp::clean_up(&mut program, &allocator);

        let codegen = Codegen::default();
        let raw = codegen.build(&program).code;
        let lines: Vec<&str> = raw.lines().collect();
        assert_eq!(program.body.len(), 2);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], r#"import { b } from "@qwik.dev/react";"#);
        assert_eq!(lines[1], r#"b.foo();"#);
    }

    #[test]
    fn test_rename_qwik_imports() {
        let source = "@builder.io/qwik-city/foo";
        let renamed = ImportCleanUp::rename_qwik_imports(source);
        assert_eq!(renamed, "@qwik.dev/router/foo");

        let source = "@builder.io/qwik-react/bar";
        let renamed = ImportCleanUp::rename_qwik_imports(source);
        assert_eq!(renamed, "@qwik.dev/react/bar");

        let source = "@builder.io/qwik/baz";
        let renamed = ImportCleanUp::rename_qwik_imports(source);
        assert_eq!(renamed, "@qwik.dev/core/baz");
    }
}

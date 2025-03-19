use oxc_allocator::{Allocator};
use oxc_ast::ast::{ImportDeclaration, ImportOrExportKind, Program, Statement};
use oxc_semantic::{SemanticBuilder, SemanticBuilderReturn};
use oxc_traverse::{traverse_mut, Traverse, TraverseCtx};
use std::collections::{HashSet};

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

        let (symbols, scopes) = semantic.into_symbol_table_and_scope_tree();

        let transform = &mut ImportCleanUp::new();

        traverse_mut(transform, allocator, program, symbols, scopes);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Key(String);

impl From<&ImportDeclaration<'_>> for Key {
    fn from(import: &ImportDeclaration) -> Self {
        let mut key = String::new();
        for specifiers in &import.specifiers {
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

        key.push_str(&import.source.value.to_string());
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
        let mut remove: Vec<usize> = Vec::new();
        let mut keys: HashSet<Key> = HashSet::new();

        for (idx, statement) in node.iter_mut().enumerate() {
            if let Statement::ImportDeclaration(import) = statement {
                let source_value = import.source.value;
                let specifiers = &mut import.specifiers;
                if let Some(specifiers) = specifiers {
                    specifiers.retain(|s| {
                        let local = s.local();
                        ctx.symbols().symbol_is_used(local.symbol_id())
                    });

                    // If all specifiers are removed, we will want to eventually remove that statement completely.
                    if specifiers.is_empty() {
                        remove.insert(0, idx);
                    }
                    
                }

                // Duplicate check
                let key: Key = Key::from(import.as_ref());
                if keys.contains(&key) {
                    remove.insert(0, idx);
                } else {
                    keys.insert(key);
                }
            }
        }

        for idx in remove.iter() {
            node.remove(*idx);
        }
    }
}

use crate::component::ImportId;
use oxc_allocator::{Box as OxcBox, IntoIn, Vec as OxcVec};
use oxc_ast::ast::{ImportOrExportKind, Statement, WithClause};
use oxc_ast::AstBuilder;
use oxc_span::{Atom, SPAN};

pub trait AstBuilderExt<'a> {
    fn create_import_statement<U: AsRef<str>>(
        self,
        names: Vec<ImportId>,
        source: U,
    ) -> Statement<'a>;
    fn create_export_statement(self, name: &str, source: &str) -> Statement<'a>;

    fn create_simple_import(self, name: &str) -> Statement<'a>;
}

impl<'a> AstBuilderExt<'a> for AstBuilder<'a> {
    fn create_import_statement<U: AsRef<str>>(
        self,
        import_ids: Vec<ImportId>,
        source: U,
    ) -> Statement<'a> {
        let mut import_decl_specifier = OxcVec::with_capacity_in(import_ids.len(), self.allocator);
        for import_id in import_ids {
            import_decl_specifier.push(import_id.into_in(self.allocator));
        }

        let raw = self.atom(&format!("'{}'", source.as_ref()));
        let source_location = self.string_literal(SPAN, self.atom(source.as_ref()), Some(raw));
        let import_decl = self.alloc_import_declaration(
            SPAN,
            Some(import_decl_specifier),
            source_location,
            None,
            None::<OxcBox<'a, WithClause<'a>>>,
            ImportOrExportKind::Value,
        );

        Statement::ImportDeclaration(import_decl)
    }

    fn create_export_statement(self, name: &str, source: &str) -> Statement<'a> {
        let exported = self.module_export_name_identifier_name(SPAN, self.atom(name));
        let local_name = self.module_export_name_identifier_name(SPAN, self.atom(name));
        let export_specifier =
            self.export_specifier(SPAN, exported, local_name, ImportOrExportKind::Value);
        let mut export_specifiers = OxcVec::new_in(self.allocator);
        export_specifiers.push(export_specifier);
        let raw = self.atom(&format!(r#""{}""#, source));
        let source_location = self.string_literal(SPAN, self.atom(source), Some(raw));
        let export_decl = self.alloc_export_named_declaration(
            SPAN,
            None,
            export_specifiers,
            Some(source_location),
            ImportOrExportKind::Value,
            None::<OxcBox<'a, WithClause<'a>>>,
        );

        Statement::ExportNamedDeclaration(export_decl)
    }

    fn create_simple_import(self, name: &str) -> Statement<'a> {
        let raw: Atom = self.atom(&format!(r#""{}""#, name));
        let source = self.expression_string_literal(SPAN, self.atom(name), Some(raw));
        let import_expression = self.expression_import(SPAN, source, None, None);
        self.statement_expression(SPAN, import_expression)
    }
}

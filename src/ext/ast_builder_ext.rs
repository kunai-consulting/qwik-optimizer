use oxc_allocator::{Box as OxcBox, IntoIn, Vec as OxcVec};
use oxc_ast::ast::{ImportDeclarationSpecifier, ImportOrExportKind, Statement, WithClause};
use oxc_ast::AstBuilder;
use oxc_span::{Atom, SPAN};

pub trait AstBuilderExt<'a> {
    fn create_import_statement<T: AsRef<str>, U: AsRef<str>>(
        self,
        names: Vec<T>,
        source: U,
    ) -> Statement<'a>;
    fn create_export_statement(self, name: &str, source: &str) -> Statement<'a>;

    fn create_simple_import(self, name: &str) -> Statement<'a>;
}

impl<'a> AstBuilderExt<'a> for AstBuilder<'a> {
    fn create_import_statement<T: AsRef<str>, U: AsRef<str>>(
        self,
        names: Vec<T>,
        source: U,
    ) -> Statement<'a> {
        let mut import_decl_specifier = OxcVec::with_capacity_in(names.len(), self.allocator);
        for name in names {
            let name: Atom<'a> = name.as_ref().into_in(self.allocator);
            let imported = self.module_export_name_identifier_name(SPAN, &name);
            let local_name = self.binding_identifier(SPAN, &name);
            let import_specifier =
                self.import_specifier(SPAN, imported, local_name, ImportOrExportKind::Value);

            import_decl_specifier.push(ImportDeclarationSpecifier::ImportSpecifier(
                OxcBox::new_in(import_specifier, self.allocator),
            ));
        }

        let raw = format!("'{}'", source.as_ref());
        let raw: Atom = raw.into_in(self.allocator);
        let source_location = self.string_literal(SPAN, source.as_ref(), Some(raw));
        let import_decl = self.import_declaration(
            SPAN,
            Some(import_decl_specifier),
            source_location,
            None,
            None::<OxcBox<'a, WithClause<'a>>>,
            ImportOrExportKind::Value,
        );

        Statement::ImportDeclaration(OxcBox::new_in(import_decl, self.allocator))
    }

    fn create_export_statement(self, name: &str, source: &str) -> Statement<'a> {
        let exported = self.module_export_name_identifier_name(SPAN, name);
        let local_name = self.module_export_name_identifier_name(SPAN, name);
        let export_specifier =
            self.export_specifier(SPAN, exported, local_name, ImportOrExportKind::Value);
        let mut export_specifiers = OxcVec::new_in(self.allocator);
        export_specifiers.push(export_specifier);
        let raw = format!(r#""{}""#, source);
        let raw: Atom = raw.into_in(self.allocator);
        let source_location = self.string_literal(SPAN, source, Some(raw));
        let export_decl = self.export_named_declaration(
            SPAN,
            None,
            export_specifiers,
            Some(source_location),
            ImportOrExportKind::Value,
            None::<OxcBox<'a, WithClause<'a>>>,
        );

        Statement::ExportNamedDeclaration(OxcBox::new_in(export_decl, self.allocator))
    }

    fn create_simple_import(self, name: &str) -> Statement<'a> {
        let raw: Atom = format!(r#""{}""#, name).into_in(self.allocator);
        let source = self.expression_string_literal(SPAN, name, Some(raw));
        let import_expression =
            self.expression_import(SPAN, source, OxcVec::new_in(self.allocator), None);
        self.statement_expression(SPAN, import_expression)
    }
}

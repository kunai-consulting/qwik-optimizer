use oxc::allocator::{Box as OxcBox, IntoIn, Vec as OxcVec};
use oxc::ast::ast::{ ImportDeclarationSpecifier, ImportOrExportKind, Statement, WithClause};
use oxc::ast::AstBuilder;
use oxc::span::{Atom, SPAN};

pub trait AstBuilderExt<'a> {
    fn qwik_import(self, name: &str, source: &str) -> Statement<'a>;
    fn qwik_export(self, name: &str, source: &str) -> Statement<'a>;
}

impl<'a> AstBuilderExt<'a> for AstBuilder<'a> {
    fn qwik_import(self, name: &str, source: &str) -> Statement<'a> {
        let imported = self.module_export_name_identifier_name(SPAN, name);
        let local_name = self.binding_identifier(SPAN, name);
        let import_specifier =
            self.import_specifier(SPAN, imported, local_name, ImportOrExportKind::Value);
        let mut import_decl_specifier = OxcVec::new_in(self.allocator);
        import_decl_specifier.push(ImportDeclarationSpecifier::ImportSpecifier(OxcBox::new_in(
            import_specifier,
            self.allocator,
        )));
        let raw = format!("'{}'", source);
        let raw: Atom = raw.into_in(self.allocator);
        let source_location = self.string_literal(SPAN, source, Some(raw));
        let import_decl = self.import_declaration(
            SPAN,
            Some(import_decl_specifier),
            source_location,
            None,
            None::<OxcBox<'a, WithClause<'a>>>,
            ImportOrExportKind::Type,
        );

        Statement::ImportDeclaration(OxcBox::new_in(import_decl, self.allocator))
    }
    
    fn qwik_export(self, name: &str, source: &str) -> Statement<'a> {
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
}

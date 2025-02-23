use oxc_allocator::{Allocator, Box as OxcBox, CloneIn, IntoIn, Vec as OxcVec};
use oxc_ast::ast::*;
use oxc_ast::*;
use oxc_codegen::Codegen;
use oxc_span::{SourceType, SPAN};
use crate::component::*;
use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QwikComponent {
    pub id: Id,
    pub source_type: SourceType,
    pub code: String,
    pub qurl: Qrl,
}

impl QwikComponent {
    pub fn new(
        source_info: &SourceInfo,
        segments: &Vec<String>,
        function: &ArrowFunctionExpression<'_>,
        target: &Target,
        scope: &Option<String>,
    ) -> Result<QwikComponent> {
        let id = Id::new(source_info, segments, target, scope);
        let source_type = source_info.try_into()?;
        let qurl = Qrl::new(
            source_info.rel_path.to_str().unwrap_or_default(),
            &id.symbol_name,
        );
        let code = Self::gen(&id, function, &source_type, &Allocator::default());
        Ok(QwikComponent {
            id,
            source_type,
            code,
            qurl,
        })
    }

    fn std_import(ast_builder: &AstBuilder) {
        let imported = ast_builder.module_export_name_identifier_name(SPAN, "qrl");
        let local_name = ast_builder.binding_identifier(SPAN, "qrl");
        let import_specifier =
            ast_builder.import_specifier(SPAN, imported, local_name, ImportOrExportKind::Value);
    }

    fn gen(
        id: &Id,
        function: &ArrowFunctionExpression,
        source_type: &SourceType,
        allocator: &Allocator,
    ) -> String {
        let name = &id.symbol_name;

        let ast_builder = AstBuilder::new(allocator);

        Self::std_import(&ast_builder);

        let id = OxcBox::new_in(ast_builder.binding_identifier(SPAN, name), allocator);
        let bind_pat = ast_builder.binding_pattern(
            BindingPatternKind::BindingIdentifier(id),
            None::<OxcBox<'_, TSTypeAnnotation<'_>>>,
            false,
        );
        let mut var_declarator = OxcVec::new_in(allocator);

        let boxed = OxcBox::new_in(function.clone_in(allocator), allocator);
        let expr = Expression::ArrowFunctionExpression(boxed);
        var_declarator.push(ast_builder.variable_declarator(
            SPAN,
            VariableDeclarationKind::Const,
            bind_pat,
            Some(expr),
            false,
        ));

        let decl = ast_builder.variable_declaration(
            SPAN,
            VariableDeclarationKind::Const,
            var_declarator,
            false,
        );
        let decl = OxcBox::new_in(decl, allocator);
        let decl = Declaration::VariableDeclaration(decl);
        let export = ast_builder.export_named_declaration(
            SPAN,
            Some(decl),
            OxcVec::new_in(allocator),
            None,
            ImportOrExportKind::Value,
            None::<OxcBox<WithClause>>,
        );
        let export = Statement::ExportNamedDeclaration(OxcBox::new_in(export, allocator));

        let mut body = OxcVec::new_in(allocator);
        body.push(export);

        let hw_export = CommonExport::BuilderIoQwik("_hW".into()).into_in(allocator);
        body.push(hw_export);

        let ast_builder = AstBuilder::new(allocator);

        let new_pgm = ast_builder.program(
            SPAN,
            *source_type,
            "",
            OxcVec::new_in(allocator),
            None,
            OxcVec::new_in(allocator),
            body,
        );

        let codegen = Codegen::new();
        codegen.build(&new_pgm).code
    }
}

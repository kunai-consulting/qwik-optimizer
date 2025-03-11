use oxc_allocator::{Allocator, Box as OxcBox, CloneIn, IntoIn, Vec as OxcVec};
use oxc_ast::ast::*;
use oxc_ast::*;
use oxc_codegen::Codegen;
use oxc_span::{SourceType, SPAN};
use crate::component::*;
use crate::prelude::*;
use crate::component::Language;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct QwikComponent {
    pub id: Id,
    pub language: Language,
    pub code: String,
    pub qurl: Qrl,
}


impl QwikComponent {
    pub fn new(
        source_info: &SourceInfo,
        segments: &Vec<String>,
        function: &ArrowFunctionExpression<'_>,
        imports: Vec<CommonImport>,
        target: &Target,
        scope: &Option<String>,
    ) -> Result<QwikComponent> {
        let language = source_info.language.clone();
        let id = Id::new(source_info, segments, target, scope);
        let qurl = Qrl::new(
            &id.local_file_name,
            &id.symbol_name,
        );
        
        let source_type: SourceType = language.into();
        
        let code = Self::gen(&id, function, imports, &source_type, &Allocator::default());
        Ok(QwikComponent {
            id,
            language: source_info.language.clone(),
            code,
            qurl,
        })
    }

    fn gen(
        id: &Id,
        function: &ArrowFunctionExpression,
        imports: Vec<CommonImport>,
        source_type: &SourceType,
        allocator: &Allocator,
    ) -> String {
        let name = &id.symbol_name;

        let ast_builder = AstBuilder::new(allocator);

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

        let imports = imports.iter().map ( |import|  {
            let statement: Statement = import.clone().into_in(allocator) ;
            statement
        });

        let mut body = ast_builder.vec_from_iter(imports);

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

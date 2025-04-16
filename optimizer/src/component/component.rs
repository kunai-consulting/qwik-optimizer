use crate::component::Language;
use crate::component::*;
use crate::segment::Segment;
use oxc_allocator::{Allocator, Box as OxcBox, CloneIn, IntoIn, Vec as OxcVec};
use oxc_ast::ast::*;
use oxc_ast::*;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_minifier::*;
use oxc_span::{SourceType, SPAN};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct QrlComponent {
    pub id: Id,
    pub language: Language,
    pub code: String,
    pub qrl: Qrl,
}

impl QrlComponent {
    pub(crate) fn new(
        source_info: &SourceInfo,
        id: Id,
        exported_expression: Expression<'_>,
        imports: Vec<Import>,
        minify: bool,
        qrl_type: QrlType,
    ) -> QrlComponent {
        let language = source_info.language.clone();
        let qrl = Qrl::new(&id.local_file_name, &id.symbol_name, qrl_type);

        let source_type: SourceType = language.into();

        let code = Self::gen(
            &id,
            exported_expression,
            imports,
            minify,
            &source_type,
            &Allocator::default(),
        );
        QrlComponent {
            id,
            language: source_info.language.clone(),
            code,
            qrl,
        }
    }

    fn gen(
        id: &Id,
        exported_expression: Expression<'_>,
        imports: Vec<Import>,
        minify: bool,
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

        var_declarator.push(ast_builder.variable_declarator(
            SPAN,
            VariableDeclarationKind::Const,
            bind_pat,
            Some(exported_expression),
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

        let imports = imports.iter().map(|import| {
            let statement: Statement = import.clone().into_in(allocator);
            statement
        });

        let mut body = ast_builder.vec_from_iter(imports);

        body.push(export);

        let ast_builder = AstBuilder::new(allocator);

        let mut new_pgm = ast_builder.program(
            SPAN,
            *source_type,
            "",
            OxcVec::new_in(allocator),
            None,
            OxcVec::new_in(allocator),
            body,
        );

        let codegen = Codegen::new();
        let codegen_options = CodegenOptions {
            annotation_comments: true,
            minify: minify,
            ..Default::default()
        };

        if minify {
            let ops = MinifierOptions {
                compress: Some(CompressOptions::default()),
                mangle: None,
                // mangle: Some(MangleOptions::default()),
            };
            let minifier = Minifier::new(ops);
            let ret = minifier.build(allocator, &mut new_pgm);
            let sym_tab = ret.symbol_table;

            codegen
                .with_options(codegen_options)
                .with_symbol_table(sym_tab)
                .build(&new_pgm)
                .code
        } else {
            codegen.with_options(codegen_options).build(&new_pgm).code
        }
    }

    /// Create a QrlComponent from an `Expression`.
    pub(crate) fn from_expression(
        expr: Expression<'_>,
        imports: Vec<Import>,
        segments: &Vec<Segment>,
        target: &Target,
        scope: &Option<String>,
        source_info: &SourceInfo,
        minify: bool,
    ) -> QrlComponent {
        let qrl_type: QrlType = segments
            .last()
            .iter()
            .flat_map(|segment| segment.qrl_type())
            .last()
            .unwrap(); // TODO Clean this up.

        let id = Id::new(source_info, segments, target, scope);

        QrlComponent::new(source_info, id, expr, imports, minify, qrl_type)
    }

    pub(crate) fn from_call_expression_argument(
        arg: &Argument,
        imports: Vec<Import>,
        segments: &Vec<Segment>,
        target: &Target,
        scope: &Option<String>,
        source_info: &SourceInfo,
        minify: bool,
        allocator: &Allocator,
    ) -> QrlComponent {
        let init = arg.clone_in(allocator).into_expression();
        Self::from_expression(init, imports, segments, target, scope, source_info, minify)
    }
}

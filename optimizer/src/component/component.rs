use crate::component::*;
use crate::segment::Segment;
use crate::transform::TransformOptions;
use crate::{component::Language, import_clean_up::ImportCleanUp};
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
        options: &TransformOptions,
        source_info: &SourceInfo,
        id: Id,
        exported_expression: Expression<'_>,
        imports: Vec<Import>,
        qrl_type: QrlType,
    ) -> QrlComponent {
        let language = source_info.language.clone();
        let qrl = Qrl::new(&id.local_file_name, &id.symbol_name, qrl_type);

        let source_type: SourceType = language.into();

        let code = Self::gen(
            options,
            &id,
            exported_expression,
            imports,
            &source_type,
            source_info,
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
        options: &TransformOptions,
        id: &Id,
        exported_expression: Expression<'_>,
        imports: Vec<Import>,
        source_type: &SourceType,
        _source_info: &SourceInfo,
        allocator: &Allocator,
    ) -> String {
        let name = &id.symbol_name;

        let ast_builder = AstBuilder::new(allocator);

        let bind_pat = ast_builder.binding_pattern_binding_identifier(SPAN, name.as_str());
        let mut var_declarator = OxcVec::new_in(allocator);

        var_declarator.push(ast_builder.variable_declarator(
            SPAN,
            VariableDeclarationKind::Const,
            bind_pat,
            None::<OxcBox<'_, TSTypeAnnotation<'_>>>,
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

        ImportCleanUp::clean_up(&mut new_pgm, allocator);

        let codegen = Codegen::new();
        let codegen_options = CodegenOptions {
            minify: options.minify,
            ..Default::default()
        };

        if options.minify {
            let ops = MinifierOptions {
                compress: Some(CompressOptions::default()),
                mangle: None,
                // mangle: Some(MangleOptions::default()),
            };
            let minifier = Minifier::new(ops);
            let ret = minifier.minify(allocator, &mut new_pgm);
            let scoping = ret.scoping;

            codegen
                .with_options(codegen_options)
                .with_scoping(scoping)
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
        scope: &Option<String>,
        options: &TransformOptions,
        source_info: &SourceInfo,
    ) -> QrlComponent {
        let qrl_type: QrlType = segments
            .last()
            .iter()
            .flat_map(|segment| segment.qrl_type())
            .last()
            .unwrap(); // TODO Clean this up.

        let id = Id::new(source_info, segments, &options.target, scope);

        QrlComponent::new(options, source_info, id, expr, imports, qrl_type)
    }

    pub(crate) fn from_call_expression_argument(
        arg: &Argument,
        imports: Vec<Import>,
        segments: &Vec<Segment>,
        scope: &Option<String>,
        options: &TransformOptions,
        source_info: &SourceInfo,
        allocator: &Allocator,
    ) -> QrlComponent {
        let init = arg.clone_in(allocator).into_expression();
        Self::from_expression(init, imports, segments, scope, options, source_info)
    }
}

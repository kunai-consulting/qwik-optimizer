use crate::code_move::transform_function_expr;
use crate::collector::Id as CollectorId;
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

/// A QRL component represents a lazy-loadable segment of code.
///
/// QrlComponent combines the component identification, generated code,
/// QRL reference, and segment metadata needed for code generation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct QrlComponent {
    /// Component identifier (symbol name, hash, etc.)
    pub id: Id,
    /// Source language (JavaScript, TypeScript, etc.)
    pub language: Language,
    /// Generated code for this segment
    pub code: String,
    /// QRL reference (path and symbol)
    pub qrl: Qrl,
    /// Segment metadata (captures, context kind, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment_data: Option<SegmentData>,
}

impl QrlComponent {
    /// Creates a new QrlComponent with optional segment data.
    pub(crate) fn new(
        options: &TransformOptions,
        source_info: &SourceInfo,
        id: Id,
        exported_expression: Expression<'_>,
        imports: Vec<Import>,
        qrl_type: QrlType,
        segment_data: Option<SegmentData>,
    ) -> QrlComponent {
        let language = source_info.language.clone();

        // Extract scoped_idents for both Qrl and transform_function_expr
        let scoped_idents: Vec<CollectorId> = segment_data
            .as_ref()
            .map(|d| d.scoped_idents.clone())
            .unwrap_or_default();

        // Create Qrl with scoped_idents for capture array generation
        let qrl = Qrl::new(&id.local_file_name, &id.symbol_name, qrl_type, scoped_idents.clone());

        let source_type: SourceType = language.into();

        let code = Self::gen(
            options,
            &id,
            exported_expression,
            imports,
            &source_type,
            source_info,
            &scoped_idents,
            &Allocator::default(),
        );
        QrlComponent {
            id,
            language: source_info.language.clone(),
            code,
            qrl,
            segment_data,
        }
    }

    fn gen(
        options: &TransformOptions,
        id: &Id,
        exported_expression: Expression<'_>,
        imports: Vec<Import>,
        source_type: &SourceType,
        _source_info: &SourceInfo,
        scoped_idents: &[CollectorId],
        allocator: &Allocator,
    ) -> String {
        let name = &id.symbol_name;

        let ast_builder = AstBuilder::new(allocator);

        // Apply useLexicalScope injection if there are captured variables
        let transformed_expression = if !scoped_idents.is_empty() {
            transform_function_expr(exported_expression, scoped_idents, allocator)
        } else {
            exported_expression
        };

        let bind_pat = ast_builder.binding_pattern_binding_identifier(SPAN, name.as_str());
        let mut var_declarator = OxcVec::new_in(allocator);

        var_declarator.push(ast_builder.variable_declarator(
            SPAN,
            VariableDeclarationKind::Const,
            bind_pat,
            None::<OxcBox<'_, TSTypeAnnotation<'_>>>,
            Some(transformed_expression),
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
        segment_data: Option<SegmentData>,
    ) -> QrlComponent {
        let qrl_type: QrlType = segments
            .last()
            .iter()
            .flat_map(|segment| segment.qrl_type())
            .last()
            .unwrap(); // TODO Clean this up.

        let id = Id::new(source_info, segments, &options.target, scope);

        QrlComponent::new(options, source_info, id, expr, imports, qrl_type, segment_data)
    }

    /// Create a QrlComponent from a call expression argument.
    pub(crate) fn from_call_expression_argument(
        arg: &Argument,
        imports: Vec<Import>,
        segments: &Vec<Segment>,
        scope: &Option<String>,
        options: &TransformOptions,
        source_info: &SourceInfo,
        segment_data: Option<SegmentData>,
        allocator: &Allocator,
    ) -> QrlComponent {
        let init = arg.clone_in(allocator).into_expression();
        Self::from_expression(init, imports, segments, scope, options, source_info, segment_data)
    }

    // --- Segment data accessors ---

    /// Returns the captured variable identifiers (scoped_idents).
    ///
    /// These are variables from the enclosing scope that need to be
    /// captured via `useLexicalScope` injection.
    pub fn scoped_idents(&self) -> &[CollectorId] {
        self.segment_data
            .as_ref()
            .map(|d| d.scoped_idents.as_slice())
            .unwrap_or(&[])
    }

    /// Returns the local identifiers used in the segment.
    ///
    /// These are used for import generation in the segment file.
    pub fn local_idents(&self) -> &[CollectorId] {
        self.segment_data
            .as_ref()
            .map(|d| d.local_idents.as_slice())
            .unwrap_or(&[])
    }

    /// Returns the parent segment name if this is a nested QRL.
    ///
    /// Nested QRLs (QRLs defined inside other QRLs) need to track
    /// their parent segment for proper resolution.
    pub fn parent_segment(&self) -> Option<&str> {
        self.segment_data
            .as_ref()
            .and_then(|d| d.parent_segment.as_deref())
    }

    /// Returns true if this segment has captured variables.
    pub fn has_captures(&self) -> bool {
        self.segment_data
            .as_ref()
            .map(|d| d.has_captures())
            .unwrap_or(false)
    }

    /// Returns the segment's context kind if segment data is present.
    pub fn ctx_kind(&self) -> Option<SegmentKind> {
        self.segment_data.as_ref().map(|d| d.ctx_kind)
    }
}

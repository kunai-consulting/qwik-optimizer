use std::path::PathBuf;
use oxc_allocator::{Allocator, Box as OxcBox, Vec as OxcVec, IntoIn, FromIn};
use oxc_ast::ast::*;
use oxc_ast::AstBuilder;
use oxc_codegen::Codegen;
use oxc_semantic::ReferenceId;
use oxc_span::{Atom, SPAN};
use oxc_index::Idx;
use crate::ext::AstBuilderExt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Qrl {
    pub rel_path: PathBuf,
    pub display_name: String,
}

impl Qrl {
    pub fn new<T: Into<PathBuf>>(rel_path: T, display_name: &str) -> Self {
        Self {
            rel_path: rel_path.into(),
            display_name: display_name.into(),
        }
    }


    fn expression<'a>(self, allocator: &'a Allocator, ast_builder: AstBuilder<'a>) -> Expression<'a> {
        let rel_path = format!(
            "./{}",
            &self.rel_path.file_stem().unwrap().to_string_lossy()
        );
        let display_name = &self.display_name;

        let mut statements = OxcVec::with_capacity_in(1, allocator);
        statements.push(ast_builder.qwik_simple_import(rel_path.as_ref()));

        let function_body = ast_builder.function_body(SPAN, OxcVec::new_in(allocator), statements);
        let func_params = ast_builder.formal_parameters(
            SPAN,
            FormalParameterKind::ArrowFormalParameters,
            OxcVec::with_capacity_in(0, allocator),
            None::<OxcBox<BindingRestElement>>,
        );
        let arrow_function = ast_builder.arrow_function_expression(
            SPAN,
            true,
            false,
            None::<OxcBox<TSTypeParameterDeclaration>>,
            func_params,
            None::<OxcBox<TSTypeAnnotation>>,
            function_body,
        );

        let refid: ReferenceId = ReferenceId::from_usize(0);
        let qurl = OxcBox::new_in(
            ast_builder.identifier_reference_with_reference_id(SPAN, "qrl", refid),
            allocator,
        );
        let qurl = Expression::Identifier(qurl);

        let raw: Atom = format!(r#""{}""#, display_name).into_in(allocator);
        let display_name_arg = OxcBox::new_in(
            ast_builder.string_literal(SPAN, display_name, Some(raw)),
            allocator,
        );

        let mut args = OxcVec::with_capacity_in(2, allocator);
        args.push(Argument::ArrowFunctionExpression(OxcBox::new_in(
            arrow_function,
            allocator,
        )));
        args.push(Argument::StringLiteral(display_name_arg));

        let call_expr = ast_builder.call_expression(
            SPAN,
            qurl,
            None::<OxcBox<TSTypeParameterInstantiation>>,
            args,
            false,
        );

        Expression::CallExpression(OxcBox::new_in(call_expr, allocator))
    }
}

impl<'a> IntoIn<'a, Expression<'a>> for Qrl {
    fn into_in(self, allocator: &'a Allocator) -> Expression<'a> {
        let ast_builder = AstBuilder::new(allocator);
        self.expression(allocator, ast_builder)
    }
}

impl<'a> IntoIn<'a, Statement<'a>> for Qrl {
    fn into_in(self, allocator: &'a Allocator) -> Statement<'a> {
        let ast_builder = AstBuilder::new(allocator);
        ast_builder.statement_expression(SPAN, self.into_in(allocator))
    }
}


#[test]
fn test_qurl() {
    let allocator = Allocator::default();
    let ast_builder = AstBuilder::new(&allocator);
    let qurl = Qrl::new(
        "./test.tsx_renderHeader_zBbHWn4e8Cg.tsx",
        "renderHeader_zBbHWn4e8Cg",
    );
    let source_type = SourceType::from_path(&qurl.rel_path).unwrap();
    let statement = qurl.into_in(&allocator);
    let pgm = ast_builder.qwik_program(vec![statement], source_type);
    let codegen = Codegen::new();
    let script = codegen.build(&pgm).code;

    let expected = r#"qrl(() => import("./test.tsx_renderHeader_zBbHWn4e8Cg"), "renderHeader_zBbHWn4e8Cg");"#;
    assert_eq!(script.trim(), expected.trim())
}

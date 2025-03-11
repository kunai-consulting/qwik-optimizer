use crate::ext::AstBuilderExt;
use oxc_allocator::{Allocator, Box as OxcBox, FromIn, IntoIn, Vec as OxcVec};
use oxc_ast::ast::*;
use oxc_ast::AstBuilder;
use oxc_codegen::Codegen;
use oxc_index::Idx;
use oxc_semantic::ReferenceId;
use oxc_span::{Atom, SPAN};
use std::path::PathBuf;

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

    /// Creates a `qrl` identifier.
    /// 
    /// # Examples
    /// ```javascript
    ///  qrl
    /// ```    
    /// This identifier will eventually be used to construct a call expression e.g. a function call to `qrl()`.
    /// ```javascript
    /// qrl(() => import("./test.tsx_renderHeader_zBbHWn4e8Cg"), "renderHeader_zBbHWn4e8Cg");
    /// ```
    /// 
    fn as_identifier<'a>(ast_builder: &AstBuilder<'a>) -> oxc_allocator::Box<'a, IdentifierReference<'a>> {
        let refid: ReferenceId = ReferenceId::from_usize(0);
        let qurl = OxcBox::new_in(
            ast_builder.identifier_reference_with_reference_id(SPAN, "qrl", refid),
            ast_builder.allocator,
        );
        qurl
    }
   
    
    /// Creates an arrow function expression that lazily imports a named module
    ///
    /// # Examples
    /// ```javascript
    /// () => import("./test.tsx_renderHeader_zBbHWn4e8Cg")
    /// ```
    /// 
    /// This arrow function expression will eventually be used to construct a call expression e.g. a function call to `qrl()`.
    /// 
    /// ```javascript
    /// qrl(() => import("./test.tsx_renderHeader_zBbHWn4e8Cg"), "renderHeader_zBbHWn4e8Cg");
    /// ```
    /// 
    fn as_arrow_function<'a>(self, ast_builder: &AstBuilder<'a>) -> ArrowFunctionExpression<'a> {
        let rel_path = self.rel_path.to_string_lossy();

        // Function Body /////////
        let mut statements = ast_builder.vec_with_capacity(1);
        statements.push(ast_builder.qwik_simple_import(rel_path.as_ref()));
        let function_body = ast_builder.function_body(SPAN, ast_builder.vec(), statements);
        let func_params = ast_builder.formal_parameters(
            SPAN,
            FormalParameterKind::ArrowFormalParameters,
            OxcVec::with_capacity_in(0, ast_builder.allocator),
            None::<OxcBox<BindingRestElement>>,
        );

        //  Arrow Function Expression ////////
         ast_builder.arrow_function_expression(
            SPAN,
            true,
            false,
            None::<OxcBox<TSTypeParameterDeclaration>>,
            func_params,
            None::<OxcBox<TSTypeAnnotation>>,
            function_body,
        )
    }

    fn as_arguments<'a>(self, ast_builder: &AstBuilder<'a>) -> OxcVec<'a, Argument<'a>> {

        let allocator = ast_builder.allocator;
        let display_name = self.display_name.clone();

        // ARG: Display name string literal ////////
        let raw: Atom = format!(r#""{}""#, display_name).into_in(allocator);
        let display_name_arg = OxcBox::new_in(
            ast_builder.string_literal(SPAN, display_name, Some(raw)),
            allocator,
        );

        let mut args = ast_builder.vec_with_capacity(2);
        let arrow_function = self.as_arrow_function(ast_builder);
        args.push(Argument::ArrowFunctionExpression(OxcBox::new_in(
            arrow_function,
            allocator,
        )));
        args.push(Argument::StringLiteral(display_name_arg));

        args
    }
    
    fn as_call_expression<'a>(self, ast_builder: &AstBuilder<'a>) -> CallExpression<'a> {
        let qrl = Self::as_identifier(ast_builder);
        let call_expr = ast_builder.call_expression(
            SPAN,
            Expression::Identifier(qrl),
            None::<OxcBox<TSTypeParameterInstantiation>>,
            self.as_arguments(ast_builder),
            false,
        );

      call_expr
    }

    /// To access this logic call `IntoIn` to convert `Qrl` to  full call `Expression`.
    /// # Examples
    /// ```rust
    /// use oxc_allocator::Allocator;
    /// use oxc_ast::ast::Expression;
    /// 
    /// let allocator = Allocator::default();
    /// let qrl = super::Qrl::new("./test.tsx_renderHeader_zBbHWn4e8Cg", "renderHeader_zBbHWn4e8Cg");
    /// let expr: Expression = qrl.into_in(&allocator);
    /// ```
    /// The resulting Javascript, when rendered, will be:
    /// ```javascript
    /// qrl(() => import("./test.tsx_renderHeader_zBbHWn4e8Cg"), "renderHeader_zBbHWn4e8Cg");
    /// 
    fn expression<'a>(self, allocator: &'a Allocator, ast_builder: &AstBuilder<'a>) -> Expression<'a> {
        // let ast_builder = AstBuilder::new(allocator);
        // let rel_path = format!(
        //     "./{}",
        //     self.rel_path.to_string_lossy()
        // );
        // let display_name = &self.display_name;
        // let allocator = ast_builder.allocator;
        //
        // // Function Body /////////
        // let mut statements = ast_builder.vec_with_capacity(1);
        // statements.push(ast_builder.qwik_simple_import(rel_path.as_ref()));
        // let function_body = ast_builder.function_body(SPAN, ast_builder.vec(), statements);
        // let func_params = ast_builder.formal_parameters(
        //     SPAN,
        //     FormalParameterKind::ArrowFormalParameters,
        //     OxcVec::with_capacity_in(0, ast_builder.allocator),
        //     None::<OxcBox<BindingRestElement>>,
        // );
        //
        // // ARG: Arrow Function Expression ////////
        // let arrow_function = ast_builder.arrow_function_expression(
        //     SPAN,
        //     true,
        //     false,
        //     None::<OxcBox<TSTypeParameterDeclaration>>,
        //     func_params,
        //     None::<OxcBox<TSTypeAnnotation>>,
        //     function_body,
        // );

        // FUNC NAME: Reference Id 'qrl' ////////
        // let qrl = self.as_identifier(ast_builder);

        // ARG: Display name string literal ////////
        // let raw: Atom = format!(r#""{}""#, display_name).into_in(allocator);
        // let display_name_arg = OxcBox::new_in(
        //     ast_builder.string_literal(SPAN, display_name, Some(raw)),
        //     allocator,
        // );
        //
        // let mut args = ast_builder.vec_with_capacity(2);
        // args.push(Argument::ArrowFunctionExpression(OxcBox::new_in(
        //     arrow_function,
        //     allocator,
        // )));
        // args.push(Argument::StringLiteral(display_name_arg));

        // Call Expression ////////
        // let call_expr = ast_builder.call_expression(
        //     SPAN,
        //     Expression::Identifier(qrl),
        //     None::<OxcBox<TSTypeParameterInstantiation>>,
        //     self.as_arguments(ast_builder),
        //     false,
        // );

        Expression::CallExpression(OxcBox::new_in(self.as_call_expression(ast_builder), allocator))
    }

    pub fn as_statement<'a>(self, ast_builder: &AstBuilder<'a>) -> Statement<'a> {
        let call_expr = self.expression(ast_builder.allocator, ast_builder);
        ast_builder.statement_expression(SPAN, call_expr)
    }
}


impl<'a> FromIn<'a, Qrl> for Expression<'a> {
    fn from_in(qrl: Qrl, allocator: &'a Allocator) -> Self {
        let ast_builder = AstBuilder::new(allocator);
        qrl.expression(allocator, &ast_builder)
    }
}

impl<'a> FromIn<'a, Qrl> for JSXExpression<'a> {
    fn from_in(qrl: Qrl, allocator: &'a Allocator) -> Self {
        let ast_builder = AstBuilder::new(allocator);
        let call_expr = qrl.as_call_expression(&ast_builder);
        JSXExpression::CallExpression(OxcBox::new_in(call_expr, allocator))
    }
}

#[test]
fn test_qurl() {
    let allocator = Allocator::default();
    let ast_builder = AstBuilder::new(&allocator);
    let qurl = Qrl::new(
        "./test.tsx_renderHeader_zBbHWn4e8Cg",
        "renderHeader_zBbHWn4e8Cg",
    );
    let source_type = SourceType::from_path("test.tsx").unwrap();
    let ast_builer = AstBuilder::new(&allocator);
    let statement = qurl.as_statement(&ast_builer);
    let pgm = ast_builder.qwik_program(vec![statement], source_type);
    let codegen = Codegen::new();
    let script = codegen.build(&pgm).code;

    let expected =
        r#"qrl(() => import("./test.tsx_renderHeader_zBbHWn4e8Cg"), "renderHeader_zBbHWn4e8Cg");"#;
    assert_eq!(script.trim(), expected.trim())
}

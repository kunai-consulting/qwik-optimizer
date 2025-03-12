use crate::component::{CommonImport, COMPONENT_QRL, QRL};
use crate::ext::AstBuilderExt;
use oxc_allocator::{Allocator, Box as OxcBox, FromIn, IntoIn, Vec as OxcVec};
use oxc_ast::ast::*;
use oxc_ast::AstBuilder;
use oxc_index::Idx;
use oxc_semantic::ReferenceId;
use oxc_span::{Atom, SPAN};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum QrlType {
    Qrl,
    ComponentQrl,
}

impl From<QrlType> for CommonImport {
    fn from(value: QrlType) -> Self {
        match value {
            QrlType::Qrl => CommonImport::qrl(),
            QrlType::ComponentQrl => {
                CommonImport::BuilderIoQwik(vec![COMPONENT_QRL.into(), QRL.into()])
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Qrl {
    pub rel_path: PathBuf,
    pub display_name: String,
    pub qrl_type: QrlType,
}

impl Qrl {
    pub fn new<T: Into<PathBuf>>(rel_path: T, display_name: &str, qrl_type: QrlType) -> Self {
        Self {
            rel_path: rel_path.into(),
            display_name: display_name.into(),
            qrl_type,
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
    fn into_identifier<'a>(&self, ast_builder: &AstBuilder<'a>) -> IdentifierReference<'a> {
        let ref_id: ReferenceId = ReferenceId::from_usize(0);
        match self.qrl_type {
            QrlType::Qrl => ast_builder.identifier_reference_with_reference_id(SPAN, QRL, ref_id),
            QrlType::ComponentQrl => {
                ast_builder.identifier_reference_with_reference_id(SPAN, COMPONENT_QRL, ref_id)
            }
        }
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
    fn into_arrow_function<'a>(self, ast_builder: &AstBuilder<'a>) -> ArrowFunctionExpression<'a> {
        let rel_path = self.rel_path.to_string_lossy();

        // Function Body /////////
        let mut statements = ast_builder.vec_with_capacity(1);
        statements.push(ast_builder.create_simple_import(rel_path.as_ref()));
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

    fn into_arguments<'a>(self, ast_builder: &AstBuilder<'a>) -> OxcVec<'a, Argument<'a>> {
        let allocator = ast_builder.allocator;
        let display_name = self.display_name.clone();

        // ARG: Display name string literal ////////
        let raw: Atom = format!(r#""{}""#, display_name).into_in(allocator);
        let display_name_arg = OxcBox::new_in(
            ast_builder.string_literal(SPAN, display_name, Some(raw)),
            allocator,
        );

        let mut args = ast_builder.vec_with_capacity(2);
        let arrow_function = self.into_arrow_function(ast_builder);
        args.push(Argument::ArrowFunctionExpression(OxcBox::new_in(
            arrow_function,
            allocator,
        )));
        args.push(Argument::StringLiteral(display_name_arg));

        args
    }

    fn into_call_expression<'a>(self, ast_builder: &AstBuilder<'a>) -> CallExpression<'a> {
        let ref_id: ReferenceId = ReferenceId::from_usize(0);
        let qrl = ast_builder.identifier_reference_with_reference_id(SPAN, QRL, ref_id);
        let qrl_type = self.qrl_type.clone();

        let args = self.into_arguments(ast_builder);
        let qrl = OxcBox::new_in(qrl, ast_builder.allocator);
        let call_expr = ast_builder.call_expression(
            SPAN,
            Expression::Identifier(qrl),
            None::<OxcBox<TSTypeParameterInstantiation>>,
            args,
            false,
        );

        match qrl_type {
            QrlType::Qrl => call_expr,

            QrlType::ComponentQrl => {
                let ident = OxcBox::new_in(
                    ast_builder.identifier_reference_with_reference_id(SPAN, COMPONENT_QRL, ref_id),
                    ast_builder.allocator,
                );
                let arg =
                    Argument::CallExpression(OxcBox::new_in(call_expr, ast_builder.allocator));
                let args = ast_builder.vec1(arg);
                ast_builder.call_expression(
                    SPAN,
                    Expression::Identifier(ident),
                    None::<OxcBox<TSTypeParameterInstantiation>>,
                    args,
                    false,
                )
            }
        }
    }

    /// To access this logic call `IntoIn` to convert `Qrl` to  full call `Expression`.
    /// # Examples
    /// ```ignore
    /// use oxc_allocator::Allocator;
    /// use oxc_ast::ast::Expression;
    ///
    ///
    /// let allocator = Allocator::default();
    /// let qrl = Qrl::new("./test.tsx_renderHeader_zBbHWn4e8Cg", "renderHeader_zBbHWn4e8Cg");
    /// let expr: Expression = qrl.into_in(&allocator);
    /// ```
    /// The resulting Javascript, when rendered, will be:
    /// ```javascript
    /// qrl(() => import("./test.tsx_renderHeader_zBbHWn4e8Cg"), "renderHeader_zBbHWn4e8Cg");
    ///
    pub(crate) fn into_expression<'a>(
        self,
        allocator: &'a Allocator,
        ast_builder: &AstBuilder<'a>,
    ) -> Expression<'a> {
        Expression::CallExpression(OxcBox::new_in(
            self.into_call_expression(ast_builder),
            allocator,
        ))
    }

    pub fn into_statement<'a>(self, ast_builder: &AstBuilder<'a>) -> Statement<'a> {
        let call_expr = self.into_expression(ast_builder.allocator, ast_builder);
        ast_builder.statement_expression(SPAN, call_expr)
    }
}

impl<'a> FromIn<'a, Qrl> for OxcVec<'a, Argument<'a>> {
    fn from_in(qrl: Qrl, allocator: &'a Allocator) -> Self {
        let ast_builder = AstBuilder::new(allocator);
        qrl.into_arguments(&ast_builder)
    }
}
impl<'a> FromIn<'a, Qrl> for IdentifierReference<'a> {
    fn from_in(qrl: Qrl, allocator: &'a Allocator) -> Self {
        let ast_builder = AstBuilder::new(allocator);
        qrl.into_identifier(&ast_builder)
    }
}

impl<'a> FromIn<'a, Qrl> for OxcBox<'a, CallExpression<'a>> {
    fn from_in(qrl: Qrl, allocator: &'a Allocator) -> OxcBox<'a, CallExpression<'a>> {
        let ast_builder = AstBuilder::new(allocator);
        OxcBox::new_in(qrl.into_call_expression(&ast_builder), allocator)
    }
}

impl<'a> FromIn<'a, Qrl> for Expression<'a> {
    fn from_in(qrl: Qrl, allocator: &'a Allocator) -> Self {
        let ast_builder = AstBuilder::new(allocator);
        qrl.into_expression(allocator, &ast_builder)
    }
}

impl<'a> FromIn<'a, Qrl> for JSXExpression<'a> {
    fn from_in(qrl: Qrl, allocator: &'a Allocator) -> Self {
        let ast_builder = AstBuilder::new(allocator);
        let call_expr = qrl.into_call_expression(&ast_builder);
        JSXExpression::CallExpression(OxcBox::new_in(call_expr, allocator))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    
    use oxc_codegen::Codegen;

    #[test]
    fn test_qurl() {
        let allocator = Allocator::default();
        let ast_builder = AstBuilder::new(&allocator);
        let qurl = Qrl::new(
            "./test.tsx_renderHeader_zBbHWn4e8Cg",
            "renderHeader_zBbHWn4e8Cg",
            QrlType::Qrl,
        );
        let statement = qurl.into_statement(&ast_builder);
        let pgm = ast_builder.program(
            SPAN,
            SourceType::tsx(),
            "",
            OxcVec::new_in(&allocator),
            None,
            OxcVec::new_in(&allocator),
            ast_builder.vec1(statement),
        );
        let codegen = Codegen::new();
        let script = codegen.build(&pgm).code;

        let expected = r#"qrl(() => import("./test.tsx_renderHeader_zBbHWn4e8Cg"), "renderHeader_zBbHWn4e8Cg");"#;
        assert_eq!(script.trim(), expected.trim())
    }
}

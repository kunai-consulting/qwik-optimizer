use crate::component::{CommonImport, Import, QRL, QRL_SUFFIX, QWIK_CORE_SOURCE};
use crate::ext::AstBuilderExt;
use oxc_allocator::{Allocator, Box as OxcBox, CloneIn, FromIn, IntoIn, Vec as OxcVec};
use oxc_ast::ast::*;
use oxc_ast::AstBuilder;
use oxc_semantic::{NodeId, ReferenceFlags, ReferenceId, ScopeId, SymbolFlags, SymbolId};
use oxc_span::{Atom, SPAN};
use oxc_traverse::TraverseCtx;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum QrlType {
    Qrl,
    PrefixedQrl(String),
    IndexedQrl(usize),
}

impl From<QrlType> for CommonImport {
    fn from(value: QrlType) -> Self {
        match value {
            QrlType::Qrl => CommonImport::qrl(),
            QrlType::IndexedQrl(_) => CommonImport::qrl(),
            QrlType::PrefixedQrl(prefix) => CommonImport::QwikCore(vec![
                format!("{}{}", prefix, QRL_SUFFIX).as_str().into(),
                QRL.into(),
            ]),
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

    /// Creates a reference id, attempting to bind it
    /// to the relevant symbol_id if it exists.
    ///
    fn make_ref_id(
        qrl_type: &QrlType,
        ctx: &mut TraverseCtx,
        symbols_by_name: &mut HashMap<String, SymbolId>,
        import_by_symbol: &mut HashMap<SymbolId, Import>,
    ) -> ReferenceId {
        match qrl_type {
            QrlType::Qrl | QrlType::IndexedQrl(_) => {
                // `qrl` is ALWAYS part of newly created expression, even if `$` was not used in the initial script.
                // If `qrl` was not explicitly imported in the original script, we need to synthesize both a SymbolId and an Import.
                let qrl_symbol_id = if !symbols_by_name.contains_key(QRL) {
                    let symbol_id = ctx.symbols_mut().create_symbol(
                        SPAN,
                        QRL,
                        SymbolFlags::Import,
                        ScopeId::new(0),
                        NodeId::DUMMY,
                    );
                    let import = Import::new(vec!["qrl".into()], QWIK_CORE_SOURCE);
                    symbols_by_name.insert(QRL.to_string(), symbol_id);
                    import_by_symbol.insert(symbol_id, import);
                    symbol_id
                } else {
                    *symbols_by_name.get(QRL).unwrap() // This should never fail based on the call above.
                };

                ctx.create_bound_reference(qrl_symbol_id, ReferenceFlags::None)
            }
            QrlType::PrefixedQrl(name) => {
                if let Some(symbol_id) = symbols_by_name.get(name) {
                    ctx.create_bound_reference(*symbol_id, ReferenceFlags::None)
                } else {
                    ctx.create_unbound_reference(name, ReferenceFlags::None)
                }
            }
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
    pub fn into_identifier_reference<'a>(
        &self,
        ctx: &mut TraverseCtx<'a>,
        symbols_by_name: &mut HashMap<String, SymbolId>,
        import_by_symbol: &mut HashMap<SymbolId, Import>,
    ) -> IdentifierReference<'a> {
        let ast = ctx.ast;
        match &self.qrl_type {
            QrlType::Qrl | QrlType::IndexedQrl(_) => {
                let ref_id =
                    Self::make_ref_id(&self.qrl_type, ctx, symbols_by_name, import_by_symbol);
                ast.identifier_reference_with_reference_id(SPAN, QRL, ref_id)
            }
            QrlType::PrefixedQrl(prefix) => {
                let ref_id =
                    Self::make_ref_id(&self.qrl_type, ctx, symbols_by_name, import_by_symbol);
                ast.identifier_reference_with_reference_id(
                    SPAN,
                    format!("{}{}", prefix, QRL_SUFFIX),
                    ref_id,
                )
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
    fn into_arrow_function<'a>(&self, ast_builder: &AstBuilder<'a>) -> ArrowFunctionExpression<'a> {
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

    fn into_arguments<'a>(&self, ast_builder: &AstBuilder<'a>) -> OxcVec<'a, Argument<'a>> {
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

    pub fn into_call_expression<'a>(
        &self,
        ctx: &mut TraverseCtx<'a>,
        symbols_by_name: &mut HashMap<String, SymbolId>,
        import_by_symbol: &mut HashMap<SymbolId, Import>,
    ) -> CallExpression<'a> {
        let ast_builder = ctx.ast;

        let qrl_ref_id = Self::make_ref_id(&QrlType::Qrl, ctx, symbols_by_name, import_by_symbol);
        let qrl = ast_builder.identifier_reference_with_reference_id(SPAN, QRL, qrl_ref_id);
        let qrl_type = self.qrl_type.clone();

        let args = self
            .into_arguments(&ast_builder)
            .clone_in(ast_builder.allocator);
        let qrl = OxcBox::new_in(qrl, ast_builder.allocator);
        let qrl_call_expr = ast_builder.call_expression(
            SPAN,
            Expression::Identifier(qrl),
            None::<OxcBox<TSTypeParameterInstantiation>>,
            args,
            false,
        );

        match qrl_type {
            QrlType::Qrl | QrlType::IndexedQrl(_) => qrl_call_expr,

            QrlType::PrefixedQrl(prefix) => {
                let ref_id =
                    Self::make_ref_id(&self.qrl_type, ctx, symbols_by_name, import_by_symbol);
                Self::make_ref_id(&self.qrl_type, ctx, symbols_by_name, import_by_symbol);
                let ident = OxcBox::new_in(
                    ast_builder.identifier_reference_with_reference_id(
                        SPAN,
                        format!("{}{}", prefix, QRL_SUFFIX),
                        ref_id,
                    ),
                    ast_builder.allocator,
                );
                let arg =
                    Argument::CallExpression(OxcBox::new_in(qrl_call_expr, ast_builder.allocator));
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
        ctx: &mut TraverseCtx<'a>,
        symbols_by_name: &mut HashMap<String, SymbolId>,
        import_by_symbol: &mut HashMap<SymbolId, Import>,
    ) -> Expression<'a> {
        Expression::CallExpression(OxcBox::new_in(
            self.into_call_expression(ctx, symbols_by_name, import_by_symbol),
            ctx.ast.allocator,
        ))
    }

    pub fn into_statement<'a>(
        self,
        ctx: &mut TraverseCtx<'a>,
        symbols_by_name: &mut HashMap<String, SymbolId>,
        import_by_symbol: &mut HashMap<SymbolId, Import>,
    ) -> Statement<'a> {
        let call_expr = self.into_expression(ctx, symbols_by_name, import_by_symbol);
        ctx.ast.statement_expression(SPAN, call_expr)
    }

    pub fn into_jsx_expression<'a>(
        self,
        ctx: &mut TraverseCtx<'a>,
        symbols_by_name: &mut HashMap<String, SymbolId>,
        import_by_symbol: &mut HashMap<SymbolId, Import>,
    ) -> JSXExpression<'a> {
        let call_expr = self.into_call_expression(ctx, symbols_by_name, import_by_symbol);
        JSXExpression::CallExpression(OxcBox::new_in(call_expr, ctx.ast.allocator))
    }
}

impl<'a> FromIn<'a, Qrl> for OxcVec<'a, Argument<'a>> {
    fn from_in(qrl: Qrl, allocator: &'a Allocator) -> Self {
        let ast_builder = AstBuilder::new(allocator);
        qrl.into_arguments(&ast_builder)
    }
}

#[cfg(test)]
mod tests {

    // #[test]
    // fn test_qurl() {
    //     let allocator = Allocator::default();
    //     let ast_builder = AstBuilder::new(&allocator);
    //     let qurl = Qrl::new(
    //         "./test.tsx_renderHeader_zBbHWn4e8Cg",
    //         "renderHeader_zBbHWn4e8Cg",
    //         QrlType::Qrl,
    //     );
    //     let statement = qurl.into_statement(&ast_builder);
    //     let pgm = ast_builder.program(
    //         SPAN,
    //         SourceType::tsx(),
    //         "",
    //         OxcVec::new_in(&allocator),
    //         None,
    //         OxcVec::new_in(&allocator),
    //         ast_builder.vec1(statement),
    //     );
    //     let codegen = Codegen::new();
    //     let script = codegen.build(&pgm).code;
    //
    //     let expected = r#"qrl(() => import("./test.tsx_renderHeader_zBbHWn4e8Cg"), "renderHeader_zBbHWn4e8Cg");"#;
    //     assert_eq!(script.trim(), expected.trim())
    // }
}

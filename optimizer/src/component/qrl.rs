use crate::collector::{ExportInfo, Id};
use crate::component::{Import, QRL, QRL_SUFFIX, QWIK_CORE_SOURCE};
use oxc_allocator::{Box as OxcBox, CloneIn, Vec as OxcVec};
use oxc_ast::ast::*;
use oxc_ast::AstBuilder;
use oxc_semantic::{NodeId, ReferenceFlags, ReferenceId, ScopeId, SymbolFlags, SymbolId};
use oxc_span::SPAN;
use oxc_traverse::TraverseCtx;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum QrlType {
    Qrl,
    PrefixedQrl(String),
    IndexedQrl(usize),
}

impl From<QrlType> for Import {
    fn from(value: QrlType) -> Self {
        match value {
            QrlType::Qrl => Import::qrl(),
            QrlType::IndexedQrl(_) => Import::qrl(),
            QrlType::PrefixedQrl(prefix) => Import::new(
                vec![
                    format!("{}{}", prefix, QRL_SUFFIX).as_str().into(),
                    QRL.into(),
                ],
                QWIK_CORE_SOURCE,
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Qrl {
    pub rel_path: PathBuf,
    pub display_name: String,
    pub qrl_type: QrlType,
    /// Captured variables from enclosing scope (sorted for deterministic output)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub scoped_idents: Vec<Id>,
    /// Source file exports referenced in the QRL body.
    /// These need to be imported in the segment file from the source file.
    /// Contains: (local_name, exported_name, is_default)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub referenced_exports: Vec<ExportInfo>,
    /// Iteration variables that become function parameters instead of captures.
    /// Used for event handlers inside loops.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub iteration_params: Vec<Id>,
}

impl Qrl {
    pub fn new_with_exports<T: Into<PathBuf>>(
        rel_path: T,
        display_name: &str,
        qrl_type: QrlType,
        scoped_idents: Vec<Id>,
        referenced_exports: Vec<ExportInfo>,
    ) -> Self {
        Self::new_with_iteration_params(
            rel_path,
            display_name,
            qrl_type,
            scoped_idents,
            referenced_exports,
            Vec::new(),
        )
    }

    pub fn new_with_iteration_params<T: Into<PathBuf>>(
        rel_path: T,
        display_name: &str,
        qrl_type: QrlType,
        scoped_idents: Vec<Id>,
        referenced_exports: Vec<ExportInfo>,
        iteration_params: Vec<Id>,
    ) -> Self {
        Self {
            rel_path: rel_path.into(),
            display_name: display_name.into(),
            qrl_type,
            scoped_idents,
            referenced_exports,
            iteration_params,
        }
    }

    /// Generate the hoisted import identifier name (e.g., "i_abc123")
    /// Uses just the hash portion to match qwik-core format
    pub fn hoisted_import_name(&self) -> String {
        // Extract hash from display_name (format: name_hash)
        // e.g., "renderHeader_div_onClick_fV2uzAL99u4" -> "fV2uzAL99u4"
        let hash = self.display_name
            .rsplit('_')
            .next()
            .unwrap_or(&self.display_name);
        format!("i_{}", hash)
    }

    /// Generate the filename for the dynamic import (e.g., "./test.tsx_component_abc123")
    /// Note: No extension on import paths - bundlers resolve these
    pub fn import_filename(&self) -> String {
        format!(
            "./{}",
            self.rel_path.file_name().unwrap().to_string_lossy()
        )
    }

    /// Creates a reference id, attempting to bind it
    /// to the relevant symbol_id if it exists.
    ///
    fn make_ref_id(
        qrl_type: &QrlType,
        ctx: &mut TraverseCtx<'_, ()>,
        symbols_by_name: &mut HashMap<String, SymbolId>,
        import_by_symbol: &mut HashMap<SymbolId, Import>,
    ) -> ReferenceId {
        match qrl_type {
            QrlType::Qrl | QrlType::IndexedQrl(_) => {
                let qrl_symbol_id = if !symbols_by_name.contains_key(QRL) {
                    let symbol_id = ctx.scoping_mut().create_symbol(
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
        ctx: &mut TraverseCtx<'a, ()>,
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
                    ast.atom(&format!("{}{}", prefix, QRL_SUFFIX)),
                    ref_id,
                )
            }
        }
    }

    pub(crate) fn into_arguments<'a>(
        &self,
        ast_builder: &AstBuilder<'a>,
        hoisted_imports: &mut Vec<(String, String)>,
    ) -> OxcVec<'a, Argument<'a>> {
        let allocator = ast_builder.allocator;

        // Generate hoisted import name and register for hoisting (with deduplication)
        let hoisted_name = self.hoisted_import_name();
        let filename = self.import_filename();

        // Check if this import is already registered (deduplication)
        if !hoisted_imports.iter().any(|(name, _)| name == &hoisted_name) {
            hoisted_imports.push((hoisted_name.clone(), filename));
        }

        let raw = ast_builder.atom(&format!(r#""{}""#, &self.display_name));
        let display_name_arg = OxcBox::new_in(
            ast_builder.string_literal(SPAN, ast_builder.atom(&self.display_name), Some(raw)),
            allocator,
        );

        let capacity = if self.scoped_idents.is_empty() { 2 } else { 3 };
        let mut args = ast_builder.vec_with_capacity(capacity);

        // Use identifier reference to hoisted import instead of inline arrow
        let import_ident = ast_builder.expression_identifier(SPAN, ast_builder.atom(&hoisted_name));
        args.push(Argument::from(import_ident));
        args.push(Argument::StringLiteral(display_name_arg));

        if !self.scoped_idents.is_empty() {
            let mut elements = ast_builder.vec_with_capacity(self.scoped_idents.len());
            for (name, _scope_id) in &self.scoped_idents {
                let ident_ref = ast_builder.expression_identifier(SPAN, ast_builder.atom(name.as_str()));
                elements.push(ArrayExpressionElement::from(ident_ref));
            }
            let captures_array = ast_builder.expression_array(SPAN, elements);
            args.push(Argument::from(captures_array));
        }

        args
    }

    pub fn into_call_expression<'a>(
        &self,
        ctx: &mut TraverseCtx<'a, ()>,
        symbols_by_name: &mut HashMap<String, SymbolId>,
        import_by_symbol: &mut HashMap<SymbolId, Import>,
        hoisted_imports: &mut Vec<(String, String)>,
    ) -> CallExpression<'a> {
        let ast_builder = ctx.ast;

        let qrl_ref_id = Self::make_ref_id(&QrlType::Qrl, ctx, symbols_by_name, import_by_symbol);
        let qrl = ast_builder.identifier_reference_with_reference_id(SPAN, QRL, qrl_ref_id);
        let qrl_type = self.qrl_type.clone();

        let args = self
            .into_arguments(&ast_builder, hoisted_imports)
            .clone_in(ast_builder.allocator);
        let qrl = OxcBox::new_in(qrl, ast_builder.allocator);

        // Create qrl() call with PURE annotation for tree-shaking
        let qrl_call_expr = ast_builder.call_expression_with_pure(
            SPAN,
            Expression::Identifier(qrl),
            None::<OxcBox<TSTypeParameterInstantiation>>,
            args,
            false,
            true, // pure: true - adds /* @__PURE__ */ annotation
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
                        ast_builder.atom(&format!("{}{}", prefix, QRL_SUFFIX)),
                        ref_id,
                    ),
                    ast_builder.allocator,
                );
                let arg =
                    Argument::CallExpression(OxcBox::new_in(qrl_call_expr, ast_builder.allocator));
                let args = ast_builder.vec1(arg);
                // Prefixed calls (like componentQrl()) also get PURE annotation
                ast_builder.call_expression_with_pure(
                    SPAN,
                    Expression::Identifier(ident),
                    None::<OxcBox<TSTypeParameterInstantiation>>,
                    args,
                    false,
                    true, // pure: true - adds /* @__PURE__ */ annotation
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
    /// let qrl = Qrl::new("./test.tsx_renderHeader_zBbHWn4e8Cg", "renderHeader_zBbHWn4e8Cg", QrlType::Qrl, vec![]);
    /// let expr: Expression = qrl.into_in(&allocator);
    /// ```
    /// The resulting Javascript, when rendered, will be:
    /// ```javascript
    /// qrl(() => import("./test.tsx_renderHeader_zBbHWn4e8Cg"), "renderHeader_zBbHWn4e8Cg");
    /// ```
    /// Or with captures:
    /// ```javascript
    /// qrl(() => import("./test.tsx_renderHeader_zBbHWn4e8Cg"), "renderHeader_zBbHWn4e8Cg", [count, name]);
    ///
    pub(crate) fn into_expression<'a>(
        self,
        ctx: &mut TraverseCtx<'a, ()>,
        symbols_by_name: &mut HashMap<String, SymbolId>,
        import_by_symbol: &mut HashMap<SymbolId, Import>,
        hoisted_imports: &mut Vec<(String, String)>,
    ) -> Expression<'a> {
        Expression::CallExpression(OxcBox::new_in(
            self.into_call_expression(ctx, symbols_by_name, import_by_symbol, hoisted_imports),
            ctx.ast.allocator,
        ))
    }

    pub fn into_statement<'a>(
        self,
        ctx: &mut TraverseCtx<'a, ()>,
        symbols_by_name: &mut HashMap<String, SymbolId>,
        import_by_symbol: &mut HashMap<SymbolId, Import>,
        hoisted_imports: &mut Vec<(String, String)>,
    ) -> Statement<'a> {
        let call_expr = self.into_expression(ctx, symbols_by_name, import_by_symbol, hoisted_imports);
        ctx.ast.statement_expression(SPAN, call_expr)
    }

    pub fn into_jsx_expression<'a>(
        self,
        ctx: &mut TraverseCtx<'a, ()>,
        symbols_by_name: &mut HashMap<String, SymbolId>,
        import_by_symbol: &mut HashMap<SymbolId, Import>,
        hoisted_imports: &mut Vec<(String, String)>,
    ) -> JSXExpression<'a> {
        let call_expr = self.into_call_expression(ctx, symbols_by_name, import_by_symbol, hoisted_imports);
        JSXExpression::CallExpression(OxcBox::new_in(call_expr, ctx.ast.allocator))
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

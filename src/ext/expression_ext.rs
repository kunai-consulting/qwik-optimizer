use crate::component::QRL_MARKER;
use oxc_ast::ast::Expression;
use oxc_semantic::SymbolId;
use oxc_traverse::TraverseCtx;
use std::collections::HashSet;

pub trait ExpressionExt {
    fn is_qrl_replaceable(&self) -> bool;
    fn get_referenced_symbols(&self, ctx: &mut TraverseCtx) -> HashSet<SymbolId>;
}

fn walk_referenced_symbols(
    expression: &Expression,
    symbols: &mut HashSet<SymbolId>,
    ctx: &mut TraverseCtx,
) {
    match expression {
        Expression::BooleanLiteral(_) => {}
        Expression::NullLiteral(_) => {}
        Expression::NumericLiteral(_) => {}
        Expression::BigIntLiteral(_) => {}
        Expression::RegExpLiteral(_) => {}
        Expression::StringLiteral(_) => {}
        Expression::TemplateLiteral(_) => {}
        Expression::Identifier(id_ref) => {
            if !id_ref.name.ends_with("$") {
                let ref_id = id_ref.reference_id();
                if let Some(symbol_id) = ctx.symbols().get_reference(ref_id).symbol_id() {
                    symbols.insert(symbol_id);
                }
            }
        }
        Expression::MetaProperty(_) => {}
        Expression::Super(_) => {}
        Expression::ArrayExpression(_) => {}
        Expression::ArrowFunctionExpression(_) => {}
        Expression::AssignmentExpression(_) => {}
        Expression::AwaitExpression(_) => {}
        Expression::BinaryExpression(_) => {}
        Expression::CallExpression(_) => {}
        Expression::ChainExpression(_) => {}
        Expression::ClassExpression(_) => {}
        Expression::ConditionalExpression(_) => {}
        Expression::FunctionExpression(_) => {}
        Expression::ImportExpression(_) => {}
        Expression::LogicalExpression(_) => {}
        Expression::NewExpression(_) => {}
        Expression::ObjectExpression(_) => {}
        Expression::ParenthesizedExpression(_) => {}
        Expression::SequenceExpression(_) => {}
        Expression::TaggedTemplateExpression(_) => {}
        Expression::ThisExpression(_) => {}
        Expression::UnaryExpression(_) => {}
        Expression::UpdateExpression(_) => {}
        Expression::YieldExpression(_) => {}
        Expression::PrivateInExpression(_) => {}
        Expression::JSXElement(_) => {}
        Expression::JSXFragment(_) => {}
        Expression::TSAsExpression(_) => {}
        Expression::TSSatisfiesExpression(_) => {}
        Expression::TSTypeAssertion(_) => {}
        Expression::TSNonNullExpression(_) => {}
        Expression::TSInstantiationExpression(_) => {}
        Expression::ComputedMemberExpression(_) => {}
        Expression::StaticMemberExpression(expr) => {
            walk_referenced_symbols(&expr.object, symbols, ctx);
        }
        Expression::PrivateFieldExpression(_) => {}
    }
}

impl ExpressionExt for Expression<'_> {
    fn is_qrl_replaceable(&self) -> bool {
        if let Expression::CallExpression(call_xpr) = self {
            if let Expression::Identifier(id_ref) = &call_xpr.callee {
                id_ref.name.ends_with(QRL_MARKER)
            } else {
                false
            }
        } else {
            false
        }
    }

    fn get_referenced_symbols(&self, ctx: &mut TraverseCtx) -> HashSet<SymbolId> {
        let mut symbols = HashSet::new();
        walk_referenced_symbols(self, &mut symbols, ctx);
        symbols
    }
}

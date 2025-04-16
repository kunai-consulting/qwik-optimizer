use crate::component::MARKER_SUFFIX;
use oxc_ast::ast::Expression;

pub trait ExpressionExt {
    fn is_qrl_replaceable(&self) -> bool;
}

impl ExpressionExt for Expression<'_> {
    fn is_qrl_replaceable(&self) -> bool {
        if let Expression::CallExpression(call_xpr) = self {
            if let Expression::Identifier(id_ref) = &call_xpr.callee {
                id_ref.name.ends_with(MARKER_SUFFIX)
            } else {
                false
            }
        } else {
            false
        }
    }
}

use oxc_ast::ast::{BindingIdentifier, VariableDeclarator};
use oxc_traverse::TraverseCtx;

pub trait RefCounter {
    fn reference_count(&self, ctx: &TraverseCtx) -> usize;
}

impl RefCounter for &BindingIdentifier<'_> {
    fn reference_count(&self, ctx: &TraverseCtx<'_>) -> usize {
        let mut count: usize = 0;
        if let Some(sym) = self.symbol_id.get() {
            count = ctx.symbols().get_resolved_references(sym).count();
        }
        count
    }
}

impl RefCounter for &VariableDeclarator<'_> {
    fn reference_count(&self, ctx: &TraverseCtx) -> usize {
        let mut count: usize = 0;
        if let Some(id) = self.id.get_binding_identifier() {
            count = id.reference_count(ctx);
        }
        count
    }
}

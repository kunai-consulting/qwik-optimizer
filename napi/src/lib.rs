#![deny(clippy::all)]
#![deny(clippy::perf)]
#![deny(clippy::nursery)]

extern crate napi;
#[macro_use]
extern crate napi_derive;

use napi::{CallContext, JsObject, JsUnknown, Result};
use qwik_optimizer::transform;

#[cfg(windows)]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[allow(clippy::needless_pass_by_value)]
#[js_function(1)]
fn transform_fs(ctx: CallContext) -> Result<JsUnknown> {
    let opts = ctx.get::<JsObject>(0)?;
    let config: transform::TransformFsOptions = ctx.env.from_js_value(opts)?;

    let result = transform::transform_fs(config).unwrap();
    ctx.env.to_js_value(&result)
}

#[allow(clippy::needless_pass_by_value)]
#[js_function(1)]
fn transform_modules(ctx: CallContext) -> Result<JsUnknown> {
    let opts = ctx.get::<JsObject>(0)?;
    let config: transform::TransformModulesOptions = ctx.env.from_js_value(opts)?;

    let result = transform::transform_modules(config).unwrap();
    ctx.env.to_js_value(&result)
}

#[module_exports]
fn init(mut exports: JsObject) -> Result<()> {
    exports.create_named_method("transform_fs", transform_fs)?;
    exports.create_named_method("transform_modules", transform_modules)?;

    Ok(())
}

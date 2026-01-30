pub mod generator;
pub mod jsx;
pub mod options;
pub mod qrl;
pub mod scope;
pub mod state;

pub use generator::{IdentType, IdPlusType, TransformGenerator};
pub use options::{transform, OptimizationResult, OptimizedApp, TransformOptions};
pub use state::{ImportTracker, JsxState};

#[allow(unused_imports)]
pub(crate) use generator::Target;
#[allow(unused_imports)]
pub(crate) use jsx::event::get_event_scope_data_from_jsx_event;
#[allow(unused_imports)]
pub(crate) use jsx::event::jsx_event_to_html_attribute;
#[allow(unused_imports)]
pub(crate) use jsx::is_bind_directive;
#[allow(unused_imports)]
pub(crate) use qrl::compute_scoped_idents;

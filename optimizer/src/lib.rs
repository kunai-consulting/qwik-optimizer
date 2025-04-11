pub mod component;
pub(crate) mod error;
pub(crate) mod ext;
pub(crate) mod prelude;

pub mod source;
#[macro_use]
pub mod macros;

mod dead_code;
mod illegal_code;
mod import_clean_up;
mod processing_failure;
mod ref_counter;
mod segment;
pub mod transform;

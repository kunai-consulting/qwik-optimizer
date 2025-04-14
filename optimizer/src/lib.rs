pub mod component;
pub(crate) mod error;
pub(crate) mod ext;
pub(crate) mod prelude;

pub mod source;

pub mod config;
#[macro_use]
pub mod macros;

mod dead_code;
mod import_clean_up;
mod entry_strategy;
pub mod js_lib_interface;
mod ref_counter;
mod segment;
pub mod transform;
mod illegal_code;
mod processing_failure;
mod transpiler;
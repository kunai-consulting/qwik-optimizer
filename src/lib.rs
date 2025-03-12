pub mod component;
pub(crate) mod error;
pub(crate) mod ext;
pub(crate) mod prelude;
pub(crate) mod sources;

#[macro_use]
pub mod macros;

mod dead_code;
mod ref_counter;
mod segment;
mod transform;

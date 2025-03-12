pub(crate) mod sources;
pub mod component;
pub(crate)  mod error;
pub(crate) mod ext;
pub(crate) mod prelude;

#[macro_use]
pub mod macros;

mod dead_code;
mod transform;
mod segment;
mod ref_counter;



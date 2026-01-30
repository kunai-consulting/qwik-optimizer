// Allow common clippy lints that are pre-existing in the codebase
// These can be addressed in a dedicated code quality phase
#![allow(clippy::ptr_arg)]
#![allow(clippy::wrong_self_convention)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::explicit_auto_deref)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::single_match)]
#![allow(clippy::redundant_pattern_matching)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::unnecessary_to_owned)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::collapsible_match)]
#![allow(clippy::redundant_guards)]
#![allow(clippy::unwrap_or_default)]

pub mod code_move;
pub mod collector;
pub mod component;
pub mod const_replace;
pub(crate) mod error;
pub(crate) mod ext;
pub mod is_const;
pub(crate) mod prelude;
pub mod props_destructuring;
pub mod inlined_fn;

pub mod source;

#[macro_use]
pub mod macros;

mod dead_code;
mod entry_strategy;
mod illegal_code;
mod import_clean_up;
pub mod js_lib_interface;
mod processing_failure;
mod ref_counter;
mod segment;
pub mod transform;

#[cfg(test)]
mod transform_tests;

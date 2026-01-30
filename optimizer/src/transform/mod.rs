//! Transform module for Qwik optimizer.
//!
//! This module handles the core AST transformation logic for converting
//! standard JavaScript/TypeScript into Qwik's lazy-loadable format.
//!
//! # Module Structure
//!
//! - `state`: JSX and import state tracking types
//! - `options`: Configuration and main transform entry point
//! - `generator`: Core AST transformation logic

pub mod generator;
pub mod jsx;
pub mod options;
pub mod state;

// Re-export main types for public API
pub use generator::{IdentType, IdPlusType, OptimizationResult, OptimizedApp, TransformGenerator};
pub use options::{transform, TransformOptions};
pub use state::{ImportTracker, JsxState};

// Re-export crate-internal items (used by is_const.rs, transform_tests.rs)
#[allow(unused_imports)]
pub(crate) use generator::{compute_scoped_idents, Target};
#[allow(unused_imports)]
pub(crate) use jsx::{get_event_scope_data_from_jsx_event, jsx_event_to_html_attribute};

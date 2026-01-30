//! Transform options and main entry point for the Qwik optimizer.
//!
//! This module contains:
//! - `TransformOptions`: Configuration for the transformation
//! - `transform()`: Main entry point for transforming source code
//! - `OptimizedApp`: Result of optimization containing body and components
//! - `OptimizationResult`: Wrapper for optimization result and errors

use std::fmt::Display;

use serde::Serialize;

use crate::component::{QrlComponent, Target};
use crate::const_replace::ConstReplacerVisitor;
use crate::entry_strategy::EntryStrategy;
use crate::prelude::*;
use crate::processing_failure::ProcessingFailure;
use crate::source::Source;

use super::generator::TransformGenerator;
use super::state::ImportTracker;

use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclarationSpecifier, Statement};
use oxc_parser::Parser;
use oxc_semantic::{SemanticBuilder, SemanticBuilderReturn};
use oxc_transformer::{JsxOptions, TransformOptions as OxcTransformOptions, Transformer, TypeScriptOptions};
use oxc_traverse::traverse_mut;

// =============================================================================
// Output Types
// =============================================================================

/// Result of optimizing a Qwik source file.
///
/// Contains the transformed body code and all extracted QRL components.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Serialize)]
pub struct OptimizedApp {
    pub body: String,
    pub components: Vec<QrlComponent>,
}

impl OptimizedApp {
    /// Find a component by its symbol name.
    pub fn get_component(&self, name: String) -> Option<&QrlComponent> {
        self.components
            .iter()
            .find(|comp| comp.id.symbol_name == name)
    }
}

impl Display for OptimizedApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let component_count = self.components.len();
        let comp_heading = format!(
            "------------------- COMPONENTS[{}] ------------------\n",
            component_count
        );

        let sep = format!("{}\n", "-".repeat(comp_heading.len()));
        let all_comps = self.components.iter().fold(String::new(), |acc, comp| {
            let heading = format!("-------- COMPONENT: {}", comp.id.symbol_name);

            let body = &comp.code;
            format!("{}\n{}\n{}\n{}", acc, heading, body, sep)
        });

        let body_heading = "------------------------ BODY -----------------------\n".to_string();

        write!(
            f,
            "{}{}{}{}",
            comp_heading, all_comps, body_heading, self.body
        )
    }
}

/// Wrapper for optimization result containing the optimized app and any errors.
pub struct OptimizationResult {
    pub optimized_app: OptimizedApp,
    pub errors: Vec<ProcessingFailure>,
}

impl OptimizationResult {
    pub fn new(optimized_app: OptimizedApp, errors: Vec<ProcessingFailure>) -> Self {
        Self {
            optimized_app,
            errors,
        }
    }
}

// =============================================================================
// Configuration
// =============================================================================

/// Configuration options for the Qwik optimizer transformation.
#[derive(Clone)]
pub struct TransformOptions {
    pub minify: bool,
    pub target: Target,
    pub transpile_ts: bool,
    pub transpile_jsx: bool,
    /// Entry strategy for determining how segments are grouped for bundling.
    pub entry_strategy: EntryStrategy,
    /// Whether this is a server build (true) or client/browser build (false).
    /// Default: true (safe default - server code is safer to run on server than client code on server)
    pub is_server: bool,
}

impl TransformOptions {
    pub fn with_transpile_ts(mut self, transpile_ts: bool) -> Self {
        self.transpile_ts = transpile_ts;
        self
    }

    pub fn with_transpile_jsx(mut self, transpile_jsx: bool) -> Self {
        self.transpile_jsx = transpile_jsx;
        self
    }

    pub fn with_is_server(mut self, is_server: bool) -> Self {
        self.is_server = is_server;
        self
    }

    /// Returns true if running in development mode (Target::Dev)
    pub fn is_dev(&self) -> bool {
        self.target == Target::Dev
    }
}

impl Default for TransformOptions {
    fn default() -> Self {
        TransformOptions {
            minify: false,
            target: Target::Dev,
            transpile_ts: false,
            transpile_jsx: false,
            entry_strategy: EntryStrategy::Segment,
            is_server: true, // Safe default
        }
    }
}

/// Main entry point for transforming source code using the Qwik optimizer.
///
/// This function parses the source code, applies TypeScript transpilation if configured,
/// performs const replacement for SSR/build mode, and runs the main transformation
/// to extract QRLs and generate optimized output.
pub fn transform(script_source: Source, options: TransformOptions) -> Result<OptimizationResult> {
    let allocator = Allocator::default();
    let source_text = script_source.source_code();
    let source_info = script_source.source_info();
    let source_type = script_source.source_info().try_into()?;

    let mut errors = Vec::new();

    let parse_return = Parser::new(&allocator, source_text, source_type).parse();
    errors.extend(parse_return.errors);

    let mut program = parse_return.program;

    if options.transpile_ts {
        let SemanticBuilderReturn {
            semantic,
            errors: _semantic_errors,
        } = SemanticBuilder::new().build(&program);
        let scoping = semantic.into_scoping();
        Transformer::new(
            &allocator,
            source_info.rel_path.as_path(),
            &OxcTransformOptions {
                typescript: TypeScriptOptions::default(),
                jsx: JsxOptions::disable(),
                ..OxcTransformOptions::default()
            },
        )
        .build_with_scoping(scoping, &mut program);
    }

    // Collect imports BEFORE const replacement (for import aliasing)
    // Skip type-only imports as they don't exist at runtime and shouldn't be captured in QRLs
    let mut import_tracker = ImportTracker::new();
    for stmt in &program.body {
        if let Statement::ImportDeclaration(import) = stmt {
            // Skip entire type-only import declarations: `import type { Foo } from '...'`
            if import.import_kind.is_type() {
                continue;
            }

            let source = import.source.value.to_string();
            if let Some(specifiers) = &import.specifiers {
                for specifier in specifiers {
                    match specifier {
                        ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                            // Skip type-only specifiers: `import { type Foo, bar } from '...'`
                            if spec.import_kind.is_type() {
                                continue;
                            }
                            let imported = spec.imported.name().to_string();
                            let local = spec.local.name.to_string();
                            import_tracker.add_import(&source, &imported, &local);
                        }
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                            let local = spec.local.name.to_string();
                            import_tracker.add_import(&source, "default", &local);
                        }
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                            let local = spec.local.name.to_string();
                            import_tracker.add_import(&source, "*", &local);
                        }
                    }
                }
            }
        }
    }

    // Apply const replacement (skip in Test mode to match SWC behavior)
    if options.target != Target::Test {
        let mut const_replacer = ConstReplacerVisitor::new(
            &allocator,
            options.is_server,
            options.is_dev(),
            &import_tracker,
        );
        const_replacer.visit_program(&mut program);
    }

    let SemanticBuilderReturn {
        semantic,
        errors: _semantic_errors,
    } = SemanticBuilder::new()
        .with_check_syntax_error(true) // Enable extra syntax error checking
        .with_cfg(true) // Build a Control Flow Graph
        .build(&program);

    let mut transform = TransformGenerator::new(source_info, options, None, &allocator);

    // let (symbols, scopes) = semantic.into_symbol_table_and_scope_tree();
    let scoping = semantic.into_scoping();

    traverse_mut(&mut transform, &allocator, &mut program, scoping, ());

    let TransformGenerator { app, errors, .. } = transform;
    Ok(OptimizationResult::new(app, errors))
}

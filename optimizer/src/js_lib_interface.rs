// Defines the interface between qwik-optimizer and the qwik JS library.
// This includes some types from the original qwik project that are otherwise
// not used in this rewrite, but needed for compatibility.

use crate::entry_strategy::*;
use crate::error::Error;
use crate::prelude::*;
use crate::processing_failure::ProcessingFailure;
use crate::source::Source;
use crate::transform::*;

use crate::component::*;
use serde::{Deserialize, Serialize};
use std::iter::Sum;

use std::cmp::Ordering;
use std::hash::{DefaultHasher, Hasher};
use std::path::{Path, PathBuf};
use std::str;

#[derive(Debug, Deserialize, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MinifyMode {
    Simplify,
    None,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformModuleInput {
    pub path: String,
    pub dev_path: Option<String>,
    pub code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformModulesOptions {
    pub src_dir: String,
    pub root_dir: Option<String>,
    pub input: Vec<TransformModuleInput>,
    pub source_maps: bool,
    pub minify: MinifyMode,
    pub transpile_ts: bool,
    pub transpile_jsx: bool,
    pub preserve_filenames: bool,
    pub entry_strategy: EntryStrategy,
    pub explicit_extensions: bool,
    pub mode: Target,
    pub scope: Option<String>,

    pub core_module: Option<String>,
    pub strip_exports: Option<Vec<String>>,
    pub strip_ctx_name: Option<Vec<String>>,
    pub strip_event_handlers: bool,
    pub reg_ctx_name: Option<Vec<String>>,
    pub is_server: Option<bool>,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransformOutput {
    pub modules: Vec<TransformModule>,
    pub diagnostics: Vec<Diagnostic>,
    pub is_type_script: bool,
    pub is_jsx: bool,
}

impl TransformOutput {
    pub fn append(mut self, output: &mut Self) -> Self {
        self.modules.append(&mut output.modules);
        self.diagnostics.append(&mut output.diagnostics);
        self.is_type_script = self.is_type_script || output.is_type_script;
        self.is_jsx = self.is_jsx || output.is_jsx;
        self
    }
}

impl Sum<Self> for TransformOutput {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(Self::default(), |x, mut y| x.append(&mut y))
    }
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransformModule {
    pub path: String,
    pub code: String,

    pub map: Option<String>,

    pub segment: Option<SegmentAnalysis>,
    pub is_entry: bool,

    #[serde(skip_serializing)]
    pub order: u64,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SegmentKind {
    Function,
    EventHandler,
    JSXProp,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SegmentAnalysis {
    pub origin: String,
    pub name: String,
    pub entry: Option<String>,
    pub display_name: String,
    pub hash: String,
    pub canonical_filename: String,
    pub path: String,
    pub extension: String,
    pub parent: Option<String>,
    pub ctx_kind: SegmentKind,
    pub ctx_name: String,
    pub captures: bool,
    pub loc: (u32, u32),
}

#[derive(Debug, Serialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SourceLocation {
    lo: usize,
    hi: usize,
    start_line: usize,
    start_col: usize,
    end_line: usize,
    end_col: usize,
}

// impl SourceLocation {
//     pub fn from(source_map: &swc_common::SourceMap, span: swc_common::Span) -> Self {
//         let start = source_map.lookup_char_pos(span.lo);
//         let end = source_map.lookup_char_pos(span.hi);
//         // - SWC's columns are exclusive, ours are inclusive (column - 1)
//         // - SWC has 0-based columns, ours are 1-based (column + 1)
//         // = +-0
//
//         Self {
//             lo: span.lo.0 as usize,
//             hi: span.hi.0 as usize,
//             start_line: start.line,
//             start_col: start.col_display + 1,
//             end_line: end.line,
//             end_col: end.col_display,
//         }
//     }
// }

impl PartialOrd for SourceLocation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.start_line.cmp(&other.start_line) {
            Ordering::Equal => self.start_col.partial_cmp(&other.start_col),
            o => Some(o),
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub category: DiagnosticCategory,
    pub code: Option<String>,
    pub file: String,
    pub message: String,
    pub highlights: Option<Vec<SourceLocation>>,
    pub suggestions: Option<Vec<String>>,
    pub scope: DiagnosticScope,
}

#[derive(Serialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticCategory {
    /// Fails the build with an error.
    Error,
    /// Logs a warning, but the build does not fail.
    Warning,
    /// An error if this is source code in the project, or a warning if in node_modules.
    SourceError,
}

#[derive(Serialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticScope {
    Optimizer,
}

fn error_to_diagnostic(error: ProcessingFailure, path: &Path) -> Diagnostic {
    let message = match error {
        ProcessingFailure::IllegalCode(code) =>
            format!(
                "Reference to identifier '{id}' can not be used inside a Qrl($) scope because it's a {expr_type}",
                id = code.identifier(), expr_type = code.expression_type()
            )
    };
    Diagnostic {
        category: DiagnosticCategory::Error,
        code: None,
        file: path.to_string_lossy().to_string(),
        message,
        highlights: None,
        suggestions: None,
        scope: DiagnosticScope::Optimizer,
    }
}

pub fn transform_modules(config: TransformModulesOptions) -> Result<TransformOutput> {
    let mut final_output = config
        .input
        .into_iter()
        .map(|input| -> Result<Option<TransformOutput>> {
            let path = Path::new(&input.path);
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let relative_path = if path.is_relative() {
                path.into()
            } else {
                pathdiff::diff_paths(path, &config.src_dir).ok_or_else(|| {
                    Error::Generic(format!(
                        "Path {} cannot be made relative to directory {}",
                        path.to_string_lossy(),
                        &config.src_dir
                    ))
                })?
            }
            .to_string_lossy()
            .to_string();
            let language = match ext {
                "ts" => Language::Typescript,
                "tsx" => Language::Typescript,
                "js" => Language::Javascript,
                "jsx" => Language::Javascript,
                "mjs" => Language::Javascript,
                "cjs" => Language::Javascript,
                _ => return Ok(None),
            };
            let OptimizationResult {
                optimized_app,
                errors,
            } = transform(
                Source::from_source(
                    input.code,
                    language,
                    Some(path.with_extension("").to_string_lossy().to_string()),
                )?,
                TransformOptions {
                    minify: match config.minify {
                        MinifyMode::Simplify => true,
                        MinifyMode::None => false,
                    },
                    target: config.mode,
                    transpile_ts: config.transpile_ts,
                    transpile_jsx: config.transpile_jsx,
                    entry_strategy: config.entry_strategy,
                    is_server: true, // Default to server build
                },
            )?;
            let mut hasher = DefaultHasher::new();
            hasher.write(relative_path.as_bytes());
            let mut modules = vec![TransformModule {
                path: relative_path.clone(),
                code: optimized_app.body,
                map: None,
                segment: None,
                is_entry: false,
                order: hasher.finish(),
            }];
            modules.extend(optimized_app.components.into_iter().map(|c| {
                TransformModule {
                    path: format!("{}.js", &c.id.local_file_name),
                    code: c.code,
                    map: None,
                    segment: Some(SegmentAnalysis {
                        origin: relative_path.clone(),
                        name: c.id.symbol_name.clone(),
                        entry: c.entry.clone(),
                        display_name: c.id.display_name,
                        hash: c.id.hash,
                        canonical_filename: PathBuf::from(&c.id.local_file_name)
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                        path: PathBuf::from(&c.id.local_file_name)
                            .parent()
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                        extension: "js".to_string(),
                        parent: c.id.scope,
                        ctx_kind: if c.id.symbol_name.starts_with("on") {
                            SegmentKind::JSXProp
                        } else {
                            SegmentKind::Function
                        },
                        ctx_name: c.id.symbol_name,
                        captures: false,
                        loc: (0, 0),
                    }),
                    is_entry: true,
                    order: c.id.sort_order,
                }
            }));
            Ok(Some(TransformOutput {
                modules,
                diagnostics: errors
                    .into_iter()
                    .map(|e| error_to_diagnostic(e, &path))
                    .collect(),
                is_type_script: config.transpile_ts, // TODO: Set this flag correctly
                is_jsx: config.transpile_jsx,        // TODO: Set this flag correctly
            }))
        })
        .sum::<Result<Option<TransformOutput>>>()?
        .unwrap_or(TransformOutput::default());

    final_output.modules.sort_unstable_by_key(|key| key.order);
    Ok(final_output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glob::glob;
    use serde_json::to_string_pretty;
    use std::path::PathBuf;

    #[test]
    fn test_example_1() {
        assert_valid_transform_debug!(EntryStrategy::Segment);
    }

    #[test]
    fn test_example_2() {
        assert_valid_transform!(EntryStrategy::Segment);
    }

    #[test]
    fn test_example_3() {
        assert_valid_transform!(EntryStrategy::Segment);
    }

    #[test]
    fn test_example_4() {
        assert_valid_transform!(EntryStrategy::Segment);
    }

    #[test]
    fn test_example_5() {
        assert_valid_transform!(EntryStrategy::Segment);
    }

    #[test]
    fn test_example_6() {
        assert_valid_transform!(EntryStrategy::Segment);
    }

    #[test]
    fn test_example_7() {
        assert_valid_transform_debug!(EntryStrategy::Segment);
    }

    #[test]
    fn test_example_8() {
        assert_valid_transform_debug!(EntryStrategy::Segment);
    }

    // #[test]
    fn test_example_9() {
        // Not removing:
        // const decl8 = 1, decl9;
        assert_valid_transform_debug!(EntryStrategy::Segment);
    }

    // #[test]
    fn test_example_10() {
        // Not converting:
        // const a = ident1 + ident3;
        // const b = ident1 + ident3;
        // to:
        // ident1, ident3;
        // ident1, ident3;
        assert_valid_transform!(EntryStrategy::Segment);
    }

    #[test]
    fn test_example_11() {
        assert_valid_transform!(EntryStrategy::Segment);
    }

    #[test]
    fn test_example_capture_imports() {
        assert_valid_transform!(EntryStrategy::Segment);
    }

    #[test]
    fn test_example_capturing_fn_class() {
        assert_valid_transform_debug!(EntryStrategy::Segment);
        /*
        assert_processing_errors!(|errors: Vec<ProcessingFailure>| {
            assert_eq!(errors.len(), 2);

            if let ProcessingFailure::IllegalCode(IllegalCodeType::Function(_, Some(name))) =
                &errors[0]
            {
                assert_eq!(name, "hola");
            } else {
                panic!("Expected function invocation to be illegal code");
            }

            if let ProcessingFailure::IllegalCode(IllegalCodeType::Class(_, Some(name))) =
                &errors[1]
            {
                assert_eq!(name, "Thing");
            } else {
                panic!("Expected class construction to be illegal code");
            }
        });
        */
    }

    #[test]
    fn test_example_jsx() {
        assert_valid_transform_debug!(EntryStrategy::Segment);
    }

    #[test]
    fn test_example_ts() {
        assert_valid_transform_debug!(EntryStrategy::Segment);
    }

    #[test]
    fn test_project_1() {
        // This should be a macro eventually
        let func_name = function_name!();
        let path = PathBuf::from("./src/test_input").join(func_name);

        println!(
            "Loading test input project directory from path: {:?}",
            &path
        );

        let result = transform_modules(TransformModulesOptions {
            input: glob(path.join("src/**/*.ts*").to_str().unwrap())
                .unwrap()
                .into_iter()
                .map(|file| {
                    let file = Path::new(".").join(file.unwrap());
                    let code = std::fs::read_to_string(&file).unwrap();
                    TransformModuleInput {
                        path: file.into_os_string().into_string().unwrap(),
                        dev_path: None,
                        code,
                    }
                })
                .collect(),
            src_dir: path.clone().into_os_string().into_string().unwrap(),
            root_dir: Some(path.clone().into_os_string().into_string().unwrap()),
            minify: MinifyMode::None,
            entry_strategy: EntryStrategy::Component,
            source_maps: false,
            transpile_ts: true,
            transpile_jsx: true,
            preserve_filenames: false,
            explicit_extensions: false,
            mode: Target::Dev,
            scope: None,

            core_module: None,
            strip_exports: None,
            strip_ctx_name: None,
            strip_event_handlers: false,
            reg_ctx_name: None,
            is_server: None,
        })
        .unwrap();

        insta::assert_yaml_snapshot!(func_name, result);
    }

    // ========== QRL Parity Tests ==========
    // These tests validate QRL transformation behavior

    /// Test 1: Basic arrow function QRL - `$(() => ...)` transforms correctly
    #[test]
    fn test_qrl_basic_arrow() {
        assert_valid_transform!(EntryStrategy::Segment);
    }

    /// Test 2: QRL with captured variables - should have `[count]` as third argument
    /// and useLexicalScope in segment
    #[test]
    fn test_qrl_with_captures() {
        assert_valid_transform!(EntryStrategy::Segment);
    }

    /// Test 3: Nested component with handler - component$ with onClick$ inside,
    /// verify parent segment linking
    #[test]
    fn test_qrl_nested_component() {
        assert_valid_transform!(EntryStrategy::Segment);
    }

    /// Test 4: Multiple QRLs in same file - all get unique symbol names
    #[test]
    fn test_qrl_multiple_qrls() {
        assert_valid_transform!(EntryStrategy::Segment);
    }

    /// Test 5: QRL in ternary expression - conditional QRL transforms correctly
    #[test]
    fn test_qrl_ternary() {
        assert_valid_transform!(EntryStrategy::Segment);
    }

    /// Test 6: Hash stability - same input produces same hash
    /// This test verifies that running the transform twice produces identical hashes
    #[test]
    fn test_qrl_hash_stability() {
        let path = PathBuf::from("./src/test_input/test_qrl_basic_arrow.tsx");
        let code = std::fs::read_to_string(&path).unwrap();

        let options1 = TransformModulesOptions {
            input: vec![TransformModuleInput {
                path: path.file_name().unwrap().to_string_lossy().to_string(),
                dev_path: None,
                code: code.clone(),
            }],
            src_dir: ".".to_string(),
            root_dir: None,
            minify: MinifyMode::None,
            entry_strategy: EntryStrategy::Segment,
            source_maps: false,
            transpile_ts: true,
            transpile_jsx: true,
            preserve_filenames: false,
            explicit_extensions: false,
            mode: Target::Test,
            scope: None,
            core_module: None,
            strip_exports: None,
            strip_ctx_name: None,
            strip_event_handlers: false,
            reg_ctx_name: None,
            is_server: None,
        };

        let options2 = TransformModulesOptions {
            input: vec![TransformModuleInput {
                path: path.file_name().unwrap().to_string_lossy().to_string(),
                dev_path: None,
                code: code.clone(),
            }],
            src_dir: ".".to_string(),
            root_dir: None,
            minify: MinifyMode::None,
            entry_strategy: EntryStrategy::Segment,
            source_maps: false,
            transpile_ts: true,
            transpile_jsx: true,
            preserve_filenames: false,
            explicit_extensions: false,
            mode: Target::Test,
            scope: None,
            core_module: None,
            strip_exports: None,
            strip_ctx_name: None,
            strip_event_handlers: false,
            reg_ctx_name: None,
            is_server: None,
        };

        let result1 = transform_modules(options1).unwrap();
        let result2 = transform_modules(options2).unwrap();

        // Verify same number of modules
        assert_eq!(result1.modules.len(), result2.modules.len(), "Module count should match");

        // Verify hashes match for all segments
        for (m1, m2) in result1.modules.iter().zip(result2.modules.iter()) {
            if let (Some(s1), Some(s2)) = (&m1.segment, &m2.segment) {
                assert_eq!(s1.hash, s2.hash, "Hashes should be stable: {} vs {}", s1.hash, s2.hash);
            }
        }
    }

    /// Test 7: Function declaration component$ - `component$(function Name() { ... })`
    /// transforms correctly
    #[test]
    fn test_qrl_function_declaration() {
        assert_valid_transform!(EntryStrategy::Segment);
    }

    // ========== Entry Strategy Integration Tests ==========
    // These tests validate the entry strategy integration with segment generation

    /// Helper function to transform code with a specific entry strategy
    fn transform_with_strategy(code: &str, strategy: EntryStrategy) -> TransformOutput {
        transform_modules(TransformModulesOptions {
            input: vec![TransformModuleInput {
                path: "test.tsx".to_string(),
                dev_path: None,
                code: code.to_string(),
            }],
            src_dir: ".".to_string(),
            root_dir: None,
            minify: MinifyMode::None,
            entry_strategy: strategy,
            source_maps: false,
            transpile_ts: true,
            transpile_jsx: true,
            preserve_filenames: false,
            explicit_extensions: false,
            mode: Target::Dev,
            scope: None,
            core_module: None,
            strip_exports: None,
            strip_ctx_name: None,
            strip_event_handlers: false,
            reg_ctx_name: None,
            is_server: None,
        })
        .unwrap()
    }

    /// Test InlineStrategy groups all segments into "entry_segments"
    #[test]
    fn test_entry_strategy_inline() {
        let code = r#"
            import { component$ } from "@qwik.dev/core";
            export const Counter = component$(() => {
                return <button onClick$={() => console.log("click")}>Click</button>;
            });
        "#;

        let result = transform_with_strategy(code, EntryStrategy::Inline);

        // Verify all segments have entry = Some("entry_segments")
        let segments: Vec<_> = result
            .modules
            .iter()
            .filter_map(|m| m.segment.as_ref())
            .collect();

        assert!(!segments.is_empty(), "Should have segments");
        for segment in segments {
            assert_eq!(
                segment.entry,
                Some("entry_segments".to_string()),
                "InlineStrategy should group all to entry_segments, got {:?}",
                segment.entry
            );
        }
    }

    /// Test SingleStrategy groups all segments into "entry_segments"
    #[test]
    fn test_entry_strategy_single() {
        let code = r#"
            import { component$ } from "@qwik.dev/core";
            export const Counter = component$(() => {
                return <button onClick$={() => console.log("click")}>Click</button>;
            });
        "#;

        let result = transform_with_strategy(code, EntryStrategy::Single);

        // Verify all segments have entry = Some("entry_segments")
        let segments: Vec<_> = result
            .modules
            .iter()
            .filter_map(|m| m.segment.as_ref())
            .collect();

        assert!(!segments.is_empty(), "Should have segments");
        for segment in segments {
            assert_eq!(
                segment.entry,
                Some("entry_segments".to_string()),
                "SingleStrategy should group all to entry_segments, got {:?}",
                segment.entry
            );
        }
    }

    /// Test PerSegmentStrategy creates separate files (entry = None)
    #[test]
    fn test_entry_strategy_segment() {
        let code = r#"
            import { component$ } from "@qwik.dev/core";
            export const Counter = component$(() => {
                return <button onClick$={() => console.log("click")}>Click</button>;
            });
        "#;

        let result = transform_with_strategy(code, EntryStrategy::Segment);

        // Verify all segments have entry = None (separate files)
        let segments: Vec<_> = result
            .modules
            .iter()
            .filter_map(|m| m.segment.as_ref())
            .collect();

        assert!(!segments.is_empty(), "Should have segments");
        for segment in segments {
            assert_eq!(
                segment.entry, None,
                "PerSegmentStrategy should produce separate files (None), got {:?}",
                segment.entry
            );
        }
    }

    /// Test HookStrategy (alias for PerSegment) creates separate files
    #[test]
    fn test_entry_strategy_hook() {
        let code = r#"
            import { component$ } from "@qwik.dev/core";
            export const Counter = component$(() => {
                return <button onClick$={() => console.log("click")}>Click</button>;
            });
        "#;

        let result = transform_with_strategy(code, EntryStrategy::Hook);

        // Verify all segments have entry = None (separate files)
        let segments: Vec<_> = result
            .modules
            .iter()
            .filter_map(|m| m.segment.as_ref())
            .collect();

        assert!(!segments.is_empty(), "Should have segments");
        for segment in segments {
            assert_eq!(
                segment.entry, None,
                "HookStrategy should produce separate files (None), got {:?}",
                segment.entry
            );
        }
    }

    /// Test PerComponentStrategy groups by component name
    #[test]
    fn test_entry_strategy_component() {
        let code = r#"
            import { component$ } from "@qwik.dev/core";
            export const Counter = component$(() => {
                return <button onClick$={() => console.log("click")}>Click</button>;
            });
        "#;

        let result = transform_with_strategy(code, EntryStrategy::Component);

        // Verify segments have entry = Some("{origin}_entry_{component}")
        let segments: Vec<_> = result
            .modules
            .iter()
            .filter_map(|m| m.segment.as_ref())
            .collect();

        assert!(!segments.is_empty(), "Should have segments");
        for segment in segments {
            assert!(
                segment.entry.is_some(),
                "PerComponentStrategy should have entry value"
            );
            let entry = segment.entry.as_ref().unwrap();
            assert!(
                entry.contains("_entry_"),
                "PerComponentStrategy entry should contain '_entry_', got {}",
                entry
            );
        }
    }

    /// Test SmartStrategy behavior for component$ segments
    /// Note: JSX event handlers (onClick$={() => ...}) don't produce separate segment files
    /// in this implementation - they're inlined QRLs. Only component$() calls produce segments.
    #[test]
    fn test_entry_strategy_smart() {
        let code = r#"
            import { component$ } from "@qwik.dev/core";
            export const Counter = component$(() => {
                const count = 0;
                return (
                    <div>
                        <button onClick$={() => console.log("no capture")}>A</button>
                        <button onClick$={() => console.log(count)}>B</button>
                    </div>
                );
            });
        "#;

        let result = transform_with_strategy(code, EntryStrategy::Smart);

        let segments: Vec<_> = result
            .modules
            .iter()
            .filter_map(|m| m.segment.as_ref())
            .collect();

        // Only the component$ call produces a segment
        assert!(!segments.is_empty(), "Should have segments");

        // SmartStrategy for component$ segments:
        // - component$ functions with context -> entry = Some(grouped by component)
        // - JSX event handlers are inlined and don't produce segments
        for segment in &segments {
            // component$ with context gets grouped
            if segment.ctx_name.contains("component") {
                assert!(
                    segment.entry.is_some(),
                    "component$ segment should have grouped entry"
                );
            }
        }
    }

    /// Test SmartStrategy with multiple components
    /// Each component$ produces a segment, grouped by its component context
    #[test]
    fn test_entry_strategy_smart_multiple_components() {
        let code = r#"
            import { component$ } from "@qwik.dev/core";

            export const CompA = component$(() => {
                const stateA = "A";
                return <button onClick$={() => console.log(stateA)}>A</button>;
            });

            export const CompB = component$(() => {
                return <button onClick$={() => console.log("stateless")}>B</button>;
            });
        "#;

        let result = transform_with_strategy(code, EntryStrategy::Smart);

        let segments: Vec<_> = result
            .modules
            .iter()
            .filter_map(|m| m.segment.as_ref())
            .collect();

        // Should have 2 segments (one per component$)
        assert_eq!(
            segments.len(), 2,
            "Should have 2 segments (one per component), got {}",
            segments.len()
        );

        // Both component$ calls produce grouped entries (with component context)
        for segment in &segments {
            assert!(
                segment.entry.is_some(),
                "component$ segment {} should have grouped entry",
                segment.name
            );
            let entry = segment.entry.as_ref().unwrap();
            assert!(
                entry.contains("_entry_"),
                "Entry should contain component grouping, got {}",
                entry
            );
        }
    }

    /// Test SmartStrategy with named QRL (has context from variable name)
    /// Named QRLs get grouped by their variable name context
    #[test]
    fn test_entry_strategy_smart_named_qrl() {
        let code = r#"
            import { $ } from "@qwik.dev/core";
            export const handler = $(() => console.log("named qrl"));
        "#;

        let result = transform_with_strategy(code, EntryStrategy::Smart);

        let segments: Vec<_> = result
            .modules
            .iter()
            .filter_map(|m| m.segment.as_ref())
            .collect();

        assert_eq!(segments.len(), 1, "Should have 1 segment");

        // Named QRL has context from variable name "handler"
        // SmartStrategy groups QRLs with context
        assert!(
            segments[0].entry.is_some(),
            "Named QRL should have grouped entry (context from variable name)"
        );
        let entry = segments[0].entry.as_ref().unwrap();
        assert!(
            entry.contains("_entry_handler"),
            "Entry should reference the variable name, got {}",
            entry
        );
    }

    /// Test that SmartStrategy behavior matches PerComponentStrategy for component$ QRLs
    #[test]
    fn test_entry_strategy_smart_vs_component() {
        let code = r#"
            import { component$ } from "@qwik.dev/core";
            export const Counter = component$(() => {
                return <div>Hello</div>;
            });
        "#;

        let smart_result = transform_with_strategy(code, EntryStrategy::Smart);
        let component_result = transform_with_strategy(code, EntryStrategy::Component);

        let smart_segments: Vec<_> = smart_result
            .modules
            .iter()
            .filter_map(|m| m.segment.as_ref())
            .collect();

        let component_segments: Vec<_> = component_result
            .modules
            .iter()
            .filter_map(|m| m.segment.as_ref())
            .collect();

        assert_eq!(smart_segments.len(), component_segments.len(), "Same number of segments");

        // For component$ QRLs, both strategies should produce grouped entries
        // (Smart delegates to component grouping for non-stateless segments)
        for (smart, comp) in smart_segments.iter().zip(component_segments.iter()) {
            assert!(smart.entry.is_some(), "Smart entry should exist");
            assert!(comp.entry.is_some(), "Component entry should exist");
            assert_eq!(
                smart.entry, comp.entry,
                "Smart and Component strategies should produce same entry for component$ QRLs"
            );
        }
    }
}

// Defines the interface between qwik-optimizer and the qwik JS library.
// This includes some types from the original qwik project that are otherwise
// not used in this rewrite, but needed for compatibility.

use crate::entry_strategy::*;
use crate::error::Error;
use crate::package_json;
use crate::prelude::*;
use crate::source::Source;
use crate::transform::*;

use crate::component::*;
use serde::{Deserialize, Serialize};
use std::iter::Sum;

use std::cmp::Ordering;
use std::fs;
use std::path::Path;
use std::str;

#[derive(Debug, Deserialize, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MinifyMode {
    Simplify,
    None,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformFsOptions {
    pub src_dir: String,
    pub root_dir: Option<String>,
    pub vendor_roots: Vec<String>,
    pub glob: Option<String>,
    pub minify: MinifyMode,
    pub entry_strategy: EntryStrategy,
    pub source_maps: bool,
    pub transpile_ts: bool,
    pub transpile_jsx: bool,
    pub preserve_filenames: bool,
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

//const BUILDER_IO_QWIK: &str = "@builder.io/qwik";

pub fn transform_fs(config: TransformFsOptions) -> Result<TransformOutput> {
    //let core_module = config.core_module.unwrap_or(BUILDER_IO_QWIK.to_string());
    let src_dir = Path::new(&config.src_dir);
    //let root_dir = config.root_dir.as_ref().map(Path::new);

    let mut paths = vec![];
    //let entry_policy = &*parse_entry_strategy(&config.entry_strategy);
    package_json::find_modules(src_dir, config.vendor_roots, &mut paths)?;

    let mut counter: u64 = 0; // TODO: Determine order correctly
    let mut next_order = || {
        counter += 1;
        counter
    };

    let iterator = paths.iter();

    let mut final_output = iterator
        .map(|path| -> Result<Option<TransformOutput>> {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let relative_path = pathdiff::diff_paths(path, &config.src_dir)
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap();
            // TODO: Is there any use for the language variable here?
            let language = match ext {
                "ts" => {
                    if config.transpile_ts {
                        Language::Typescript
                    } else {
                        println!("AAAA - Skipping disabled TS file: {relative_path}");
                        return Ok(None);
                    }
                }
                "tsx" => {
                    if config.transpile_ts && config.transpile_jsx {
                        Language::Typescript
                    } else {
                        println!("AAAA - Skipping disabled TSX file: {relative_path}");
                        return Ok(None);
                    }
                }
                "js" => Language::Javascript,
                "jsx" => {
                    if config.transpile_jsx {
                        Language::Javascript
                    } else {
                        println!("AAAA - Skipping disabled JSX file: {relative_path}");
                        return Ok(None);
                    }
                }
                _ => {
                    println!("AAAA - Skipping file with unrecognized extension: {relative_path}");
                    return Ok(None);
                }
            };
            println!("AAAA - Transforming file {relative_path}");
            let r = transform(
                Source::from_file(path)?,
                TransformOptions {
                    minify: match config.minify {
                        MinifyMode::Simplify => true,
                        MinifyMode::None => false,
                    },
                    target: config.mode,
                },
            )?;
            let mut modules = vec![TransformModule {
                path: relative_path.clone(),
                code: r.body,
                map: None,
                segment: None,
                is_entry: false,
                order: next_order(),
            }];
            modules.extend(r.components.into_iter().map(|c| TransformModule {
                path: relative_path.clone(),
                code: c.code,
                map: None,
                segment: None,
                is_entry: true,
                order: next_order(),
            }));
            Ok(Some(TransformOutput {
                modules,
                diagnostics: vec![],                 // TODO: Collect diagnostics
                is_type_script: config.transpile_ts, // TODO: Set this flag correctly
                is_jsx: config.transpile_jsx,        // TODO: Set this flag correctly
            }))
        })
        .sum::<Result<Option<TransformOutput>>>()?
        .unwrap_or(TransformOutput::default());

    final_output.modules.sort_unstable_by_key(|key| key.order);

    Ok(final_output)
}

pub fn transform_modules(config: TransformModulesOptions) -> Result<TransformOutput> {
    Err(Error::Generic("Not yet implemented".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_project_1() {
        // This should be a macro eventually
        let func_name = function_name!();
        let path = PathBuf::from("./src/test_input").join(func_name);

        println!(
            "Loading test input project directory from path: {:?}",
            &path
        );

        let result = transform_fs(TransformFsOptions {
            src_dir: path.clone().into_os_string().into_string().unwrap(),
            root_dir: Some(path.clone().into_os_string().into_string().unwrap()),
            vendor_roots: vec![],
            glob: None,
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
}

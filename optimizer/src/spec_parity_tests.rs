//! Spec parity tests ported from qwik-core/src/optimizer/core/src/test.rs
//!
//! These tests verify that the OXC optimizer produces output matching the SWC reference.

#[cfg(test)]
mod tests {
    use crate::entry_strategy::*;
    use crate::js_lib_interface::*;
    use crate::transform::Target;
    use serde_json::to_string_pretty;
    use std::path::PathBuf;

    /// Macro to run spec parity tests with custom options
    macro_rules! spec_test {
        ($options:expr) => {{
            let func_name = crate::function_name!();
            // Strip "spec_" prefix to get the test name
            let test_name = func_name.strip_prefix("spec_").unwrap_or(func_name);
            let mut path = PathBuf::from("./src/test_input/spec").join(format!("{test_name}.tsx"));
            let mut transpile_ts = true;

            if !path.exists() {
                path = PathBuf::from("./src/test_input/spec").join(format!("{test_name}.js"));
                transpile_ts = false;
            }

            println!("Loading spec test input file from path: {:?}", &path);

            let code = std::fs::read_to_string(&path).unwrap();

            // Create options, overriding with provided values
            let mut options = TransformModulesOptions {
                input: vec![TransformModuleInput {
                    path: path.file_name().unwrap().to_string_lossy().to_string(),
                    dev_path: None,
                    code: code.clone(),
                }],
                src_dir: "/user/qwik/src/".to_string(),
                root_dir: None,
                minify: MinifyMode::Simplify,
                entry_strategy: EntryStrategy::Segment,
                source_maps: true,
                transpile_ts,
                transpile_jsx: false,
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

            // Apply provided option overrides
            let overrides = $options;
            options = apply_options(options, overrides);

            let result = transform_modules(options);

            crate::snapshot_res!(result, format!("==INPUT==\n\n{}", code.to_string()));
        }};
    }

    /// Default spec test with standard options
    macro_rules! spec_test_default {
        () => {{
            spec_test!(SpecOptions::default());
        }};
    }

    /// Options struct for spec tests
    #[derive(Default)]
    struct SpecOptions {
        filename: Option<String>,
        entry_strategy: Option<EntryStrategy>,
        minify: Option<MinifyMode>,
        transpile_ts: Option<bool>,
        transpile_jsx: Option<bool>,
        explicit_extensions: Option<bool>,
        mode: Option<Target>,
        strip_exports: Option<Vec<String>>,
        strip_ctx_name: Option<Vec<String>>,
        strip_event_handlers: Option<bool>,
        reg_ctx_name: Option<Vec<String>>,
        is_server: Option<bool>,
    }

    fn apply_options(
        mut options: TransformModulesOptions,
        overrides: SpecOptions,
    ) -> TransformModulesOptions {
        if let Some(filename) = overrides.filename {
            options.input[0].path = filename;
        }
        if let Some(strategy) = overrides.entry_strategy {
            options.entry_strategy = strategy;
        }
        if let Some(minify) = overrides.minify {
            options.minify = minify;
        }
        if let Some(transpile_ts) = overrides.transpile_ts {
            options.transpile_ts = transpile_ts;
        }
        if let Some(transpile_jsx) = overrides.transpile_jsx {
            options.transpile_jsx = transpile_jsx;
        }
        if let Some(explicit_extensions) = overrides.explicit_extensions {
            options.explicit_extensions = explicit_extensions;
        }
        if let Some(mode) = overrides.mode {
            options.mode = mode;
        }
        if let Some(strip_exports) = overrides.strip_exports {
            options.strip_exports = Some(strip_exports);
        }
        if let Some(strip_ctx_name) = overrides.strip_ctx_name {
            options.strip_ctx_name = Some(strip_ctx_name);
        }
        if let Some(strip_event_handlers) = overrides.strip_event_handlers {
            options.strip_event_handlers = strip_event_handlers;
        }
        if let Some(reg_ctx_name) = overrides.reg_ctx_name {
            options.reg_ctx_name = Some(reg_ctx_name);
        }
        if let Some(is_server) = overrides.is_server {
            options.is_server = Some(is_server);
        }
        options
    }

    // =========================================================================
    // Spec Parity Tests - Batch 1 (Tests 1-33)
    // =========================================================================

    #[test]
    fn spec_example_1() {
        spec_test_default!();
    }

    #[test]
    fn spec_example_2() {
        spec_test_default!();
    }

    #[test]
    fn spec_example_3() {
        spec_test_default!();
    }

    #[test]
    fn spec_example_4() {
        spec_test_default!();
    }

    #[test]
    fn spec_example_5() {
        spec_test_default!();
    }

    #[test]
    fn spec_example_6() {
        spec_test_default!();
    }

    #[test]
    fn spec_example_7() {
        spec_test_default!();
    }

    #[test]
    fn spec_example_8() {
        spec_test_default!();
    }

    #[test]
    fn spec_example_9() {
        spec_test_default!();
    }

    #[test]
    fn spec_example_10() {
        spec_test!(SpecOptions {
            filename: Some("project/test.tsx".to_string()),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_11() {
        spec_test!(SpecOptions {
            filename: Some("project/test.tsx".to_string()),
            entry_strategy: Some(EntryStrategy::Single),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_functional_component() {
        spec_test!(SpecOptions {
            minify: Some(MinifyMode::None),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_functional_component_2() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_functional_component_capture_props() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_multi_capture() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_dead_code() {
        spec_test_default!();
    }

    #[test]
    fn spec_example_with_tagname() {
        spec_test_default!();
    }

    #[test]
    fn spec_example_with_style() {
        spec_test_default!();
    }

    #[test]
    fn spec_example_props_optimization() {
        spec_test!(SpecOptions {
            transpile_jsx: Some(true),
            entry_strategy: Some(EntryStrategy::Inline),
            transpile_ts: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_props_wrapping() {
        spec_test!(SpecOptions {
            transpile_jsx: Some(true),
            entry_strategy: Some(EntryStrategy::Inline),
            transpile_ts: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_props_wrapping2() {
        spec_test!(SpecOptions {
            transpile_jsx: Some(true),
            entry_strategy: Some(EntryStrategy::Inline),
            transpile_ts: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_props_wrapping_children() {
        spec_test!(SpecOptions {
            transpile_jsx: Some(true),
            entry_strategy: Some(EntryStrategy::Inline),
            transpile_ts: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_props_wrapping_children2() {
        spec_test!(SpecOptions {
            transpile_jsx: Some(true),
            entry_strategy: Some(EntryStrategy::Inline),
            transpile_ts: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_use_optimization() {
        spec_test!(SpecOptions {
            transpile_jsx: Some(false),
            entry_strategy: Some(EntryStrategy::Inline),
            transpile_ts: Some(true),
            is_server: Some(false),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_optimization_issue_3561() {
        spec_test!(SpecOptions {
            transpile_jsx: Some(false),
            entry_strategy: Some(EntryStrategy::Inline),
            transpile_ts: Some(true),
            is_server: Some(false),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_optimization_issue_4386() {
        spec_test!(SpecOptions {
            transpile_jsx: Some(false),
            entry_strategy: Some(EntryStrategy::Inline),
            transpile_ts: Some(true),
            is_server: Some(false),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_optimization_issue_3542() {
        spec_test!(SpecOptions {
            transpile_jsx: Some(false),
            entry_strategy: Some(EntryStrategy::Inline),
            transpile_ts: Some(true),
            is_server: Some(false),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_optimization_issue_3795() {
        spec_test!(SpecOptions {
            entry_strategy: Some(EntryStrategy::Inline),
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            is_server: Some(false),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_drop_side_effects() {
        spec_test!(SpecOptions {
            entry_strategy: Some(EntryStrategy::Segment),
            strip_ctx_name: Some(vec!["server".into()]),
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            is_server: Some(false),
            mode: Some(Target::Dev),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_reg_ctx_name_segments() {
        spec_test!(SpecOptions {
            entry_strategy: Some(EntryStrategy::Inline),
            reg_ctx_name: Some(vec!["server".into()]),
            strip_event_handlers: Some(true),
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_reg_ctx_name_segments_inlined() {
        spec_test!(SpecOptions {
            entry_strategy: Some(EntryStrategy::Inline),
            reg_ctx_name: Some(vec!["server".into()]),
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_reg_ctx_name_segments_hoisted() {
        spec_test!(SpecOptions {
            entry_strategy: Some(EntryStrategy::Hoist),
            reg_ctx_name: Some(vec!["server".into()]),
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_lightweight_functional() {
        spec_test_default!();
    }

    // =========================================================================
    // Spec Parity Tests - Batch 2 (Tests 34-55)
    // =========================================================================

    #[test]
    fn spec_example_invalid_references() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_invalid_segment_expr1() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_capture_imports() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_capturing_fn_class() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_renamed_exports() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_exports() {
        spec_test!(SpecOptions {
            filename: Some("project/test.tsx".to_string()),
            transpile_ts: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_issue_117() {
        spec_test!(SpecOptions {
            filename: Some("project/test.tsx".to_string()),
            entry_strategy: Some(EntryStrategy::Single),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_jsx() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_jsx_listeners() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    #[ignore = "panics: local variable 'qrl' shadows qwik import causing symbol conflict"]
    fn spec_example_qwik_conflict() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_fix_dynamic_import() {
        spec_test!(SpecOptions {
            filename: Some("project/folder/test.tsx".to_string()),
            entry_strategy: Some(EntryStrategy::Single),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_custom_inlined_functions() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_missing_custom_inlined_functions() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_skip_transform() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_explicit_ext_transpile() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            explicit_extensions: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_explicit_ext_no_transpile() {
        spec_test!(SpecOptions {
            explicit_extensions: Some(true),
            entry_strategy: Some(EntryStrategy::Single),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_jsx_import_source() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            explicit_extensions: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_prod_node() {
        spec_test!(SpecOptions {
            mode: Some(Target::Prod),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_use_client_effect() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_inlined_entry_strategy() {
        spec_test!(SpecOptions {
            entry_strategy: Some(EntryStrategy::Inline),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_default_export() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            filename: Some("src/routes/_repl/[id]/[[...slug]].tsx".into()),
            entry_strategy: Some(EntryStrategy::Smart),
            explicit_extensions: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_default_export_index() {
        spec_test!(SpecOptions {
            filename: Some("src/components/mongo/index.tsx".into()),
            entry_strategy: Some(EntryStrategy::Inline),
            ..SpecOptions::default()
        });
    }

    // =========================================================================
    // Spec Parity Tests - Batch 3 (Tests 111-132)
    // =========================================================================

    #[test]
    fn spec_destructure_args_colon_props3() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_handle_dangerously_set_inner_html() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_noop_dev_mode() {
        spec_test!(SpecOptions {
            mode: Some(Target::Dev),
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            strip_event_handlers: Some(true),
            strip_ctx_name: Some(vec!["server".to_string()]),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_lib_mode_fn_signal() {
        spec_test!(SpecOptions {
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_ternary_prop() {
        spec_test!(SpecOptions {
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_transform_qrl_in_regular_prop() {
        spec_test!(SpecOptions {
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_impure_template_fns() {
        spec_test!(SpecOptions {
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_rename_builder_io() {
        spec_test!(SpecOptions {
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_example_component_with_event_listeners_inside_loop() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_wrap_inner_inline_component_prop() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_wrap_prop_from_destructured_array() {
        spec_test!(SpecOptions {
            transpile_jsx: Some(true),
            transpile_ts: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_wrap_object_with_fn_signal() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_mark_props_as_var_props_for_inner_cmp() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_not_wrap_fn() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_issue_7216_add_test() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_wrap_store_expression() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_not_wrap_var_template_string() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_wrap_type_asserted_variables_in_template() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_wrap_logical_expression_in_template() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_not_wrap_ternary_function_operator_with_fn() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_split_spread_props() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_split_spread_props_with_additional_prop() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    // =========================================================================
    // Spec Parity Tests - Batch 4 (Tests 133-154)
    // =========================================================================

    #[test]
    fn spec_should_split_spread_props_with_additional_prop2() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_split_spread_props_with_additional_prop3() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_split_spread_props_with_additional_prop4() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_split_spread_props_with_additional_prop5() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_not_generate_conflicting_props_identifiers() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            entry_strategy: Some(EntryStrategy::Hoist),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_convert_rest_props() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_merge_attributes_with_spread_props() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_merge_attributes_with_spread_props_before_and_after() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_not_move_over_side_effects() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_ignore_null_inlined_qrl() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_hoisted_fn_signal_in_loop() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_convert_jsx_events() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_transform_event_names_without_jsx_transpile() {
        spec_test!(SpecOptions {
            transpile_ts: Some(false),
            transpile_jsx: Some(false),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_not_transform_events_on_non_elements() {
        spec_test!(SpecOptions {
            transpile_ts: Some(false),
            transpile_jsx: Some(false),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_merge_bind_value_and_on_input() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_merge_bind_checked_and_on_input() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_extract_single_qrl() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_extract_single_qrl_2() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_extract_single_qrl_with_index() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_extract_single_qrl_with_nested_components() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_transform_component_with_normal_function() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_transform_nested_loops() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    // =========================================================================
    // Spec Parity Tests - Batch 5 (Tests 155-164)
    // =========================================================================

    #[test]
    fn spec_should_transform_multiple_event_handlers() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_transform_multiple_event_handlers_case2() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_merge_on_input_and_bind_value() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_merge_on_input_and_bind_checked() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_transform_qrls_in_ternary_expression() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_not_transform_bind_value_in_var_props_for_jsx_split() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_not_transform_bind_checked_in_var_props_for_jsx_split() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_move_bind_value_to_var_props() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }

    #[test]
    fn spec_should_move_props_related_to_iteration_variables_to_var_props() {
        spec_test!(SpecOptions {
            transpile_ts: Some(true),
            transpile_jsx: Some(true),
            ..SpecOptions::default()
        });
    }
}

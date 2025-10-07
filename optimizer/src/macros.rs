#[macro_export]
macro_rules! function_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        // Remove "::f" suffix (6 chars) and everything before the last "::"
        let name = &name[..name.len() - 3];
        name.rsplit("::").next().unwrap_or(name)
    }};
}

#[macro_export]
macro_rules! snapshot_res {
    ($res: expr, $prefix: expr) => {
        match $res {
            Ok(v) => {
                let mut output: String = $prefix;

                for module in &v.modules {
                    let is_entry = if module.is_entry { "(ENTRY POINT)" } else { "" };
                    output += format!(
                        "\n============================= {} {}==\n\n{}\n\n{:?}",
                        module.path, is_entry, module.code, module.map
                    )
                    .as_str();
                    if let Some(segment) = &module.segment {
                        let segment = to_string_pretty(&segment).unwrap();
                        output += &format!("\n/*\n{}\n*/", segment);
                    }
                }
                output += format!(
                    "\n== DIAGNOSTICS ==\n\n{}",
                    to_string_pretty(&v.diagnostics).unwrap()
                )
                .as_str();
                insta::assert_snapshot!(output);
            }
            Err(err) => {
                insta::assert_snapshot!(err);
            }
        }
    };
}

#[macro_export]
macro_rules! _assert_valid_transform {
    ($debug:literal, $entry_strategy:expr) => {{
        let func_name = function_name!();
        let mut path = PathBuf::from("./src/test_input").join(format!("{func_name}.tsx"));
        let mut transpile_ts = true;

        if !path.exists() {
            path = PathBuf::from("./src/test_input").join(format!("{func_name}.js"));
            transpile_ts = false;
        }

        println!("Loading test input file from path: {:?}", &path);

        let code = std::fs::read_to_string(&path).unwrap();
        let options = TransformModulesOptions {
            input: vec![TransformModuleInput {
                path: path.file_name().unwrap().to_string_lossy().to_string(),
                dev_path: None,
                code: code.clone(),
            }],
            src_dir: ".".to_string(),
            root_dir: None,
            minify: MinifyMode::None,
            entry_strategy: $entry_strategy,
            source_maps: true,
            transpile_ts,
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

        let result = transform_modules(options);

        if $debug == true {
            println!("{:?}", result);
        }

        snapshot_res!(result, format!("==INPUT==\n\n{}", code.to_string()));
    }};
}

#[macro_export]
macro_rules! assert_valid_transform {
    ($options:expr) => {{
        _assert_valid_transform!(false, $options);
    }};
}

#[macro_export]
macro_rules! assert_valid_transform_debug {
    ($options:expr) => {{
        _assert_valid_transform!(true, $options);
    }};
}

#[macro_export]
macro_rules! assert_processing_errors {
    ($verifier:expr) => {{
        let func_name = function_name!();
        let mut path = PathBuf::from("./src/test_input").join(format!("{func_name}.tsx"));
        let mut lang = crate::component::Language::Typescript;

        if !path.exists() {
            path = PathBuf::from("./src/test_input").join(format!("{func_name}.js"));
            lang = crate::component::Language::Javascript;
        }

        println!("Loading test input file from path: {:?}", &path);

        let source_code = std::fs::read_to_string(&path).unwrap();

        let source_input =
            Source::from_source(source_code, lang, Some("test".to_string())).unwrap();
        let errors: Vec<ProcessingFailure> = transform(source_input, TransformOptions::default())
            .unwrap()
            .errors;

        ($verifier)(errors)
    }};
}

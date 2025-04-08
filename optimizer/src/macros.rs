
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
macro_rules! _assert_valid_transform {
    ($input:literal) => {{
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
        let result = transform(source_input, TransformOptions::default()).unwrap().optimized_app;

        if $input == true {
            println!("{}", result);
        }

        let body_snap_name = format!("{}_body", func_name);

        // Return a clone of the cached result
        insta::assert_yaml_snapshot!(body_snap_name, result.body);

        for comp in result.components {
            let comp_snap_name = format!("{}_{}", func_name, comp.id.symbol_name);
            insta::assert_yaml_snapshot!(comp_snap_name, comp.code);
        }
    }};
}

#[macro_export]
macro_rules! assert_valid_transform {
    () => {{
        _assert_valid_transform!(false);
    }};
}

#[macro_export]
macro_rules! assert_valid_transform_debug {
    () => {{
        _assert_valid_transform!(true);
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
        let errors: Vec<ProcessingFailure> = transform(source_input).unwrap().errors;

        ($verifier)(errors)
    }};
}

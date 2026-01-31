# Testing Patterns

**Analysis Date:** 2026-01-29

## Test Framework

**Runner:**
- Rust built-in test harness (no external test framework)
- Tests invoked with `cargo test` (standard Rust testing)
- Snapshot testing via `insta` crate v1.42.1 with YAML support

**Assertion Library:**
- `assert_eq!`, `assert_ne!` macros from standard library
- Insta snapshot assertions via `insta::assert_snapshot!()` and `insta::assert_yaml_snapshot!()`

**Run Commands:**
```bash
cargo test                  # Run all tests in optimizer workspace
cargo test --lib           # Run library tests only
cargo test --test <name>   # Run specific test
cargo test -- --nocapture # Run with output visible
cargo test -- --show-output # Alternative output display
```

## Test File Organization

**Location:**
- Co-located with source code in same files
- Module-level tests in `#[cfg(test)] mod tests { ... }` blocks
- Integration tests via snapshot inputs in `src/test_input/` directory
- Snapshots stored in `src/snapshots/` with `.snap` file extension

**Naming:**
- Module tests: `test_<descriptor>()` pattern
- Examples: `test_example_1()`, `test_example_2()`, `test_calculate_hash()`, `test_project_1()`
- Test input files: `test_example_<N>.<ext>` (tsx/js) and `test_project_1/` directory

**Structure:**
```
optimizer/src/
├── component/
│   └── id.rs
│       └── #[cfg(test)]
│           └── mod tests
│               ├── #[test] fn test_1()
│               ├── #[test] fn test_2()
│               └── ...
├── js_lib_interface.rs
│   └── #[cfg(test)]
│       └── mod tests
│           ├── #[test] fn test_example_1()
│           ├── #[test] fn test_example_2()
│           └── ...
├── test_input/
│   ├── test_example_1.tsx
│   ├── test_example_2.tsx
│   ├── ...
│   └── test_project_1/
│       ├── src/
│       └── Cargo.toml
└── snapshots/
    ├── qwik_optimizer__component__id__tests__escapes_a_name.snap
    ├── qwik_optimizer__js_lib_interface__tests__example_1.snap
    └── ...
```

## Test Structure

**Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_hash() {
        let (_, hash0) = Id::calculate_hash("./app.js", "a_b_c", &None);
        let (_, hash1) = Id::calculate_hash("./app.js", "a_b_c", &Some("scope".to_string()));
        assert_eq!(hash0, "0RVAWYCCxyk");
        assert_ne!(hash1, hash0);
    }
}
```

**Patterns:**
- Setup via direct struct construction (no factories)
- Assertions via `assert_eq!`, `assert_ne!` macros
- Error cases tested via snapshot (expected_output)
- No teardown patterns detected (immutable test data)

## Mocking

**Framework:** No mocking framework detected

**Patterns:**
- Uses real file I/O in tests (`std::fs::read_to_string()`)
- Test input files in `src/test_input/` serve as fixtures
- Snapshot testing captures full output for comparison
- No trait mocks; uses concrete implementations

**What to Mock:**
- Not applicable; this codebase uses real file systems and snapshots

**What NOT to Mock:**
- File I/O is real; tests intentionally use actual files
- AST operations are real via `oxc_*` libraries
- Output is captured as snapshots rather than mocking I/O

## Fixtures and Factories

**Test Data:**
```rust
#[test]
fn creates_a_id() {
    let source_info0 = SourceInfo::new("app.js").unwrap();
    let id0 = Id::new(
        &source_info0,
        &vec![
            Segment::Named("a".to_string()),
            Segment::Named("b".to_string()),
            Segment::Named("c".to_string()),
        ],
        &Target::Dev,
        &Option::None,
    );

    let expected0 = Id {
        display_name: "app.js_a_b_c".to_string(),
        symbol_name: format!("a_b_c_{}", hash0),
        local_file_name: "app.js_a_b_c_tZuivXMgs2w".to_string(),
        hash: hash0,
        sort_order,
        scope: None,
    };

    assert_eq!(id0, expected0);
}
```

**Location:**
- Fixtures stored as files in `src/test_input/test_example_*.tsx` and `test_example_*.js`
- Expected outputs stored as snapshots in `src/snapshots/` (YAML format)
- Direct construction of complex types in-test for small examples

## Coverage

**Requirements:** Not enforced (no coverage configuration found in Cargo.toml)

**View Coverage:**
```bash
cargo tarpaulin --out Html  # If tarpaulin is installed
# or
rustup component add llvm-tools-embedded
RUSTFLAGS="-C instrument-coverage" cargo test
```

## Test Types

**Unit Tests:**
- Scope: Individual functions and methods
- Approach: Direct function calls with assertions
- Examples:
  - `escapes_a_name()`: Tests `Id::sanitize()` function
  - `test_calculate_hash()`: Tests hash calculation logic
  - `can_load_from_file()`: Tests `Source::from_file()` loading

**Integration Tests:**
- Scope: Full transformation pipeline
- Approach: Load fixture files, run `transform_modules()`, compare snapshots
- Examples:
  - `test_example_1()` through `test_example_8()`: Process example TSX/JSX files
  - `test_project_1()`: Process entire project directory with glob patterns
  - Snapshots verify entire output structure (code + metadata)

**Snapshot Tests:**
- Framework: `insta` crate with YAML serialization
- Format: Test input shown, followed by all output modules and diagnostics
- Strategy: Compare full serialized output (code + modules + diagnostics)
- Example snapshot includes:
  ```yaml
  ==INPUT==
  // Original source code

  ============================= ./test_example_1.tsx_renderHeader_component_l1SEbA0PBzg.js (ENTRY POINT)==

  // Generated code

  /*
  { metadata json }
  */

  == DIAGNOSTICS ==
  // Any errors or warnings
  ```

**E2E Tests:**
- Not explicitly structured as separate E2E suite
- Real file I/O tests approximate E2E (e.g., `test_project_1()`)

## Common Patterns

**Async Testing:**
```rust
// No async tests detected
// NAPI binding uses tokio::task::spawn_blocking for CPU work
// Tests are synchronous (no #[tokio::test] attributes)
```

**Error Testing:**
```rust
// Via snapshot comparison of error messages
// Disabled tests with comments explaining known issues:
// #[test]
// fn test_example_9() {
//     // Not removing:
//     // const decl8 = 1, decl9;
//     assert_valid_transform_debug!(EntryStrategy::Segment);
// }

// Error assertions via custom macro:
#[macro_export]
macro_rules! assert_processing_errors {
    (|errors: Vec<ProcessingFailure>| {
        assert_eq!(errors.len(), 2);
        if let ProcessingFailure::IllegalCode(IllegalCodeType::Function(_, Some(name))) = &errors[0] {
            assert_eq!(name, "hola");
        }
    });
}
```

**File-Based Tests:**
```rust
#[test]
fn test_project_1() {
    let func_name = function_name!();
    let path = PathBuf::from("./src/test_input").join(func_name);

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
        // ... config fields ...
    })
    .unwrap();

    insta::assert_yaml_snapshot!(func_name, result);
}
```

## Test Macros

**Helper Macros:**
```rust
#[macro_export]
macro_rules! function_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        let name = &name[..name.len() - 3];
        name.rsplit("::").next().unwrap_or(name)
    }};
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

        let code = std::fs::read_to_string(&path).unwrap();
        let options = TransformModulesOptions { /* ... */ };
        let result = transform_modules(options);

        if $debug == true {
            println!("{:?}", result);
        }

        snapshot_res!(result, format!("==INPUT==\n\n{}", code.to_string()));
    }};
}

#[macro_export]
macro_rules! assert_valid_transform {
    () => {
        _assert_valid_transform!(false, EntryStrategy::Segment)
    };
}

#[macro_export]
macro_rules! assert_valid_transform_debug {
    ($entry_strategy:expr) => {
        _assert_valid_transform!(true, $entry_strategy)
    };
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
                    ).as_str();
                    if let Some(segment) = &module.segment {
                        let segment = to_string_pretty(&segment).unwrap();
                        output += &format!("\n/*\n{}\n*/", segment);
                    }
                }
                insta::assert_snapshot!(output);
            }
            Err(err) => {
                insta::assert_snapshot!(err);
            }
        }
    };
}
```

**Usage:**
```rust
#[test]
fn test_example_1() {
    assert_valid_transform_debug!(EntryStrategy::Segment);
}

#[test]
fn test_example_2() {
    assert_valid_transform!(EntryStrategy::Segment);
}
```

## Test Status

**Test Count:** 30+ unit/integration tests

**Disabled Tests:** 2 tests have `#[test]` commented out:
- `test_example_9()`: Issue with variable declaration removal
- `test_example_10()`: Issue with constant expression handling

**Coverage Areas:**
- Component ID generation and hashing
- Name sanitization and escaping
- AST transformation and code generation
- Module splitting and segmentation
- File loading and path handling
- Full integration examples (TSX, JSX, TypeScript)
- Project-level transformations

---

*Testing analysis: 2026-01-29*

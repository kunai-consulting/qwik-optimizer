//! Snapshot Verification Test
//!
//! Compares OXC optimizer snapshots against qwik-core reference snapshots.
//! This test FAILS if snapshots don't match.
//!
//! # Usage
//! ```bash
//! cargo test --test snapshot_verify -- --nocapture
//! ```

use similar::TextDiff;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn oxc_snapshots_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/snapshots")
}

fn qwik_core_snapshots_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("qwik-core/src/snapshots")
}

fn extract_oxc_test_name(filename: &str) -> Option<&str> {
    const PREFIX: &str = "qwik_optimizer__spec_parity_tests__tests__spec_";
    const SUFFIX: &str = ".snap";

    if filename.starts_with(PREFIX) && filename.ends_with(SUFFIX) {
        Some(&filename[PREFIX.len()..filename.len() - SUFFIX.len()])
    } else {
        None
    }
}

fn oxc_to_qwik_core_filename(test_name: &str) -> String {
    format!("qwik_core__test__{}.snap", test_name)
}

/// Extract snapshot content after the ==INPUT== section.
/// This gets the actual transformed output, not the input code.
fn extract_output_content(content: &str) -> String {
    // Find the first entry point marker (actual output starts there)
    if let Some(pos) = content.find("(ENTRY POINT)==") {
        // Back up to find the start of this line (the === marker)
        let before = &content[..pos];
        if let Some(line_start) = before.rfind('\n') {
            return content[line_start + 1..].to_string();
        }
    }

    // Fallback: skip everything before first ========= after INPUT
    if let Some(input_pos) = content.find("==INPUT==") {
        let after_input = &content[input_pos..];
        if let Some(sep_pos) = after_input.find("\n==") {
            return after_input[sep_pos + 1..].to_string();
        }
    }

    content.to_string()
}

/// Minimal normalization - only strip insta metadata header, normalize line endings
fn normalize_for_comparison(content: &str) -> String {
    let content = content.replace("\r\n", "\n");

    // Strip insta header (lines before ==INPUT== or first ===)
    let content = if let Some(pos) = content.find("==INPUT==") {
        &content[pos..]
    } else if let Some(pos) = content.find("===") {
        &content[pos..]
    } else {
        &content
    };

    content.trim().to_string()
}

#[test]
fn verify_snapshots_match_qwik_core() {
    let oxc_dir = oxc_snapshots_dir();
    let qwik_core_dir = qwik_core_snapshots_dir();

    // Build map of qwik-core snapshots
    let mut qwik_core_snapshots: HashMap<String, PathBuf> = HashMap::new();
    for entry in fs::read_dir(&qwik_core_dir).expect("Failed to read qwik-core snapshots dir") {
        let entry = entry.expect("Failed to read entry");
        let filename = entry.file_name().to_string_lossy().to_string();
        if filename.ends_with(".snap") {
            qwik_core_snapshots.insert(filename, entry.path());
        }
    }

    let mut compared = 0;
    let mut matches = 0;
    let mut failures: Vec<(String, String)> = Vec::new();
    let mut oxc_only: Vec<String> = Vec::new();

    for entry in fs::read_dir(&oxc_dir).expect("Failed to read OXC snapshots dir") {
        let entry = entry.expect("Failed to read entry");
        let filename = entry.file_name().to_string_lossy().to_string();

        let test_name = match extract_oxc_test_name(&filename) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let qwik_core_filename = oxc_to_qwik_core_filename(&test_name);

        let qwik_core_path = match qwik_core_snapshots.get(&qwik_core_filename) {
            Some(path) => path,
            None => {
                oxc_only.push(test_name);
                continue;
            }
        };

        let oxc_content = fs::read_to_string(entry.path()).expect("Failed to read OXC snapshot");
        let qwik_content = fs::read_to_string(qwik_core_path).expect("Failed to read qwik-core snapshot");

        let oxc_normalized = normalize_for_comparison(&oxc_content);
        let qwik_normalized = normalize_for_comparison(&qwik_content);

        compared += 1;

        if oxc_normalized == qwik_normalized {
            matches += 1;
        } else {
            // Generate diff
            let diff = TextDiff::from_lines(&qwik_normalized, &oxc_normalized);
            let diff_str = diff
                .unified_diff()
                .context_radius(3)
                .header("qwik-core (expected)", "oxc (actual)")
                .to_string();
            failures.push((test_name, diff_str));
        }
    }

    // Print results
    println!("\n============================================================");
    println!("SNAPSHOT VERIFICATION RESULTS");
    println!("============================================================\n");
    println!("Compared: {}", compared);
    println!("Matches:  {}", matches);
    println!("Failures: {}", failures.len());
    println!("OXC-only: {} (skipped)", oxc_only.len());

    if !oxc_only.is_empty() {
        println!("\nOXC-only tests (no qwik-core equivalent):");
        for name in &oxc_only {
            println!("  - {}", name);
        }
    }

    if !failures.is_empty() {
        println!("\n============================================================");
        println!("FAILURES ({} snapshots differ)", failures.len());
        println!("============================================================\n");

        // Show first 5 failures in detail
        for (i, (name, diff)) in failures.iter().enumerate() {
            if i >= 5 {
                println!("\n... and {} more failures (run with --nocapture to see all)", failures.len() - 5);
                break;
            }
            println!("--- {} ---", name);
            // Truncate very long diffs
            if diff.len() > 3000 {
                println!("{}...\n[truncated, {} more chars]", &diff[..3000], diff.len() - 3000);
            } else {
                println!("{}", diff);
            }
            println!();
        }

        // List all failing test names
        println!("\nAll failing tests:");
        for (name, _) in &failures {
            println!("  - {}", name);
        }
    }

    // ACTUALLY FAIL if there are differences
    assert_eq!(
        failures.len(),
        0,
        "\n\n{} of {} snapshots do not match qwik-core!\nRun with --nocapture to see diffs.\n",
        failures.len(),
        compared
    );

    println!("\nSUCCESS: All {} snapshots match qwik-core!", compared);
}

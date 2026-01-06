//! Golden file integration tests for KDL → Hurl translation.
//!
//! These tests compare the translator output against expected `.hurl` files
//! to catch regressions and verify correctness.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use kdl::KdlDocument;
use pretty_assertions::assert_eq;

use hurl_adapters_lib::formats::kdl::translate_to_string;
use hurl_core::parser::parse_hurl_file;

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Translates a KDL fixture file to a Hurl string and validates the output.
fn translate_fixture(name: &str) -> String {
    let kdl_path = fixture_dir().join(format!("{name}.kdl"));
    let kdl_input =
        fs::read_to_string(&kdl_path).unwrap_or_else(|_| panic!("Failed to read {}", kdl_path.display()));
    let doc: KdlDocument = kdl_input.parse().expect("Failed to parse KDL");
    let hurl_output = translate_to_string(&doc).expect("Failed to translate");

    // Validate that the generated Hurl is parseable
    parse_hurl_file(&hurl_output).unwrap_or_else(|e| {
        panic!(
            "Generated Hurl for '{name}' is invalid:\n{e:?}\n\nGenerated output:\n{hurl_output}"
        )
    });

    hurl_output
}

/// Discovers all fixture names by scanning for .kdl files.
fn discover_fixtures() -> Vec<String> {
    fs::read_dir(fixture_dir())
        .expect("Failed to read fixtures directory")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension()? == "kdl" {
                path.file_stem()?.to_str().map(String::from)
            } else {
                None
            }
        })
        .collect()
}

/// Runs a golden file test by comparing translator output to expected .hurl file.
fn run_golden_test(name: &str) {
    let hurl_path = fixture_dir().join(format!("{name}.hurl"));
    let expected_hurl = fs::read_to_string(&hurl_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", hurl_path.display()));

    let actual_hurl = translate_fixture(name);

    assert_eq!(
        actual_hurl, expected_hurl,
        "Golden file mismatch for '{name}'"
    );
}

#[test]
fn test_crud_workflow() {
    run_golden_test("crud_workflow");
}

#[test]
fn test_auth_flow() {
    run_golden_test("auth_flow");
}

/// Helper test to generate golden files - run with:
/// `cargo test --test golden generate_golden -- --ignored --nocapture`
#[test]
#[ignore = "Only run manually to regenerate golden files"]
fn generate_golden_files() {
    for name in discover_fixtures() {
        let hurl_path = fixture_dir().join(format!("{name}.hurl"));
        let hurl_output = translate_fixture(&name);

        fs::write(&hurl_path, &hurl_output)
            .unwrap_or_else(|_| panic!("Failed to write {}", hurl_path.display()));

        println!("Generated: {}", hurl_path.display());
        println!("--- {name} ---\n{hurl_output}");
    }
}

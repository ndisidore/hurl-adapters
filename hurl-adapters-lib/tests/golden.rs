//! Golden file integration tests for KDL → Hurl translation.
//!
//! These tests compare the translator output against expected `.hurl` files
//! to catch regressions and verify correctness.

use std::fs;
use std::path::Path;

use kdl::KdlDocument;
use pretty_assertions::assert_eq;

use hurl_adapters_lib::formats::kdl::translate_to_string;

/// Runs a golden file test by comparing translator output to expected .hurl file.
fn run_golden_test(name: &str) {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");

    let kdl_path = fixture_dir.join(format!("{}.kdl", name));
    let hurl_path = fixture_dir.join(format!("{}.hurl", name));

    let kdl_input =
        fs::read_to_string(&kdl_path).unwrap_or_else(|_| panic!("Failed to read {:?}", kdl_path));

    let expected_hurl = fs::read_to_string(&hurl_path)
        .unwrap_or_else(|_| panic!("Failed to read {:?}", hurl_path));

    let doc: KdlDocument = kdl_input.parse().expect("Failed to parse KDL");
    let actual_hurl = translate_to_string(&doc).expect("Failed to translate");

    assert_eq!(
        actual_hurl, expected_hurl,
        "Golden file mismatch for '{}'",
        name
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
#[ignore]
fn generate_golden_files() {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let fixtures = ["crud_workflow", "auth_flow"];

    for name in fixtures {
        let kdl_path = fixture_dir.join(format!("{}.kdl", name));
        let hurl_path = fixture_dir.join(format!("{}.hurl", name));

        let kdl_input = fs::read_to_string(&kdl_path)
            .unwrap_or_else(|_| panic!("Failed to read {:?}", kdl_path));

        let doc: KdlDocument = kdl_input.parse().expect("Failed to parse KDL");
        let hurl_output = translate_to_string(&doc).expect("Failed to translate");

        fs::write(&hurl_path, &hurl_output)
            .unwrap_or_else(|_| panic!("Failed to write {:?}", hurl_path));

        println!("Generated: {:?}", hurl_path);
        println!("--- {} ---\n{}", name, hurl_output);
    }
}

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use maplibre_style_spec::validate::parse_and_validate_style;
use serde_json::Value;

/// Upstream reject-parity harness:
/// - expected validity comes from `*.output.json` emptiness
/// - actual validity is whether `parse_and_validate_style` accepts the document
///   (decode into the generated `MaplibreStyleSpecification` plus rules in `validate.rs`).
#[test]
fn upstream_style_spec_reject_parity() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root should exist")
        .to_path_buf();
    let fixtures_dir = workspace_root.join("upstream/test/integration/style-spec/tests");

    assert!(
        fixtures_dir.exists(),
        "fixture directory does not exist: {}",
        fixtures_dir.display()
    );

    // Keep this empty by default. Add fixture **filenames** here only for JS-only cases that
    // Rust intentionally does not model (see project notes on prototype pollution / parity).
    let skip_list: BTreeSet<&str> = BTreeSet::new();
    let mut examined = 0usize;
    let mut skipped = 0usize;
    let mut mismatches: Vec<String> = Vec::new();

    for input_path in list_input_fixtures(&fixtures_dir) {
        let file_name = input_path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("input fixture filename should be valid UTF-8");

        if skip_list.contains(file_name) {
            skipped += 1;
            continue;
        }

        examined += 1;
        let output_path = paired_output_path(&input_path);

        let input_json = fs::read_to_string(&input_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", input_path.display()));
        let output_json = fs::read_to_string(&output_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", output_path.display()));

        let expected_valid = expected_validity(&output_json, &output_path);
        let actual = parse_and_validate_style(&input_json);
        let actual_valid = actual.is_ok();

        if expected_valid != actual_valid {
            let err = actual.err().unwrap_or_default();
            mismatches.push(format!(
                "{} -> expected={}, actual={}{}",
                file_name,
                validity_word(expected_valid),
                validity_word(actual_valid),
                if err.is_empty() {
                    String::new()
                } else {
                    format!(", error={err}")
                }
            ));
        }
    }

    assert!(
        examined > 0,
        "no fixtures discovered under {}",
        fixtures_dir.display()
    );

    if !mismatches.is_empty() {
        panic!(
            "reject-parity mismatches: {} out of {} examined ({} skipped)\n{}",
            mismatches.len(),
            examined,
            skipped,
            mismatches.join("\n")
        );
    }
}

fn list_input_fixtures(fixtures_dir: &Path) -> Vec<PathBuf> {
    let mut fixtures = fs::read_dir(fixtures_dir)
        .unwrap_or_else(|e| panic!("failed to list {}: {e}", fixtures_dir.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|e| panic!("failed to read fixture dir entry: {e}"))
                .path()
        })
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.ends_with(".input.json"))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    fixtures.sort();
    fixtures
}

fn paired_output_path(input_path: &Path) -> PathBuf {
    let input_name = input_path
        .file_name()
        .and_then(|n| n.to_str())
        .expect("input fixture filename should be valid UTF-8");
    let output_name = input_name.replace(".input.json", ".output.json");
    input_path.with_file_name(output_name)
}

fn expected_validity(output_json: &str, output_path: &Path) -> bool {
    let output_value: Value = serde_json::from_str(output_json).unwrap_or_else(|e| {
        panic!(
            "failed to parse expected output {}: {e}",
            output_path.display()
        )
    });
    let output_entries = output_value.as_array().unwrap_or_else(|| {
        panic!(
            "expected output must be a JSON array: {}",
            output_path.display()
        )
    });
    output_entries.is_empty()
}

fn validity_word(valid: bool) -> &'static str {
    if valid { "valid" } else { "invalid" }
}

//! End-to-end check: a realistic style triggers unary simplifications across filters and paint.

use std::path::Path;

use maplibre_style_optimizer::{
    OptPasses, load_intermediate_spec_from_v8_path, optimize_style_json_value,
};

fn fixture_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn sample_mir() -> maplibre_style_spec::mir::MirSpec {
    let v8 = Path::new(env!("CARGO_MANIFEST_DIR")).join("../upstream/src/reference/v8.json");
    load_intermediate_spec_from_v8_path(&v8).expect("load v8.json")
}

#[test]
fn unary_simplification_fixture_matches_expected() {
    let before_path = fixture_dir().join("unary_simplification_style.json");
    let expected_path = fixture_dir().join("unary_simplification_style.expected.json");

    let mut value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&before_path).unwrap()).unwrap();
    let expected: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&expected_path).unwrap()).unwrap();

    optimize_style_json_value(
        &mut value,
        &sample_mir(),
        &OptPasses {
            simplify_unary: true,
            ..Default::default()
        },
    );

    assert_eq!(
        value, expected,
        "optimized style should match tests/fixtures/unary_simplification_style.expected.json"
    );
}

#[test]
fn unary_simplification_fixture_noop_when_disabled() {
    let before_path = fixture_dir().join("unary_simplification_style.json");
    let mut value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&before_path).unwrap()).unwrap();
    let original = value.clone();

    optimize_style_json_value(&mut value, &sample_mir(), &OptPasses::default());

    assert_eq!(value, original);
}

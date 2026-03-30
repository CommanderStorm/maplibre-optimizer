//! E2e tests for constant folding and stats-driven optimization passes.
#![expect(clippy::needless_pass_by_value, clippy::cast_sign_loss)]

use std::collections::BTreeMap;
use std::path::Path;

use insta::assert_yaml_snapshot;
use maplibre_style_optimizer::stats::{
    GeometryTypeStats, LayerStats, PropertyStats, SourceStats, TileStatistics,
};
use maplibre_style_optimizer::{
    OptPasses, load_intermediate_spec_from_v8_path, optimize_style_json_value,
    optimize_style_json_value_with_stats,
};
use maplibre_style_spec::mir::MirSpec;

fn sample_mir() -> MirSpec {
    let v8 = Path::new(env!("CARGO_MANIFEST_DIR")).join("../upstream/src/reference/v8.json");
    load_intermediate_spec_from_v8_path(&v8).expect("load v8.json")
}

fn fold_passes() -> OptPasses {
    OptPasses {
        constant_fold: true,
        constant_fold_stats: true,
        ..Default::default()
    }
}

fn simplify_passes() -> OptPasses {
    OptPasses {
        constant_fold: true,
        constant_fold_stats: true,
        simplify_unary: true,
        simplify_expressions: true,
        ..Default::default()
    }
}

fn make_stats(layer_name: &str, layer_stats: LayerStats) -> TileStatistics {
    let mut layers = BTreeMap::new();
    layers.insert(layer_name.to_string(), layer_stats);
    let mut sources = BTreeMap::new();
    sources.insert("src".to_string(), SourceStats { layers });
    TileStatistics {
        sources,
        sample_rate: 1.0,
    }
}

/// Minimal valid style wrapping a single layer with a filter.
fn style_with_filter(filter: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "version": 8,
        "sources": {"src": {"type": "vector", "url": "x"}},
        "layers": [{"id": "l", "type": "fill", "source": "src", "source-layer": "lyr", "filter": filter}]
    })
}

/// Minimal valid style wrapping a single layer with a paint property.
fn style_with_paint(prop: &str, value: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "version": 8,
        "sources": {"src": {"type": "vector", "url": "x"}},
        "layers": [{"id": "l", "type": "fill", "source": "src", "source-layer": "lyr", "paint": {prop: value}}]
    })
}

/// Minimal valid style wrapping a single layer with both a filter and a paint property.
fn style_with_filter_and_paint(
    filter: serde_json::Value,
    prop: &str,
    value: serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "version": 8,
        "sources": {"src": {"type": "vector", "url": "x"}},
        "layers": [{"id": "l", "type": "fill", "source": "src", "source-layer": "lyr", "filter": filter, "paint": {prop: value}}]
    })
}

/// Minimal valid style wrapping a single symbol layer with a layout property.
fn style_with_layout(prop: &str, value: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "version": 8,
        "sources": {"src": {"type": "vector", "url": "x"}},
        "layers": [{"id": "l", "type": "symbol", "source": "src", "source-layer": "lyr", "layout": {prop: value}}]
    })
}

// ── Redundant properties ─────────────────────────────────────────────────

#[test]
fn fold_get_with_redundant_properties() {
    let mir = sample_mir();
    let mut v = style_with_filter(serde_json::json!([
        "==",
        ["get", "name", ["properties"]],
        "v"
    ]));
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0], @r#"
    filter:
      - "=="
      - - get
        - name
      - v
    id: l
    source: src
    source-layer: lyr
    type: fill
    "#);
}

#[test]
fn fold_has_with_redundant_properties() {
    let mir = sample_mir();
    let mut v = style_with_filter(serde_json::json!(["has", "name", ["properties"]]));
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0], @r"
    filter:
      - has
      - name
    id: l
    source: src
    source-layer: lyr
    type: fill
    ");
}

// ── Boolean algebra ──────────────────────────────────────────────────────

#[test]
fn fold_empty_all_to_true() {
    let mir = sample_mir();
    let mut v = style_with_filter(serde_json::json!(["all"]));
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["filter"], @"true");
}

#[test]
fn fold_empty_any_to_false() {
    let mir = sample_mir();
    let mut v = style_with_filter(serde_json::json!(["any"]));
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["filter"], @"false");
}

#[test]
fn fold_literal_comparison() {
    let mir = sample_mir();
    let mut v = style_with_filter(serde_json::json!(["==", 2, 3]));
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["filter"], @"false");
}

#[test]
fn fold_pure_arithmetic() {
    let mir = sample_mir();
    let mut v = style_with_filter(serde_json::json!(["+", 1, 2]));
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["filter"], @"3");
}

#[test]
fn fold_string_concat() {
    let mir = sample_mir();
    let mut v = style_with_filter(serde_json::json!(["concat", "hello", " ", "world"]));
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["filter"], @"hello world");
}

// ── Stats: has fold ──────────────────────────────────────────────────────

#[test]
fn has_folds_false_when_property_absent() {
    let mir = sample_mir();
    let stats = make_stats(
        "lyr",
        LayerStats {
            total_features: 100,
            properties: BTreeMap::new(),
            ..Default::default()
        },
    );
    let mut v = style_with_filter(serde_json::json!(["has", "missing"]));
    optimize_style_json_value_with_stats(&mut v, &mir, &fold_passes(), Some(&stats));
    assert_yaml_snapshot!(v["layers"][0]["filter"], @"false");
}

#[test]
fn has_folds_true_when_always_present() {
    let mir = sample_mir();
    let stats = make_stats(
        "lyr",
        LayerStats {
            total_features: 100,
            properties: BTreeMap::from([(
                "name".to_string(),
                PropertyStats::String {
                    present_count: 100,
                    cardinality: 5,
                    value_counts: None,
                },
            )]),
            ..Default::default()
        },
    );
    let mut v = style_with_filter(serde_json::json!(["has", "name"]));
    optimize_style_json_value_with_stats(&mut v, &mir, &fold_passes(), Some(&stats));
    assert_yaml_snapshot!(v["layers"][0]["filter"], @"true");
}

#[test]
fn has_unchanged_when_partially_present() {
    let mir = sample_mir();
    let stats = make_stats(
        "lyr",
        LayerStats {
            total_features: 100,
            properties: BTreeMap::from([(
                "name".to_string(),
                PropertyStats::String {
                    present_count: 50,
                    cardinality: 3,
                    value_counts: None,
                },
            )]),
            ..Default::default()
        },
    );
    let mut v = style_with_filter(serde_json::json!(["has", "name"]));
    let original = v["layers"][0]["filter"].clone();
    optimize_style_json_value_with_stats(&mut v, &mir, &fold_passes(), Some(&stats));
    assert_eq!(v["layers"][0]["filter"], original);
}

// ── Stats: geometry-type fold ────────────────────────────────────────────

#[test]
fn geometry_type_eq_folds_true_when_only_type() {
    let mir = sample_mir();
    let stats = make_stats(
        "lyr",
        LayerStats {
            total_features: 100,
            geometry_types: GeometryTypeStats {
                point: 100,
                linestring: 0,
                polygon: 0,
                unknown: 0,
            },
            ..Default::default()
        },
    );
    // Use symbol layer — accepts any geometry, so stats determine the fold.
    let mut v = serde_json::json!({
        "version": 8,
        "sources": {"src": {"type": "vector", "url": "x"}},
        "layers": [{"id": "l", "type": "symbol", "source": "src", "source-layer": "lyr",
                     "filter": ["==", ["geometry-type"], "Point"]}]
    });
    optimize_style_json_value_with_stats(&mut v, &mir, &fold_passes(), Some(&stats));
    assert_yaml_snapshot!(v["layers"][0]["filter"], @"true");
}

#[test]
fn geometry_type_eq_folds_false_when_absent() {
    let mir = sample_mir();
    let stats = make_stats(
        "lyr",
        LayerStats {
            total_features: 100,
            geometry_types: GeometryTypeStats {
                point: 100,
                linestring: 0,
                polygon: 0,
                unknown: 0,
            },
            ..Default::default()
        },
    );
    // Use symbol layer — accepts any geometry, so stats determine the fold.
    let mut v = serde_json::json!({
        "version": 8,
        "sources": {"src": {"type": "vector", "url": "x"}},
        "layers": [{"id": "l", "type": "symbol", "source": "src", "source-layer": "lyr",
                     "filter": ["==", ["geometry-type"], "Polygon"]}]
    });
    optimize_style_json_value_with_stats(&mut v, &mir, &fold_passes(), Some(&stats));
    assert_yaml_snapshot!(v["layers"][0]["filter"], @"false");
}

// ── Stats: comparison fold ───────────────────────────────────────────────

fn rank_stats(min: i64, max: i64) -> TileStatistics {
    make_stats(
        "lyr",
        LayerStats {
            total_features: 100,
            properties: BTreeMap::from([(
                "rank".to_string(),
                PropertyStats::Integer {
                    present_count: 100,
                    min,
                    max,
                    cardinality: (max - min + 1) as u64,
                    value_counts: None,
                },
            )]),
            ..Default::default()
        },
    )
}

#[test]
fn comparison_lt_folds_false_when_min_ge_n() {
    let mir = sample_mir();
    let stats = rank_stats(5, 10);
    let mut v = style_with_filter(serde_json::json!(["<", ["get", "rank"], 3]));
    optimize_style_json_value_with_stats(&mut v, &mir, &fold_passes(), Some(&stats));
    assert_yaml_snapshot!(v["layers"][0]["filter"], @"false");
}

#[test]
fn comparison_gte_folds_true_when_min_ge_n() {
    let mir = sample_mir();
    let stats = rank_stats(5, 10);
    let mut v = style_with_filter(serde_json::json!([">=", ["get", "rank"], 3]));
    optimize_style_json_value_with_stats(&mut v, &mir, &fold_passes(), Some(&stats));
    assert_yaml_snapshot!(v["layers"][0]["filter"], @"true");
}

#[test]
fn comparison_eq_folds_false_when_out_of_range() {
    let mir = sample_mir();
    let stats = rank_stats(5, 10);
    let mut v = style_with_filter(serde_json::json!(["==", ["get", "rank"], 99]));
    optimize_style_json_value_with_stats(&mut v, &mir, &fold_passes(), Some(&stats));
    assert_yaml_snapshot!(v["layers"][0]["filter"], @"false");
}

// ── Stats: in pruning ────────────────────────────────────────────────────

fn class_stats(values: &[(&str, u64)]) -> TileStatistics {
    let total: u64 = values.iter().map(|(_, c)| c).sum();
    make_stats(
        "lyr",
        LayerStats {
            total_features: total,
            properties: BTreeMap::from([(
                "class".to_string(),
                PropertyStats::String {
                    present_count: total,
                    cardinality: values.len() as u64,
                    value_counts: Some(values.iter().map(|(k, v)| (k.to_string(), *v)).collect()),
                },
            )]),
            ..Default::default()
        },
    )
}

#[test]
fn in_prune_removes_dead_values() {
    let mir = sample_mir();
    let stats = class_stats(&[("lake", 60), ("river", 40)]);
    let mut v = style_with_filter(serde_json::json!([
        "in",
        ["get", "class"],
        ["literal", ["lake", "ocean", "sea"]]
    ]));
    optimize_style_json_value_with_stats(&mut v, &mir, &fold_passes(), Some(&stats));
    assert_yaml_snapshot!(v["layers"][0]["filter"], @r#"
    - "=="
    - - get
      - class
    - lake
    "#);
}

#[test]
fn in_prune_all_dead_folds_to_false() {
    let mir = sample_mir();
    let stats = class_stats(&[("lake", 60), ("river", 40)]);
    let mut v = style_with_filter(serde_json::json!([
        "in",
        ["get", "class"],
        ["literal", ["ocean", "sea"]]
    ]));
    optimize_style_json_value_with_stats(&mut v, &mir, &fold_passes(), Some(&stats));
    assert_yaml_snapshot!(v["layers"][0]["filter"], @"false");
}

// ── Stats: match pruning ─────────────────────────────────────────────────

#[test]
fn match_prune_removes_dead_arm() {
    let mir = sample_mir();
    let stats = class_stats(&[("lake", 60), ("river", 40)]);
    let mut v = style_with_paint(
        "fill-color",
        serde_json::json!([
            "match",
            ["get", "class"],
            "lake",
            "#00f",
            "ocean",
            "#009",
            "#ccc"
        ]),
    );
    optimize_style_json_value_with_stats(&mut v, &mir, &fold_passes(), Some(&stats));
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r##"
    fill-color:
      - match
      - - get
        - class
      - lake
      - "#00f"
      - "#ccc"
    "##);
}

// ── Stats: coalesce fold ─────────────────────────────────────────────────

#[test]
fn coalesce_truncates_after_always_present_arm() {
    let mir = sample_mir();
    let stats = make_stats(
        "lyr",
        LayerStats {
            total_features: 100,
            properties: BTreeMap::from([(
                "name".to_string(),
                PropertyStats::String {
                    present_count: 100,
                    cardinality: 10,
                    value_counts: None,
                },
            )]),
            ..Default::default()
        },
    );
    let mut v = style_with_layout(
        "text-field",
        serde_json::json!(["coalesce", ["get", "name"], ["get", "name_en"], "fallback"]),
    );
    optimize_style_json_value_with_stats(&mut v, &mir, &fold_passes(), Some(&stats));
    assert_yaml_snapshot!(v["layers"][0]["layout"], @r"
    text-field:
      - get
      - name
    ");
}

// ── Stats: data ramp pruning ─────────────────────────────────────────────

#[test]
fn step_prune_above_max() {
    let mir = sample_mir();
    let stats = rank_stats(1, 5);
    let mut v = style_with_paint(
        "fill-opacity",
        serde_json::json!(["step", ["get", "rank"], 0.1, 3, 0.5, 7, 0.9]),
    );
    optimize_style_json_value_with_stats(&mut v, &mir, &fold_passes(), Some(&stats));
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r"
    fill-opacity:
      - step
      - - get
        - rank
      - 0.1
      - 3
      - 0.5
    ");
}

#[test]
fn interpolate_prune_above_max() {
    let mir = sample_mir();
    let stats = rank_stats(1, 5);
    let mut v = style_with_paint(
        "fill-opacity",
        serde_json::json!([
            "interpolate",
            ["linear"],
            ["get", "rank"],
            1,
            0.5,
            3,
            1.5,
            5,
            2.5,
            8,
            4.0,
            10,
            5.0
        ]),
    );
    optimize_style_json_value_with_stats(&mut v, &mir, &fold_passes(), Some(&stats));
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r"
    fill-opacity:
      - interpolate
      - - linear
      - - get
        - rank
      - 1
      - 0.5
      - 3
      - 1.5
      - 5
      - 2.5
    ");
}

// ── SCCP: sparse conditional constant propagation ─────────────────────

#[test]
fn sccp_case_substitutes_get_in_output() {
    let mir = sample_mir();
    let mut v = style_with_paint(
        "fill-color",
        serde_json::json!([
            "case",
            ["==", ["get", "type"], "road"],
            ["get", "type"],
            "other"
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r#"
    fill-color:
      - case
      - - "=="
        - - get
          - type
        - road
      - road
      - other
    "#);
}

#[test]
fn sccp_case_commuted_operands() {
    let mir = sample_mir();
    let mut v = style_with_paint(
        "fill-color",
        serde_json::json!([
            "case",
            ["==", "road", ["get", "type"]],
            ["get", "type"],
            "other"
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r#"
    fill-color:
      - case
      - - "=="
        - road
        - - get
          - type
      - road
      - other
    "#);
}

#[test]
fn sccp_case_enables_concat_fold() {
    let mir = sample_mir();
    let mut v = style_with_paint(
        "fill-color",
        serde_json::json!([
            "case",
            ["==", ["get", "type"], "road"],
            ["concat", ["get", "type"], "-area"],
            "other"
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r#"
    fill-color:
      - case
      - - "=="
        - - get
          - type
        - road
      - road-area
      - other
    "#);
}

#[test]
fn sccp_match_substitutes_input() {
    let mir = sample_mir();
    let mut v = style_with_paint(
        "fill-color",
        serde_json::json!([
            "match",
            ["get", "class"],
            "park",
            ["get", "class"],
            "default"
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r#"
    fill-color:
      - match
      - - get
        - class
      - park
      - park
      - default
    "#);
}

#[test]
fn sccp_match_enables_concat_fold() {
    let mir = sample_mir();
    let mut v = style_with_paint(
        "fill-color",
        serde_json::json!([
            "match",
            ["get", "class"],
            "park",
            ["concat", ["get", "class"], "-area"],
            "default"
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r#"
    fill-color:
      - match
      - - get
        - class
      - park
      - park-area
      - default
    "#);
}

#[test]
fn sccp_match_skips_multi_label_arms() {
    let mir = sample_mir();
    let mut v = style_with_paint(
        "fill-color",
        serde_json::json!([
            "match",
            ["get", "class"],
            ["park", "garden"],
            ["get", "class"],
            "default"
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    // Output should be unchanged — multi-label arm is not substituted
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r#"
    fill-color:
      - match
      - - get
        - class
      - - park
        - garden
      - - get
        - class
      - default
    "#);
}

#[test]
fn sccp_case_skips_not_equal_condition() {
    let mir = sample_mir();
    let mut v = style_with_paint(
        "fill-color",
        serde_json::json!([
            "case",
            ["!=", ["get", "type"], "road"],
            ["get", "type"],
            "other"
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    // Output should be unchanged — != doesn't constrain the value
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r#"
    fill-color:
      - case
      - - "!="
        - - get
          - type
        - road
      - - get
        - type
      - other
    "#);
}

// ── Filter-to-property constant propagation ───────────────────────────

#[test]
fn filter_propagation_match_folds_to_literal() {
    let mir = sample_mir();
    let mut v = style_with_filter_and_paint(
        serde_json::json!(["==", ["get", "class"], "road"]),
        "fill-color",
        serde_json::json!([
            "match",
            ["get", "class"],
            "road",
            "#333",
            "rail",
            "#666",
            "#999"
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r##"
    fill-color: "#333"
    "##);
}

#[test]
fn filter_propagation_case_folds_true_branch() {
    let mir = sample_mir();
    let mut v = style_with_filter_and_paint(
        serde_json::json!(["==", ["get", "kind"], "park"]),
        "fill-color",
        serde_json::json!(["case", ["==", ["get", "kind"], "park"], "#0f0", "#ccc"]),
    );
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r##"
    fill-color: "#0f0"
    "##);
}

#[test]
fn filter_propagation_all_extracts_multiple_constraints() {
    let mir = sample_mir();
    let mut v = style_with_filter_and_paint(
        serde_json::json!([
            "all",
            ["==", ["get", "class"], "road"],
            ["==", ["get", "type"], 3]
        ]),
        "fill-color",
        serde_json::json!([
            "match",
            ["get", "class"],
            "road",
            "#333",
            "rail",
            "#666",
            "#999"
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r##"
    fill-color: "#333"
    "##);
}

#[test]
fn filter_propagation_commuted_equality() {
    let mir = sample_mir();
    let mut v = style_with_filter_and_paint(
        serde_json::json!(["==", "road", ["get", "class"]]),
        "fill-color",
        serde_json::json!([
            "match",
            ["get", "class"],
            "road",
            "#333",
            "rail",
            "#666",
            "#999"
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r##"
    fill-color: "#333"
    "##);
}

#[test]
fn filter_propagation_no_leak_across_layers() {
    let mir = sample_mir();
    // Two layers: first has filter, second doesn't — constraints shouldn't leak.
    let mut v = serde_json::json!({
        "version": 8,
        "sources": {"src": {"type": "vector", "url": "x"}},
        "layers": [
            {
                "id": "a", "type": "fill", "source": "src", "source-layer": "lyr",
                "filter": ["==", ["get", "class"], "road"],
                "paint": {"fill-color": ["match", ["get", "class"], "road", "#333", "#999"]}
            },
            {
                "id": "b", "type": "fill", "source": "src", "source-layer": "lyr",
                "paint": {"fill-color": ["match", ["get", "class"], "road", "#333", "#999"]}
            }
        ]
    });
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    // First layer should be folded.
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r##"
    fill-color: "#333"
    "##);
    // Second layer should remain data-driven (no filter constraint).
    assert_yaml_snapshot!(v["layers"][1]["paint"], @r##"
    fill-color:
      - match
      - - get
        - class
      - road
      - "#333"
      - "#999"
    "##);
}

// ── Filter-to-property: `has` constraint ──────────────────────────────

#[test]
fn filter_propagation_has_eliminates_coalesce_fallback() {
    let mir = sample_mir();
    let mut v = style_with_filter_and_paint(
        serde_json::json!(["has", "name"]),
        "fill-color",
        serde_json::json!(["coalesce", ["get", "name"], "unnamed"]),
    );
    // simplify_passes needed so try_simplify_coalesce unwraps single-arg coalesce.
    optimize_style_json_value(&mut v, &mir, &simplify_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r##"
    fill-color:
      - get
      - name
    "##);
}

#[test]
fn filter_propagation_has_coalesce_not_first_arg() {
    let mir = sample_mir();
    // `has` guarantees "alt_name" exists — it's the second coalesce arg.
    // The first arg (["get", "name"]) is NOT guaranteed, so it stays.
    let mut v = style_with_filter_and_paint(
        serde_json::json!(["has", "alt_name"]),
        "fill-color",
        serde_json::json!(["coalesce", ["get", "name"], ["get", "alt_name"], "fallback"]),
    );
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r##"
    fill-color:
      - coalesce
      - - get
        - name
      - - get
        - alt_name
    "##);
}

#[test]
fn filter_propagation_has_nested() {
    let mir = sample_mir();
    // `has` constraint should propagate into nested expressions.
    let mut v = style_with_filter_and_paint(
        serde_json::json!(["has", "name"]),
        "fill-color",
        serde_json::json!([
            "case",
            ["==", ["get", "type"], "primary"],
            ["coalesce", ["get", "name"], "unnamed"],
            "#ccc"
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &simplify_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r##"
    fill-color:
      - case
      - - "=="
        - - get
          - type
        - primary
      - - get
        - name
      - "#ccc"
    "##);
}

// ── Filter-to-property: `in`/domain constraint ───────────────────────

#[test]
fn filter_propagation_in_prunes_match_arms() {
    let mir = sample_mir();
    let mut v = style_with_filter_and_paint(
        serde_json::json!(["in", ["get", "class"], ["literal", ["road", "rail"]]]),
        "fill-color",
        serde_json::json!([
            "match",
            ["get", "class"],
            "road",
            "#333",
            "rail",
            "#666",
            "water",
            "#00f",
            "#999"
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r##"
    fill-color:
      - match
      - - get
        - class
      - road
      - "#333"
      - rail
      - "#666"
      - "#999"
    "##);
}

#[test]
fn filter_propagation_in_collapses_to_fallback() {
    let mir = sample_mir();
    // Domain is ["park"] but match only has "road" and "rail" arms → all pruned → fallback.
    let mut v = style_with_filter_and_paint(
        serde_json::json!(["in", ["get", "class"], ["literal", ["park"]]]),
        "fill-color",
        serde_json::json!([
            "match",
            ["get", "class"],
            "road",
            "#333",
            "rail",
            "#666",
            "#999"
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r##"
    fill-color: "#999"
    "##);
}

#[test]
fn filter_propagation_in_with_array_labels() {
    let mir = sample_mir();
    // Grouped labels: ["road", "rail"] maps to "#333". Domain is ["road", "water"].
    // "rail" should be pruned from the group.
    let mut v = style_with_filter_and_paint(
        serde_json::json!(["in", ["get", "class"], ["literal", ["road", "water"]]]),
        "fill-color",
        serde_json::json!([
            "match",
            ["get", "class"],
            ["road", "rail"],
            "#333",
            "water",
            "#00f",
            "#999"
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r##"
    fill-color:
      - match
      - - get
        - class
      - road
      - "#333"
      - water
      - "#00f"
      - "#999"
    "##);
}

// ── Filter-to-property: range constraint ─────────────────────────────

#[test]
fn filter_propagation_range_folds_ge_to_true() {
    let mir = sample_mir();
    // Filter: scalerank >= 3. Property: scalerank >= 1 → always true.
    let mut v = style_with_filter_and_paint(
        serde_json::json!([">=", ["get", "scalerank"], 3]),
        "fill-color",
        serde_json::json!(["case", [">=", ["get", "scalerank"], 1], "#0f0", "#ccc"]),
    );
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r##"
    fill-color: "#0f0"
    "##);
}

#[test]
fn filter_propagation_range_folds_lt_to_false() {
    let mir = sample_mir();
    // Filter: scalerank >= 3. Property: scalerank < 2 → always false.
    let mut v = style_with_filter_and_paint(
        serde_json::json!([">=", ["get", "scalerank"], 3]),
        "fill-color",
        serde_json::json!(["case", ["<", ["get", "scalerank"], 2], "#0f0", "#ccc"]),
    );
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r##"
    fill-color: "#ccc"
    "##);
}

#[test]
fn filter_propagation_range_no_fold_ambiguous() {
    let mir = sample_mir();
    // Filter: scalerank >= 3. Property: scalerank >= 5 → can't determine.
    let mut v = style_with_filter_and_paint(
        serde_json::json!([">=", ["get", "scalerank"], 3]),
        "fill-color",
        serde_json::json!(["case", [">=", ["get", "scalerank"], 5], "#0f0", "#ccc"]),
    );
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r##"
    fill-color:
      - case
      - - ">="
        - - get
          - scalerank
        - 5
      - "#0f0"
      - "#ccc"
    "##);
}

#[test]
fn filter_propagation_range_commuted() {
    let mir = sample_mir();
    // Filter: `3 <= scalerank` (commuted form of scalerank >= 3).
    // Property: scalerank >= 1 → always true.
    let mut v = style_with_filter_and_paint(
        serde_json::json!(["<=", 3, ["get", "scalerank"]]),
        "fill-color",
        serde_json::json!(["case", [">=", ["get", "scalerank"], 1], "#0f0", "#ccc"]),
    );
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r##"
    fill-color: "#0f0"
    "##);
}

// ── Filter-to-property: mixed constraints ────────────────────────────

#[test]
fn filter_propagation_all_with_mixed_constraints() {
    let mir = sample_mir();
    // `all` with `has` + `==`: both should apply.
    let mut v = style_with_filter_and_paint(
        serde_json::json!(["all", ["has", "name"], ["==", ["get", "class"], "road"]]),
        "fill-color",
        serde_json::json!([
            "match",
            ["get", "class"],
            "road",
            ["coalesce", ["get", "name"], "unnamed"],
            "#ccc"
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &simplify_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"], @r##"
    fill-color:
      - get
      - name
    "##);
}

// ── Distributive factoring ────────────────────────────────────────────

#[test]
fn distributive_factor_any_of_alls() {
    let mir = sample_mir();
    // ["any", ["all", A, B], ["all", A, C]] → ["all", A, ["any", B, C]]
    let mut v = style_with_filter(serde_json::json!([
        "any",
        ["all", ["has", "name"], ["==", ["get", "class"], "road"]],
        ["all", ["has", "name"], ["==", ["get", "class"], "rail"]]
    ]));
    optimize_style_json_value(&mut v, &mir, &simplify_passes());
    // Factoring produces ["all", ["has","name"], ["any", ==road, ==rail]]
    // then any_to_in rewrites the inner any to ["in", ...].
    assert_yaml_snapshot!(v["layers"][0]["filter"], @r#"
    - all
    - - has
      - name
    - - in
      - - get
        - class
      - - literal
        - - road
          - rail
    "#);
}

#[test]
fn distributive_factor_all_of_anys() {
    let mir = sample_mir();
    // ["all", ["any", A, B], ["any", A, C]] → ["any", A, ["all", B, C]]
    let mut v = style_with_filter(serde_json::json!([
        "all",
        ["any", ["has", "name"], ["==", ["get", "class"], "road"]],
        ["any", ["has", "name"], ["==", ["get", "class"], "rail"]]
    ]));
    optimize_style_json_value(&mut v, &mir, &simplify_passes());
    // Factoring extracts ["has","name"], absorption may also interact.
    assert_yaml_snapshot!(v["layers"][0]["filter"], @r"
    - has
    - name
    ");
}

#[test]
fn distributive_factor_multiple_common() {
    let mir = sample_mir();
    // ["any", ["all", A, B, C], ["all", A, B, D]] → ["all", A, B, ["any", C, D]]
    let mut v = style_with_filter(serde_json::json!([
        "any",
        [
            "all",
            ["has", "name"],
            ["has", "rank"],
            ["==", ["get", "class"], "road"]
        ],
        [
            "all",
            ["has", "name"],
            ["has", "rank"],
            ["==", ["get", "class"], "rail"]
        ]
    ]));
    optimize_style_json_value(&mut v, &mir, &simplify_passes());
    // Factoring then any_to_in on the remainder.
    assert_yaml_snapshot!(v["layers"][0]["filter"], @r#"
    - all
    - - has
      - name
    - - has
      - rank
    - - in
      - - get
        - class
      - - literal
        - - road
          - rail
    "#);
}

#[test]
fn distributive_factor_no_common_unchanged() {
    let mir = sample_mir();
    // No common factor → no change.
    let mut v = style_with_filter(serde_json::json!([
        "any",
        ["all", ["has", "name"], ["==", ["get", "class"], "road"]],
        ["all", ["has", "rank"], ["==", ["get", "class"], "rail"]]
    ]));
    let original = v["layers"][0]["filter"].clone();
    optimize_style_json_value(&mut v, &mir, &simplify_passes());
    assert_eq!(v["layers"][0]["filter"], original);
}

#[test]
fn distributive_factor_single_child_unchanged() {
    let mir = sample_mir();
    // Only one child → no factoring possible.
    let mut v = style_with_filter(serde_json::json!([
        "any",
        ["all", ["has", "name"], ["==", ["get", "class"], "road"]]
    ]));
    optimize_style_json_value(&mut v, &mir, &simplify_passes());
    // simplify_unary unwraps: ["any", X] → X
    assert_yaml_snapshot!(v["layers"][0]["filter"], @r#"
    - all
    - - has
      - name
    - - "=="
      - - get
        - class
      - road
    "#);
}

// ── Zoom comparison folding ──────────────────────────────────────────────

/// Style with minzoom set on the layer.
fn style_with_filter_and_minzoom(filter: serde_json::Value, minzoom: f64) -> serde_json::Value {
    serde_json::json!({
        "version": 8,
        "sources": {"src": {"type": "vector", "url": "x"}},
        "layers": [{
            "id": "l", "type": "fill", "source": "src", "source-layer": "lyr",
            "minzoom": minzoom,
            "filter": filter
        }]
    })
}

fn style_with_paint_and_minzoom(
    prop: &str,
    value: serde_json::Value,
    minzoom: f64,
) -> serde_json::Value {
    serde_json::json!({
        "version": 8,
        "sources": {"src": {"type": "vector", "url": "x"}},
        "layers": [{
            "id": "l", "type": "fill", "source": "src", "source-layer": "lyr",
            "minzoom": minzoom,
            "paint": {prop: value}
        }]
    })
}

/// Passes that trigger zoom folding + re-fold cleanup.
fn zoom_fold_passes() -> OptPasses {
    OptPasses {
        constant_fold: true,
        simplify_unary: true,
        simplify_expressions: true,
        // Need a structural pass to trigger the re-fold (step 3b).
        cleanup: true,
        ..Default::default()
    }
}

#[test]
fn zoom_fold_gte_true() {
    // [">=", ["zoom"], 5] with minzoom 10 → true
    let mir = sample_mir();
    let mut v = style_with_paint_and_minzoom(
        "fill-opacity",
        serde_json::json!(["case", [">=", ["zoom"], 5], 0.8, 0.2]),
        10.0,
    );
    optimize_style_json_value(&mut v, &mir, &zoom_fold_passes());
    // case folds to true branch → 0.8
    assert_yaml_snapshot!(v["layers"][0]["paint"]["fill-opacity"], @"0.8");
}

#[test]
fn zoom_fold_lt_false() {
    // ["<", ["zoom"], 5] with minzoom 8 → false
    let mir = sample_mir();
    let mut v = style_with_paint_and_minzoom(
        "fill-opacity",
        serde_json::json!(["case", ["<", ["zoom"], 5], 0.8, 0.2]),
        8.0,
    );
    optimize_style_json_value(&mut v, &mir, &zoom_fold_passes());
    // case folds to false (fallback) → 0.2
    assert_yaml_snapshot!(v["layers"][0]["paint"]["fill-opacity"], @"0.2");
}

#[test]
fn zoom_fold_inside_all() {
    // Zoom guard inside ["all", ...] in a case condition → partial fold + simplification.
    let mir = sample_mir();
    let mut v = style_with_paint_and_minzoom(
        "fill-opacity",
        serde_json::json!([
            "case",
            ["all", [">=", ["zoom"], 5], ["has", "name"]],
            0.8,
            0.2
        ]),
        10.0,
    );
    optimize_style_json_value(&mut v, &mir, &zoom_fold_passes());
    // [">=", ["zoom"], 5] → true, ["all", true, ["has", "name"]] → ["has", "name"]
    assert_yaml_snapshot!(v["layers"][0]["paint"]["fill-opacity"], @r#"
    - case
    - - has
      - name
    - 0.8
    - 0.2
    "#);
}

#[test]
fn zoom_fold_eq_false() {
    // ["==", ["zoom"], 3] with minzoom 5 → false
    let mir = sample_mir();
    let mut v = style_with_paint_and_minzoom(
        "fill-opacity",
        serde_json::json!(["case", ["==", ["zoom"], 3], 0.8, 0.2]),
        5.0,
    );
    optimize_style_json_value(&mut v, &mir, &zoom_fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"]["fill-opacity"], @"0.2");
}

#[test]
fn zoom_fold_commuted_lte_false() {
    // [">=", 5, ["zoom"]] means zoom <= 5. With minzoom 8 → false.
    let mir = sample_mir();
    let mut v = style_with_paint_and_minzoom(
        "fill-opacity",
        serde_json::json!(["case", [">=", 5, ["zoom"]], 0.8, 0.2]),
        8.0,
    );
    optimize_style_json_value(&mut v, &mir, &zoom_fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"]["fill-opacity"], @"0.2");
}

#[test]
fn zoom_fold_undetermined_no_change() {
    // [">=", ["zoom"], 15] with minzoom 10 → undetermined, no folding.
    let mir = sample_mir();
    let mut v = style_with_paint_and_minzoom(
        "fill-opacity",
        serde_json::json!(["case", [">=", ["zoom"], 15], 0.8, 0.2]),
        10.0,
    );
    optimize_style_json_value(&mut v, &mir, &zoom_fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"]["fill-opacity"], @r#"
    - case
    - - ">="
      - - zoom
      - 15
    - 0.8
    - 0.2
    "#);
}

#[test]
fn zoom_fold_no_minzoom_no_change() {
    // No minzoom → no folding.
    let mir = sample_mir();
    let mut v = style_with_paint(
        "fill-opacity",
        serde_json::json!(["case", [">=", ["zoom"], 5], 0.8, 0.2]),
    );
    optimize_style_json_value(&mut v, &mir, &zoom_fold_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"]["fill-opacity"], @r#"
    - case
    - - ">="
      - - zoom
      - 5
    - 0.8
    - 0.2
    "#);
}

#[test]
fn zoom_fold_in_filter() {
    // Zoom comparison in filter also gets folded.
    let mir = sample_mir();
    let mut v = style_with_filter_and_minzoom(
        serde_json::json!(["all", [">=", ["zoom"], 5], ["has", "name"]]),
        10.0,
    );
    optimize_style_json_value(&mut v, &mir, &zoom_fold_passes());
    // [">=", ["zoom"], 5] → true, ["all", true, ["has", "name"]] → ["has", "name"]
    assert_yaml_snapshot!(v["layers"][0]["filter"], @r#"
    - has
    - name
    "#);
}

// ── Stats: match arm reordering ─────────────────────────────────────────

#[test]
fn match_arms_reordered_by_frequency() {
    let mir = sample_mir();
    let mut vc = indexmap::IndexMap::new();
    vc.insert("rare".to_string(), 10_u64);
    vc.insert("common".to_string(), 1000);
    vc.insert("medium".to_string(), 100);
    let stats = make_stats(
        "lyr",
        LayerStats {
            total_features: 1110,
            properties: BTreeMap::from([(
                "class".to_string(),
                PropertyStats::String {
                    present_count: 1110,
                    cardinality: 3,
                    value_counts: Some(vc),
                },
            )]),
            ..Default::default()
        },
    );
    // Arms are in alphabetical order: common, medium, rare.
    // After reordering, they should be: common (1000), medium (100), rare (10).
    let mut v = style_with_paint(
        "fill-color",
        serde_json::json!([
            "match",
            ["get", "class"],
            "rare",
            "red",
            "common",
            "blue",
            "medium",
            "green",
            "black"
        ]),
    );
    optimize_style_json_value_with_stats(&mut v, &mir, &simplify_passes(), Some(&stats));
    assert_yaml_snapshot!(v["layers"][0]["paint"]["fill-color"], @r#"
    - match
    - - get
      - class
    - common
    - blue
    - medium
    - green
    - rare
    - red
    - black
    "#);
}

#[test]
fn match_arms_unchanged_when_already_ordered() {
    let mir = sample_mir();
    let mut vc = indexmap::IndexMap::new();
    vc.insert("a".to_string(), 100_u64);
    vc.insert("b".to_string(), 10);
    let stats = make_stats(
        "lyr",
        LayerStats {
            total_features: 110,
            properties: BTreeMap::from([(
                "class".to_string(),
                PropertyStats::String {
                    present_count: 110,
                    cardinality: 2,
                    value_counts: Some(vc),
                },
            )]),
            ..Default::default()
        },
    );
    let mut v = style_with_paint(
        "fill-color",
        serde_json::json!(["match", ["get", "class"], "a", "red", "b", "blue", "black"]),
    );
    let original = v["layers"][0]["paint"]["fill-color"].clone();
    optimize_style_json_value_with_stats(&mut v, &mir, &simplify_passes(), Some(&stats));
    assert_eq!(v["layers"][0]["paint"]["fill-color"], original);
}

#[test]
fn match_arms_not_reordered_without_value_counts() {
    let mir = sample_mir();
    let stats = make_stats(
        "lyr",
        LayerStats {
            total_features: 100,
            properties: BTreeMap::from([(
                "class".to_string(),
                PropertyStats::String {
                    present_count: 100,
                    cardinality: 50,
                    value_counts: None,
                },
            )]),
            ..Default::default()
        },
    );
    let mut v = style_with_paint(
        "fill-color",
        serde_json::json!(["match", ["get", "class"], "b", "red", "a", "blue", "black"]),
    );
    let original = v["layers"][0]["paint"]["fill-color"].clone();
    optimize_style_json_value_with_stats(&mut v, &mir, &simplify_passes(), Some(&stats));
    assert_eq!(v["layers"][0]["paint"]["fill-color"], original);
}

// ── Equivalence substitution ──────────────────────────────────────────

#[test]
fn equivalence_substitution_in_subsumption() {
    let mir = sample_mir();
    let mut v = style_with_filter(serde_json::json!([
        "all",
        ["==", ["get", "class"], "road"],
        [
            "in",
            ["get", "class"],
            ["literal", ["road", "rail", "path"]]
        ]
    ]));
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"], @r#"
    - filter:
        - all
        - - "=="
          - - get
            - class
          - road
      id: l
      source: src
      source-layer: lyr
      type: fill
    "#);
}

#[test]
fn equivalence_substitution_comparison() {
    let mir = sample_mir();
    let mut v = style_with_filter(serde_json::json!([
        "all",
        ["==", ["get", "x"], 5],
        [">=", ["get", "x"], 3]
    ]));
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"], @r#"
    - filter:
        - all
        - - "=="
          - - get
            - x
          - 5
      id: l
      source: src
      source-layer: lyr
      type: fill
    "#);
}

#[test]
fn equivalence_substitution_no_binding_without_equality() {
    let mir = sample_mir();
    let mut v = style_with_filter(serde_json::json!([
        "all",
        [">=", ["get", "x"], 5],
        ["in", ["get", "x"], ["literal", [5, 6, 7]]]
    ]));
    let original_filter = v["layers"][0]["filter"].clone();
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_eq!(v["layers"][0]["filter"], original_filter);
}

#[test]
fn equivalence_substitution_multiple_equalities() {
    let mir = sample_mir();
    let mut v = style_with_filter(serde_json::json!([
        "all",
        ["==", ["get", "class"], "road"],
        ["==", ["get", "subclass"], "main"],
        [
            "in",
            ["get", "class"],
            ["literal", ["road", "rail", "path"]]
        ]
    ]));
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"], @r#"
    - filter:
        - all
        - - "=="
          - - get
            - class
          - road
        - - "=="
          - - get
            - subclass
          - main
      id: l
      source: src
      source-layer: lyr
      type: fill
    "#);
}

#[test]
fn equivalence_substitution_commuted_equality() {
    let mir = sample_mir();
    let mut v = style_with_filter(serde_json::json!([
        "all",
        ["==", "road", ["get", "class"]],
        [
            "in",
            ["get", "class"],
            ["literal", ["road", "rail", "path"]]
        ]
    ]));
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"], @r#"
    - filter:
        - all
        - - "=="
          - road
          - - get
            - class
      id: l
      source: src
      source-layer: lyr
      type: fill
    "#);
}

#[test]
fn equivalence_substitution_flat_walk_only() {
    let mir = sample_mir();
    let mut v = style_with_filter(serde_json::json!([
        "all",
        ["==", ["get", "class"], "road"],
        [
            "any",
            ["in", ["get", "class"], ["literal", ["road", "rail"]]],
            ["==", ["get", "class"], "path"]
        ]
    ]));
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"], @r#"
    - filter:
        - all
        - - "=="
          - - get
            - class
          - road
      id: l
      source: src
      source-layer: lyr
      type: fill
    "#);
}

#[test]
fn equivalence_substitution_no_self_modification() {
    let mir = sample_mir();
    let mut v = style_with_filter(serde_json::json!([
        "all",
        ["==", ["get", "class"], "road"],
        ["!=", ["get", "class"], "rail"]
    ]));
    optimize_style_json_value(&mut v, &mir, &fold_passes());
    assert_yaml_snapshot!(v["layers"], @r#"
    - filter:
        - all
        - - "=="
          - - get
            - class
          - road
      id: l
      source: src
      source-layer: lyr
      type: fill
    "#);
}

// ── case chain linearization ────────────────────────────────────────────────

#[test]
fn case_flatten_two_level() {
    let mir = sample_mir();
    let mut v = style_with_paint(
        "fill-opacity",
        serde_json::json!([
            "case",
            ["==", ["get", "kind"], "park"],
            0.8,
            ["case", ["==", ["get", "kind"], "water"], 0.6, 0.2]
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &simplify_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"]["fill-opacity"], @r"
    - match
    - - get
      - kind
    - park
    - 0.8
    - water
    - 0.6
    - 0.2
    ");
}

#[test]
fn case_flatten_three_level() {
    let mir = sample_mir();
    let mut v = style_with_paint(
        "fill-opacity",
        serde_json::json!([
            "case",
            ["==", ["get", "kind"], "park"],
            0.9,
            [
                "case",
                ["==", ["get", "kind"], "water"],
                0.7,
                ["case", ["==", ["get", "kind"], "sand"], 0.5, 0.1]
            ]
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &simplify_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"]["fill-opacity"], @r#"
    - case
    - - "=="
      - - get
        - kind
      - park
    - 0.9
    - - match
      - - get
        - kind
      - water
      - 0.7
      - sand
      - 0.5
      - 0.1
    "#);
}

#[test]
fn case_flatten_noop_non_case_fallback() {
    let mir = sample_mir();
    let mut v = style_with_paint(
        "fill-opacity",
        serde_json::json!(["case", ["==", ["get", "kind"], "park"], 0.8, 0.2]),
    );
    optimize_style_json_value(&mut v, &mir, &simplify_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"]["fill-opacity"], @r#"
    - case
    - - "=="
      - - get
        - kind
      - park
    - 0.8
    - 0.2
    "#);
}

// ── Interpolation curve canonicalization ──────────────────────────────

#[test]
fn canonicalize_exponential_1_to_linear() {
    let mir = sample_mir();
    let mut v = style_with_paint(
        "fill-opacity",
        serde_json::json!(["interpolate", ["exponential", 1], ["zoom"], 0, 0.0, 10, 1.0]),
    );
    optimize_style_json_value(&mut v, &mir, &simplify_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"]["fill-opacity"], @r"
    - interpolate
    - - linear
    - - zoom
    - 0
    - 0
    - 10
    - 1
    ");
}

#[test]
fn canonicalize_exponential_1_hcl() {
    let mir = sample_mir();
    let mut v = style_with_paint(
        "fill-color",
        serde_json::json!([
            "interpolate-hcl",
            ["exponential", 1],
            ["zoom"],
            0,
            "#000",
            10,
            "#fff"
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &simplify_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"]["fill-color"], @r##"
    - interpolate-hcl
    - - linear
    - - zoom
    - 0
    - "#000"
    - 10
    - "#fff"
    "##);
}

// ── case → match conversion ───────────────────────────────────────────

#[test]
fn case_to_match_string_labels() {
    let mir = sample_mir();
    let mut v = style_with_paint(
        "fill-color",
        serde_json::json!([
            "case",
            ["==", ["get", "class"], "road"],
            "red",
            ["==", ["get", "class"], "rail"],
            "blue",
            "gray"
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &simplify_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"]["fill-color"], @r#"
    - match
    - - get
      - class
    - road
    - red
    - rail
    - blue
    - gray
    "#);
}

#[test]
fn case_to_match_numeric_labels() {
    let mir = sample_mir();
    let mut v = style_with_paint(
        "fill-color",
        serde_json::json!([
            "case",
            ["==", ["get", "level"], 1],
            "a",
            ["==", ["get", "level"], 2],
            "b",
            "c"
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &simplify_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"]["fill-color"], @r#"
    - match
    - - get
      - level
    - 1
    - a
    - 2
    - b
    - c
    "#);
}

#[test]
fn case_to_match_commuted_equality() {
    let mir = sample_mir();
    let mut v = style_with_paint(
        "fill-color",
        serde_json::json!([
            "case",
            ["==", "road", ["get", "class"]],
            "red",
            ["==", "rail", ["get", "class"]],
            "blue",
            "gray"
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &simplify_passes());
    assert_yaml_snapshot!(v["layers"][0]["paint"]["fill-color"], @r#"
    - match
    - - get
      - class
    - road
    - red
    - rail
    - blue
    - gray
    "#);
}

#[test]
fn case_to_match_single_arm_no_conversion() {
    let mir = sample_mir();
    let mut v = style_with_paint(
        "fill-color",
        serde_json::json!(["case", ["==", ["get", "class"], "road"], "red", "gray"]),
    );
    optimize_style_json_value(&mut v, &mir, &simplify_passes());
    // Single arm stays as case — not worth converting.
    assert_yaml_snapshot!(v["layers"][0]["paint"]["fill-color"], @r#"
    - case
    - - "=="
      - - get
        - class
      - road
    - red
    - gray
    "#);
}

#[test]
fn case_to_match_non_uniform_expr_no_conversion() {
    let mir = sample_mir();
    let mut v = style_with_paint(
        "fill-color",
        serde_json::json!([
            "case",
            ["==", ["get", "class"], "road"],
            "red",
            ["==", ["get", "type"], "rail"],
            "blue",
            "gray"
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &simplify_passes());
    // Different get expressions — stays as case.
    assert_yaml_snapshot!(v["layers"][0]["paint"]["fill-color"], @r#"
    - case
    - - "=="
      - - get
        - class
      - road
    - red
    - - "=="
      - - get
        - type
      - rail
    - blue
    - gray
    "#);
}

#[test]
fn case_to_match_bool_labels_no_conversion() {
    let mir = sample_mir();
    let mut v = style_with_paint(
        "fill-color",
        serde_json::json!([
            "case",
            ["==", ["get", "x"], true],
            "a",
            ["==", ["get", "x"], false],
            "b",
            "c"
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &simplify_passes());
    // Bool labels are rejected by match spec — stays as case.
    assert_yaml_snapshot!(v["layers"][0]["paint"]["fill-color"], @r#"
    - case
    - - "=="
      - - get
        - x
      - true
    - a
    - - "=="
      - - get
        - x
      - false
    - b
    - c
    "#);
}

#[test]
fn case_to_match_collator_no_conversion() {
    let mir = sample_mir();
    let mut v = style_with_paint(
        "fill-color",
        serde_json::json!([
            "case",
            ["==", ["get", "class"], "road", {"locale": "en"}], "red",
            ["==", ["get", "class"], "rail", {"locale": "en"}], "blue",
            "gray"
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &simplify_passes());
    // Collator (4-element ==) is rejected by extract_eq_chain — stays as case.
    assert_yaml_snapshot!(v["layers"][0]["paint"]["fill-color"], @r#"
    - case
    - - "=="
      - - get
        - class
      - road
      - locale: en
    - red
    - - "=="
      - - get
        - class
      - rail
      - locale: en
    - blue
    - gray
    "#);
}

#[test]
fn case_to_match_in_filter() {
    let mir = sample_mir();
    let mut v = style_with_filter(serde_json::json!([
        "case",
        ["==", ["get", "class"], "road"],
        true,
        ["==", ["get", "class"], "rail"],
        true,
        false
    ]));
    optimize_style_json_value(&mut v, &mir, &simplify_passes());
    assert_yaml_snapshot!(v["layers"][0]["filter"], @r"
    - in
    - - get
      - class
    - - literal
      - - road
        - rail
    ");
}

#[test]
fn nested_case_flatten_then_match() {
    let mir = sample_mir();
    // Outer case has only 1 arm → won't convert, but inner case has 2 → converts.
    // flatten_case runs per-node BEFORE case_to_match, but children are processed
    // bottom-up, so the inner case becomes match before the outer can flatten it.
    let mut v = style_with_paint(
        "fill-color",
        serde_json::json!([
            "case",
            ["==", ["get", "class"], "road"],
            "red",
            ["==", ["get", "class"], "water"],
            "aqua",
            [
                "case",
                ["==", ["get", "class"], "rail"],
                "blue",
                ["==", ["get", "class"], "path"],
                "green",
                "gray"
            ]
        ]),
    );
    optimize_style_json_value(&mut v, &mir, &simplify_passes());
    // Both outer and inner are independently converted to match.
    assert_yaml_snapshot!(v["layers"][0]["paint"]["fill-color"], @r#"
    - match
    - - get
      - class
    - road
    - red
    - water
    - aqua
    - - match
      - - get
        - class
      - rail
      - blue
      - path
      - green
      - gray
    "#);
}

#[test]
fn no_canonicalize_exponential_non_1() {
    let mir = sample_mir();
    let mut v = style_with_paint(
        "fill-opacity",
        serde_json::json!([
            "interpolate",
            ["exponential", 1.5],
            ["zoom"],
            0,
            0.0,
            10,
            1.0
        ]),
    );
    let original = v["layers"][0]["paint"]["fill-opacity"].clone();
    optimize_style_json_value(&mut v, &mir, &simplify_passes());
    assert_eq!(v["layers"][0]["paint"]["fill-opacity"], original);
}

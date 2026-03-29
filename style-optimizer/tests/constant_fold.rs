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
    let mut v = style_with_filter(serde_json::json!(["==", ["geometry-type"], "Point"]));
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
    let mut v = style_with_filter(serde_json::json!(["==", ["geometry-type"], "Polygon"]));
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

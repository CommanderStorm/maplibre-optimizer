//! Style JSON rewrite pipeline: expression normalisation, dead-code removal, metadata, reordering.

mod dead;
mod defaults;
mod expr;
mod metadata;
mod selectivity;
mod source_util;
mod strip;
mod walk;

use crate::stats::TileStatistics;
use dead::DeadEliminationVisitor;
use defaults::StripDefaultsVisitor;
use expr::{NormalizeFoldVisitor, ReorderSelectivityVisitor};
use maplibre_style_spec::mir::IntermediateSpec;
use metadata::MetadataRefinementVisitor;
use serde_json::Value;
use source_util::precompute_vector_layer_info;
use strip::{CleanupVisitor, StripMetadataVisitor};
use walk::walk_style_mut;

const NORMALIZE_FOLD_FIXPOINT_CAP: usize = 8;

/// Toggleable optimization passes (see crate-level docs).
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OptPasses {
    /// Unary boolean simplifications: `["any", e]` / `["all", e]` → `e`, and `["!", ["!", e]]` → `e`.
    pub simplify_unary: bool,
    /// `["!", ["==", a, b]]` → `["!=", a, b]` and similar (requires MIR operators).
    pub expression_kind: bool,
    /// Constant folding for comparisons, boolean `any`/`all`/`!`, arithmetic, strings, and colors.
    pub constant_fold: bool,
    /// Drop layers with always-false filters and remove unused `sources` entries.
    pub dead_elimination: bool,
    /// Raise `minzoom` / tighten `maxzoom` from `["zoom"]` comparisons inside `filter`.
    /// Also removes zoom predicates that are fully captured by the extracted bounds.
    pub metadata_refinement: bool,
    /// Static selectivity reordering of `any` / `all` operands (literals first/last for short-circuit hints).
    pub selectivity_reorder: bool,
    /// Remove `metadata` keys from root and layers.
    pub strip_metadata: bool,
    /// Remove paint/layout properties equal to their spec-defined defaults.
    pub strip_defaults: bool,
    /// Simplify `interpolate`/`step` with identical stops and deduplicate `match` arms.
    pub simplify_expressions: bool,
    /// Remove empty `paint`/`layout` objects, `visibility:none` layers, and zero-opacity layers.
    pub cleanup: bool,
}

impl OptPasses {
    /// Enable all optimization passes.
    #[must_use]
    pub fn all() -> Self {
        Self {
            simplify_unary: true,
            expression_kind: true,
            constant_fold: true,
            dead_elimination: true,
            metadata_refinement: true,
            selectivity_reorder: true,
            strip_metadata: true,
            strip_defaults: true,
            simplify_expressions: true,
            cleanup: true,
        }
    }
}

fn wants_normalize_fold(passes: &OptPasses) -> bool {
    passes.simplify_unary
        || passes.expression_kind
        || passes.constant_fold
        || passes.simplify_expressions
}

/// Apply enabled passes in pipeline order (backward-compatible, no stats).
///
/// Pipeline order:
/// 1. `strip_metadata` — remove noise before analysis
/// 2. Fixpoint loop (`simplify_unary`, `expression_kind`, `constant_fold`, `simplify_expressions`)
/// 3. `dead_elimination` — remove always-false layers, unused sources
/// 4. `metadata_refinement` — extract zoom bounds (+ remove redundant zoom predicates)
/// 5. `strip_defaults` — remove default-valued properties
/// 6. `selectivity_reorder` — reorder for short-circuit after folding removes literals
/// 7. `cleanup` — remove empty objects, invisible layers, re-run source cleanup
pub fn optimize_style_json_value(v: &mut Value, mir: &IntermediateSpec, passes: &OptPasses) {
    optimize_style_json_value_with_stats(v, mir, passes, None);
}

/// Apply enabled passes with optional data-driven statistics.
pub fn optimize_style_json_value_with_stats(
    v: &mut Value,
    mir: &IntermediateSpec,
    passes: &OptPasses,
    stats: Option<&TileStatistics>,
) {
    if !passes.strip_metadata
        && !wants_normalize_fold(passes)
        && !passes.dead_elimination
        && !passes.metadata_refinement
        && !passes.strip_defaults
        && !passes.selectivity_reorder
        && !passes.cleanup
    {
        return;
    }

    // 1. Strip metadata early (before analysis).
    if passes.strip_metadata {
        walk_style_mut(v, mir, &mut StripMetadataVisitor);
    }

    // Pre-compute vector layer info for stats-driven passes.
    let layer_info = stats.map(|_| precompute_vector_layer_info(v));

    // 2. Normalize / fold fixpoint.
    if wants_normalize_fold(passes) {
        for _ in 0..NORMALIZE_FOLD_FIXPOINT_CAP {
            let mut visitor = NormalizeFoldVisitor {
                mir,
                passes,
                stats,
                layer_info: layer_info.as_deref(),
                changed: false,
            };
            walk_style_mut(v, mir, &mut visitor);
            if !visitor.changed {
                break;
            }
        }
    }

    // 3. Dead elimination (recompute layer info since folding may have changed layers).
    if passes.dead_elimination {
        let layer_info = stats.map(|_| precompute_vector_layer_info(v));
        walk_style_mut(
            v,
            mir,
            &mut DeadEliminationVisitor {
                stats,
                layer_info: layer_info.as_deref(),
            },
        );
    }

    // 4. Metadata refinement (recompute after dead elimination).
    if passes.metadata_refinement {
        let layer_info = stats.map(|_| precompute_vector_layer_info(v));
        walk_style_mut(
            v,
            mir,
            &mut MetadataRefinementVisitor {
                stats,
                layer_info: layer_info.as_deref(),
            },
        );
    }

    // 5. Strip defaults.
    if passes.strip_defaults {
        walk_style_mut(v, mir, &mut StripDefaultsVisitor { mir });
    }

    // 6. Selectivity reorder (recompute after all mutations).
    if passes.selectivity_reorder {
        let layer_info = stats.map(|_| precompute_vector_layer_info(v));
        walk_style_mut(
            v,
            mir,
            &mut ReorderSelectivityVisitor {
                mir,
                stats,
                layer_info: layer_info.as_deref(),
            },
        );
    }

    // 7. Cleanup.
    if passes.cleanup {
        walk_style_mut(v, mir, &mut CleanupVisitor);
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::load_intermediate_spec_from_v8_path;

    fn sample_mir() -> IntermediateSpec {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../upstream/src/reference/v8.json");
        load_intermediate_spec_from_v8_path(&path).expect("v8.json")
    }

    fn passes_unary_only() -> OptPasses {
        OptPasses {
            simplify_unary: true,
            ..Default::default()
        }
    }

    #[test]
    fn simplify_unary_any_in_filter() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "layers": [{ "id": "x", "type": "fill", "filter": ["any", ["==", 1, 1]] }]
        });
        optimize_style_json_value(&mut v, &mir, &passes_unary_only());
        assert_eq!(v["layers"][0]["filter"], serde_json::json!(["==", 1, 1]));
    }

    #[test]
    fn simplify_unary_disabled_is_noop() {
        let mir = sample_mir();
        let original = serde_json::json!({
            "layers": [{ "id": "x", "type": "fill", "filter": ["any", ["==", 1, 1]] }]
        });
        let mut v = original.clone();
        optimize_style_json_value(&mut v, &mir, &OptPasses::default());
        assert_eq!(v, original);
    }

    #[test]
    fn simplify_nested_unary_any() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "layers": [{ "id": "x", "type": "fill", "filter": ["any", ["any", ["==", 1, 1]]] }]
        });
        optimize_style_json_value(&mut v, &mir, &passes_unary_only());
        assert_eq!(v["layers"][0]["filter"], serde_json::json!(["==", 1, 1]));
    }

    #[test]
    fn simplify_unary_all() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "layers": [{ "id": "x", "type": "fill", "filter": ["all", ["==", 1, 1]] }]
        });
        optimize_style_json_value(&mut v, &mir, &passes_unary_only());
        assert_eq!(v["layers"][0]["filter"], serde_json::json!(["==", 1, 1]));
    }

    #[test]
    fn bare_any_single_element_unchanged() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "layers": [{ "id": "x", "type": "fill", "filter": ["any"] }]
        });
        let expected = v.clone();
        optimize_style_json_value(&mut v, &mir, &passes_unary_only());
        assert_eq!(v, expected);
    }

    #[test]
    fn simplify_double_not() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "layers": [{ "id": "x", "type": "fill", "filter": ["!", ["!", ["has", "x"]]] }]
        });
        optimize_style_json_value(&mut v, &mir, &passes_unary_only());
        assert_eq!(v["layers"][0]["filter"], serde_json::json!(["has", "x"]));
    }

    #[test]
    fn simplify_triple_not() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "layers": [{ "id": "x", "type": "fill", "filter": ["!", ["!", ["!", ["has", "x"]]]] }]
        });
        optimize_style_json_value(&mut v, &mir, &passes_unary_only());
        assert_eq!(
            v["layers"][0]["filter"],
            serde_json::json!(["!", ["has", "x"]])
        );
    }

    #[test]
    fn negated_eq_to_neq() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "layers": [{ "id": "x", "type": "fill", "filter": ["!", ["==", ["get", "a"], 1]] }]
        });
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                expression_kind: true,
                ..Default::default()
            },
        );
        assert_eq!(
            v["layers"][0]["filter"],
            serde_json::json!(["!=", ["get", "a"], 1])
        );
    }

    #[test]
    fn fold_literal_eq() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "layers": [{ "id": "x", "type": "fill", "filter": ["==", 2, 3] }]
        });
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                constant_fold: true,
                ..Default::default()
            },
        );
        assert_eq!(
            v["layers"][0]["filter"],
            serde_json::json!(["literal", false])
        );
    }

    #[test]
    fn dead_elim_removes_false_filter_layer_and_unused_source() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "version": 8,
            "sources": {
                "a": { "type": "vector", "url": "https://example/tiles.json" },
                "b": { "type": "vector", "url": "https://example/b.json" }
            },
            "layers": [
                {
                    "id": "x",
                    "type": "fill",
                    "source": "a",
                    "source-layer": "s",
                    "filter": ["literal", false]
                },
                {
                    "id": "y",
                    "type": "line",
                    "source": "b",
                    "source-layer": "r",
                    "filter": ["==", 1, 1]
                }
            ]
        });
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                dead_elimination: true,
                ..Default::default()
            },
        );
        let expected = serde_json::json!({
            "version": 8,
            "sources": {
                "b": { "type": "vector", "url": "https://example/b.json" }
            },
            "layers": [
                {
                    "id": "y",
                    "type": "line",
                    "source": "b",
                    "source-layer": "r",
                    "filter": ["==", 1, 1]
                }
            ]
        });
        assert_eq!(v, expected);
    }

    #[test]
    fn metadata_refine_minzoom_from_filter() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "version": 8,
            "layers": [{
                "id": "x",
                "type": "fill",
                "filter": ["all", [">=", ["zoom"], 7]]
            }]
        });
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                metadata_refinement: true,
                ..Default::default()
            },
        );
        let min = v["layers"][0]["minzoom"].as_f64().expect("minzoom");
        assert!((min - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn reorder_any_puts_literal_true_first() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "layers": [{
                "id": "x",
                "type": "fill",
                "filter": ["any", ["==", 1, 2], ["literal", true]]
            }]
        });
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                selectivity_reorder: true,
                ..Default::default()
            },
        );
        let arr = v["layers"][0]["filter"].as_array().unwrap();
        assert_eq!(arr[1], serde_json::json!(["literal", true]));
    }

    #[test]
    fn arithmetic_fold_addition() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "layers": [{ "id": "x", "type": "fill", "filter": ["+", 1, 2] }]
        });
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                constant_fold: true,
                ..Default::default()
            },
        );
        assert_eq!(
            v["layers"][0]["filter"],
            serde_json::json!(["literal", 3.0])
        );
    }

    #[test]
    fn string_fold_concat() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "layers": [{ "id": "x", "type": "fill", "filter": ["concat", "hello", " ", "world"] }]
        });
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                constant_fold: true,
                ..Default::default()
            },
        );
        assert_eq!(
            v["layers"][0]["filter"],
            serde_json::json!(["literal", "hello world"])
        );
    }

    #[test]
    fn interpolate_all_same_stops_collapsed() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "layers": [{ "id": "x", "type": "fill", "paint": {
                "fill-opacity": ["interpolate", ["linear"], ["zoom"], 5, 0.5, 10, 0.5, 15, 0.5]
            }}]
        });
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                simplify_expressions: true,
                ..Default::default()
            },
        );
        assert_eq!(
            v["layers"][0]["paint"]["fill-opacity"],
            serde_json::json!(["literal", 0.5])
        );
    }

    #[test]
    fn step_all_same_outputs_collapsed() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "layers": [{ "id": "x", "type": "line", "paint": {
                "line-width": ["step", ["zoom"], 2, 10, 2, 15, 2]
            }}]
        });
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                simplify_expressions: true,
                ..Default::default()
            },
        );
        assert_eq!(
            v["layers"][0]["paint"]["line-width"],
            serde_json::json!(["literal", 2])
        );
    }

    #[test]
    fn match_arms_deduplicated() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "layers": [{ "id": "x", "type": "line", "paint": {
                "line-color": ["match", ["get", "class"],
                    "motorway", "#ff0000",
                    "trunk", "#ff0000",
                    "primary", "#ff6600",
                    "#cccccc"
                ]
            }}]
        });
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                simplify_expressions: true,
                ..Default::default()
            },
        );
        let color = &v["layers"][0]["paint"]["line-color"];
        let arr = color.as_array().unwrap();
        assert_eq!(arr[0], serde_json::json!("match"));
        // motorway and trunk should now be grouped
        assert_eq!(arr[2], serde_json::json!(["motorway", "trunk"]));
        assert_eq!(arr[3], serde_json::json!("#ff0000"));
    }

    #[test]
    fn strip_metadata_removes_noise() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "metadata": { "maputnik:renderer": "mbgljs" },
            "layers": [{ "id": "x", "type": "fill", "metadata": { "group": "water" } }]
        });
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                strip_metadata: true,
                ..Default::default()
            },
        );
        assert!(v.get("metadata").is_none());
        assert!(v["layers"][0].get("metadata").is_none());
    }

    #[test]
    fn strip_defaults_removes_fill_opacity_one() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "layers": [{ "id": "x", "type": "fill", "paint": { "fill-opacity": 1, "fill-color": "#f00" } }]
        });
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                strip_defaults: true,
                ..Default::default()
            },
        );
        assert!(v["layers"][0]["paint"].get("fill-opacity").is_none());
        assert!(v["layers"][0]["paint"].get("fill-color").is_some());
    }

    #[test]
    fn cleanup_removes_invisible_layer() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "version": 8,
            "sources": { "s": { "type": "vector", "url": "x" } },
            "layers": [{ "id": "x", "type": "fill", "source": "s", "source-layer": "l",
                "layout": { "visibility": "none" } }]
        });
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                cleanup: true,
                ..Default::default()
            },
        );
        assert_eq!(v["layers"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn all_passes_enabled_does_not_panic() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "version": 8,
            "metadata": { "editor": "maputnik" },
            "sources": {
                "openmaptiles": { "type": "vector", "url": "https://example/tiles.json" }
            },
            "layers": [
                {
                    "id": "background",
                    "type": "background",
                    "paint": { "background-color": "#f8f4f0", "background-opacity": 1 }
                },
                {
                    "id": "water",
                    "type": "fill",
                    "source": "openmaptiles",
                    "source-layer": "water",
                    "filter": ["all", [">=", ["zoom"], 5], ["==", ["geometry-type"], "Polygon"]],
                    "paint": { "fill-color": "#a0c8f0", "fill-opacity": 1 }
                }
            ]
        });
        let passes = OptPasses::all();
        optimize_style_json_value(&mut v, &mir, &passes);
        // Running again must be a no-op (idempotency).
        let mut again = v.clone();
        optimize_style_json_value(&mut again, &mir, &passes);
        assert_eq!(v, again);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn zoom_predicate_removed_after_metadata_extraction() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "layers": [{
                "id": "x",
                "type": "fill",
                "filter": ["all", [">=", ["zoom"], 7], ["==", ["get", "class"], "river"]]
            }]
        });
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                metadata_refinement: true,
                ..Default::default()
            },
        );
        // minzoom extracted
        assert_eq!(v["layers"][0]["minzoom"].as_f64().unwrap(), 7.0);
        // zoom predicate removed from filter, leaving just the class check
        assert_eq!(
            v["layers"][0]["filter"],
            serde_json::json!(["==", ["get", "class"], "river"])
        );
    }

    // ── Stats-driven tests ──────────────────────────────────────────────────────

    use crate::stats::{GeometryTypeStats, LayerStats, SourceStats};
    use std::collections::BTreeMap;

    fn make_stats(layer_name: &str, layer_stats: LayerStats) -> TileStatistics {
        let mut layers = BTreeMap::new();
        layers.insert(layer_name.to_string(), layer_stats);
        let mut sources = BTreeMap::new();
        sources.insert(
            "openmaptiles".to_string(),
            SourceStats { layers },
        );
        TileStatistics { sources }
    }

    #[test]
    fn dead_elim_geometry_type_mismatch() {
        let mir = sample_mir();
        let stats = make_stats(
            "water",
            LayerStats {
                total_features: 1000,
                geometry_types: GeometryTypeStats {
                    unknown: 0,
                    point: 1000,
                    linestring: 0,
                    polygon: 0,
                },
                ..Default::default()
            },
        );
        let mut v = serde_json::json!({
            "version": 8,
            "sources": { "openmaptiles": { "type": "vector", "url": "x" } },
            "layers": [{
                "id": "water-fill",
                "type": "fill",
                "source": "openmaptiles",
                "source-layer": "water"
            }]
        });
        optimize_style_json_value_with_stats(
            &mut v,
            &mir,
            &OptPasses {
                dead_elimination: true,
                ..Default::default()
            },
            Some(&stats),
        );
        assert_eq!(v["layers"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn dead_elim_geometry_type_match_preserved() {
        let mir = sample_mir();
        let stats = make_stats(
            "water",
            LayerStats {
                total_features: 1000,
                geometry_types: GeometryTypeStats {
                    unknown: 0,
                    point: 0,
                    linestring: 0,
                    polygon: 1000,
                },
                ..Default::default()
            },
        );
        let mut v = serde_json::json!({
            "version": 8,
            "sources": { "openmaptiles": { "type": "vector", "url": "x" } },
            "layers": [{
                "id": "water-fill",
                "type": "fill",
                "source": "openmaptiles",
                "source-layer": "water"
            }]
        });
        optimize_style_json_value_with_stats(
            &mut v,
            &mir,
            &OptPasses {
                dead_elimination: true,
                ..Default::default()
            },
            Some(&stats),
        );
        assert_eq!(v["layers"].as_array().unwrap().len(), 1);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn metadata_refinement_zoom_from_stats() {
        let mir = sample_mir();
        let mut fbz = BTreeMap::new();
        fbz.insert(6u8, 100u64);
        fbz.insert(7, 500);
        fbz.insert(14, 9000);
        let stats = make_stats(
            "water",
            LayerStats {
                total_features: 9600,
                features_by_zoom: fbz,
                ..Default::default()
            },
        );
        let mut v = serde_json::json!({
            "version": 8,
            "sources": { "openmaptiles": { "type": "vector", "url": "x" } },
            "layers": [{
                "id": "water-fill",
                "type": "fill",
                "source": "openmaptiles",
                "source-layer": "water"
            }]
        });
        optimize_style_json_value_with_stats(
            &mut v,
            &mir,
            &OptPasses {
                metadata_refinement: true,
                ..Default::default()
            },
            Some(&stats),
        );
        assert_eq!(v["layers"][0]["minzoom"].as_f64().unwrap(), 6.0);
        assert_eq!(v["layers"][0]["maxzoom"].as_f64().unwrap(), 14.0);
    }

    #[test]
    fn id_fold_when_no_feature_ids() {
        let mir = sample_mir();
        let stats = make_stats(
            "water",
            LayerStats {
                total_features: 1000,
                has_feature_ids: false,
                ..Default::default()
            },
        );
        let mut v = serde_json::json!({
            "version": 8,
            "sources": { "openmaptiles": { "type": "vector", "url": "x" } },
            "layers": [{
                "id": "water-fill",
                "type": "fill",
                "source": "openmaptiles",
                "source-layer": "water",
                "filter": ["==", ["id"], 5]
            }]
        });
        optimize_style_json_value_with_stats(
            &mut v,
            &mir,
            &OptPasses {
                constant_fold: true,
                ..Default::default()
            },
            Some(&stats),
        );
        assert_eq!(
            v["layers"][0]["filter"],
            serde_json::json!(["literal", false])
        );
    }
}

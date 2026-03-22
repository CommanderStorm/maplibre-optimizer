//! Style optimization pipeline: expression normalisation, dead-code removal, metadata, reordering.
//!
//! Two entry points:
//!
//! - [`optimize_style`] is the typed entry point.  All passes operate on
//!   `&mut MaplibreStyleSpecification` as the canonical representation.
//!   Expression passes temporarily serialize to JSON for the schema-guided
//!   walker, then deserialize back.  All generated expression types accept
//!   an `Any` fallback variant, so optimizer-produced forms like
//!   `["literal", x]` round-trip correctly.
//!
//! - [`optimize_style_json_value`] / [`optimize_style_json_value_with_stats`]
//!   are thin wrappers: they deserialize the JSON into a typed struct, call
//!   the same [`run_pipeline`], and serialize back.

mod defaults;
pub(crate) mod expr;
pub(crate) mod metadata;
mod selectivity;
pub(crate) mod source_util;
pub(crate) mod typed_passes;
mod walk;

use defaults::StripDefaultsVisitor;
use expr::{NormalizeFoldVisitor, ReorderSelectivityVisitor};
use maplibre_style_spec::mir::MirSpec;
use maplibre_style_spec::spec::MaplibreStyleSpecification;
use serde_json::Value;
use source_util::precompute_vector_layer_info;
use typed_passes::{
    cleanup_typed, dead_elimination_typed, metadata_refinement_typed,
    precompute_vector_layer_info_typed, strip_metadata_typed,
};
use walk::walk_style_mut;

use crate::stats::TileStatistics;

const NORMALIZE_FOLD_FIXPOINT_CAP: usize = 8;

/// Toggleable optimization passes.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OptPasses {
    pub simplify_unary: bool,
    pub expression_kind: bool,
    pub constant_fold: bool,
    pub dead_elimination: bool,
    pub metadata_refinement: bool,
    pub selectivity_reorder: bool,
    pub strip_metadata: bool,
    pub strip_defaults: bool,
    pub simplify_expressions: bool,
    pub cleanup: bool,
}

impl OptPasses {
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

// ── Public entry points ─────────────────────────────────────────────────────

/// Convenience wrapper (no stats).
pub fn optimize_style_json_value(v: &mut Value, mir: &MirSpec, passes: &OptPasses) {
    optimize_style_json_value_with_stats(v, mir, passes, None);
}

/// JSON entry point.  Deserializes `v` into a typed struct, runs the typed
/// pipeline, and serializes back.
pub fn optimize_style_json_value_with_stats(
    v: &mut Value,
    mir: &MirSpec,
    passes: &OptPasses,
    stats: Option<&TileStatistics>,
) {
    let Ok(mut style) = serde_json::from_value::<MaplibreStyleSpecification>(v.clone()) else {
        return;
    };
    run_pipeline(&mut style, mir, passes, stats);
    if let Ok(updated) = serde_json::to_value(&style) {
        *v = updated;
    }
}

/// Primary typed entry point.  The typed struct is the canonical
/// representation; no whole-style JSON roundtrip is performed.
pub fn optimize_style(
    style: &mut MaplibreStyleSpecification,
    mir: &MirSpec,
    passes: &OptPasses,
    stats: Option<&TileStatistics>,
) {
    run_pipeline(style, mir, passes, stats);
}

// ── Pipeline implementation ─────────────────────────────────────────────────

/// Typed-primary pipeline shared by both entry points.
///
/// Structural passes (`strip_metadata`, `dead_elimination`, `metadata_refinement`,
/// `cleanup`) work directly on the typed struct.  Expression passes temporarily
/// serialize to JSON for the schema-guided walker, then deserialize back; the
/// `Any` fallback variants on all generated expression types guarantee that
/// optimizer-produced forms (e.g. `["literal", x]`) survive the round-trip.
fn run_pipeline(
    style: &mut MaplibreStyleSpecification,
    mir: &MirSpec,
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

    // 1. Strip metadata (typed, direct).
    if passes.strip_metadata {
        strip_metadata_typed(style);
    }

    // 2. Expression passes: serialize → JSON walker → deserialize back.
    if (wants_normalize_fold(passes) || passes.strip_defaults || passes.selectivity_reorder)
        && let Ok(mut v) = serde_json::to_value(&*style)
    {
        run_json_expression_passes(&mut v, mir, passes, stats);
        if let Ok(updated) = serde_json::from_value(v) {
            *style = updated;
        }
    }

    // 3. Dead elimination (typed, direct).
    if passes.dead_elimination {
        let layer_info = stats.map(|_| precompute_vector_layer_info_typed(style));
        dead_elimination_typed(style, stats, layer_info.as_deref());
    }

    // 4. Metadata refinement (typed, direct).
    if passes.metadata_refinement {
        let layer_info = stats.map(|_| precompute_vector_layer_info_typed(style));
        metadata_refinement_typed(style, stats, layer_info.as_deref());
    }

    // 5. Cleanup (typed, direct).
    if passes.cleanup {
        cleanup_typed(style);
    }
}

/// Run expression-level passes on the JSON value.
fn run_json_expression_passes(
    v: &mut Value,
    mir: &MirSpec,
    passes: &OptPasses,
    stats: Option<&TileStatistics>,
) {
    let needs = wants_normalize_fold(passes) || passes.strip_defaults || passes.selectivity_reorder;
    if !needs {
        return;
    }

    let layer_info = stats.map(|_| precompute_vector_layer_info(v));

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

    if passes.strip_defaults {
        walk_style_mut(v, mir, &mut StripDefaultsVisitor { mir });
    }

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
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::load_intermediate_spec_from_v8_path;

    fn sample_mir() -> MirSpec {
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
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[{"id":"x","type":"fill","filter":["any",["==",1,1]]}]});
        optimize_style_json_value(&mut v, &mir, &passes_unary_only());
        assert_eq!(v["layers"][0]["filter"], serde_json::json!(["==", 1, 1]));
    }

    #[test]
    fn simplify_unary_disabled_is_noop() {
        let mir = sample_mir();
        let original = serde_json::json!({"version":8,"sources":{},"layers":[{"id":"x","type":"fill","filter":["any",["==",1,1]]}]});
        let mut v = original.clone();
        optimize_style_json_value(&mut v, &mir, &OptPasses::default());
        assert_eq!(v, original);
    }

    #[test]
    fn simplify_nested_unary_any() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[{"id":"x","type":"fill","filter":["any",["any",["==",1,1]]]}]});
        optimize_style_json_value(&mut v, &mir, &passes_unary_only());
        assert_eq!(v["layers"][0]["filter"], serde_json::json!(["==", 1, 1]));
    }

    #[test]
    fn simplify_unary_all() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[{"id":"x","type":"fill","filter":["all",["==",1,1]]}]});
        optimize_style_json_value(&mut v, &mir, &passes_unary_only());
        assert_eq!(v["layers"][0]["filter"], serde_json::json!(["==", 1, 1]));
    }

    #[test]
    fn bare_any_single_element_unchanged() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[{"id":"x","type":"fill","filter":["any"]}]});
        let expected = v.clone();
        optimize_style_json_value(&mut v, &mir, &passes_unary_only());
        assert_eq!(v, expected);
    }

    #[test]
    fn simplify_double_not() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[{"id":"x","type":"fill","filter":["!",["!",["has","x"]]]}]});
        optimize_style_json_value(&mut v, &mir, &passes_unary_only());
        assert_eq!(v["layers"][0]["filter"], serde_json::json!(["has", "x"]));
    }

    #[test]
    fn simplify_triple_not() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[{"id":"x","type":"fill","filter":["!",["!",["!",["has","x"]]]]}]});
        optimize_style_json_value(&mut v, &mir, &passes_unary_only());
        assert_eq!(
            v["layers"][0]["filter"],
            serde_json::json!(["!", ["has", "x"]])
        );
    }

    #[test]
    fn negated_eq_to_neq() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[{"id":"x","type":"fill","filter":["!",["==",["get","a"],1]]}]});
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
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[{"id":"x","type":"fill","filter":["==",2,3]}]});
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
        let mut v = serde_json::json!({"version":8,"sources":{"a":{"type":"vector","url":"https://example/tiles.json"},"b":{"type":"vector","url":"https://example/b.json"}},"layers":[{"id":"x","type":"fill","source":"a","source-layer":"s","filter":["literal",false]},{"id":"y","type":"line","source":"b","source-layer":"r","filter":["==",1,1]}]});
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                dead_elimination: true,
                ..Default::default()
            },
        );
        assert_eq!(v["sources"].as_object().unwrap().len(), 1);
        assert_eq!(v["layers"].as_array().unwrap().len(), 1);
        assert_eq!(v["layers"][0]["id"], "y");
    }

    #[test]
    fn metadata_refine_minzoom_from_filter() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[{"id":"x","type":"fill","filter":["all",[">=",["zoom"],7]]}]});
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                metadata_refinement: true,
                ..Default::default()
            },
        );
        assert!((v["layers"][0]["minzoom"].as_f64().expect("minzoom") - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn reorder_any_puts_literal_true_first() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[{"id":"x","type":"fill","filter":["any",["==",1,2],["literal",true]]}]});
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                selectivity_reorder: true,
                ..Default::default()
            },
        );
        assert_eq!(
            v["layers"][0]["filter"].as_array().unwrap()[1],
            serde_json::json!(["literal", true])
        );
    }

    #[test]
    fn arithmetic_fold_addition() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[{"id":"x","type":"fill","filter":["+",1,2]}]});
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
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[{"id":"x","type":"fill","filter":["concat","hello"," ","world"]}]});
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
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[{"id":"x","type":"fill","paint":{"fill-opacity":["interpolate",["linear"],["zoom"],5,0.5,10,0.5,15,0.5]}}]});
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
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[{"id":"x","type":"line","paint":{"line-width":["step",["zoom"],2,10,2,15,2]}}]});
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
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[{"id":"x","type":"line","paint":{"line-color":["match",["get","class"],"motorway","#ff0000","trunk","#ff0000","primary","#ff6600","#cccccc"]}}]});
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                simplify_expressions: true,
                ..Default::default()
            },
        );
        let arr = v["layers"][0]["paint"]["line-color"].as_array().unwrap();
        assert_eq!(arr[0], "match");
        assert_eq!(arr[2], serde_json::json!(["motorway", "trunk"]));
    }

    #[test]
    fn strip_metadata_removes_noise() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"metadata":{"maputnik:renderer":"mbgljs"},"layers":[{"id":"x","type":"fill","metadata":{"group":"water"}}]});
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
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[{"id":"x","type":"fill","paint":{"fill-opacity":1,"fill-color":"#f00"}}]});
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
        let mut v = serde_json::json!({"version":8,"sources":{"s":{"type":"vector","url":"x"}},"layers":[{"id":"x","type":"fill","source":"s","source-layer":"l","layout":{"visibility":"none"}}]});
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
        let mut v = serde_json::json!({"version":8,"metadata":{"editor":"maputnik"},"sources":{"openmaptiles":{"type":"vector","url":"https://example/tiles.json"}},"layers":[{"id":"background","type":"background","paint":{"background-color":"#f8f4f0","background-opacity":1}},{"id":"water","type":"fill","source":"openmaptiles","source-layer":"water","filter":["all",[">=",["zoom"],5],["==",["geometry-type"],"Polygon"]],"paint":{"fill-color":"#a0c8f0","fill-opacity":1}}]});
        let passes = OptPasses::all();
        optimize_style_json_value(&mut v, &mir, &passes);
        let mut again = v.clone();
        optimize_style_json_value(&mut again, &mir, &passes);
        assert_eq!(v, again);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn zoom_predicate_removed_after_metadata_extraction() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[{"id":"x","type":"fill","filter":["all",[">=",["zoom"],7],["==",["get","class"],"river"]]}]});
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                metadata_refinement: true,
                ..Default::default()
            },
        );
        assert_eq!(v["layers"][0]["minzoom"].as_f64().unwrap(), 7.0);
        assert_eq!(
            v["layers"][0]["filter"],
            serde_json::json!(["==", ["get", "class"], "river"])
        );
    }

    // ── Stats-driven tests ──────────────────────────────────────────────────

    use std::collections::BTreeMap;

    use crate::stats::{GeometryTypeStats, LayerStats, SourceStats};

    fn make_stats(layer_name: &str, layer_stats: LayerStats) -> TileStatistics {
        let mut layers = BTreeMap::new();
        layers.insert(layer_name.to_string(), layer_stats);
        let mut sources = BTreeMap::new();
        sources.insert("openmaptiles".to_string(), SourceStats { layers });
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
        let mut v = serde_json::json!({"version":8,"sources":{"openmaptiles":{"type":"vector","url":"x"}},"layers":[{"id":"water-fill","type":"fill","source":"openmaptiles","source-layer":"water"}]});
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
        let mut v = serde_json::json!({"version":8,"sources":{"openmaptiles":{"type":"vector","url":"x"}},"layers":[{"id":"water-fill","type":"fill","source":"openmaptiles","source-layer":"water"}]});
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
        let mut v = serde_json::json!({"version":8,"sources":{"openmaptiles":{"type":"vector","url":"x"}},"layers":[{"id":"water-fill","type":"fill","source":"openmaptiles","source-layer":"water"}]});
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
        let mut v = serde_json::json!({"version":8,"sources":{"openmaptiles":{"type":"vector","url":"x"}},"layers":[{"id":"water-fill","type":"fill","source":"openmaptiles","source-layer":"water","filter":["==",["id"],5]}]});
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

    // ── Typed entry point tests ─────────────────────────────────────────────

    #[test]
    fn typed_strip_metadata() {
        let mir = sample_mir();
        let mut style: MaplibreStyleSpecification = serde_json::from_value(serde_json::json!({"version":8,"sources":{},"metadata":{"x":"y"},"layers":[{"id":"x","type":"fill","metadata":{"g":"w"}}]})).unwrap();
        optimize_style(
            &mut style,
            &mir,
            &OptPasses {
                strip_metadata: true,
                ..Default::default()
            },
            None,
        );
        assert!(style.metadata.is_none());
        assert!(style.layers[0].common().unwrap().metadata.is_none());
    }

    #[test]
    fn typed_all_passes_idempotent() {
        let mir = sample_mir();
        let mut style: MaplibreStyleSpecification = serde_json::from_value(serde_json::json!({"version":8,"sources":{"openmaptiles":{"type":"vector","url":"https://example/tiles.json"}},"layers":[{"id":"water","type":"fill","source":"openmaptiles","source-layer":"water","filter":["all",[">=",["zoom"],5],["==",["geometry-type"],"Polygon"]],"paint":{"fill-color":"#a0c8f0"}}]})).unwrap();
        let passes = OptPasses::all();
        optimize_style(&mut style, &mir, &passes, None);
        let first = serde_json::to_value(&style).unwrap();
        optimize_style(&mut style, &mir, &passes, None);
        let second = serde_json::to_value(&style).unwrap();
        assert_eq!(first, second);
    }
}

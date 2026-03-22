//! Style optimization pipeline: expression normalisation, dead-code removal, metadata, reordering.
//!
//! Two entry points:
//!
//! - [`optimize_style`] operates on the typed [`MaplibreStyleSpecification`].
//!   Structural passes work directly on typed structs; expression passes use a
//!   parallel JSON `Value`.
//!
//! - [`optimize_style_json_value`] / [`optimize_style_json_value_with_stats`]
//!   operate on raw `serde_json::Value`.  They build a typed struct for
//!   structural passes and fall back to JSON for expression passes.

pub mod advisory_rewrite;
mod defaults;
pub(crate) mod expr;
pub(crate) mod expr_util;
pub mod field_analysis;
pub(crate) mod metadata;
mod selectivity;
pub(crate) mod source_util;
pub(crate) mod typed_passes;
mod walk;

use defaults::StripDefaultsVisitor;
use expr::{NormalizeFoldVisitor, ReorderSelectivityVisitor};
use maplibre_style_spec::mir::IntermediateSpec;
use maplibre_style_spec::spec::MaplibreStyleSpecification;
use serde_json::Value;
use source_util::precompute_vector_layer_info;
use typed_passes::{
    cleanup_typed, dead_elimination_typed, metadata_refinement_typed,
    precompute_vector_layer_info_typed, strip_metadata_typed,
};
use walk::walk_style_mut;

use crate::advisory::DataRewriteAdvisory;
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
    /// When enabled, compute a data rewrite advisory and rewrite expressions to match.
    pub data_advisory: bool,
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
            data_advisory: false,
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
pub fn optimize_style_json_value(v: &mut Value, mir: &IntermediateSpec, passes: &OptPasses) {
    optimize_style_json_value_with_stats(v, mir, passes, None);
}

/// Primary JSON entry point.  Structural passes use typed structs internally;
/// expression passes operate on the JSON directly.
///
/// Returns a [`DataRewriteAdvisory`] when `passes.data_advisory` is enabled and stats are provided.
pub fn optimize_style_json_value_with_stats(
    v: &mut Value,
    mir: &IntermediateSpec,
    passes: &OptPasses,
    stats: Option<&TileStatistics>,
) -> Option<DataRewriteAdvisory> {
    run_pipeline(v, mir, passes, stats)
}

/// Primary typed entry point.  Builds a parallel JSON value for expression
/// passes and syncs results between the two representations.
pub fn optimize_style(
    style: &mut MaplibreStyleSpecification,
    mir: &IntermediateSpec,
    passes: &OptPasses,
    stats: Option<&TileStatistics>,
) -> Option<DataRewriteAdvisory> {
    let Ok(mut v) = serde_json::to_value(&*style) else {
        return None;
    };
    let advisory = run_pipeline(&mut v, mir, passes, stats);
    // Sync final JSON state back to the typed struct (best-effort).
    if let Ok(updated) = serde_json::from_value::<MaplibreStyleSpecification>(v) {
        *style = updated;
    }
    advisory
}

// ── Pipeline implementation ─────────────────────────────────────────────────

/// Single pipeline that takes a mutable JSON `Value` and uses typed structs
/// internally for the structural passes.
fn run_pipeline(
    v: &mut Value,
    mir: &IntermediateSpec,
    passes: &OptPasses,
    stats: Option<&TileStatistics>,
) -> Option<DataRewriteAdvisory> {
    if !passes.strip_metadata
        && !wants_normalize_fold(passes)
        && !passes.dead_elimination
        && !passes.metadata_refinement
        && !passes.strip_defaults
        && !passes.selectivity_reorder
        && !passes.cleanup
        && !passes.data_advisory
    {
        return None;
    }

    // 1. Strip metadata (typed).
    if passes.strip_metadata
        && let Ok(mut style) = serde_json::from_value::<MaplibreStyleSpecification>(v.clone())
    {
        strip_metadata_typed(&mut style);
        sync_typed_to_json(&style, v);
    }

    // 2. Expression passes (JSON walker).
    run_json_expression_passes(v, mir, passes, stats);

    // 2b. Data advisory: field analysis → advisory → expression rewrite.
    let advisory = if passes.data_advisory
        && let Some(stats) = stats
    {
        let layer_info = precompute_vector_layer_info(v);
        let field_analysis = field_analysis::analyze_fields(v, mir, &layer_info);
        let adv = crate::advisory::compute_advisory(&field_analysis, stats);
        advisory_rewrite::apply_advisory(v, mir, &layer_info, &adv);
        Some(adv)
    } else {
        None
    };

    // 3. Dead elimination (typed).
    if passes.dead_elimination
        && let Ok(mut style) = serde_json::from_value::<MaplibreStyleSpecification>(v.clone())
    {
        let layer_info = stats.map(|_| precompute_vector_layer_info_typed(&style));
        dead_elimination_typed(&mut style, stats, layer_info.as_deref());
        sync_typed_to_json(&style, v);
    }

    // 4. Metadata refinement (typed).
    if passes.metadata_refinement
        && let Ok(mut style) = serde_json::from_value::<MaplibreStyleSpecification>(v.clone())
    {
        let layer_info = stats.map(|_| precompute_vector_layer_info_typed(&style));
        metadata_refinement_typed(&mut style, stats, layer_info.as_deref());
        sync_typed_to_json(&style, v);
    }

    // 5. Cleanup (typed).
    if passes.cleanup
        && let Ok(mut style) = serde_json::from_value::<MaplibreStyleSpecification>(v.clone())
    {
        cleanup_typed(&mut style);
        sync_typed_to_json(&style, v);
    }

    advisory
}

/// Run expression-level passes on the JSON value.
fn run_json_expression_passes(
    v: &mut Value,
    mir: &IntermediateSpec,
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

// ── Sync helpers ────────────────────────────────────────────────────────────

/// Sync typed struct → JSON.  Merges typed scalar fields into the JSON while
/// preserving paint/layout from the JSON side.
fn sync_typed_to_json(style: &MaplibreStyleSpecification, v: &mut Value) {
    let Some(obj) = v.as_object_mut() else { return };

    if style.metadata.is_none() {
        obj.remove("metadata");
    }

    if let Ok(sources_val) = serde_json::to_value(&style.sources) {
        obj.insert("sources".to_string(), sources_val);
    }

    if let Ok(typed_layers_val) = serde_json::to_value(&style.layers) {
        if let (Some(json_layers), Some(typed_layers)) = (
            obj.get("layers").and_then(Value::as_array).cloned(),
            typed_layers_val.as_array(),
        ) {
            if json_layers.len() == typed_layers.len() {
                let merged: Vec<Value> = json_layers
                    .into_iter()
                    .zip(typed_layers)
                    .map(|(jl, tl)| merge_layer_json(jl, tl))
                    .collect();
                obj.insert("layers".to_string(), Value::Array(merged));
            } else {
                obj.insert("layers".to_string(), typed_layers_val);
            }
        } else {
            obj.insert("layers".to_string(), typed_layers_val);
        }
    }
}

/// Merge typed scalar fields into a JSON layer, keeping original paint/layout.
fn merge_layer_json(mut original: Value, typed: &Value) -> Value {
    let (Some(orig_obj), Some(typed_obj)) = (original.as_object_mut(), typed.as_object()) else {
        return original;
    };
    for key in &[
        "id",
        "type",
        "source",
        "source-layer",
        "filter",
        "minzoom",
        "maxzoom",
        "metadata",
        "ref",
    ] {
        match typed_obj.get(*key) {
            Some(val) if !val.is_null() => {
                orig_obj.insert((*key).to_string(), val.clone());
            }
            _ => {
                orig_obj.remove(*key);
            }
        }
    }
    original
}

// ── Tests ───────────────────────────────────────────────────────────────────

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

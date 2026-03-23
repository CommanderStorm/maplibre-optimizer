//! Style optimization pipeline: expression normalisation, dead-code removal, metadata, reordering.
//!
//! Two entry points:
//!
//! - [`optimize_style`] is the typed entry point.  Structural passes work
//!   directly on `&mut MaplibreStyleSpecification`; expression passes
//!   temporarily serialize to JSON for the schema-guided walker and sync
//!   only filter changes back (paint/layout in the typed struct is left as-is
//!   because `["literal", x]` produced by constant-folding is not accepted by
//!   the generated numeric paint property types).
//!
//! - [`optimize_style_json_value`] / [`optimize_style_json_value_with_stats`]
//!   run expression passes directly on the JSON (no typed round-trip, so all
//!   optimizer-produced forms survive).  Structural passes then run in one
//!   batch: one deserialize → all structural passes → one `sync_typed_to_json`.

mod cleanup;
mod dead;
mod defaults;
pub(crate) mod expr;
mod metadata;
pub(crate) mod selectivity;
pub(crate) mod source_util;
mod strip;
pub(crate) mod walk;
mod zoom;

use cleanup::cleanup;
use dead::dead_elimination;
use defaults::StripDefaultsVisitor;
use expr::{NormalizeFoldVisitor, ReorderSelectivityVisitor};
use maplibre_style_spec::mir::MirSpec;
use maplibre_style_spec::spec::{AnyLayer, LayerFilter, MaplibreStyleSpecification};
use metadata::metadata_refinement;
use serde_json::Value;
use source_util::{precompute_vector_layer_info, precompute_vector_layer_info_typed};
use strip::strip_metadata;
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

fn wants_expression_passes(passes: &OptPasses) -> bool {
    wants_normalize_fold(passes) || passes.strip_defaults || passes.selectivity_reorder
}

fn wants_structural_passes(passes: &OptPasses) -> bool {
    passes.strip_metadata || passes.dead_elimination || passes.metadata_refinement || passes.cleanup
}

// ── Public entry points ─────────────────────────────────────────────────────

/// Convenience wrapper (no stats).
pub fn optimize_style_json_value(v: &mut Value, mir: &MirSpec, passes: &OptPasses) {
    optimize_style_json_value_with_stats(v, mir, passes, None);
}

/// JSON entry point.  Expression passes run directly on the JSON so that all
/// optimizer-produced forms (e.g. `["literal", 0.5]` from constant-folding a
/// numeric paint property) are preserved verbatim.  Structural passes then run
/// in a single deserialize → run → `sync_typed_to_json` cycle.
pub fn optimize_style_json_value_with_stats(
    v: &mut Value,
    mir: &MirSpec,
    passes: &OptPasses,
    stats: Option<&TileStatistics>,
) {
    if !wants_expression_passes(passes) && !wants_structural_passes(passes) {
        return;
    }

    // 1. Expression passes directly on JSON (no typed round-trip).
    run_json_expression_passes(v, mir, passes, stats);

    // 2. Structural passes — one deserialize + run all + one sync.
    if wants_structural_passes(passes)
        && let Ok(mut style) = serde_json::from_value::<MaplibreStyleSpecification>(v.clone())
    {
        run_structural_passes(&mut style, passes, stats);
        sync_typed_to_json(&style, v);
    }
}

/// Typed entry point.  Structural passes work directly on the typed struct.
/// Expression passes serialize to JSON, run the walker, then sync only filter
/// changes back; paint/layout simplification results in the typed output are
/// dropped because `["literal", x]` is not a valid form for generated numeric
/// paint types.  This limitation will be resolved in Phase 3.
pub fn optimize_style(
    style: &mut MaplibreStyleSpecification,
    mir: &MirSpec,
    passes: &OptPasses,
    stats: Option<&TileStatistics>,
) {
    run_pipeline(style, mir, passes, stats);
}

// ── Pipeline implementation ─────────────────────────────────────────────────

/// Typed-primary pipeline used by [`optimize_style`].
fn run_pipeline(
    style: &mut MaplibreStyleSpecification,
    mir: &MirSpec,
    passes: &OptPasses,
    stats: Option<&TileStatistics>,
) {
    if !wants_expression_passes(passes) && !wants_structural_passes(passes) {
        return;
    }

    // 1. Strip metadata (typed, direct) — before expression passes so that the
    //    serialized JSON excludes metadata.
    if passes.strip_metadata {
        strip_metadata(style);
    }

    // 2. Expression passes: serialize → JSON walker → sync only filter changes back.
    if wants_expression_passes(passes)
        && let Ok(mut v) = serde_json::to_value(&*style)
    {
        run_json_expression_passes(&mut v, mir, passes, stats);
        sync_filters_from_json(style, &v);
    }

    // 3–5. Remaining structural passes (typed, direct).
    run_structural_passes(style, passes, stats);
}

/// Run structural passes in order.  `strip_metadata` is idempotent and
/// harmless to call even if it already ran in [`run_pipeline`].
fn run_structural_passes(
    style: &mut MaplibreStyleSpecification,
    passes: &OptPasses,
    stats: Option<&TileStatistics>,
) {
    if passes.strip_metadata {
        strip_metadata(style);
    }

    if passes.dead_elimination {
        let layer_info = stats.map(|_| precompute_vector_layer_info_typed(style));
        dead_elimination(style, stats, layer_info.as_deref());
    }

    if passes.metadata_refinement {
        let layer_info = stats.map(|_| precompute_vector_layer_info_typed(style));
        metadata_refinement(style, stats, layer_info.as_deref());
    }

    if passes.cleanup {
        cleanup(style);
    }
}

/// Sync only the `filter` field of each layer from a post-expression-pass JSON
/// value back into the typed struct.  Filters that fail to deserialize (should
/// not happen with well-formed expression-pass output) are silently dropped.
fn sync_filters_from_json(style: &mut MaplibreStyleSpecification, v: &Value) {
    let Some(json_layers) = v.get("layers").and_then(Value::as_array) else {
        return;
    };
    for (typed_layer, json_layer) in style.layers.iter_mut().zip(json_layers.iter()) {
        let new_filter = json_layer
            .get("filter")
            .filter(|f| !f.is_null())
            .cloned()
            .and_then(LayerFilter::from_value);
        match typed_layer {
            AnyLayer::Typed(t) => t.common_mut().filter = new_filter,
            AnyLayer::Ref(r) => r.filter = new_filter,
        }
    }
}

// ── Sync helpers (JSON entry point only) ────────────────────────────────────

/// Sync typed struct → JSON.  Merges typed scalar fields into the JSON while
/// preserving paint/layout from the JSON side (which has expression-pass results).
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
    fn cleanup_removes_zero_opacity_background() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[
            {"id":"bg","type":"background","paint":{"background-opacity":0}}
        ]});
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
    fn cleanup_removes_zero_opacity_color_relief() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{"s":{"type":"raster-dem","url":"x"}},"layers":[
            {"id":"cr","type":"color-relief","source":"s","paint":{"color-relief-opacity":0}}
        ]});
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
    fn cleanup_removes_zero_opacity_symbol() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{"s":{"type":"vector","url":"x"}},"layers":[
            {"id":"sym","type":"symbol","source":"s","source-layer":"l","paint":{"icon-opacity":0,"text-opacity":0}}
        ]});
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
    fn cleanup_keeps_symbol_with_one_nonzero_opacity() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{"s":{"type":"vector","url":"x"}},"layers":[
            {"id":"sym","type":"symbol","source":"s","source-layer":"l","paint":{"icon-opacity":0,"text-opacity":1}}
        ]});
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                cleanup: true,
                ..Default::default()
            },
        );
        assert_eq!(v["layers"].as_array().unwrap().len(), 1);
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

    // ── Paint-based minzoom inference tests ─────────────────────────────────

    #[test]
    #[allow(clippy::float_cmp)]
    fn paint_minzoom_interpolate_leading_zeros() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[
            {"id":"x","type":"line","paint":{"line-width":["interpolate",["linear"],["zoom"],13.5,0,14,2.5]}}
        ]});
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                metadata_refinement: true,
                ..Default::default()
            },
        );
        assert_eq!(v["layers"][0]["minzoom"].as_f64().unwrap(), 13.5);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn paint_minzoom_step_zero_default() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[
            {"id":"x","type":"line","paint":{"line-width":["step",["zoom"],0,15,2]}}
        ]});
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                metadata_refinement: true,
                ..Default::default()
            },
        );
        assert_eq!(v["layers"][0]["minzoom"].as_f64().unwrap(), 15.0);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn paint_minzoom_multiple_zero_stops() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[
            {"id":"x","type":"line","paint":{"line-width":["interpolate",["linear"],["zoom"],5,0,10,0,14,2.5]}}
        ]});
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                metadata_refinement: true,
                ..Default::default()
            },
        );
        assert_eq!(v["layers"][0]["minzoom"].as_f64().unwrap(), 10.0);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn paint_minzoom_combined_width_opacity() {
        let mir = sample_mir();
        // line-width: last zero stop at 10 (transitions 10→14),
        // line-opacity: zero until step at 12.
        // Both must be non-zero → max(10, 12) = 12.
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[
            {"id":"x","type":"line","paint":{
                "line-width":["interpolate",["linear"],["zoom"],10,0,14,2],
                "line-opacity":["step",["zoom"],0,12,1]
            }}
        ]});
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                metadata_refinement: true,
                ..Default::default()
            },
        );
        assert_eq!(v["layers"][0]["minzoom"].as_f64().unwrap(), 12.0);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn paint_minzoom_existing_tighter_preserved() {
        let mir = sample_mir();
        // Existing minzoom: 16, paint suggests 14 → stays 16.
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[
            {"id":"x","type":"line","minzoom":16,"paint":{"line-width":["interpolate",["linear"],["zoom"],10,0,14,2]}}
        ]});
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                metadata_refinement: true,
                ..Default::default()
            },
        );
        assert_eq!(v["layers"][0]["minzoom"].as_f64().unwrap(), 16.0);
    }

    #[test]
    fn paint_minzoom_non_zoom_expression_no_change() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[
            {"id":"x","type":"line","paint":{"line-width":["*",["get","w"],2]}}
        ]});
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                metadata_refinement: true,
                ..Default::default()
            },
        );
        assert!(v["layers"][0].get("minzoom").is_none());
    }

    #[test]
    fn paint_minzoom_all_stops_zero_no_minzoom() {
        let mir = sample_mir();
        // All stops zero → INFINITY → we skip setting minzoom (let cleanup handle).
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[
            {"id":"x","type":"line","paint":{"line-width":["interpolate",["linear"],["zoom"],5,0,10,0]}}
        ]});
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                metadata_refinement: true,
                ..Default::default()
            },
        );
        assert!(v["layers"][0].get("minzoom").is_none());
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn paint_minzoom_end_to_end_full_pipeline() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{"s":{"type":"vector","url":"x"}},"layers":[
            {"id":"road","type":"line","source":"s","source-layer":"transportation",
             "filter":["==",["get","class"],"motorway"],
             "paint":{"line-width":["interpolate",["linear"],["zoom"],5,0,10,0,14,2.5]}}
        ]});
        let passes = OptPasses::all();
        optimize_style_json_value(&mut v, &mir, &passes);
        assert!(v["layers"][0]["minzoom"].as_f64().unwrap() >= 10.0);
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

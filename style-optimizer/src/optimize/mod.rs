//! Style optimization pipeline: expression normalisation, dead-code removal, metadata, reordering.
//!
//! Two entry points:
//!
//! - [`optimize_style`] is the typed entry point.  Structural passes work
//!   directly on `&mut MaplibreStyleSpecification`; expression passes
//!   temporarily serialize to JSON for the schema-guided walker, then
//!   deserialize the result back to replace the full typed struct (the
//!   JSON walker's `NormalizeFoldVisitor` unwraps `["literal", scalar]`
//!   back to bare scalars, so the post-pass JSON round-trips cleanly).
//!
//! - [`optimize_style_json_value`] / [`optimize_style_json_value_with_stats`]
//!   run expression passes directly on the JSON (no typed round-trip, so all
//!   optimizer-produced forms survive).  Structural passes then run in one
//!   batch: one deserialize → all structural passes → one `sync_typed_to_json`.

mod cleanup;
mod color;
mod dead;
mod defaults;
pub(crate) mod expr;
mod merge;
mod metadata;
mod ramp;
pub(crate) mod selectivity;
pub(crate) mod source_util;
mod strip;
pub(crate) mod walk;
mod zoom;

use cleanup::cleanup;
use color::MinifyColorsVisitor;
use dead::dead_elimination;
use defaults::StripDefaultsVisitor;
use expr::{NormalizeFoldVisitor, ReorderSelectivityVisitor};
use maplibre_style_spec::mir::MirSpec;
use maplibre_style_spec::spec::MaplibreStyleSpecification;
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
    pub minify_colors: bool,
    pub cleanup: bool,
    pub layer_merge: bool,
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
            minify_colors: true,
            cleanup: true,
            layer_merge: true,
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
    wants_normalize_fold(passes)
        || passes.strip_defaults
        || passes.selectivity_reorder
        || passes.minify_colors
}

fn wants_structural_passes(passes: &OptPasses) -> bool {
    passes.strip_metadata
        || passes.dead_elimination
        || passes.metadata_refinement
        || passes.cleanup
        || passes.layer_merge
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

    // 3. Zoom-bounded stop pruning — runs after structural passes have
    //    tightened zoom bounds (metadata_refinement) and synced them to JSON.
    if passes.simplify_expressions {
        ramp::prune_zoom_stops(v);
    }

    // 3b. Re-fold after structural changes: metadata_refinement may remove
    //     zoom predicates leaving residual wrappers (e.g. `["all", x]`),
    //     and ramp pruning may collapse ramps to bare literals.
    if wants_normalize_fold(passes) && wants_structural_passes(passes) {
        run_normalize_fold_only(v, mir, passes, stats);
    }

    // 4. Layer merging — runs on JSON after all other passes so that dead
    //    layers are gone and expressions are simplified before grouping.
    if passes.layer_merge {
        merge::layer_merge(v, mir);

        // 4b. Re-run expression passes on merge-generated expressions:
        //     layer_merge synthesises new case/match/any expressions that
        //     benefit from fold/simplify, default stripping, and
        //     selectivity reordering.
        run_json_expression_passes(v, mir, passes, stats);

        // 4c. Prune zoom stops in newly-synthesised properties.
        if passes.simplify_expressions {
            ramp::prune_zoom_stops(v);
        }
    }
}

/// Typed entry point.  Delegates to the JSON pipeline so that expression-pass
/// results for all properties (paint, layout, and filter) are preserved.  The
/// final JSON is deserialized back into a typed struct.
pub fn optimize_style(
    style: &mut MaplibreStyleSpecification,
    mir: &MirSpec,
    passes: &OptPasses,
    stats: Option<&TileStatistics>,
) {
    let Ok(mut v) = serde_json::to_value(&*style) else {
        return;
    };
    optimize_style_json_value_with_stats(&mut v, mir, passes, stats);
    if let Ok(updated) = serde_json::from_value::<MaplibreStyleSpecification>(v) {
        *style = updated;
    }
}

/// Run structural passes in order.
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

// ── Sync helpers (JSON entry point only) ────────────────────────────────────

/// Sync typed struct → JSON.  Merges typed scalar fields into the JSON while
/// preserving paint/layout from the JSON side (which has expression-pass results).
fn sync_typed_to_json(style: &MaplibreStyleSpecification, v: &mut Value) {
    let Some(obj) = v.as_object_mut() else { return };

    if style.metadata.is_none() {
        obj.remove("metadata");
    }

    // Sync root defaults that were stripped by cleanup.
    for key in ["bearing", "pitch", "roll", "state", "transition"] {
        let is_none = match key {
            "bearing" => style.bearing.is_none(),
            "pitch" => style.pitch.is_none(),
            "roll" => style.roll.is_none(),
            "state" => style.state.is_none(),
            "transition" => style.transition.is_none(),
            _ => false,
        };
        if is_none {
            obj.remove(key);
        }
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

/// Replace the JSON layer with the typed serialization.
///
/// The typed struct was deserialized from the JSON *after* expression passes,
/// so it already contains those results.  Using it as the authoritative source
/// ensures that structural-pass changes to paint/layout (empty-object
/// removal, zoom-stop pruning, …) are reflected in the output.
fn merge_layer_json(_original: Value, typed: &Value) -> Value {
    typed.clone()
}

/// Run expression-level passes on the JSON value.
fn run_json_expression_passes(
    v: &mut Value,
    mir: &MirSpec,
    passes: &OptPasses,
    stats: Option<&TileStatistics>,
) {
    if !wants_expression_passes(passes) {
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

    if passes.minify_colors {
        walk_style_mut(v, mir, &mut MinifyColorsVisitor);
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

/// Run only the normalize/fold fixpoint — a lightweight cleanup pass for
/// residual wrappers left by structural changes (e.g. `["all", x]` after a
/// predicate was removed, or a ramp collapsed to a literal).
fn run_normalize_fold_only(
    v: &mut Value,
    mir: &MirSpec,
    passes: &OptPasses,
    stats: Option<&TileStatistics>,
) {
    let layer_info = stats.map(|_| precompute_vector_layer_info(v));
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

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::path::Path;

    use insta::assert_yaml_snapshot;

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
        assert_yaml_snapshot!(v["layers"][0], @r#"
        filter:
          - "=="
          - 1
          - 1
        id: x
        type: fill
        "#);
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
        assert_yaml_snapshot!(v["layers"][0], @r#"
        filter:
          - "=="
          - 1
          - 1
        id: x
        type: fill
        "#);
    }

    #[test]
    fn simplify_unary_all() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[{"id":"x","type":"fill","filter":["all",["==",1,1]]}]});
        optimize_style_json_value(&mut v, &mir, &passes_unary_only());
        assert_yaml_snapshot!(v["layers"][0], @r#"
        filter:
          - "=="
          - 1
          - 1
        id: x
        type: fill
        "#);
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
        assert_yaml_snapshot!(v["layers"][0], @r"
        filter:
          - has
          - x
        id: x
        type: fill
        ");
    }

    #[test]
    fn simplify_triple_not() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[{"id":"x","type":"fill","filter":["!",["!",["!",["has","x"]]]]}]});
        optimize_style_json_value(&mut v, &mir, &passes_unary_only());
        assert_yaml_snapshot!(v["layers"][0], @r#"
        filter:
          - "!"
          - - has
            - x
        id: x
        type: fill
        "#);
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
        assert_yaml_snapshot!(v["layers"][0], @r#"
        filter:
          - "!="
          - - get
            - a
          - 1
        id: x
        type: fill
        "#);
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
        assert_yaml_snapshot!(v["layers"][0], @r"
        filter: false
        id: x
        type: fill
        ");
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
        assert_yaml_snapshot!(v, @r#"
        layers:
          - filter:
              - "=="
              - 1
              - 1
            id: y
            source: b
            source-layer: r
            type: line
        sources:
          b:
            type: vector
            url: "https://example/b.json"
        version: 8
        "#);
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
        assert_yaml_snapshot!(v["layers"][0], @r"
        filter: true
        id: x
        minzoom: 7
        type: fill
        ");
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
        assert_yaml_snapshot!(v["layers"][0], @r#"
        filter:
          - any
          - - literal
            - true
          - - "=="
            - 1
            - 2
        id: x
        type: fill
        "#);
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
        assert_yaml_snapshot!(v["layers"][0], @r"
        filter: 3
        id: x
        type: fill
        ");
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
        assert_yaml_snapshot!(v["layers"][0], @r"
        filter: hello world
        id: x
        type: fill
        ");
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
        assert_yaml_snapshot!(v["layers"][0], @r"
        id: x
        paint:
          fill-opacity: 0.5
        type: fill
        ");
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
        assert_yaml_snapshot!(v["layers"][0], @r"
        id: x
        paint:
          line-width: 2
        type: line
        ");
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
        assert_yaml_snapshot!(v["layers"][0], @r##"
        id: x
        paint:
          line-color:
            - match
            - - get
              - class
            - - motorway
              - trunk
            - "#ff0000"
            - primary
            - "#ff6600"
            - "#cccccc"
        type: line
        "##);
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
        assert_yaml_snapshot!(v, @r"
        layers:
          - id: x
            type: fill
        sources: {}
        version: 8
        ");
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
        assert_yaml_snapshot!(v["layers"][0], @r##"
        id: x
        paint:
          fill-color: "#f00"
        type: fill
        "##);
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
        assert_yaml_snapshot!(v["layers"], @"[]");
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
        assert_yaml_snapshot!(v["layers"], @"[]");
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
        assert_yaml_snapshot!(v["layers"], @"[]");
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
        assert_yaml_snapshot!(v["layers"], @"[]");
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
        assert_yaml_snapshot!(v["layers"], @r"
        - id: sym
          paint:
            icon-opacity: 0
            text-opacity: 1
          source: s
          source-layer: l
          type: symbol
        ");
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
        assert_yaml_snapshot!(v["layers"][0], @r#"
        filter:
          - "=="
          - - get
            - class
          - river
        id: x
        minzoom: 7
        type: fill
        "#);
    }

    // ── Paint-based minzoom inference tests ─────────────────────────────────

    #[test]
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
        assert_yaml_snapshot!(v["layers"][0], @r"
        id: x
        minzoom: 13.5
        paint:
          line-width:
            - interpolate
            - - linear
            - - zoom
            - 13.5
            - 0
            - 14
            - 2.5
        type: line
        ");
    }

    #[test]
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
        assert_yaml_snapshot!(v["layers"][0], @r"
        id: x
        minzoom: 15
        paint:
          line-width:
            - step
            - - zoom
            - 0
            - 15
            - 2
        type: line
        ");
    }

    #[test]
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
        assert_yaml_snapshot!(v["layers"][0], @r"
        id: x
        minzoom: 10
        paint:
          line-width:
            - interpolate
            - - linear
            - - zoom
            - 5
            - 0
            - 10
            - 0
            - 14
            - 2.5
        type: line
        ");
    }

    #[test]
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
        assert_yaml_snapshot!(v["layers"][0], @r"
        id: x
        minzoom: 12
        paint:
          line-opacity:
            - step
            - - zoom
            - 0
            - 12
            - 1
          line-width:
            - interpolate
            - - linear
            - - zoom
            - 10
            - 0
            - 14
            - 2
        type: line
        ");
    }

    #[test]
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
        assert_yaml_snapshot!(v["layers"][0], @r"
        id: x
        minzoom: 16
        paint:
          line-width:
            - interpolate
            - - linear
            - - zoom
            - 10
            - 0
            - 14
            - 2
        type: line
        ");
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
        assert_yaml_snapshot!(v["layers"][0], @r#"
        id: x
        paint:
          line-width:
            - "*"
            - - get
              - w
            - 2
        type: line
        "#);
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
        assert_yaml_snapshot!(v["layers"][0], @r"
        id: x
        paint:
          line-width:
            - interpolate
            - - linear
            - - zoom
            - 5
            - 0
            - 10
            - 0
        type: line
        ");
    }

    #[test]
    fn paint_minzoom_end_to_end_full_pipeline() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{"s":{"type":"vector","url":"x"}},"layers":[
            {"id":"road","type":"line","source":"s","source-layer":"transportation",
             "filter":["==",["get","class"],"motorway"],
             "paint":{"line-width":["interpolate",["linear"],["zoom"],5,0,10,0,14,2.5]}}
        ]});
        let passes = OptPasses::all();
        optimize_style_json_value(&mut v, &mir, &passes);
        assert_yaml_snapshot!(v["layers"][0], @r#"
        filter:
          - "=="
          - - get
            - class
          - motorway
        id: road
        minzoom: 10
        paint:
          line-width:
            - interpolate
            - - linear
            - - zoom
            - 10
            - 0
            - 14
            - 2.5
        source: s
        source-layer: transportation
        type: line
        "#);
    }

    // ── Stats-driven tests ──────────────────────────────────────────────────

    use std::collections::BTreeMap;

    use crate::stats::{GeometryTypeStats, LayerStats, SourceStats};

    fn make_stats(layer_name: &str, layer_stats: LayerStats) -> TileStatistics {
        let mut layers = BTreeMap::new();
        layers.insert(layer_name.to_string(), layer_stats);
        let mut sources = BTreeMap::new();
        sources.insert("openmaptiles".to_string(), SourceStats { layers });
        TileStatistics {
            sources,
            sample_rate: 1.0,
        }
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
        assert_yaml_snapshot!(v["layers"], @"[]");
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
        assert_yaml_snapshot!(v["layers"][0], @r"
        id: water-fill
        maxzoom: 14
        minzoom: 6
        source: openmaptiles
        source-layer: water
        type: fill
        ");
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
        assert_yaml_snapshot!(v["layers"][0], @r"
        filter: false
        id: water-fill
        source: openmaptiles
        source-layer: water
        type: fill
        ");
    }

    // ── Single-value property folding ──────────────────────────────────────

    #[test]
    fn fold_get_single_value_string() {
        let mir = sample_mir();
        let stats = make_stats(
            "water",
            LayerStats {
                total_features: 500,
                properties: BTreeMap::from([(
                    "class".to_string(),
                    crate::stats::PropertyStats::String {
                        present_count: 500,
                        cardinality: 1,
                        value_counts: Some(indexmap::IndexMap::from([("lake".to_string(), 500)])),
                    },
                )]),
                ..Default::default()
            },
        );
        let mut v = serde_json::json!({"version":8,"sources":{"openmaptiles":{"type":"vector","url":"x"}},"layers":[{"id":"w","type":"fill","source":"openmaptiles","source-layer":"water","filter":["==",["get","class"],"lake"]}]});
        optimize_style_json_value_with_stats(
            &mut v,
            &mir,
            &OptPasses {
                constant_fold: true,
                ..Default::default()
            },
            Some(&stats),
        );
        assert_yaml_snapshot!(v["layers"][0], @r"
        filter: true
        id: w
        source: openmaptiles
        source-layer: water
        type: fill
        ");
    }

    #[test]
    fn fold_get_single_value_integer() {
        let mir = sample_mir();
        let stats = make_stats(
            "water",
            LayerStats {
                total_features: 100,
                properties: BTreeMap::from([(
                    "level".to_string(),
                    crate::stats::PropertyStats::Integer {
                        present_count: 100,
                        min: 3,
                        max: 3,
                        cardinality: 1,
                        value_counts: Some(BTreeMap::from([(3, 100)])),
                    },
                )]),
                ..Default::default()
            },
        );
        let mut v = serde_json::json!({"version":8,"sources":{"openmaptiles":{"type":"vector","url":"x"}},"layers":[{"id":"w","type":"fill","source":"openmaptiles","source-layer":"water","filter":[">=",["get","level"],2]}]});
        optimize_style_json_value_with_stats(
            &mut v,
            &mir,
            &OptPasses {
                constant_fold: true,
                ..Default::default()
            },
            Some(&stats),
        );
        assert_yaml_snapshot!(v["layers"][0], @r"
        filter: true
        id: w
        source: openmaptiles
        source-layer: water
        type: fill
        ");
    }

    #[test]
    fn no_fold_get_when_cardinality_gt_1() {
        let mir = sample_mir();
        let stats = make_stats(
            "water",
            LayerStats {
                total_features: 500,
                properties: BTreeMap::from([(
                    "class".to_string(),
                    crate::stats::PropertyStats::String {
                        present_count: 500,
                        cardinality: 2,
                        value_counts: Some(indexmap::IndexMap::from([
                            ("lake".to_string(), 300),
                            ("river".to_string(), 200),
                        ])),
                    },
                )]),
                ..Default::default()
            },
        );
        let mut v = serde_json::json!({"version":8,"sources":{"openmaptiles":{"type":"vector","url":"x"}},"layers":[{"id":"w","type":"fill","source":"openmaptiles","source-layer":"water","filter":["==",["get","class"],"lake"]}]});
        let original_filter = v["layers"][0]["filter"].clone();
        optimize_style_json_value_with_stats(
            &mut v,
            &mir,
            &OptPasses {
                constant_fold: true,
                ..Default::default()
            },
            Some(&stats),
        );
        // Should NOT fold — property has two distinct values.
        assert_eq!(v["layers"][0]["filter"], original_filter);
    }

    #[test]
    fn no_fold_get_when_not_present_on_all_features() {
        let mir = sample_mir();
        let stats = make_stats(
            "water",
            LayerStats {
                total_features: 500,
                properties: BTreeMap::from([(
                    "class".to_string(),
                    crate::stats::PropertyStats::String {
                        present_count: 400, // not on all features
                        cardinality: 1,
                        value_counts: Some(indexmap::IndexMap::from([("lake".to_string(), 400)])),
                    },
                )]),
                ..Default::default()
            },
        );
        let mut v = serde_json::json!({"version":8,"sources":{"openmaptiles":{"type":"vector","url":"x"}},"layers":[{"id":"w","type":"fill","source":"openmaptiles","source-layer":"water","filter":["==",["get","class"],"lake"]}]});
        let original_filter = v["layers"][0]["filter"].clone();
        optimize_style_json_value_with_stats(
            &mut v,
            &mir,
            &OptPasses {
                constant_fold: true,
                ..Default::default()
            },
            Some(&stats),
        );
        // Should NOT fold — property is not present on all features.
        assert_eq!(v["layers"][0]["filter"], original_filter);
    }

    #[test]
    fn fold_get_in_paint_expression() {
        let mir = sample_mir();
        let stats = make_stats(
            "water",
            LayerStats {
                total_features: 100,
                properties: BTreeMap::from([(
                    "depth".to_string(),
                    crate::stats::PropertyStats::Integer {
                        present_count: 100,
                        min: 5,
                        max: 5,
                        cardinality: 1,
                        value_counts: Some(BTreeMap::from([(5, 100)])),
                    },
                )]),
                ..Default::default()
            },
        );
        // ["get","depth"] in a paint expression should also be folded.
        let mut v = serde_json::json!({"version":8,"sources":{"openmaptiles":{"type":"vector","url":"x"}},"layers":[{"id":"w","type":"fill","source":"openmaptiles","source-layer":"water","paint":{"fill-opacity":["case",[">=",["get","depth"],3],1,0.5]}}]});
        optimize_style_json_value_with_stats(
            &mut v,
            &mir,
            &OptPasses {
                constant_fold: true,
                ..Default::default()
            },
            Some(&stats),
        );
        assert_yaml_snapshot!(v["layers"][0], @r"
        id: w
        paint:
          fill-opacity: 1
        source: openmaptiles
        source-layer: water
        type: fill
        ");
    }

    #[test]
    fn literal_scalar_inside_expression_unwrapped() {
        // ["literal", 1] (scalar) inside an expression is unwrapped to bare 1.
        // ["+", ["literal", 1], ["get", "foo"]] → ["+", 1, ["get", "foo"]]
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[{
            "id":"x","type":"fill",
            "paint":{"fill-opacity":["+",["literal",1],["get","foo"]]}
        }]});
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                constant_fold: true,
                simplify_unary: true,
                simplify_expressions: true,
                ..Default::default()
            },
        );
        assert_eq!(
            v["layers"][0]["paint"]["fill-opacity"],
            serde_json::json!(["+", 1, ["get", "foo"]])
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

    // ── Trivially-true filter removal ───────────────────────────────────

    #[test]
    fn fold_empty_all_to_true() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[
            {"id":"x","type":"fill","filter":["all"]}
        ]});
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                constant_fold: true,
                cleanup: true,
                ..Default::default()
            },
        );
        assert_yaml_snapshot!(v["layers"][0], @r"
        id: x
        type: fill
        ");
    }

    #[test]
    fn fold_empty_any_to_false() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[
            {"id":"x","type":"fill","filter":["any"]}
        ]});
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                constant_fold: true,
                ..Default::default()
            },
        );
        assert_yaml_snapshot!(v["layers"][0], @r"
        filter: false
        id: x
        type: fill
        ");
    }

    // ── Color minification ──────────────────────────────────────────────

    #[test]
    fn minify_rgba_in_paint() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[
            {"id":"x","type":"fill","paint":{"fill-color":"rgba(255,255,255,1)"}}
        ]});
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                minify_colors: true,
                ..Default::default()
            },
        );
        assert_yaml_snapshot!(v["layers"][0], @r##"
        id: x
        paint:
          fill-color: "#fff"
        type: fill
        "##);
    }

    #[test]
    fn minify_rgb_in_paint() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[
            {"id":"x","type":"fill","paint":{"fill-color":"rgb(0,0,0)"}}
        ]});
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                minify_colors: true,
                ..Default::default()
            },
        );
        assert_yaml_snapshot!(v["layers"][0], @r##"
        id: x
        paint:
          fill-color: "#000"
        type: fill
        "##);
    }

    #[test]
    fn minify_hsl_in_paint() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[
            {"id":"x","type":"fill","paint":{"fill-color":"hsl(0, 0%, 100%)"}}
        ]});
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                minify_colors: true,
                ..Default::default()
            },
        );
        assert_yaml_snapshot!(v["layers"][0], @r##"
        id: x
        paint:
          fill-color: "#fff"
        type: fill
        "##);
    }

    // ── Root default stripping ──────────────────────────────────────────

    #[test]
    fn strip_root_bearing_zero() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[],"bearing":0,"pitch":0,"roll":0});
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                cleanup: true,
                ..Default::default()
            },
        );
        assert_yaml_snapshot!(v, @r"
        layers: []
        sources: {}
        version: 8
        ");
    }

    #[test]
    fn preserve_root_nonzero_bearing() {
        let mir = sample_mir();
        let mut v = serde_json::json!({"version":8,"sources":{},"layers":[],"bearing":45});
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                cleanup: true,
                ..Default::default()
            },
        );
        assert_yaml_snapshot!(v, @r"
        bearing: 45
        layers: []
        sources: {}
        version: 8
        ");
    }
}

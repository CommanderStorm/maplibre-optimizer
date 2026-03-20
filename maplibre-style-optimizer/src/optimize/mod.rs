//! Style JSON rewrite pipeline: expression normalisation, dead-code removal, metadata, reordering.

mod dead;
mod expr;
mod metadata;
mod walk;

use dead::DeadEliminationVisitor;
use expr::{NormalizeFoldVisitor, ReorderSelectivityVisitor};
use maplibre_style_spec::mir::IntermediateSpec;
use metadata::MetadataRefinementVisitor;
use serde_json::Value;
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
    /// Constant folding for comparisons, boolean `any`/`all`, and `!` on literals (style-static only).
    pub constant_fold: bool,
    /// Drop layers with always-false filters and remove unused `sources` entries.
    pub dead_elimination: bool,
    /// Raise `minzoom` / tighten `maxzoom` from `["zoom"]` comparisons inside `filter` (`all` only).
    pub metadata_refinement: bool,
    /// Static selectivity reordering of `any` / `all` operands (literals first/last for short-circuit hints).
    pub selectivity_reorder: bool,
}

fn wants_normalize_fold(passes: &OptPasses) -> bool {
    passes.simplify_unary || passes.expression_kind || passes.constant_fold
}

/// Apply enabled passes in pipeline order with a fixpoint on normalisation + folding.
pub fn optimize_style_json_value(v: &mut Value, mir: &IntermediateSpec, passes: &OptPasses) {
    if !wants_normalize_fold(passes)
        && !passes.dead_elimination
        && !passes.metadata_refinement
        && !passes.selectivity_reorder
    {
        return;
    }

    if wants_normalize_fold(passes) {
        for _ in 0..NORMALIZE_FOLD_FIXPOINT_CAP {
            let mut visitor = NormalizeFoldVisitor {
                mir,
                passes,
                changed: false,
            };
            walk_style_mut(v, mir, &mut visitor);
            if !visitor.changed {
                break;
            }
        }
    }

    if passes.dead_elimination {
        walk_style_mut(v, mir, &mut DeadEliminationVisitor);
    }

    if passes.metadata_refinement {
        walk_style_mut(v, mir, &mut MetadataRefinementVisitor);
    }

    if passes.selectivity_reorder {
        walk_style_mut(v, mir, &mut ReorderSelectivityVisitor { mir });
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
}

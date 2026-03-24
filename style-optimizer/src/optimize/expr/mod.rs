//! Expression-tree passes: normalisation, constant folding, selectivity reordering.

mod fold;
mod reorder;
mod simplify;
pub(crate) mod util;

use fold::{
    try_algebraic_simplify, try_dead_branch_case, try_dead_branch_match_literal,
    try_filter_contradiction, try_fold_boolean_algebra, try_fold_coalesce_from_stats,
    try_fold_comparison, try_fold_comparison_from_stats, try_fold_geometry_type_from_stats,
    try_fold_get_from_stats, try_fold_has_from_stats, try_fold_not, try_fold_pure_operator,
    try_fold_redundant_coercion, try_fold_redundant_properties, try_negate_comparison,
    try_predicate_subsumption, try_prune_in_from_stats, try_prune_match_from_stats,
    try_range_tightening,
};
use maplibre_style_spec::mir::MirSpec;
use reorder::{LayerContext, reorder_selectivity};
use serde_json::Value;
use simplify::{
    try_boolean_flattening, try_demorgan, try_inline_let_var, try_merge_in_expressions,
    try_rewrite_any_to_in, try_simplify_case, try_simplify_coalesce,
    try_simplify_interpolate_or_step, try_simplify_match, try_simplify_single_in,
};
pub(crate) use util::extract_json_literal;

use super::OptPasses;
use super::source_util::VectorLayerInfo;
use super::walk::{PropertyContext, StyleVisitor};
use crate::stats::TileStatistics;

// ── Visitors ──────────────────────────────────────────────────────────────────

pub(crate) struct NormalizeFoldVisitor<'a> {
    pub mir: &'a MirSpec,
    pub passes: &'a OptPasses,
    pub stats: Option<&'a TileStatistics>,
    pub layer_info: Option<&'a [Option<VectorLayerInfo>]>,
    pub changed: bool,
}

impl StyleVisitor for NormalizeFoldVisitor<'_> {
    fn visit_filter(&mut self, layer_index: usize, _: &str, filter: &mut Value) {
        // Stats-driven: fold ["id"] → ["literal", null] when no feature IDs.
        if self.passes.constant_fold
            && should_fold_id(self.stats, self.layer_info, layer_index)
            && fold_id_to_null(filter)
        {
            self.changed = true;
        }
        // Stats-driven: fold ["has", p] → true when present in all features.
        if self.passes.constant_fold {
            fold_has_in_tree(
                filter,
                self.stats,
                self.layer_info,
                layer_index,
                &mut self.changed,
            );
        }
        // Stats-driven: fold ["get", p] → ["literal", v] when property is constant.
        if self.passes.constant_fold {
            fold_get_in_tree(
                filter,
                self.stats,
                self.layer_info,
                layer_index,
                &mut self.changed,
            );
        }
        // Stats-driven: fold ["=="|"!=", ["geometry-type"], T] → true/false.
        if self.passes.constant_fold {
            fold_geometry_type_in_tree(
                filter,
                self.stats,
                self.layer_info,
                layer_index,
                &mut self.changed,
            );
        }
        // Stats-driven: fold comparisons to true/false when provable from min/max.
        if self.passes.constant_fold {
            fold_comparison_in_tree(
                filter,
                self.stats,
                self.layer_info,
                layer_index,
                &mut self.changed,
            );
        }
        // Stats-driven: prune dead values from "in" expressions.
        if self.passes.constant_fold {
            fold_in_tree(
                filter,
                self.stats,
                self.layer_info,
                layer_index,
                &mut self.changed,
                try_prune_in_from_stats,
            );
        }
        // Stats-driven: prune dead arms from "match" expressions.
        if self.passes.constant_fold {
            fold_in_tree(
                filter,
                self.stats,
                self.layer_info,
                layer_index,
                &mut self.changed,
                try_prune_match_from_stats,
            );
        }
        // Stats-driven: remove dead coalesce arms.
        if self.passes.constant_fold {
            fold_in_tree(
                filter,
                self.stats,
                self.layer_info,
                layer_index,
                &mut self.changed,
                try_fold_coalesce_from_stats,
            );
        }
        normalize_and_fold(filter, self.mir, self.passes, &mut self.changed);
    }

    fn visit_property(&mut self, ctx: &PropertyContext<'_>, value: &mut Value) {
        // Stats-driven: fold ["get", p] → ["literal", v] in paint/layout expressions too.
        if self.passes.constant_fold {
            fold_get_in_tree(
                value,
                self.stats,
                self.layer_info,
                ctx.layer_index,
                &mut self.changed,
            );
        }
        // Stats-driven: fold comparisons in paint/layout expressions too.
        if self.passes.constant_fold {
            fold_comparison_in_tree(
                value,
                self.stats,
                self.layer_info,
                ctx.layer_index,
                &mut self.changed,
            );
        }
        // Stats-driven: prune dead values from "in"/"match"/"coalesce" in paint/layout too.
        if self.passes.constant_fold {
            fold_in_tree(
                value,
                self.stats,
                self.layer_info,
                ctx.layer_index,
                &mut self.changed,
                try_prune_in_from_stats,
            );
            fold_in_tree(
                value,
                self.stats,
                self.layer_info,
                ctx.layer_index,
                &mut self.changed,
                try_prune_match_from_stats,
            );
            fold_in_tree(
                value,
                self.stats,
                self.layer_info,
                ctx.layer_index,
                &mut self.changed,
                try_fold_coalesce_from_stats,
            );
        }
        normalize_and_fold(value, self.mir, self.passes, &mut self.changed);
        // Unwrap ["literal", scalar] → scalar for paint/layout properties.
        // Only safe for non-array, non-object types (numbers, strings, bools, null).
        // Arrays/objects inside ["literal", ...] encode literal array/object values
        // that would be misinterpreted as expressions if unwrapped.
        if let Value::Array(arr) = value
            && arr.len() == 2
            && arr[0].as_str() == Some("literal")
            && !arr[1].is_array()
            && !arr[1].is_object()
        {
            *value = arr[1].take();
            self.changed = true;
        }
    }
}

fn should_fold_id(
    stats: Option<&TileStatistics>,
    layer_info: Option<&[Option<VectorLayerInfo>]>,
    layer_index: usize,
) -> bool {
    let Some(stats) = stats else { return false };
    let Some(infos) = layer_info else {
        return false;
    };
    let Some(Some(info)) = infos.get(layer_index) else {
        return false;
    };
    stats
        .layer_stats(&info.source, &info.source_layer)
        .is_some_and(|ls| !ls.has_feature_ids)
}

/// Recursively replace `["id"]` with `["literal", null]`.
fn fold_id_to_null(v: &mut Value) -> bool {
    let Value::Array(arr) = v else {
        return false;
    };
    if arr.len() == 1 && arr[0].as_str() == Some("id") {
        *arr = vec![Value::String("literal".to_string()), Value::Null];
        return true;
    }
    let mut changed = false;
    for child in arr.iter_mut() {
        changed |= fold_id_to_null(child);
    }
    changed
}

/// Recursively fold `["has", p]` → `true` when stats confirm the property is always present.
fn fold_has_in_tree(
    v: &mut Value,
    stats: Option<&TileStatistics>,
    layer_info: Option<&[Option<VectorLayerInfo>]>,
    layer_index: usize,
    changed: &mut bool,
) {
    let Value::Array(arr) = v else {
        return;
    };
    if try_fold_has_from_stats(arr, stats, layer_info, layer_index) {
        *changed = true;
        return;
    }
    for child in arr.iter_mut() {
        fold_has_in_tree(child, stats, layer_info, layer_index, changed);
    }
}

/// Recursively fold `["get", p]` → `["literal", v]` when stats show a single constant value.
fn fold_get_in_tree(
    v: &mut Value,
    stats: Option<&TileStatistics>,
    layer_info: Option<&[Option<VectorLayerInfo>]>,
    layer_index: usize,
    changed: &mut bool,
) {
    let Value::Array(arr) = v else {
        return;
    };
    if try_fold_get_from_stats(arr, stats, layer_info, layer_index) {
        *changed = true;
        return;
    }
    for child in arr.iter_mut() {
        fold_get_in_tree(child, stats, layer_info, layer_index, changed);
    }
}

/// Recursively fold `["=="|"!=", ["geometry-type"], T]` → `true`/`false` based on geometry stats.
fn fold_geometry_type_in_tree(
    v: &mut Value,
    stats: Option<&TileStatistics>,
    layer_info: Option<&[Option<VectorLayerInfo>]>,
    layer_index: usize,
    changed: &mut bool,
) {
    let Value::Array(arr) = v else {
        return;
    };
    if try_fold_geometry_type_from_stats(arr, stats, layer_info, layer_index) {
        *changed = true;
        return;
    }
    for child in arr.iter_mut() {
        fold_geometry_type_in_tree(child, stats, layer_info, layer_index, changed);
    }
}

/// Recursively fold comparisons to `true`/`false` when stats prove the result is constant.
fn fold_comparison_in_tree(
    v: &mut Value,
    stats: Option<&TileStatistics>,
    layer_info: Option<&[Option<VectorLayerInfo>]>,
    layer_index: usize,
    changed: &mut bool,
) {
    let Value::Array(arr) = v else {
        return;
    };
    if try_fold_comparison_from_stats(arr, stats, layer_info, layer_index) {
        *changed = true;
        return;
    }
    for child in arr.iter_mut() {
        fold_comparison_in_tree(child, stats, layer_info, layer_index, changed);
    }
}

type StatsFoldFn =
    fn(&mut Vec<Value>, Option<&TileStatistics>, Option<&[Option<VectorLayerInfo>]>, usize) -> bool;

/// Generic recursive walker for stats-driven folds on expression arrays.
fn fold_in_tree(
    v: &mut Value,
    stats: Option<&TileStatistics>,
    layer_info: Option<&[Option<VectorLayerInfo>]>,
    layer_index: usize,
    changed: &mut bool,
    try_fold: StatsFoldFn,
) {
    let Value::Array(arr) = v else {
        return;
    };
    if try_fold(arr, stats, layer_info, layer_index) {
        *changed = true;
        return;
    }
    for child in arr.iter_mut() {
        fold_in_tree(child, stats, layer_info, layer_index, changed, try_fold);
    }
}

pub(crate) struct ReorderSelectivityVisitor<'a> {
    pub mir: &'a MirSpec,
    pub stats: Option<&'a TileStatistics>,
    pub layer_info: Option<&'a [Option<VectorLayerInfo>]>,
}

impl ReorderSelectivityVisitor<'_> {
    fn layer_context(&self, layer_index: usize) -> Option<LayerContext<'_>> {
        let stats = self.stats?;
        let info = self.layer_info?.get(layer_index)?.as_ref()?;
        // Verify stats exist for this layer.
        stats.layer_stats(&info.source, &info.source_layer)?;
        Some(LayerContext {
            source: &info.source,
            source_layer: &info.source_layer,
        })
    }
}

impl StyleVisitor for ReorderSelectivityVisitor<'_> {
    fn visit_filter(&mut self, layer_index: usize, _: &str, filter: &mut Value) {
        let ctx = self.layer_context(layer_index);
        reorder_selectivity(filter, self.mir, self.stats, ctx.as_ref());
    }

    fn visit_property(&mut self, ctx: &PropertyContext<'_>, value: &mut Value) {
        let lctx = self.layer_context(ctx.layer_index);
        reorder_selectivity(value, self.mir, self.stats, lctx.as_ref());
    }
}

// ── Recursive walkers ─────────────────────────────────────────────────────────

fn normalize_and_fold(v: &mut Value, mir: &MirSpec, passes: &OptPasses, changed: &mut bool) {
    match v {
        Value::Array(arr) => {
            for x in arr.iter_mut() {
                normalize_and_fold(x, mir, passes, changed);
            }
            if !arr.is_empty()
                && arr[0]
                    .as_str()
                    .is_some_and(|op| mir.expressions.operators.contains_key(op))
            {
                rewrite_expression_array(arr, mir, passes, changed);
            }
            if passes.simplify_unary && arr.len() == 2 {
                match arr.first().and_then(Value::as_str) {
                    Some("any" | "all") => {
                        let inner = arr[1].take();
                        *v = inner;
                        *changed = true;
                        normalize_and_fold(v, mir, passes, changed);
                    }
                    Some("!") => {
                        if let Value::Array(inner) = &arr[1]
                            && inner.len() == 2
                            && inner[0].as_str() == Some("!")
                        {
                            let grand = inner[1].clone();
                            *v = grand;
                            *changed = true;
                            normalize_and_fold(v, mir, passes, changed);
                        }
                    }
                    _ => {}
                }
            }
        }
        Value::Object(map) => {
            for x in map.values_mut() {
                normalize_and_fold(x, mir, passes, changed);
            }
        }
        _ => {}
    }
}

// ── Per-node rewriting ────────────────────────────────────────────────────────

fn rewrite_expression_array(
    arr: &mut Vec<Value>,
    mir: &MirSpec,
    passes: &OptPasses,
    changed: &mut bool,
) {
    while apply_one_rewrite_pass(arr, mir, passes) {
        *changed = true;
    }
}

fn apply_one_rewrite_pass(arr: &mut Vec<Value>, mir: &MirSpec, passes: &OptPasses) -> bool {
    if passes.expression_kind && try_negate_comparison(arr, mir) {
        return true;
    }
    if passes.constant_fold {
        if try_fold_boolean_algebra(arr) {
            return true;
        }
        if try_fold_not(arr) {
            return true;
        }
        if try_fold_comparison(arr) {
            return true;
        }
        if try_fold_pure_operator(arr, mir) {
            return true;
        }
        if try_algebraic_simplify(arr) {
            return true;
        }
        if try_dead_branch_case(arr) {
            return true;
        }
        if try_dead_branch_match_literal(arr) {
            return true;
        }
        if try_filter_contradiction(arr) {
            return true;
        }
        if try_range_tightening(arr) {
            return true;
        }
        if try_predicate_subsumption(arr) {
            return true;
        }
        if try_fold_redundant_coercion(arr) {
            return true;
        }
        if try_fold_redundant_properties(arr) {
            return true;
        }
    }
    if passes.simplify_expressions {
        if try_simplify_interpolate_or_step(arr) {
            return true;
        }
        if try_simplify_match(arr) {
            return true;
        }
        if try_rewrite_any_to_in(arr) {
            return true;
        }
        if try_simplify_case(arr) {
            return true;
        }
        if try_simplify_coalesce(arr) {
            return true;
        }
        if try_boolean_flattening(arr) {
            return true;
        }
        if try_demorgan(arr) {
            return true;
        }
        if try_simplify_single_in(arr) {
            return true;
        }
        if try_merge_in_expressions(arr) {
            return true;
        }
        if try_inline_let_var(arr) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use serde_json::json;

    use super::*;
    use crate::load_intermediate_spec_from_v8_path;

    fn sample_mir() -> MirSpec {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../upstream/src/reference/v8.json");
        load_intermediate_spec_from_v8_path(&path).expect("v8.json")
    }

    fn all_passes() -> OptPasses {
        OptPasses {
            constant_fold: true,
            simplify_unary: true,
            expression_kind: true,
            simplify_expressions: true,
            ..Default::default()
        }
    }

    #[test]
    fn normalize_and_fold_removes_properties_in_comparison() {
        let mir = sample_mir();
        let passes = all_passes();
        let mut expr = json!(["==", ["get", "k", ["properties"]], "v"]);
        let mut changed = false;
        normalize_and_fold(&mut expr, &mir, &passes, &mut changed);
        assert!(changed);
        assert_eq!(expr, json!(["==", ["get", "k"], "v"]));
    }
}

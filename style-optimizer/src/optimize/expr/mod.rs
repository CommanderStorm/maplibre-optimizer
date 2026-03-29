//! Expression-tree passes: normalisation, constant folding, selectivity reordering.

mod fold;
mod fold_stats;
mod reorder;
mod simplify;
pub(crate) mod util;

use fold::{
    try_algebraic_simplify, try_dead_branch_case, try_dead_branch_match_literal,
    try_filter_contradiction, try_fold_boolean_algebra, try_fold_comparison, try_fold_not,
    try_fold_pure_operator, try_fold_redundant_coercion, try_fold_redundant_properties,
    try_negate_comparison, try_predicate_subsumption, try_range_tightening,
};
use fold_stats::{
    try_fold_coalesce_from_stats, try_fold_comparison_from_stats,
    try_fold_geometry_type_from_stats, try_fold_get_from_stats, try_fold_has_from_stats,
    try_prune_data_ramp_from_stats, try_prune_in_from_stats, try_prune_match_from_stats,
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

// ── Stats-driven tree fold ───────────────────────────────────────────────────

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

// ── Rule table ───────────────────────────────────────────────────────────────

/// Which pass flag enables this rule.
enum RuleGate {
    ExpressionKind,
    ConstantFold,
    SimplifyExpressions,
}

/// Where the rule applies in the visitor.
enum RuleScope {
    /// Peephole: applied per expression node inside `rewrite_expression_array`.
    Peephole,
    /// Tree fold on filters only (stats-driven, recursive).
    FilterOnly,
    /// Tree fold on both filters and properties (stats-driven, recursive).
    FilterAndProperty,
}

enum RewriteRule {
    /// Rule that operates on the expression array alone.
    Pure {
        gate: RuleGate,
        scope: RuleScope,
        apply: fn(&mut Vec<Value>) -> bool,
    },
    /// Rule that also needs the MIR spec.
    WithMir {
        gate: RuleGate,
        scope: RuleScope,
        apply: fn(&mut Vec<Value>, &MirSpec) -> bool,
    },
    /// Stats-driven rule that needs tile statistics and layer info.
    WithStats {
        gate: RuleGate,
        scope: RuleScope,
        apply: StatsFoldFn,
    },
}

impl RuleGate {
    const fn enabled(&self, passes: &OptPasses) -> bool {
        match self {
            Self::ExpressionKind => passes.expression_kind,
            Self::ConstantFold => passes.constant_fold,
            Self::SimplifyExpressions => passes.simplify_expressions,
        }
    }
}

impl RewriteRule {
    const fn gate(&self) -> &RuleGate {
        match self {
            Self::Pure { gate, .. } | Self::WithMir { gate, .. } | Self::WithStats { gate, .. } => {
                gate
            }
        }
    }

    const fn scope(&self) -> &RuleScope {
        match self {
            Self::Pure { scope, .. }
            | Self::WithMir { scope, .. }
            | Self::WithStats { scope, .. } => scope,
        }
    }

    /// Try to apply as a peephole rewrite (single node). Returns true if rewritten.
    fn apply_peephole(&self, arr: &mut Vec<Value>, mir: &MirSpec) -> bool {
        match self {
            Self::Pure { apply, .. } => apply(arr),
            Self::WithMir { apply, .. } => apply(arr, mir),
            Self::WithStats { .. } => false,
        }
    }

    /// Try to apply as a recursive tree fold (stats-driven).
    fn apply_tree_fold(
        &self,
        v: &mut Value,
        stats: Option<&TileStatistics>,
        layer_info: Option<&[Option<VectorLayerInfo>]>,
        layer_index: usize,
        changed: &mut bool,
    ) {
        if let Self::WithStats { apply, .. } = self {
            fold_in_tree(v, stats, layer_info, layer_index, changed, *apply);
        }
    }
}

static RULES: &[RewriteRule] = &[
    // ── expression_kind ──
    RewriteRule::WithMir {
        gate: RuleGate::ExpressionKind,
        scope: RuleScope::Peephole,
        apply: try_negate_comparison,
    },
    // ── constant_fold: peephole ──
    RewriteRule::Pure {
        gate: RuleGate::ConstantFold,
        scope: RuleScope::Peephole,
        apply: try_fold_boolean_algebra,
    },
    RewriteRule::Pure {
        gate: RuleGate::ConstantFold,
        scope: RuleScope::Peephole,
        apply: try_fold_not,
    },
    RewriteRule::Pure {
        gate: RuleGate::ConstantFold,
        scope: RuleScope::Peephole,
        apply: try_fold_comparison,
    },
    RewriteRule::WithMir {
        gate: RuleGate::ConstantFold,
        scope: RuleScope::Peephole,
        apply: try_fold_pure_operator,
    },
    RewriteRule::Pure {
        gate: RuleGate::ConstantFold,
        scope: RuleScope::Peephole,
        apply: try_algebraic_simplify,
    },
    RewriteRule::Pure {
        gate: RuleGate::ConstantFold,
        scope: RuleScope::Peephole,
        apply: try_dead_branch_case,
    },
    RewriteRule::Pure {
        gate: RuleGate::ConstantFold,
        scope: RuleScope::Peephole,
        apply: try_dead_branch_match_literal,
    },
    RewriteRule::Pure {
        gate: RuleGate::ConstantFold,
        scope: RuleScope::Peephole,
        apply: try_filter_contradiction,
    },
    RewriteRule::Pure {
        gate: RuleGate::ConstantFold,
        scope: RuleScope::Peephole,
        apply: try_range_tightening,
    },
    RewriteRule::Pure {
        gate: RuleGate::ConstantFold,
        scope: RuleScope::Peephole,
        apply: try_predicate_subsumption,
    },
    RewriteRule::Pure {
        gate: RuleGate::ConstantFold,
        scope: RuleScope::Peephole,
        apply: try_fold_redundant_coercion,
    },
    RewriteRule::Pure {
        gate: RuleGate::ConstantFold,
        scope: RuleScope::Peephole,
        apply: try_fold_redundant_properties,
    },
    // ── constant_fold: stats-driven tree folds ──
    RewriteRule::WithStats {
        gate: RuleGate::ConstantFold,
        scope: RuleScope::FilterOnly,
        apply: try_fold_has_from_stats,
    },
    RewriteRule::WithStats {
        gate: RuleGate::ConstantFold,
        scope: RuleScope::FilterAndProperty,
        apply: try_fold_get_from_stats,
    },
    RewriteRule::WithStats {
        gate: RuleGate::ConstantFold,
        scope: RuleScope::FilterOnly,
        apply: try_fold_geometry_type_from_stats,
    },
    RewriteRule::WithStats {
        gate: RuleGate::ConstantFold,
        scope: RuleScope::FilterAndProperty,
        apply: try_fold_comparison_from_stats,
    },
    RewriteRule::WithStats {
        gate: RuleGate::ConstantFold,
        scope: RuleScope::FilterAndProperty,
        apply: try_prune_in_from_stats,
    },
    RewriteRule::WithStats {
        gate: RuleGate::ConstantFold,
        scope: RuleScope::FilterAndProperty,
        apply: try_prune_match_from_stats,
    },
    RewriteRule::WithStats {
        gate: RuleGate::ConstantFold,
        scope: RuleScope::FilterAndProperty,
        apply: try_fold_coalesce_from_stats,
    },
    RewriteRule::WithStats {
        gate: RuleGate::ConstantFold,
        scope: RuleScope::FilterAndProperty,
        apply: try_prune_data_ramp_from_stats,
    },
    // ── simplify_expressions ──
    RewriteRule::Pure {
        gate: RuleGate::SimplifyExpressions,
        scope: RuleScope::Peephole,
        apply: try_simplify_interpolate_or_step,
    },
    RewriteRule::Pure {
        gate: RuleGate::SimplifyExpressions,
        scope: RuleScope::Peephole,
        apply: try_simplify_match,
    },
    RewriteRule::Pure {
        gate: RuleGate::SimplifyExpressions,
        scope: RuleScope::Peephole,
        apply: try_rewrite_any_to_in,
    },
    RewriteRule::Pure {
        gate: RuleGate::SimplifyExpressions,
        scope: RuleScope::Peephole,
        apply: try_simplify_case,
    },
    RewriteRule::Pure {
        gate: RuleGate::SimplifyExpressions,
        scope: RuleScope::Peephole,
        apply: try_simplify_coalesce,
    },
    RewriteRule::Pure {
        gate: RuleGate::SimplifyExpressions,
        scope: RuleScope::Peephole,
        apply: try_boolean_flattening,
    },
    RewriteRule::Pure {
        gate: RuleGate::SimplifyExpressions,
        scope: RuleScope::Peephole,
        apply: try_demorgan,
    },
    RewriteRule::Pure {
        gate: RuleGate::SimplifyExpressions,
        scope: RuleScope::Peephole,
        apply: try_simplify_single_in,
    },
    RewriteRule::Pure {
        gate: RuleGate::SimplifyExpressions,
        scope: RuleScope::Peephole,
        apply: try_merge_in_expressions,
    },
    RewriteRule::Pure {
        gate: RuleGate::SimplifyExpressions,
        scope: RuleScope::Peephole,
        apply: try_inline_let_var,
    },
];

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
        if self.passes.constant_fold {
            // Special case: fold ["id"] → ["literal", null] when no feature IDs.
            if should_fold_id(self.stats, self.layer_info, layer_index) && fold_id_to_null(filter) {
                self.changed = true;
            }
            for rule in RULES {
                if !matches!(
                    rule.scope(),
                    RuleScope::FilterOnly | RuleScope::FilterAndProperty
                ) {
                    continue;
                }
                if !rule.gate().enabled(self.passes) {
                    continue;
                }
                rule.apply_tree_fold(
                    filter,
                    self.stats,
                    self.layer_info,
                    layer_index,
                    &mut self.changed,
                );
            }
        }
        normalize_and_fold(filter, self.mir, self.passes, &mut self.changed);
    }

    fn visit_property(&mut self, ctx: &PropertyContext<'_>, value: &mut Value) {
        if self.passes.constant_fold {
            for rule in RULES {
                if !matches!(rule.scope(), RuleScope::FilterAndProperty) {
                    continue;
                }
                if !rule.gate().enabled(self.passes) {
                    continue;
                }
                rule.apply_tree_fold(
                    value,
                    self.stats,
                    self.layer_info,
                    ctx.layer_index,
                    &mut self.changed,
                );
            }
        }
        normalize_and_fold(value, self.mir, self.passes, &mut self.changed);
        // Unwrap ["literal", scalar] → scalar.
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
            if arr.len() == 2 {
                match arr.first().and_then(Value::as_str) {
                    Some("any" | "all") if passes.simplify_unary => {
                        let inner = arr[1].take();
                        *v = inner;
                        *changed = true;
                        normalize_and_fold(v, mir, passes, changed);
                    }
                    Some("!") if passes.simplify_unary => {
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
                    // Unwrap ["literal", scalar] → scalar.  Scalars (numbers,
                    // strings, booleans, null) are unambiguous in expression
                    // context, so the wrapper is redundant.  This ensures the
                    // JSON form is canonical for typed round-trip stability.
                    Some("literal") if !arr[1].is_array() && !arr[1].is_object() => {
                        *v = arr[1].take();
                        *changed = true;
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
    RULES
        .iter()
        .filter(|r| matches!(r.scope(), RuleScope::Peephole) && r.gate().enabled(passes))
        .any(|r| r.apply_peephole(arr, mir))
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

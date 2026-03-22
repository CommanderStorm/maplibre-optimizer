//! Selectivity-based reordering of `any`/`all` operands.

use maplibre_style_spec::mir::MirSpec;
use serde_json::Value;

use super::util::bool_literal;
use crate::optimize::selectivity::estimate_selectivity;
use crate::stats::TileStatistics;

pub(super) struct LayerContext<'a> {
    pub source: &'a str,
    pub source_layer: &'a str,
}

pub(super) fn reorder_selectivity(
    v: &mut Value,
    mir: &MirSpec,
    stats: Option<&TileStatistics>,
    ctx: Option<&LayerContext<'_>>,
) {
    match v {
        Value::Array(arr) => {
            for x in arr.iter_mut() {
                reorder_selectivity(x, mir, stats, ctx);
            }
            maybe_reorder_any_all(arr, mir, stats, ctx);
        }
        Value::Object(map) => {
            for x in map.values_mut() {
                reorder_selectivity(x, mir, stats, ctx);
            }
        }
        _ => {}
    }
}

pub(super) fn maybe_reorder_any_all(
    arr: &mut Vec<Value>,
    mir: &MirSpec,
    stats: Option<&TileStatistics>,
    ctx: Option<&LayerContext<'_>>,
) {
    let op = match arr.first().and_then(Value::as_str) {
        Some("any") => "any",
        Some("all") => "all",
        _ => return,
    };
    if !mir.expressions.operators.contains_key(op) || arr.len() < 3 {
        return;
    }
    let head = arr[0].clone();
    let mut ops: Vec<Value> = arr.iter().skip(1).cloned().collect();

    // Check if we have data-driven selectivity available.
    let has_stats = stats.is_some() && ctx.is_some();

    if has_stats {
        let stats = stats.unwrap();
        let ctx = ctx.unwrap();
        // Data-driven: sort by estimated selectivity.
        // For "all": ascending (lowest selectivity = most likely false = best short-circuit).
        // For "any": descending (highest selectivity = most likely true = best short-circuit).
        // Unknown selectivity operands go in the middle.
        let mut with_sel: Vec<(Value, Option<f64>)> = ops
            .into_iter()
            .map(|v| {
                let sel = estimate_selectivity(&v, ctx.source, ctx.source_layer, stats);
                (v, sel)
            })
            .collect();

        with_sel.sort_by(|(_, a), (_, b)| {
            let key = |s: &Option<f64>| match s {
                Some(v) => (1, ordered_float(*v)),
                None => (1, ordered_float(0.5)), // middle
            };
            let (a_tier, a_val) = key(a);
            let (b_tier, b_val) = key(b);
            if op == "all" {
                // ascending selectivity
                a_tier.cmp(&b_tier).then(
                    a_val
                        .partial_cmp(&b_val)
                        .unwrap_or(std::cmp::Ordering::Equal),
                )
            } else {
                // descending selectivity
                b_tier.cmp(&a_tier).then(
                    b_val
                        .partial_cmp(&a_val)
                        .unwrap_or(std::cmp::Ordering::Equal),
                )
            }
        });

        // Also honor literal placement: true first in any, false first in all.
        with_sel.sort_by_key(|(v, _)| match (op, bool_literal(v)) {
            ("any", Some(true)) | ("all", Some(false)) => 0,
            ("any", Some(false)) | ("all", Some(true)) => 2,
            _ => 1,
        });

        ops = with_sel.into_iter().map(|(v, _)| v).collect();
    } else {
        // Static reordering: only move literals.
        if op == "any" {
            ops.sort_by_key(|v| match bool_literal(v) {
                Some(true) => 0,
                None => 1,
                Some(false) => 2,
            });
        } else {
            ops.sort_by_key(|v| match bool_literal(v) {
                Some(false) => 0,
                None => 1,
                Some(true) => 2,
            });
        }
    }

    let mut out = vec![head];
    out.extend(ops);
    *arr = out;
}

/// Wrapper for f64 ordering that handles NaN.
fn ordered_float(f: f64) -> f64 {
    if f.is_nan() { 0.5 } else { f }
}

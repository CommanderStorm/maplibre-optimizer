//! Tighten layer `minzoom` / `maxzoom` from zoom predicates in `filter`.

use serde_json::Value;

use super::expr::extract_json_literal;
use super::walk::StyleVisitor;

// ── Visitor ───────────────────────────────────────────────────────────────────

pub(crate) struct MetadataRefinementVisitor;

impl StyleVisitor for MetadataRefinementVisitor {
    fn visit_layer(&mut self, _: usize, _: &str, layer: &mut Value) {
        refine_layer_zoom_metadata(layer);
    }
}

// ── Implementation ────────────────────────────────────────────────────────────

fn refine_layer_zoom_metadata(layer: &mut Value) {
    let Some(obj) = layer.as_object_mut() else {
        return;
    };
    let Some(filter) = obj.get("filter").cloned() else {
        return;
    };

    let (lb_raw, ub_raw) = zoom_bounds_from_expression(&filter);
    let lb = lb_raw.map(zoom_lower_to_min_zoom);
    let ub = ub_raw.map(zoom_upper_to_max_zoom);

    if let Some(bound) = lb {
        match obj.get("minzoom").and_then(Value::as_f64) {
            Some(cur) => {
                if bound > cur {
                    obj.insert("minzoom".to_string(), json_number(bound));
                }
            }
            None => {
                obj.insert("minzoom".to_string(), json_number(bound));
            }
        }
    }

    if let Some(bound) = ub {
        match obj.get("maxzoom").and_then(Value::as_f64) {
            Some(cur) => {
                let tightened = cur.min(bound);
                obj.insert("maxzoom".to_string(), json_number(tightened));
            }
            None => {
                obj.insert("maxzoom".to_string(), json_number(bound));
            }
        }
    }

    // Pass 7: remove zoom predicates from the filter that were fully captured by
    // the minzoom/maxzoom we just set.
    let adopted_min = obj.get("minzoom").and_then(Value::as_f64);
    let adopted_max = obj.get("maxzoom").and_then(Value::as_f64);
    if (adopted_min.is_some() || adopted_max.is_some())
        && let Some(filter_mut) = obj.get_mut("filter")
    {
        remove_consumed_zoom_predicates(filter_mut, adopted_min, adopted_max);
    }
}

fn json_number(n: f64) -> Value {
    serde_json::Number::from_f64(n).map_or(Value::from(0), Value::Number)
}

/// Integer zoom level lower bound implied by comparisons (inclusive).
fn zoom_lower_to_min_zoom(n: f64) -> f64 {
    n.ceil()
}

/// Integer zoom level upper bound implied by comparisons (inclusive maxzoom semantics).
fn zoom_upper_to_max_zoom(n: f64) -> f64 {
    n.floor()
}

fn is_zoom_expr(val: &Value) -> bool {
    matches!(
        val,
        Value::Array(arr) if arr.len() == 1 && arr[0].as_str() == Some("zoom")
    )
}

/// Merge bounds from subexpressions: all must hold → intersect intervals.
fn zoom_bounds_from_expression(expr: &Value) -> (Option<f64>, Option<f64>) {
    match expr {
        Value::Array(arr) if !arr.is_empty() => {
            let head = arr[0].as_str();
            match head {
                Some("all") => {
                    let mut acc_low = None;
                    let mut acc_high = None;
                    for child in arr.iter().skip(1) {
                        let (child_low, child_high) = zoom_bounds_from_expression(child);
                        acc_low = match (acc_low, child_low) {
                            (None, next) => next,
                            (Some(acc), Some(next)) => Some(acc.max(next)),
                            (Some(acc), None) => Some(acc),
                        };
                        acc_high = match (acc_high, child_high) {
                            (None, next) => next,
                            (Some(acc), Some(next)) => Some(acc.min(next)),
                            (Some(acc), None) => Some(acc),
                        };
                    }
                    (acc_low, acc_high)
                }
                Some("any") => {
                    // Pass 8: union of all branch bounds.
                    // A bound is only valid if ALL branches contribute that bound direction
                    // (if any branch has no restriction, the `any` has no restriction).
                    let mut acc_low: Option<f64> = None;
                    let mut acc_high: Option<f64> = None;
                    let mut all_have_low = true;
                    let mut all_have_high = true;

                    for child in arr.iter().skip(1) {
                        let (cl, ch) = zoom_bounds_from_expression(child);
                        if cl.is_none() {
                            all_have_low = false;
                        }
                        if ch.is_none() {
                            all_have_high = false;
                        }
                        acc_low = match (acc_low, cl) {
                            (None, v) => v,
                            (Some(a), Some(b)) => Some(a.min(b)), // union → minimum
                            (Some(a), None) => Some(a),
                        };
                        acc_high = match (acc_high, ch) {
                            (None, v) => v,
                            (Some(a), Some(b)) => Some(a.max(b)), // union → maximum
                            (Some(a), None) => Some(a),
                        };
                    }

                    (
                        if all_have_low { acc_low } else { None },
                        if all_have_high { acc_high } else { None },
                    )
                }
                _ => zoom_bounds_from_compare(arr),
            }
        }
        _ => (None, None),
    }
}

/// Parse `[cmp, zoom|lit, lit|zoom]` (both orders).
fn zoom_bounds_from_compare(arr: &[Value]) -> (Option<f64>, Option<f64>) {
    if arr.len() != 3 {
        return (None, None);
    }
    let Some(op) = arr[0].as_str() else {
        return (None, None);
    };
    let (left, right) = (&arr[1], &arr[2]);
    let (zoom_first, other) = if is_zoom_expr(left) {
        (true, right)
    } else if is_zoom_expr(right) {
        (false, left)
    } else {
        return (None, None);
    };
    let Some(lit) = extract_json_literal(other) else {
        return (None, None);
    };
    let Some(n) = lit.as_f64() else {
        return (None, None);
    };

    match (op, zoom_first) {
        (">=", true) | ("<=", false) => (Some(n), None),
        (">=", false) | ("<=", true) => (None, Some(n)),
        (">", true) | ("<", false) => (Some(next_zoom(n)), None),
        (">", false) | ("<", true) => (None, Some(prev_zoom(n))),
        _ => (None, None),
    }
}

fn next_zoom(n: f64) -> f64 {
    n.floor() + 1.0
}

fn prev_zoom(n: f64) -> f64 {
    n.ceil() - 1.0
}

// ── Pass 7: Remove consumed zoom predicates ────────────────────────────────────

/// Remove zoom predicates from an `["all", ...]` filter that were exactly captured
/// by the layer's minzoom/maxzoom.
///
/// Only removes `[">=", ["zoom"], N]` / `["<=", ["zoom"], N]` (and their reversed forms)
/// where N is an integer that exactly matches the adopted bound.
fn remove_consumed_zoom_predicates(
    filter: &mut Value,
    adopted_min: Option<f64>,
    adopted_max: Option<f64>,
) {
    let Value::Array(arr) = &*filter else {
        return;
    };
    if arr.first().and_then(Value::as_str) != Some("all") {
        return;
    }

    let kept: Vec<Value> = std::iter::once(Value::String("all".to_string()))
        .chain(
            arr[1..]
                .iter()
                .filter(|child| !is_exact_zoom_predicate_consumed(child, adopted_min, adopted_max))
                .cloned(),
        )
        .collect();

    if kept.len() == arr.len() {
        return; // nothing removed
    }

    *filter = match kept.len() {
        1 => serde_json::json!(["literal", true]),
        2 => kept[1].clone(),
        _ => Value::Array(kept),
    };
}

/// Return true if `pred` is a simple `[>=/<= , ["zoom"], N]` predicate with integer N
/// whose bound is exactly the adopted minzoom or maxzoom.
fn is_exact_zoom_predicate_consumed(
    pred: &Value,
    adopted_min: Option<f64>,
    adopted_max: Option<f64>,
) -> bool {
    let Value::Array(arr) = pred else {
        return false;
    };
    if arr.len() != 3 {
        return false;
    }
    let Some(op) = arr[0].as_str() else {
        return false;
    };

    let (zoom_first, other) = if is_zoom_expr(&arr[1]) {
        (true, &arr[2])
    } else if is_zoom_expr(&arr[2]) {
        (false, &arr[1])
    } else {
        return false;
    };

    let Some(n) = extract_json_literal(other).and_then(|v| v.as_f64()) else {
        return false;
    };
    if n.fract() != 0.0 {
        return false; // only integer bounds are safe to remove
    }

    match (op, zoom_first) {
        // zoom >= N  or  N <= zoom  → lower bound N, must match adopted_min
        (">=", true) | ("<=", false) => adopted_min.is_some_and(|m| (m - n).abs() < f64::EPSILON),
        // zoom <= N  or  N >= zoom  → upper bound N, must match adopted_max
        ("<=", true) | (">=", false) => adopted_max.is_some_and(|m| (m - n).abs() < f64::EPSILON),
        _ => false,
    }
}

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
                    // Conservative: OR does not give a shared tightening without more analysis.
                    (None, None)
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

//! Zoom-bound extraction from filter expressions.
//!
//! Shared helpers used by [`super::typed_passes::metadata_refinement_typed`].

use serde_json::Value;

use super::expr::extract_json_literal;

// ── Zoom-bound extraction ────────────────────────────────────────────────────

fn is_zoom_expr(val: &Value) -> bool {
    matches!(
        val,
        Value::Array(arr) if arr.len() == 1 && arr[0].as_str() == Some("zoom")
    )
}

pub(super) fn zoom_bounds_from_expression(expr: &Value) -> (Option<f64>, Option<f64>) {
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
                            (Some(a), Some(b)) => Some(a.min(b)),
                            (Some(a), None) => Some(a),
                        };
                        acc_high = match (acc_high, ch) {
                            (None, v) => v,
                            (Some(a), Some(b)) => Some(a.max(b)),
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

// ── Remove consumed zoom predicates ────────────────────────────────────────

pub(super) fn remove_consumed_zoom_predicates(
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
        return;
    }

    *filter = match kept.len() {
        1 => serde_json::json!(["literal", true]),
        2 => kept[1].clone(),
        _ => Value::Array(kept),
    };
}

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
        return false;
    }

    match (op, zoom_first) {
        (">=", true) | ("<=", false) => adopted_min.is_some_and(|m| (m - n).abs() < f64::EPSILON),
        ("<=", true) | (">=", false) => adopted_max.is_some_and(|m| (m - n).abs() < f64::EPSILON),
        _ => false,
    }
}

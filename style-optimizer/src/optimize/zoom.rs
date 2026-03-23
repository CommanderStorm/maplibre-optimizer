//! Zoom-bound extraction from filter expressions.
//!
//! Shared helpers used by [`super::metadata::metadata_refinement`].

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

// ── Paint-based visibility minzoom ──────────────────────────────────────────

/// Analyze a paint property expression and return the zoom level at which it
/// first becomes non-zero.
///
/// Returns `Some(f64::INFINITY)` when the value is statically zero at all zooms
/// (caller should let cleanup handle that).  Returns `None` when no constraint
/// can be derived (non-zoom expression, data-driven, non-numeric stops, etc.).
#[allow(clippy::float_cmp)]
pub(super) fn visibility_minzoom_from_value(value: &Value) -> Option<f64> {
    match value {
        // Scalar literal (bare or wrapped in ["literal", n]).
        Value::Number(n) => {
            if n.as_f64() == Some(0.0) {
                Some(f64::INFINITY)
            } else {
                None
            }
        }
        Value::Array(arr) if arr.len() == 2 && arr[0].as_str() == Some("literal") => {
            let n = arr[1].as_f64()?;
            if n == 0.0 { Some(f64::INFINITY) } else { None }
        }
        Value::Array(arr) if arr.len() >= 4 => {
            let head = arr[0].as_str()?;
            match head {
                "interpolate" | "interpolate-hcl" | "interpolate-lab" => {
                    visibility_minzoom_interpolate(arr)
                }
                "step" => visibility_minzoom_step(arr),
                _ => None,
            }
        }
        _ => None,
    }
}

/// `["interpolate", curve, ["zoom"], z1, v1, z2, v2, ...]`
#[allow(clippy::float_cmp)]
fn visibility_minzoom_interpolate(arr: &[Value]) -> Option<f64> {
    // arr[1] = curve, arr[2] = input (must be ["zoom"]), then pairs.
    if arr.len() < 5 {
        return None;
    }
    if !is_zoom_expr(&arr[2]) {
        return None;
    }
    let pairs = &arr[3..];
    if !pairs.len().is_multiple_of(2) {
        return None;
    }

    let mut last_zero_stop = None;
    let mut all_zero = true;

    for chunk in pairs.chunks_exact(2) {
        let z = chunk[0].as_f64()?;
        let v = extract_json_literal(&chunk[1]).and_then(|v| v.as_f64())?;
        if v == 0.0 {
            last_zero_stop = Some(z);
        } else {
            all_zero = false;
        }
    }

    if all_zero {
        return Some(f64::INFINITY);
    }
    last_zero_stop
}

/// `["step", ["zoom"], default, z1, v1, z2, v2, ...]`
#[allow(clippy::float_cmp)]
fn visibility_minzoom_step(arr: &[Value]) -> Option<f64> {
    if arr.len() < 4 {
        return None;
    }
    if !is_zoom_expr(&arr[1]) {
        return None;
    }

    // Default output (before first stop).
    let default_val = extract_json_literal(&arr[2]).and_then(|v| v.as_f64())?;
    let pairs = &arr[3..];
    if !pairs.len().is_multiple_of(2) {
        return None;
    }

    if default_val != 0.0 {
        // Default is non-zero → visible from the start.
        return None;
    }

    // Walk stops to find the first non-zero value.
    for chunk in pairs.chunks_exact(2) {
        let z = chunk[0].as_f64()?;
        let v = extract_json_literal(&chunk[1]).and_then(|v| v.as_f64())?;
        if v != 0.0 {
            return Some(z);
        }
    }

    // All stops are zero.
    Some(f64::INFINITY)
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

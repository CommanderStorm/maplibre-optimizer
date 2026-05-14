//! Zoom-bounded stop pruning for `step` and `interpolate` expressions.
//!
//! Runs after structural passes have tightened layer zoom bounds
//! (via `metadata_refinement`) and synced them to JSON.  Property values are
//! deserialized into typed expression enums so the pruning logic is
//! structurally safe rather than doing raw JSON array indexing.

use maplibre_style_spec::shared_expr::{ColorExpression, NumericExpression};
use maplibre_style_spec::spec::{
    Any, Color, ColorOrArrayOfColor, Number, NumberLiteral,
    NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection,
};
use serde_json::Value;

/// Walk all layers and prune out-of-range stops from zoom-driven
/// `step`/`interpolate` expressions.
pub(super) fn prune_zoom_stops(style: &mut Value) {
    let Some(layers) = style
        .as_object_mut()
        .and_then(|o| o.get_mut("layers"))
        .and_then(Value::as_array_mut)
    else {
        return;
    };

    for layer in layers.iter_mut() {
        let Some(obj) = layer.as_object_mut() else {
            continue;
        };
        let minzoom = obj.get("minzoom").and_then(Value::as_f64);
        let maxzoom = obj.get("maxzoom").and_then(Value::as_f64);

        if minzoom.is_none() && maxzoom.is_none() {
            continue;
        }

        for section in ["paint", "layout"] {
            let Some(props) = obj.get_mut(section).and_then(Value::as_object_mut) else {
                continue;
            };
            for value in props.values_mut() {
                try_prune_property(value, minzoom, maxzoom);
            }
        }
    }
}

/// Try to deserialize a property value as a typed expression, prune it,
/// and serialize back if changed.  After serialization, collapse trivial
/// ramps (zero stops for step, single/identical stops for interpolate)
/// to their bare output value.
fn try_prune_property(value: &mut Value, minzoom: Option<f64>, maxzoom: Option<f64>) {
    // Try numeric first (most common for zoom-driven ramps like opacity, width).
    if let Ok(mut expr) = serde_json::from_value::<NumericExpression>(value.clone())
        && prune_numeric(&mut expr, minzoom, maxzoom)
        && let Ok(mut v) = serde_json::to_value(&expr)
    {
        collapse_trivial_ramp(&mut v);
        *value = v;
        return;
    }
    // Then color (e.g. fill-color, line-color).
    if let Ok(mut expr) = serde_json::from_value::<ColorExpression>(value.clone())
        && prune_color(&mut expr, minzoom, maxzoom)
        && let Ok(mut v) = serde_json::to_value(&expr)
    {
        collapse_trivial_ramp(&mut v);
        *value = v;
    }
}

/// Collapse a serialized ramp expression to its bare value when all stops
/// have been pruned away or all remaining outputs are identical.
fn collapse_trivial_ramp(v: &mut Value) {
    let Value::Array(arr) = v else { return };
    let Some(op) = arr.first().and_then(Value::as_str) else {
        return;
    };
    match op {
        "step" => {
            // ["step", input, default] with no stops → default
            if arr.len() == 3 {
                *v = arr[2].clone();
                return;
            }
            // All stop outputs equal default → default
            let default = &arr[2];
            if arr.len() >= 5 && arr[4..].iter().step_by(2).all(|out| out == default) {
                *v = default.clone();
            }
        }
        "interpolate" | "interpolate-hcl" | "interpolate-lab" => {
            // ["interpolate", method, input, z, v] with 1 stop → v
            if arr.len() == 5 {
                *v = arr[4].clone();
                return;
            }
            // All stop outputs identical → that value
            if arr.len() >= 5 {
                let first = &arr[4];
                if arr[4..].iter().step_by(2).all(|out| out == first) {
                    *v = first.clone();
                }
            }
        }
        _ => {}
    }
}

// ── Typed dispatch ──────────────────────────────────────────────────────────

fn prune_numeric(expr: &mut NumericExpression, min: Option<f64>, max: Option<f64>) -> bool {
    match expr {
        NumericExpression::Number(num) => prune_number(num, min, max),
        NumericExpression::Ramp(ramp) => prune_wide_ramp(ramp, min, max),
    }
}

fn prune_color(expr: &mut ColorExpression, min: Option<f64>, max: Option<f64>) -> bool {
    match expr {
        ColorExpression::Color(c) => prune_color_operator(c, min, max),
        ColorExpression::Ramp(ramp) => prune_color_ramp(ramp, min, max),
        ColorExpression::Interpolate(ramp) => prune_wide_ramp(ramp, min, max),
    }
}

fn prune_number(num: &mut Number, min: Option<f64>, max: Option<f64>) -> bool {
    match num {
        Number::AnyExpr(any) => prune_any(any, min, max),
        _ => false,
    }
}

fn prune_color_operator(c: &mut Color, min: Option<f64>, max: Option<f64>) -> bool {
    match c {
        Color::AnyExpr(any) => prune_any(any, min, max),
        _ => false,
    }
}

/// Handle step expressions that deserialized through the `Any` escape hatch.
fn prune_any(any: &mut Any, min: Option<f64>, max: Option<f64>) -> bool {
    match any {
        Any::Step((input, default, stops)) => {
            if !matches!(**input, Number::Zoom) {
                return false;
            }
            prune_step_stops(default, stops, min, max)
        }
        _ => false,
    }
}

fn prune_wide_ramp(
    ramp: &mut NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection,
    min: Option<f64>,
    max: Option<f64>,
) -> bool {
    match ramp {
        NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection::Interpolate((
            _method,
            input,
            stops,
        )) => {
            if !matches!(input, Number::Zoom) {
                return false;
            }
            prune_interpolate_stops(stops, min, max)
        }
        NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection::Step((input, default, stops)) => {
            if !matches!(input, Number::Zoom) {
                return false;
            }
            prune_step_stops(default, stops, min, max)
        }
    }
}

fn prune_color_ramp(ramp: &mut ColorOrArrayOfColor, min: Option<f64>, max: Option<f64>) -> bool {
    match ramp {
        ColorOrArrayOfColor::InterpolateHcl((_method, input, stops))
        | ColorOrArrayOfColor::InterpolateLab((_method, input, stops)) => {
            if !matches!(input, Number::Zoom) {
                return false;
            }
            prune_interpolate_stops(stops, min, max)
        }
        ColorOrArrayOfColor::Step((input, default, stops)) => {
            if !matches!(input, Number::Zoom) {
                return false;
            }
            prune_step_stops(default, stops, min, max)
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Extract the f64 zoom level from a stop threshold.
fn stop_zoom(z: &NumberLiteral) -> Option<f64> {
    serde_json::to_value(z).ok().and_then(|v| v.as_f64())
}

// ── Generic stop pruning (works on any output type) ─────────────────────────

/// Prune stops from a step expression.
///
/// - Below minzoom: the last stop at or below M becomes the new default.
/// - Above maxzoom: stops above N are removed.
fn prune_step_stops<T: PartialEq + Clone>(
    default: &mut T,
    stops: &mut Vec<(NumberLiteral, T)>,
    minzoom: Option<f64>,
    maxzoom: Option<f64>,
) -> bool {
    let mut changed = false;

    // Above maxzoom: remove stops whose threshold exceeds maxzoom.
    if let Some(max) = maxzoom {
        let before = stops.len();
        stops.retain(|(z, _)| stop_zoom(z).is_none_or(|z| z <= max));
        changed |= stops.len() < before;
    }

    // Below minzoom: absorb stops at or below minzoom into default.
    if let Some(min) = minzoom {
        let last_absorbed = stops
            .iter()
            .rposition(|(z, _)| stop_zoom(z).is_some_and(|z| z <= min));
        if let Some(idx) = last_absorbed {
            *default = stops[idx].1.clone();
            stops.drain(..=idx);
            changed = true;
        }
    }

    changed
}

/// Prune stops from an interpolate expression.
///
/// Must keep boundary stops for correct interpolation at range edges.
/// - Below minzoom: keep the last stop at or below M, remove earlier ones.
/// - Above maxzoom: keep the first stop at or above N, remove later ones.
fn prune_interpolate_stops<T: PartialEq + Clone>(
    stops: &mut Vec<(NumberLiteral, T)>,
    minzoom: Option<f64>,
    maxzoom: Option<f64>,
) -> bool {
    if stops.len() < 2 {
        return false;
    }

    let mut changed = false;

    // Above maxzoom: keep the first stop >= max, remove everything after.
    if let Some(max) = maxzoom
        && let Some(boundary) = stops
            .iter()
            .position(|(z, _)| stop_zoom(z).is_some_and(|z| z >= max))
    {
        let keep = boundary + 1;
        if keep < stops.len() {
            stops.truncate(keep);
            changed = true;
        }
    }

    // Below minzoom: keep the last stop <= min, remove earlier ones.
    if let Some(min) = minzoom
        && let Some(keep_from) = stops
            .iter()
            .rposition(|(z, _)| stop_zoom(z).is_some_and(|z| z <= min))
        && keep_from > 0
    {
        stops.drain(..keep_from);
        changed = true;
    }

    changed
}

// ── Zoom comparison folding ──────────────────────────────────────────────────

/// Walk all layers and fold zoom comparisons to boolean literals when the
/// layer's minzoom makes them deterministic.
///
/// Only minzoom is safe — tiles overzoom beyond maxzoom, so maxzoom cannot
/// prove a comparison unreachable.
pub(super) fn fold_zoom_comparisons(style: &mut Value) {
    let Some(layers) = style
        .as_object_mut()
        .and_then(|o| o.get_mut("layers"))
        .and_then(Value::as_array_mut)
    else {
        return;
    };

    for layer in layers.iter_mut() {
        let Some(obj) = layer.as_object_mut() else {
            continue;
        };
        let Some(minzoom) = obj.get("minzoom").and_then(Value::as_f64) else {
            continue;
        };

        // Walk filter.
        if let Some(filter) = obj.get_mut("filter") {
            fold_zoom_in_expr(filter, minzoom);
        }

        // Walk paint and layout property values.
        for section in ["paint", "layout"] {
            let Some(props) = obj.get_mut(section).and_then(Value::as_object_mut) else {
                continue;
            };
            for value in props.values_mut() {
                fold_zoom_in_expr(value, minzoom);
            }
        }
    }
}

/// Recursively fold zoom comparisons within an expression.
fn fold_zoom_in_expr(v: &mut Value, minzoom: f64) {
    let Value::Array(arr) = v else { return };

    // Recurse into children first (bottom-up).
    for child in arr.iter_mut() {
        fold_zoom_in_expr(child, minzoom);
    }

    // Check if this node is a zoom comparison we can fold.
    if arr.len() != 3 {
        return;
    }
    let Some(op) = arr[0].as_str() else { return };
    if !matches!(op, ">=" | ">" | "<=" | "<" | "==" | "!=") {
        return;
    }

    let (zoom_first, other) = if super::zoom::is_zoom_expr(&arr[1]) {
        (true, &arr[2])
    } else if super::zoom::is_zoom_expr(&arr[2]) {
        (false, &arr[1])
    } else {
        return;
    };

    let Some(lit) = super::expr::extract_json_literal(other) else {
        return;
    };
    let Some(n) = lit.as_f64() else { return };

    // Normalize to zoom-first form: op' such that `zoom op' n`.
    let effective_op = if zoom_first {
        op
    } else {
        match op {
            ">=" => "<=",
            ">" => "<",
            "<=" => ">=",
            "<" => ">",
            _ => op, // == and != are symmetric
        }
    };

    // Apply truth table: given zoom >= minzoom always holds.
    let folded = match effective_op {
        ">=" => {
            if minzoom >= n {
                Some(true)
            } else {
                None
            }
        }
        ">" => {
            if minzoom > n {
                Some(true)
            } else {
                None
            }
        }
        "<=" => {
            if minzoom > n {
                Some(false)
            } else {
                None
            }
        }
        "<" => {
            if minzoom >= n {
                Some(false)
            } else {
                None
            }
        }
        "==" => {
            if n < minzoom {
                Some(false)
            } else {
                None
            }
        }
        "!=" => {
            if n < minzoom {
                Some(true)
            } else {
                None
            }
        }
        _ => None,
    };

    if let Some(result) = folded {
        *v = Value::Array(vec![
            Value::String("literal".to_string()),
            Value::Bool(result),
        ]);
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn make_style(minzoom: Option<f64>, maxzoom: Option<f64>, prop: Value) -> Value {
        let fill_opacity = prop;
        let mut layer = json!({
            "id": "test",
            "type": "fill",
            "source": "src",
            "paint": { "fill-opacity": fill_opacity }
        });
        if let Some(min) = minzoom {
            layer["minzoom"] = json!(min);
        }
        if let Some(max) = maxzoom {
            layer["maxzoom"] = json!(max);
        }
        json!({ "layers": [layer] })
    }

    fn get_prop(style: &Value) -> &Value {
        &style["layers"][0]["paint"]["fill-opacity"]
    }

    // ── Step pruning ──────────────────────────────────────────────────────

    #[test]
    fn step_prune_below_minzoom() {
        let mut style = make_style(
            Some(10.0),
            None,
            json!(["step", ["zoom"], 0, 5, 1, 8, 2, 12, 3]),
        );
        prune_zoom_stops(&mut style);
        assert_eq!(get_prop(&style), &json!(["step", ["zoom"], 2, 12, 3]));
    }

    #[test]
    fn step_prune_above_maxzoom() {
        let mut style = make_style(
            None,
            Some(16.0),
            json!(["step", ["zoom"], 0, 10, 1, 14, 2, 18, 3]),
        );
        prune_zoom_stops(&mut style);
        assert_eq!(
            get_prop(&style),
            &json!(["step", ["zoom"], 0, 10, 1, 14, 2])
        );
    }

    #[test]
    fn step_prune_both_collapses() {
        let mut style = make_style(
            Some(12.0),
            Some(14.0),
            json!(["step", ["zoom"], 0, 5, 1, 10, 2, 18, 3]),
        );
        prune_zoom_stops(&mut style);
        // After pruning: default=2, no stops → serializes as bare 2.
        assert_eq!(get_prop(&style), &json!(2));
    }

    #[test]
    fn step_no_prune_data_driven() {
        let mut style = make_style(
            Some(10.0),
            Some(16.0),
            json!(["step", ["get", "pop"], 0, 100, 1, 500, 2]),
        );
        let original = style.clone();
        prune_zoom_stops(&mut style);
        assert_eq!(style, original);
    }

    // ── Interpolate pruning ───────────────────────────────────────────────

    #[test]
    fn interpolate_prune_below_minzoom() {
        let mut style = make_style(
            Some(10.0),
            None,
            json!(["interpolate", ["linear"], ["zoom"], 2, 0, 5, 1, 8, 2, 14, 3]),
        );
        prune_zoom_stops(&mut style);
        assert_eq!(
            get_prop(&style),
            &json!(["interpolate", ["linear"], ["zoom"], 8, 2, 14, 3])
        );
    }

    #[test]
    fn interpolate_prune_above_maxzoom() {
        let mut style = make_style(
            None,
            Some(12.0),
            json!([
                "interpolate",
                ["linear"],
                ["zoom"],
                5,
                1,
                10,
                2,
                14,
                3,
                20,
                4
            ]),
        );
        prune_zoom_stops(&mut style);
        assert_eq!(
            get_prop(&style),
            &json!(["interpolate", ["linear"], ["zoom"], 5, 1, 10, 2, 14, 3])
        );
    }

    #[test]
    fn interpolate_single_stop_collapses() {
        let mut style = make_style(
            Some(10.0),
            Some(11.0),
            json!([
                "interpolate",
                ["linear"],
                ["zoom"],
                2,
                0,
                10,
                5,
                11,
                5,
                20,
                10
            ]),
        );
        prune_zoom_stops(&mut style);
        // After pruning: stops [z10→5, z11→5] → identical outputs → bare 5.
        assert_eq!(get_prop(&style), &json!(5));
    }

    #[test]
    fn interpolate_no_prune_data_driven() {
        let mut style = make_style(
            Some(10.0),
            Some(16.0),
            json!(["interpolate", ["linear"], ["get", "pop"], 0, 0, 100, 1]),
        );
        let original = style.clone();
        prune_zoom_stops(&mut style);
        assert_eq!(style, original);
    }

    #[test]
    fn no_zoom_bounds_no_pruning() {
        let mut style = make_style(None, None, json!(["step", ["zoom"], 0, 5, 1, 10, 2]));
        let original = style.clone();
        prune_zoom_stops(&mut style);
        assert_eq!(style, original);
    }
}

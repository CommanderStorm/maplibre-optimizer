//! Stats-driven constant folding passes: fold expressions using tile statistics.

use serde_json::Value;

use super::util::{extract_json_literal, replace_arr_with_value};

/// Fold `["has", p]` → `["literal", true]` when statistics show the property is present in
/// every feature, or → `["literal", false]` when the property is never present.
pub(super) fn try_fold_has_from_stats(
    arr: &mut Vec<Value>,
    stats: Option<&crate::stats::TileStatistics>,
    layer_info: Option<&[Option<crate::optimize::source_util::VectorLayerInfo>]>,
    layer_index: usize,
) -> bool {
    if arr.len() != 2 || arr[0].as_str() != Some("has") {
        return false;
    }
    let Some(prop_name) = arr[1].as_str() else {
        return false;
    };
    let Some(stats) = stats else {
        return false;
    };
    let Some(infos) = layer_info else {
        return false;
    };
    let Some(Some(info)) = infos.get(layer_index) else {
        return false;
    };
    let Some(layer_stats) = stats.layer_stats(&info.source, &info.source_layer) else {
        return false;
    };
    if layer_stats.total_features == 0 {
        return false;
    }
    let Some(prop_stats) = layer_stats.properties.get(prop_name) else {
        // Property not in stats at all — never observed on any feature.
        *arr = vec![Value::String("literal".to_string()), Value::Bool(false)];
        return true;
    };
    if prop_stats.present_count() == 0 {
        // Property exists in stats but was never present on any feature.
        *arr = vec![Value::String("literal".to_string()), Value::Bool(false)];
        return true;
    }
    if prop_stats.present_count() == layer_stats.total_features {
        *arr = vec![Value::String("literal".to_string()), Value::Bool(true)];
        return true;
    }
    false
}

/// Fold `["==", ["geometry-type"], "Point"]` → `true`/`false` (and `!=` variant) based on
/// geometry type statistics.
///
/// - `==`: fold to `true` if the queried type is the only non-zero type; fold to `false` if
///   its count is 0.
/// - `!=`: inverse of the above.
pub(super) fn try_fold_geometry_type_from_stats(
    arr: &mut Vec<Value>,
    stats: Option<&crate::stats::TileStatistics>,
    layer_info: Option<&[Option<crate::optimize::source_util::VectorLayerInfo>]>,
    layer_index: usize,
) -> bool {
    if arr.len() != 3 {
        return false;
    }
    let Some(op) = arr[0].as_str() else {
        return false;
    };
    if op != "==" && op != "!=" {
        return false;
    }

    // Detect pattern: one operand is ["geometry-type"], the other is a string literal.
    let (geom_type_str, _geom_idx) = if is_geometry_type_expr(&arr[1]) && arr[2].is_string() {
        (arr[2].as_str().unwrap(), 1)
    } else if is_geometry_type_expr(&arr[2]) && arr[1].is_string() {
        (arr[1].as_str().unwrap(), 2)
    } else {
        return false;
    };

    // Only handle the three standard geometry type strings.
    if !matches!(geom_type_str, "Point" | "LineString" | "Polygon") {
        return false;
    }

    let Some(stats) = stats else {
        return false;
    };
    let Some(infos) = layer_info else {
        return false;
    };
    let Some(Some(info)) = infos.get(layer_index) else {
        return false;
    };
    let Some(layer_stats) = stats.layer_stats(&info.source, &info.source_layer) else {
        return false;
    };
    if layer_stats.total_features == 0 {
        return false;
    }

    let gt = &layer_stats.geometry_types;
    let queried_count = match geom_type_str {
        "Point" => gt.point,
        "LineString" => gt.linestring,
        "Polygon" => gt.polygon,
        _ => unreachable!(),
    };

    // Count how many geometry types are present (excluding unknown).
    let non_zero_types =
        u8::from(gt.point > 0) + u8::from(gt.linestring > 0) + u8::from(gt.polygon > 0);

    let fold_value = if queried_count == 0 {
        // This geometry type never appears → == is false, != is true.
        Some(op == "!=")
    } else if non_zero_types == 1 && queried_count > 0 {
        // This is the only geometry type → == is true, != is false.
        Some(op == "==")
    } else {
        None
    };

    if let Some(val) = fold_value {
        *arr = vec![Value::String("literal".to_string()), Value::Bool(val)];
        return true;
    }
    false
}

/// Check if a value is `["geometry-type"]` (a 1-element array).
fn is_geometry_type_expr(v: &Value) -> bool {
    if let Value::Array(a) = v {
        a.len() == 1 && a[0].as_str() == Some("geometry-type")
    } else {
        false
    }
}

/// Fold `["get", p]` → `["literal", v]` when statistics show the property has exactly one
/// value across all features (cardinality == 1, present on every feature).
pub(super) fn try_fold_get_from_stats(
    arr: &mut Vec<Value>,
    stats: Option<&crate::stats::TileStatistics>,
    layer_info: Option<&[Option<crate::optimize::source_util::VectorLayerInfo>]>,
    layer_index: usize,
) -> bool {
    if arr.len() != 2 || arr[0].as_str() != Some("get") {
        return false;
    }
    let Some(prop_name) = arr[1].as_str() else {
        return false;
    };
    let Some(stats) = stats else {
        return false;
    };
    let Some(infos) = layer_info else {
        return false;
    };
    let Some(Some(info)) = infos.get(layer_index) else {
        return false;
    };
    let Some(layer_stats) = stats.layer_stats(&info.source, &info.source_layer) else {
        return false;
    };
    if layer_stats.total_features == 0 {
        return false;
    }
    let Some(prop_stats) = layer_stats.properties.get(prop_name) else {
        return false;
    };
    // Must be present on every feature and have exactly one distinct value.
    if prop_stats.present_count() != layer_stats.total_features {
        return false;
    }
    let Some(literal) = single_value_literal(prop_stats) else {
        return false;
    };
    replace_arr_with_value(arr, literal);
    true
}

/// Extract the sole value from a single-cardinality property as a JSON literal.
fn single_value_literal(stats: &crate::stats::PropertyStats) -> Option<Value> {
    use crate::stats::PropertyStats;
    match stats {
        PropertyStats::Bool {
            present_count,
            true_count,
        } => {
            // Bool always has cardinality ≤ 2. Single-value when all true or all false.
            if *true_count == *present_count {
                Some(Value::Bool(true))
            } else if *true_count == 0 {
                Some(Value::Bool(false))
            } else {
                None
            }
        }
        PropertyStats::Integer {
            cardinality,
            value_counts,
            ..
        } => {
            if *cardinality != 1 {
                return None;
            }
            let counts = value_counts.as_ref()?;
            let (&val, _) = counts.iter().next()?;
            Some(Value::Number(val.into()))
        }
        PropertyStats::UnsignedInteger {
            cardinality,
            value_counts,
            ..
        } => {
            if *cardinality != 1 {
                return None;
            }
            let counts = value_counts.as_ref()?;
            let (&val, _) = counts.iter().next()?;
            Some(Value::Number(val.into()))
        }
        PropertyStats::Double {
            cardinality,
            min,
            max,
            ..
        } => {
            // Doubles don't have value_counts, but if cardinality==1 then min==max.
            if *cardinality != 1 {
                return None;
            }
            serde_json::Number::from_f64(*min)
                .filter(|_| (min - max).abs() < f64::EPSILON)
                .map(Value::Number)
        }
        PropertyStats::String {
            cardinality,
            value_counts,
            ..
        } => {
            if *cardinality != 1 {
                return None;
            }
            let counts = value_counts.as_ref()?;
            let (val, _) = counts.iter().next()?;
            Some(Value::String(val.clone()))
        }
        PropertyStats::Mixed { .. } => None,
    }
}

/// Fold comparisons (`<`, `<=`, `>`, `>=`, `==`, `!=`) to `true`/`false` when tile statistics
/// prove the result is constant across all features.
pub(super) fn try_fold_comparison_from_stats(
    arr: &mut Vec<Value>,
    stats: Option<&crate::stats::TileStatistics>,
    layer_info: Option<&[Option<crate::optimize::source_util::VectorLayerInfo>]>,
    layer_index: usize,
) -> bool {
    if let Some(result) = try_fold_comparison_inner(arr, stats, layer_info, layer_index) {
        *arr = vec![Value::String("literal".to_string()), Value::Bool(result)];
        return true;
    }
    false
}

fn try_fold_comparison_inner(
    arr: &[Value],
    stats: Option<&crate::stats::TileStatistics>,
    layer_info: Option<&[Option<crate::optimize::source_util::VectorLayerInfo>]>,
    layer_index: usize,
) -> Option<bool> {
    use super::util::{get_prop_name, is_get_expr, json_as_i64, json_as_u64};
    use crate::stats::PropertyStats;

    if arr.len() != 3 {
        return None;
    }
    let Some(op @ ("<" | "<=" | ">" | ">=" | "==" | "!=")) = arr[0].as_str() else {
        return None;
    };

    // Extract ["get", prop] from one operand and literal from the other.
    // Normalize so we always have: op(get(prop), n).
    // If the get is on the right, flip the operator.
    let (prop, lit, effective_op) = if is_get_expr(&arr[1]) {
        let prop = get_prop_name(&arr[1])?;
        let lit = extract_json_literal(&arr[2])?;
        (prop, lit, op)
    } else if is_get_expr(&arr[2]) {
        let prop = get_prop_name(&arr[2])?;
        let lit = extract_json_literal(&arr[1])?;
        // Flip: ["<", n, ["get", p]] ≡ [">", ["get", p], n]
        let flipped = match op {
            "<" => ">",
            "<=" => ">=",
            ">" => "<",
            ">=" => "<=",
            other => other, // == and != are symmetric
        };
        (prop, lit, flipped)
    } else {
        return None;
    };

    let infos = layer_info?;
    let info = infos.get(layer_index)?.as_ref()?;
    let layer_stats = stats?.layer_stats(&info.source, &info.source_layer)?;
    if layer_stats.total_features == 0 {
        return None;
    }
    let prop_stats = layer_stats.properties.get(prop)?;
    let all_present = prop_stats.present_count() == layer_stats.total_features;

    match prop_stats {
        PropertyStats::Integer {
            min,
            max,
            value_counts,
            ..
        } => {
            let n = json_as_i64(&lit)?;
            fold_comparison_numeric(effective_op, &n, min, max, all_present, || {
                value_counts.as_ref().map(|vc| vc.contains_key(&n))
            })
        }
        PropertyStats::UnsignedInteger {
            min,
            max,
            value_counts,
            ..
        } => {
            let n = json_as_u64(&lit)?;
            fold_comparison_numeric(effective_op, &n, min, max, all_present, || {
                value_counts.as_ref().map(|vc| vc.contains_key(&n))
            })
        }
        PropertyStats::Double { min, max, .. } => {
            let n = lit.as_f64()?;
            fold_comparison_numeric(effective_op, &n, min, max, all_present, || None)
        }
        PropertyStats::Bool {
            present_count,
            true_count,
        } => {
            let lit_bool = lit.as_bool()?;
            fold_comparison_bool(
                effective_op,
                lit_bool,
                *true_count,
                *present_count,
                layer_stats.total_features,
            )
        }
        _ => None,
    }
}

/// Prune dead values from `["in", ["get", prop], ["literal", [v1, v2, ...]]]` using stats.
///
/// Values not present in the property's `value_counts` are removed. Empty array → `false`.
/// Single element → `["==", ["get", prop], v]`.
/// Guard: only when `sample_rate == 1.0`.
pub(super) fn try_prune_in_from_stats(
    arr: &mut Vec<Value>,
    stats: Option<&crate::stats::TileStatistics>,
    layer_info: Option<&[Option<crate::optimize::source_util::VectorLayerInfo>]>,
    layer_index: usize,
) -> bool {
    use super::util::get_prop_name;

    if arr.len() != 3 || arr[0].as_str() != Some("in") {
        return false;
    }
    let Some(prop) = get_prop_name(&arr[1]) else {
        return false;
    };
    // The third arg must be ["literal", [...]].
    let Value::Array(lit_wrapper) = &arr[2] else {
        return false;
    };
    if lit_wrapper.len() != 2 || lit_wrapper[0].as_str() != Some("literal") {
        return false;
    }
    let Value::Array(values) = &lit_wrapper[1] else {
        return false;
    };

    let Some(layer_stats) = super::util::resolve_layer_stats(stats, layer_info, layer_index)
    else {
        return false;
    };

    // If prop is unknown, all values are dead.
    let prop_stats = layer_stats.properties.get(prop);

    let kept: Vec<Value> = values
        .iter()
        .filter(|v| value_exists_in_stats(v, prop_stats))
        .cloned()
        .collect();

    // Check if all values are covered: every value in value_counts appears in
    // the `in` list AND the property is present on 100% of features → fold to true.
    // Use `kept` (post-prune) since those are exactly the values that exist in data.
    if let Some(prop_stats) = prop_stats
        && prop_stats.present_count() == layer_stats.total_features
        && all_values_covered_by_in(prop_stats, &kept)
    {
        *arr = vec![Value::String("literal".to_string()), Value::Bool(true)];
        return true;
    }

    if kept.len() == values.len() {
        return false;
    }

    if kept.is_empty() {
        *arr = vec![Value::String("literal".to_string()), Value::Bool(false)];
        return true;
    }
    if kept.len() == 1 {
        let get_expr = arr[1].clone();
        *arr = vec![
            Value::String("==".to_string()),
            get_expr,
            kept.into_iter().next().unwrap(),
        ];
        return true;
    }
    // Rebuild with pruned values.
    arr[2] = Value::Array(vec![
        Value::String("literal".to_string()),
        Value::Array(kept),
    ]);
    true
}

/// Check whether a JSON value exists in the property's `value_counts`.
fn value_exists_in_stats(v: &Value, prop_stats: Option<&crate::stats::PropertyStats>) -> bool {
    use super::util::{json_as_i64, json_as_u64};
    use crate::stats::PropertyStats;

    let Some(ps) = prop_stats else {
        return false;
    };
    match ps {
        PropertyStats::String {
            value_counts: Some(vc),
            ..
        } => v.as_str().is_some_and(|s| vc.contains_key(s)),
        PropertyStats::Integer {
            value_counts: Some(vc),
            ..
        } => json_as_i64(v).is_some_and(|n| vc.contains_key(&n)),
        PropertyStats::UnsignedInteger {
            value_counts: Some(vc),
            ..
        } => json_as_u64(v).is_some_and(|n| vc.contains_key(&n)),
        // No value_counts available — conservatively keep.
        _ => true,
    }
}

/// Check whether every value in the property's `value_counts` appears in the `in` list.
/// Returns `false` conservatively if `value_counts` is unavailable.
fn all_values_covered_by_in(prop_stats: &crate::stats::PropertyStats, in_values: &[Value]) -> bool {
    use super::util::{json_as_i64, json_as_u64};
    use crate::stats::PropertyStats;

    match prop_stats {
        PropertyStats::String {
            value_counts: Some(vc),
            ..
        } => vc
            .keys()
            .all(|k| in_values.iter().any(|v| v.as_str() == Some(k))),
        PropertyStats::Integer {
            value_counts: Some(vc),
            ..
        } => vc
            .keys()
            .all(|k| in_values.iter().any(|v| json_as_i64(v) == Some(*k))),
        PropertyStats::UnsignedInteger {
            value_counts: Some(vc),
            ..
        } => vc
            .keys()
            .all(|k| in_values.iter().any(|v| json_as_u64(v) == Some(*k))),
        // No value_counts available — conservatively don't fold.
        _ => false,
    }
}

/// Prune dead arms from `["match", ["get", prop], label, out, ..., fallback]` using stats.
///
/// Arms whose labels don't exist in `value_counts` are removed. All arms pruned → fallback.
/// Guard: only when `sample_rate == 1.0`.
pub(super) fn try_prune_match_from_stats(
    arr: &mut Vec<Value>,
    stats: Option<&crate::stats::TileStatistics>,
    layer_info: Option<&[Option<crate::optimize::source_util::VectorLayerInfo>]>,
    layer_index: usize,
) -> bool {
    use super::util::get_prop_name;

    if arr.first().and_then(Value::as_str) != Some("match") {
        return false;
    }
    // ["match", input, label1, out1, ..., fallback] — min 5 elements, odd length.
    if arr.len() < 5 || arr.len().is_multiple_of(2) {
        return false;
    }
    let Some(prop) = get_prop_name(&arr[1]) else {
        return false;
    };

    let Some(layer_stats) = super::util::resolve_layer_stats(stats, layer_info, layer_index)
    else {
        return false;
    };
    let prop_stats = layer_stats.properties.get(prop);

    let arm_count = (arr.len() - 3) / 2;
    let mut arms_to_remove: Vec<usize> = Vec::new();

    for i in 0..arm_count {
        let label_idx = 2 + i * 2;
        let label = &arr[label_idx];

        let keep = match label {
            Value::Array(labels) => labels.iter().any(|v| value_exists_in_stats(v, prop_stats)),
            single => value_exists_in_stats(single, prop_stats),
        };

        if !keep {
            arms_to_remove.push(i);
        } else if let Value::Array(labels) = label {
            // Filter individual values within array labels.
            let kept: Vec<Value> = labels
                .iter()
                .filter(|v| value_exists_in_stats(v, prop_stats))
                .cloned()
                .collect();
            if kept.len() < labels.len() {
                // Will handle via mutation below after we know we're changing something.
                arms_to_remove.push(usize::MAX); // sentinel — partial prune handled separately
            }
        }
    }

    // Check for partial label pruning (array labels with some dead values).
    let mut changed = false;
    for i in 0..arm_count {
        let label_idx = 2 + i * 2;
        if let Value::Array(labels) = &arr[label_idx] {
            let kept: Vec<Value> = labels
                .iter()
                .filter(|v| value_exists_in_stats(v, prop_stats))
                .cloned()
                .collect();
            if kept.len() < labels.len() && !kept.is_empty() {
                if kept.len() == 1 {
                    arr[label_idx] = kept.into_iter().next().unwrap();
                } else {
                    arr[label_idx] = Value::Array(kept);
                }
                changed = true;
            }
        }
    }

    // Remove fully dead arms (in reverse to preserve indices).
    let dead_arms: Vec<usize> = arms_to_remove
        .into_iter()
        .filter(|&i| i != usize::MAX)
        .collect();
    if dead_arms.is_empty() && !changed {
        return false;
    }
    for &i in dead_arms.iter().rev() {
        let label_idx = 2 + i * 2;
        // Remove output first (higher index), then label.
        arr.remove(label_idx + 1);
        arr.remove(label_idx);
    }

    // All arms removed → replace with fallback.
    if arr.len() == 3 {
        let fallback = arr[2].clone();
        replace_arr_with_value(arr, fallback);
        return true;
    }

    !dead_arms.is_empty() || changed
}

/// Remove dead arms from `["coalesce", arm1, arm2, ...]` when stats prove a `["get", prop]`
/// arm is always non-null (present on all features), making subsequent arms unreachable.
///
/// Guard: only when `sample_rate == 1.0`.
pub(super) fn try_fold_coalesce_from_stats(
    arr: &mut Vec<Value>,
    stats: Option<&crate::stats::TileStatistics>,
    layer_info: Option<&[Option<crate::optimize::source_util::VectorLayerInfo>]>,
    layer_index: usize,
) -> bool {
    use super::util::get_prop_name;

    if arr.first().and_then(Value::as_str) != Some("coalesce") {
        return false;
    }
    if arr.len() < 3 {
        return false;
    }

    let Some(layer_stats) = super::util::resolve_layer_stats(stats, layer_info, layer_index)
    else {
        return false;
    };

    // First pass: remove ["get", prop] arms where prop has present_count == 0
    // (always null, so coalesce skips them). Skip index 0 ("coalesce" operator).
    let before_len = arr.len();
    let mut first = true;
    arr.retain(|v| {
        if first {
            first = false;
            return true; // keep "coalesce" operator
        }
        if let Some(prop) = get_prop_name(v) {
            let never_present = layer_stats
                .properties
                .get(prop)
                .is_none_or(|ps| ps.present_count() == 0);
            return !never_present;
        }
        true // keep non-get arms (literals, other expressions)
    });
    let changed = arr.len() < before_len;
    // After removing never-present arms, unwrap or collapse.
    if changed {
        if arr.len() == 1 {
            // All arms removed — coalesce with no arms evaluates to null.
            *arr = vec![Value::String("literal".to_string()), Value::Null];
            return true;
        }
        if arr.len() == 2 {
            let inner = arr[1].clone();
            replace_arr_with_value(arr, inner);
            return true;
        }
    }

    // Second pass: find the first ["get", prop] arm where prop is present on
    // all features, and truncate everything after it.
    for i in 1..arr.len() {
        let Some(prop) = get_prop_name(&arr[i]) else {
            continue;
        };
        let Some(prop_stats) = layer_stats.properties.get(prop) else {
            continue;
        };
        if prop_stats.present_count() == layer_stats.total_features && i + 1 < arr.len() {
            // Truncate everything after this arm.
            arr.truncate(i + 1);
            // Unwrap single-arm coalesce.
            if arr.len() == 2 {
                let inner = arr[1].clone();
                replace_arr_with_value(arr, inner);
            }
            return true;
        }
    }
    changed
}

/// Prune unreachable stops from property-driven `step` and `interpolate` expressions
/// using min/max from `PropertyStats`.
///
/// **Step above max**: Remove stops with threshold > max(prop) — unreachable by any value.
/// Null input returns the default, unaffected by high stops.
///
/// **Step below min**: Absorb stops with threshold ≤ min(prop) into default — only safe
/// if property is 100% present (otherwise null values still need the original default).
///
/// **Interpolate above max**: Keep the first boundary stop ≥ max, remove later ones.
///
/// **Interpolate below min**: Keep the last boundary stop ≤ min, remove earlier ones.
///
/// Guard: `sample_rate == 1.0`.
pub(super) fn try_prune_data_ramp_from_stats(
    arr: &mut Vec<Value>,
    stats: Option<&crate::stats::TileStatistics>,
    layer_info: Option<&[Option<crate::optimize::source_util::VectorLayerInfo>]>,
    layer_index: usize,
) -> bool {
    use super::util::get_prop_name;
    use crate::stats::PropertyStats;

    let Some(op) = arr.first().and_then(Value::as_str) else {
        return false;
    };
    let is_step = op == "step";
    let is_interpolate = matches!(op, "interpolate" | "interpolate-hcl" | "interpolate-lab");
    if !is_step && !is_interpolate {
        return false;
    }

    // Step: ["step", input, default, t1, v1, t2, v2, ...]
    // Interpolate: ["interpolate", method, input, t1, v1, t2, v2, ...]
    let (input_idx, first_stop_idx) = if is_step { (1, 3) } else { (2, 3) };

    if arr.len() <= first_stop_idx {
        return false;
    }

    // Input must be ["get", prop].
    let Some(prop) = get_prop_name(&arr[input_idx]) else {
        return false;
    };

    let Some(layer_stats) = super::util::resolve_layer_stats(stats, layer_info, layer_index)
    else {
        return false;
    };
    let Some(prop_stats) = layer_stats.properties.get(prop) else {
        return false;
    };

    // Extract min/max as f64. Precision loss is acceptable: these are threshold
    // comparisons, not exact equality checks. Bail on non-finite values (corrupted stats).
    #[expect(
        clippy::cast_precision_loss,
        reason = "threshold comparisons tolerate rounding"
    )]
    let (min_val, max_val) = match prop_stats {
        PropertyStats::Integer { min, max, .. } => (*min as f64, *max as f64),
        PropertyStats::UnsignedInteger { min, max, .. } => (*min as f64, *max as f64),
        PropertyStats::Double { min, max, .. } if min.is_finite() && max.is_finite() => {
            (*min, *max)
        }
        _ => return false,
    };

    let all_present = prop_stats.present_count() == layer_stats.total_features;

    if is_step {
        prune_data_step(arr, first_stop_idx, min_val, max_val, all_present)
    } else {
        prune_data_interpolate(arr, first_stop_idx, min_val, max_val)
    }
}

/// Prune stops from a property-driven step expression.
fn prune_data_step(
    arr: &mut Vec<Value>,
    first_stop_idx: usize,
    min_val: f64,
    max_val: f64,
    all_present: bool,
) -> bool {
    let mut changed = false;
    let default_idx = first_stop_idx - 1; // index 2 for step

    // Above max: remove stops with threshold > max.
    // Stops are (threshold, value) pairs starting at first_stop_idx.
    let mut i = first_stop_idx;
    while i + 1 < arr.len() {
        if arr[i].as_f64().is_some_and(|t| t > max_val) {
            // Remove this stop and all after it.
            arr.truncate(i);
            changed = true;
            break;
        }
        i += 2;
    }

    // Below min (only safe if property is 100% present): absorb stops with
    // threshold ≤ min into default.
    if all_present {
        let mut last_absorbed = None;
        let mut i = first_stop_idx;
        while i + 1 < arr.len() {
            if arr[i].as_f64().is_some_and(|t| t <= min_val) {
                last_absorbed = Some(i);
            } else {
                break;
            }
            i += 2;
        }
        if let Some(last_idx) = last_absorbed {
            // The value of the last absorbed stop becomes the new default.
            arr[default_idx] = arr[last_idx + 1].clone();
            // Remove all absorbed stops.
            arr.drain(first_stop_idx..=last_idx + 1);
            changed = true;
        }
    }

    // Collapse trivial: if no stops remain, replace with default.
    if arr.len() == first_stop_idx {
        let val = arr[default_idx].clone();
        replace_arr_with_value(arr, val);
        return true;
    }

    // Collapse trivial: all stop outputs equal default.
    if arr.len() > first_stop_idx {
        let default = &arr[default_idx];
        if arr[first_stop_idx + 1..]
            .iter()
            .step_by(2)
            .all(|v| v == default)
        {
            let val = default.clone();
            replace_arr_with_value(arr, val);
            return true;
        }
    }

    changed
}

/// Prune stops from a property-driven interpolate expression.
fn prune_data_interpolate(
    arr: &mut Vec<Value>,
    first_stop_idx: usize,
    min_val: f64,
    max_val: f64,
) -> bool {
    // Stops are (threshold, value) pairs starting at first_stop_idx.
    let stop_count = (arr.len() - first_stop_idx) / 2;
    if stop_count < 2 {
        return false;
    }

    let mut changed = false;

    // Above max: keep the first stop with threshold ≥ max, remove everything after.
    let mut boundary_above = None;
    let mut i = first_stop_idx;
    while i + 1 < arr.len() {
        if arr[i].as_f64().is_some_and(|t| t >= max_val) {
            boundary_above = Some(i);
            break;
        }
        i += 2;
    }
    if let Some(keep_through) = boundary_above {
        let keep_len = keep_through + 2; // include value after threshold
        if keep_len < arr.len() {
            arr.truncate(keep_len);
            changed = true;
        }
    }

    // Below min: keep the last stop with threshold ≤ min, remove earlier ones.
    let mut keep_from = None;
    let mut i = first_stop_idx;
    while i + 1 < arr.len() {
        if arr[i].as_f64().is_some_and(|t| t <= min_val) {
            keep_from = Some(i);
        } else {
            break;
        }
        i += 2;
    }
    if let Some(keep_idx) = keep_from
        && keep_idx > first_stop_idx
    {
        arr.drain(first_stop_idx..keep_idx);
        changed = true;
    }

    // Collapse trivial: single stop → bare value.
    let remaining_stops = (arr.len() - first_stop_idx) / 2;
    if remaining_stops == 1 {
        let val = arr[first_stop_idx + 1].clone();
        replace_arr_with_value(arr, val);
        return true;
    }

    // Collapse trivial: all stop outputs identical → bare value.
    if remaining_stops >= 2 {
        let first = &arr[first_stop_idx + 1];
        if arr[first_stop_idx + 1..]
            .iter()
            .step_by(2)
            .all(|v| v == first)
        {
            let val = first.clone();
            replace_arr_with_value(arr, val);
            return true;
        }
    }

    changed
}

/// Fold `==`/`!=` comparisons against a boolean property.
fn fold_comparison_bool(
    op: &str,
    lit_bool: bool,
    true_count: u64,
    present_count: u64,
    total: u64,
) -> Option<bool> {
    let all_true = true_count == present_count && present_count == total;
    let all_false = true_count == 0 && present_count == total;
    // ("==", true) and ("!=", false) have the same logic, as do ("==", false) and ("!=", true).
    let checking_true = (op == "==" && lit_bool) || (op == "!=" && !lit_bool);
    if checking_true {
        if true_count == 0 {
            Some(false)
        } else if all_true {
            Some(true)
        } else {
            None
        }
    } else if op == "==" || op == "!=" {
        if all_true {
            Some(false)
        } else if all_false {
            Some(true)
        } else {
            None
        }
    } else {
        None
    }
}

/// Generic comparison folding using min/max bounds.
///
/// `value_in_counts` is called only for `==`/`!=` to check if `n` exists in `value_counts`.
/// Returns `Some(bool)` if the comparison can be folded, `None` otherwise.
fn fold_comparison_numeric<T: PartialOrd, F: FnOnce() -> Option<bool>>(
    op: &str,
    n: &T,
    min: &T,
    max: &T,
    all_present: bool,
    value_in_counts: F,
) -> Option<bool> {
    match op {
        "<" => {
            if min >= n {
                Some(false)
            } else if max < n && all_present {
                Some(true)
            } else {
                None
            }
        }
        "<=" => {
            if min > n {
                Some(false)
            } else if max <= n && all_present {
                Some(true)
            } else {
                None
            }
        }
        ">" => {
            if max <= n {
                Some(false)
            } else if min > n && all_present {
                Some(true)
            } else {
                None
            }
        }
        ">=" => {
            if max < n {
                Some(false)
            } else if min >= n && all_present {
                Some(true)
            } else {
                None
            }
        }
        "==" => {
            if n < min || n > max {
                Some(false)
            } else if let Some(false) = value_in_counts() {
                Some(false)
            } else {
                None
            }
        }
        "!=" => {
            if n < min || n > max {
                if all_present { Some(true) } else { None }
            } else if let Some(false) = value_in_counts() {
                if all_present { Some(true) } else { None }
            } else {
                None
            }
        }
        _ => None,
    }
}



#[cfg(test)]
mod tests {
    use insta::assert_yaml_snapshot;
    use serde_json::{Value, json};

    use super::{
        try_fold_coalesce_from_stats, try_fold_comparison_from_stats,
        try_fold_geometry_type_from_stats, try_fold_has_from_stats,
        try_prune_data_ramp_from_stats, try_prune_in_from_stats, try_prune_match_from_stats,
    };

    // ── Stats-driven fold helpers ────────────────────────────────────────

    use std::collections::BTreeMap;

    use crate::optimize::source_util::VectorLayerInfo;
    use crate::stats::{GeometryTypeStats, LayerStats, PropertyStats, SourceStats, TileStatistics};

    fn make_stats(layer_name: &str, layer_stats: LayerStats) -> TileStatistics {
        let mut layers = BTreeMap::new();
        layers.insert(layer_name.to_string(), layer_stats);
        let mut sources = BTreeMap::new();
        sources.insert("src".to_string(), SourceStats { layers });
        TileStatistics {
            sources,
            sample_rate: 1.0,
        }
    }

    fn make_layer_info() -> Vec<Option<VectorLayerInfo>> {
        vec![Some(VectorLayerInfo {
            source: "src".to_string(),
            source_layer: "lyr".to_string(),
        })]
    }

    // ── has→false tests ─────────────────────────────────────────────────

    #[test]
    fn has_folds_false_when_property_absent_from_stats() {
        let stats = make_stats(
            "lyr",
            LayerStats {
                total_features: 100,
                properties: BTreeMap::new(),
                ..Default::default()
            },
        );
        let info = make_layer_info();
        let mut arr: Vec<Value> = serde_json::from_value(json!(["has", "missing"])).unwrap();
        assert!(try_fold_has_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - false
        ");
    }

    #[test]
    fn has_folds_false_when_present_count_zero() {
        let mut props = BTreeMap::new();
        props.insert(
            "empty".to_string(),
            PropertyStats::String {
                present_count: 0,
                cardinality: 0,
                value_counts: None,
            },
        );
        let stats = make_stats(
            "lyr",
            LayerStats {
                total_features: 100,
                properties: props,
                ..Default::default()
            },
        );
        let info = make_layer_info();
        let mut arr: Vec<Value> = serde_json::from_value(json!(["has", "empty"])).unwrap();
        assert!(try_fold_has_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - false
        ");
    }

    #[test]
    fn has_folds_true_when_always_present() {
        let mut props = BTreeMap::new();
        props.insert(
            "name".to_string(),
            PropertyStats::String {
                present_count: 100,
                cardinality: 5,
                value_counts: None,
            },
        );
        let stats = make_stats(
            "lyr",
            LayerStats {
                total_features: 100,
                properties: props,
                ..Default::default()
            },
        );
        let info = make_layer_info();
        let mut arr: Vec<Value> = serde_json::from_value(json!(["has", "name"])).unwrap();
        assert!(try_fold_has_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - true
        ");
    }

    #[test]
    fn has_no_fold_when_partially_present() {
        let mut props = BTreeMap::new();
        props.insert(
            "name".to_string(),
            PropertyStats::String {
                present_count: 50,
                cardinality: 5,
                value_counts: None,
            },
        );
        let stats = make_stats(
            "lyr",
            LayerStats {
                total_features: 100,
                properties: props,
                ..Default::default()
            },
        );
        let info = make_layer_info();
        let mut arr: Vec<Value> = serde_json::from_value(json!(["has", "name"])).unwrap();
        assert!(!try_fold_has_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
    }

    #[test]
    fn has_no_fold_when_zero_features() {
        let stats = make_stats(
            "lyr",
            LayerStats {
                total_features: 0,
                ..Default::default()
            },
        );
        let info = make_layer_info();
        let mut arr: Vec<Value> = serde_json::from_value(json!(["has", "x"])).unwrap();
        assert!(!try_fold_has_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
    }

    // ── geometry-type fold tests ────────────────────────────────────────

    #[test]
    fn geometry_type_eq_folds_true_when_only_type() {
        let stats = make_stats(
            "lyr",
            LayerStats {
                total_features: 500,
                geometry_types: GeometryTypeStats {
                    point: 500,
                    linestring: 0,
                    polygon: 0,
                    unknown: 0,
                },
                ..Default::default()
            },
        );
        let info = make_layer_info();
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["==", ["geometry-type"], "Point"])).unwrap();
        assert!(try_fold_geometry_type_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - true
        ");
    }

    #[test]
    fn geometry_type_eq_folds_false_when_absent() {
        let stats = make_stats(
            "lyr",
            LayerStats {
                total_features: 500,
                geometry_types: GeometryTypeStats {
                    point: 0,
                    linestring: 500,
                    polygon: 0,
                    unknown: 0,
                },
                ..Default::default()
            },
        );
        let info = make_layer_info();
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["==", ["geometry-type"], "Point"])).unwrap();
        assert!(try_fold_geometry_type_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - false
        ");
    }

    #[test]
    fn geometry_type_neq_folds_true_when_absent() {
        let stats = make_stats(
            "lyr",
            LayerStats {
                total_features: 500,
                geometry_types: GeometryTypeStats {
                    point: 0,
                    linestring: 500,
                    polygon: 0,
                    unknown: 0,
                },
                ..Default::default()
            },
        );
        let info = make_layer_info();
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["!=", ["geometry-type"], "Point"])).unwrap();
        assert!(try_fold_geometry_type_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - true
        ");
    }

    #[test]
    fn geometry_type_neq_folds_false_when_only_type() {
        let stats = make_stats(
            "lyr",
            LayerStats {
                total_features: 500,
                geometry_types: GeometryTypeStats {
                    point: 500,
                    linestring: 0,
                    polygon: 0,
                    unknown: 0,
                },
                ..Default::default()
            },
        );
        let info = make_layer_info();
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["!=", ["geometry-type"], "Point"])).unwrap();
        assert!(try_fold_geometry_type_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - false
        ");
    }

    #[test]
    fn geometry_type_no_fold_when_mixed_types() {
        let stats = make_stats(
            "lyr",
            LayerStats {
                total_features: 500,
                geometry_types: GeometryTypeStats {
                    point: 200,
                    linestring: 300,
                    polygon: 0,
                    unknown: 0,
                },
                ..Default::default()
            },
        );
        let info = make_layer_info();
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["==", ["geometry-type"], "Point"])).unwrap();
        assert!(!try_fold_geometry_type_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
    }

    #[test]
    fn geometry_type_reversed_operand_order() {
        let stats = make_stats(
            "lyr",
            LayerStats {
                total_features: 500,
                geometry_types: GeometryTypeStats {
                    point: 500,
                    linestring: 0,
                    polygon: 0,
                    unknown: 0,
                },
                ..Default::default()
            },
        );
        let info = make_layer_info();
        // String literal on the left, geometry-type on the right.
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["==", "Point", ["geometry-type"]])).unwrap();
        assert!(try_fold_geometry_type_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - true
        ");
    }

    #[test]
    fn geometry_type_no_fold_zero_features() {
        let stats = make_stats(
            "lyr",
            LayerStats {
                total_features: 0,
                ..Default::default()
            },
        );
        let info = make_layer_info();
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["==", ["geometry-type"], "Point"])).unwrap();
        assert!(!try_fold_geometry_type_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
    }

    // ── comparison fold tests ─────────────────────────────────────────

    fn int_stats(
        min: i64,
        max: i64,
        present: u64,
        total: u64,
    ) -> (TileStatistics, Vec<Option<VectorLayerInfo>>) {
        let mut props = BTreeMap::new();
        props.insert(
            "x".to_string(),
            PropertyStats::Integer {
                present_count: present,
                min,
                max,
                cardinality: 10,
                value_counts: None,
            },
        );
        let stats = make_stats(
            "lyr",
            LayerStats {
                total_features: total,
                properties: props,
                ..Default::default()
            },
        );
        (stats, make_layer_info())
    }

    #[test]
    fn lt_folds_false_when_min_ge_n() {
        // ["<", ["get", "x"], 2] with min=5 → always false
        let (stats, info) = int_stats(5, 10, 100, 100);
        let mut arr: Vec<Value> = serde_json::from_value(json!(["<", ["get", "x"], 2])).unwrap();
        assert!(try_fold_comparison_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - false
        ");
    }

    #[test]
    fn lt_folds_true_when_max_lt_n_and_all_present() {
        // ["<", ["get", "x"], 20] with max=10, present=total → always true
        let (stats, info) = int_stats(5, 10, 100, 100);
        let mut arr: Vec<Value> = serde_json::from_value(json!(["<", ["get", "x"], 20])).unwrap();
        assert!(try_fold_comparison_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - true
        ");
    }

    #[test]
    fn lt_no_fold_when_not_all_present() {
        // ["<", ["get", "x"], 20] with max=10 but present < total → can't fold to true
        let (stats, info) = int_stats(5, 10, 80, 100);
        let mut arr: Vec<Value> = serde_json::from_value(json!(["<", ["get", "x"], 20])).unwrap();
        assert!(!try_fold_comparison_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
    }

    #[test]
    fn lte_folds_false_when_min_gt_n() {
        // ["<=", ["get", "x"], 4] with min=5 → always false
        let (stats, info) = int_stats(5, 10, 100, 100);
        let mut arr: Vec<Value> = serde_json::from_value(json!(["<=", ["get", "x"], 4])).unwrap();
        assert!(try_fold_comparison_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - false
        ");
    }

    #[test]
    fn gte_folds_true_when_min_ge_n_and_all_present() {
        // [">=", ["get", "x"], 5] with min=5, present=total → always true
        let (stats, info) = int_stats(5, 10, 100, 100);
        let mut arr: Vec<Value> = serde_json::from_value(json!([">=", ["get", "x"], 5])).unwrap();
        assert!(try_fold_comparison_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - true
        ");
    }

    #[test]
    fn gt_reversed_operand() {
        // [">", 10, ["get", "x"]] ≡ ["<", ["get", "x"], 10] → with min=5, can't fold
        // But [">", 2, ["get", "x"]] ≡ ["<", ["get", "x"], 2] → with min=5 → false
        let (stats, info) = int_stats(5, 10, 100, 100);
        let mut arr: Vec<Value> = serde_json::from_value(json!([">", 2, ["get", "x"]])).unwrap();
        assert!(try_fold_comparison_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - false
        ");
    }

    #[test]
    fn eq_folds_false_when_out_of_range() {
        // ["==", ["get", "x"], 100] with min=0, max=10 → always false
        let (stats, info) = int_stats(0, 10, 100, 100);
        let mut arr: Vec<Value> = serde_json::from_value(json!(["==", ["get", "x"], 100])).unwrap();
        assert!(try_fold_comparison_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - false
        ");
    }

    #[test]
    fn neq_folds_true_when_out_of_range_and_all_present() {
        // ["!=", ["get", "x"], 100] with min=0, max=10, present=total → always true
        let (stats, info) = int_stats(0, 10, 100, 100);
        let mut arr: Vec<Value> = serde_json::from_value(json!(["!=", ["get", "x"], 100])).unwrap();
        assert!(try_fold_comparison_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - true
        ");
    }

    #[test]
    fn double_lt_folds_false_when_min_ge_n() {
        // ["<", ["get", "x"], 0.5] with Double min=1.0 → always false
        let mut props = BTreeMap::new();
        props.insert(
            "x".to_string(),
            PropertyStats::Double {
                present_count: 100,
                min: 1.0,
                max: 5.0,
                cardinality: 10,
            },
        );
        let stats = make_stats(
            "lyr",
            LayerStats {
                total_features: 100,
                properties: props,
                ..Default::default()
            },
        );
        let info = make_layer_info();
        let mut arr: Vec<Value> = serde_json::from_value(json!(["<", ["get", "x"], 0.5])).unwrap();
        assert!(try_fold_comparison_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - false
        ");
    }

    #[test]
    fn eq_folds_false_with_value_counts() {
        // ["==", ["get", "x"], 7] with value_counts={2: 500, 4: 4000} → 7 not in counts → false
        let mut int_vc = BTreeMap::new();
        int_vc.insert(2i64, 500u64);
        int_vc.insert(4, 4000);
        let mut props = BTreeMap::new();
        props.insert(
            "x".to_string(),
            PropertyStats::Integer {
                present_count: 4500,
                min: 2,
                max: 4,
                cardinality: 2,
                value_counts: Some(int_vc),
            },
        );
        let stats = make_stats(
            "lyr",
            LayerStats {
                total_features: 4500,
                properties: props,
                ..Default::default()
            },
        );
        let info = make_layer_info();
        let mut arr: Vec<Value> = serde_json::from_value(json!(["==", ["get", "x"], 7])).unwrap();
        assert!(try_fold_comparison_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - false
        ");
    }

    #[test]
    fn non_numeric_property_no_fold() {
        // String property → comparison folding not supported
        let mut props = BTreeMap::new();
        props.insert(
            "x".to_string(),
            PropertyStats::String {
                present_count: 100,
                cardinality: 5,
                value_counts: None,
            },
        );
        let stats = make_stats(
            "lyr",
            LayerStats {
                total_features: 100,
                properties: props,
                ..Default::default()
            },
        );
        let info = make_layer_info();
        let mut arr: Vec<Value> = serde_json::from_value(json!(["<", ["get", "x"], 5])).unwrap();
        assert!(!try_fold_comparison_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
    }

    // ── Bool comparison fold tests ────────────────────────────────────

    fn bool_stats(
        true_count: u64,
        present: u64,
        total: u64,
    ) -> (TileStatistics, Vec<Option<VectorLayerInfo>>) {
        let mut props = BTreeMap::new();
        props.insert(
            "bridge".to_string(),
            PropertyStats::Bool {
                present_count: present,
                true_count,
            },
        );
        let stats = make_stats(
            "lyr",
            LayerStats {
                total_features: total,
                properties: props,
                ..Default::default()
            },
        );
        (stats, make_layer_info())
    }

    #[test]
    fn bool_eq_true_folds_false_when_no_trues() {
        let (stats, info) = bool_stats(0, 100, 100);
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["==", ["get", "bridge"], true])).unwrap();
        assert!(try_fold_comparison_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - false
        ");
    }

    #[test]
    fn bool_eq_true_folds_true_when_all_true() {
        let (stats, info) = bool_stats(100, 100, 100);
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["==", ["get", "bridge"], true])).unwrap();
        assert!(try_fold_comparison_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - true
        ");
    }

    #[test]
    fn bool_eq_false_folds_true_when_all_false() {
        let (stats, info) = bool_stats(0, 100, 100);
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["==", ["get", "bridge"], false])).unwrap();
        assert!(try_fold_comparison_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - true
        ");
    }

    #[test]
    fn bool_eq_false_folds_false_when_all_true() {
        let (stats, info) = bool_stats(100, 100, 100);
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["==", ["get", "bridge"], false])).unwrap();
        assert!(try_fold_comparison_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - false
        ");
    }

    #[test]
    fn bool_neq_true_folds_false_when_all_true() {
        let (stats, info) = bool_stats(100, 100, 100);
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["!=", ["get", "bridge"], true])).unwrap();
        assert!(try_fold_comparison_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - false
        ");
    }

    #[test]
    fn bool_no_fold_when_mixed() {
        let (stats, info) = bool_stats(50, 100, 100);
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["==", ["get", "bridge"], true])).unwrap();
        assert!(!try_fold_comparison_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
    }

    #[test]
    fn bool_no_fold_when_not_all_present() {
        let (stats, info) = bool_stats(0, 50, 100);
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["==", ["get", "bridge"], false])).unwrap();
        assert!(!try_fold_comparison_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
    }

    // ── in pruning tests ─────────────────────────────────────────────

    use indexmap::IndexMap;



    fn string_stats_with_values(
        values: &[&str],
        total: u64,
    ) -> (TileStatistics, Vec<Option<VectorLayerInfo>>) {
        let mut vc = IndexMap::new();
        for &v in values {
            vc.insert(v.to_string(), 10);
        }
        let mut props = BTreeMap::new();
        props.insert(
            "kind".to_string(),
            PropertyStats::String {
                present_count: total,
                cardinality: values.len() as u64,
                value_counts: Some(vc),
            },
        );
        let stats = make_stats(
            "lyr",
            LayerStats {
                total_features: total,
                properties: props,
                ..Default::default()
            },
        );
        (stats, make_layer_info())
    }

    fn string_stats_with_sample_rate(
        values: &[&str],
        total: u64,
        sample_rate: f64,
    ) -> (TileStatistics, Vec<Option<VectorLayerInfo>>) {
        let (mut stats, info) = string_stats_with_values(values, total);
        stats.sample_rate = sample_rate;
        (stats, info)
    }

    #[test]
    fn in_prune_removes_dead_values() {
        let (stats, info) = string_stats_with_values(&["a", "b"], 100);
        let mut arr: Vec<Value> = serde_json::from_value(json!([
            "in",
            ["get", "kind"],
            ["literal", ["a", "b", "c", "d"]]
        ]))
        .unwrap();
        assert!(try_prune_in_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        // After pruning "c","d", remaining ["a","b"] covers all values and
        // present_count == total → folds to true.
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - true
        ");
    }

    #[test]
    fn in_prune_all_dead_folds_to_false() {
        let (stats, info) = string_stats_with_values(&["a", "b"], 100);
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["in", ["get", "kind"], ["literal", ["x", "y", "z"]]]))
                .unwrap();
        assert!(try_prune_in_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - false
        ");
    }

    #[test]
    fn in_prune_single_rewrites_to_eq() {
        let (stats, info) = string_stats_with_values(&["a", "b"], 100);
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["in", ["get", "kind"], ["literal", ["a", "x"]]]))
                .unwrap();
        assert!(try_prune_in_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r#"
        - "=="
        - - get
          - kind
        - a
        "#);
    }

    #[test]
    fn in_prune_skipped_when_sample_rate_below_1() {
        let (stats, info) = string_stats_with_sample_rate(&["a", "b"], 100, 0.5);
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["in", ["get", "kind"], ["literal", ["a", "x"]]]))
                .unwrap();
        assert!(!try_prune_in_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
    }

    #[test]
    fn in_prune_no_change_when_all_present() {
        let (stats, info) = string_stats_with_values(&["a", "b", "c"], 100);
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["in", ["get", "kind"], ["literal", ["a", "b"]]]))
                .unwrap();
        assert!(!try_prune_in_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
    }

    // ── match pruning tests ──────────────────────────────────────────

    #[test]
    fn match_prune_removes_dead_arm() {
        let (stats, info) = string_stats_with_values(&["a", "b"], 100);
        let mut arr: Vec<Value> = serde_json::from_value(json!([
            "match",
            ["get", "kind"],
            "a",
            "A",
            "c",
            "C",
            "fallback"
        ]))
        .unwrap();
        assert!(try_prune_match_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - match
        - - get
          - kind
        - a
        - A
        - fallback
        ");
    }

    #[test]
    fn match_prune_all_dead_folds_to_fallback() {
        let (stats, info) = string_stats_with_values(&["a", "b"], 100);
        let mut arr: Vec<Value> = serde_json::from_value(json!([
            "match",
            ["get", "kind"],
            "x",
            "X",
            "y",
            "Y",
            "fallback"
        ]))
        .unwrap();
        assert!(try_prune_match_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - fallback
        ");
    }

    #[test]
    fn match_prune_array_label_partial() {
        let (stats, info) = string_stats_with_values(&["a"], 100);
        let mut arr: Vec<Value> = serde_json::from_value(json!([
            "match",
            ["get", "kind"],
            ["a", "x"],
            "out",
            "fallback"
        ]))
        .unwrap();
        assert!(try_prune_match_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - match
        - - get
          - kind
        - a
        - out
        - fallback
        ");
    }

    #[test]
    fn match_prune_skipped_when_sample_rate_below_1() {
        let (stats, info) = string_stats_with_sample_rate(&["a", "b"], 100, 0.5);
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["match", ["get", "kind"], "x", "X", "fallback"]))
                .unwrap();
        assert!(!try_prune_match_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
    }

    // ── coalesce fold tests ──────────────────────────────────────────

    fn coalesce_stats(
        props: Vec<(&str, u64)>,
        total: u64,
    ) -> (TileStatistics, Vec<Option<VectorLayerInfo>>) {
        let mut prop_map = BTreeMap::new();
        for (name, present) in props {
            prop_map.insert(
                name.to_string(),
                PropertyStats::String {
                    present_count: present,
                    cardinality: 5,
                    value_counts: None,
                },
            );
        }
        let stats = make_stats(
            "lyr",
            LayerStats {
                total_features: total,
                properties: prop_map,
                ..Default::default()
            },
        );
        (stats, make_layer_info())
    }

    #[test]
    fn coalesce_truncates_dead_arms() {
        let (stats, info) = coalesce_stats(vec![("name", 100), ("alt_name", 80)], 100);
        let mut arr: Vec<Value> = serde_json::from_value(json!([
            "coalesce",
            ["get", "name"],
            ["get", "alt_name"],
            "default"
        ]))
        .unwrap();
        assert!(try_fold_coalesce_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        // "name" is always present → truncate alt_name and default, unwrap single arm.
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - get
        - name
        ");
    }

    #[test]
    fn coalesce_keeps_arm_when_not_always_present() {
        let (stats, info) = coalesce_stats(vec![("name", 80), ("alt_name", 100)], 100);
        let mut arr: Vec<Value> = serde_json::from_value(json!([
            "coalesce",
            ["get", "name"],
            ["get", "alt_name"],
            "default"
        ]))
        .unwrap();
        assert!(try_fold_coalesce_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        // "name" not always present, but "alt_name" is → truncate "default".
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - coalesce
        - - get
          - name
        - - get
          - alt_name
        ");
    }

    #[test]
    fn coalesce_no_change_when_none_always_present() {
        let (stats, info) = coalesce_stats(vec![("name", 80), ("alt_name", 80)], 100);
        let mut arr: Vec<Value> = serde_json::from_value(json!([
            "coalesce",
            ["get", "name"],
            ["get", "alt_name"],
            "default"
        ]))
        .unwrap();
        assert!(!try_fold_coalesce_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
    }

    #[test]
    fn coalesce_skipped_when_sample_rate_below_1() {
        let (mut stats, info) = coalesce_stats(vec![("name", 100)], 100);
        stats.sample_rate = 0.5;
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["coalesce", ["get", "name"], "default"])).unwrap();
        assert!(!try_fold_coalesce_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
    }

    // ── coalesce: never-present arm removal ──────────────────────────

    #[test]
    fn coalesce_removes_never_present_arms() {
        let (stats, info) = coalesce_stats(vec![("name", 0), ("alt_name", 80)], 100);
        let mut arr: Vec<Value> = serde_json::from_value(json!([
            "coalesce",
            ["get", "name"],
            ["get", "alt_name"],
            "default"
        ]))
        .unwrap();
        assert!(try_fold_coalesce_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - coalesce
        - - get
          - alt_name
        - default
        ");
    }

    #[test]
    fn coalesce_removes_unknown_prop_arm() {
        // "missing" not in stats at all → never present.
        let (stats, info) = coalesce_stats(vec![("name", 80)], 100);
        let mut arr: Vec<Value> = serde_json::from_value(json!([
            "coalesce",
            ["get", "missing"],
            ["get", "name"],
            "default"
        ]))
        .unwrap();
        assert!(try_fold_coalesce_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - coalesce
        - - get
          - name
        - default
        ");
    }

    #[test]
    fn coalesce_collapses_when_all_get_arms_never_present() {
        let (stats, info) = coalesce_stats(vec![("a", 0), ("b", 0)], 100);
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["coalesce", ["get", "a"], ["get", "b"], "fallback"]))
                .unwrap();
        assert!(try_fold_coalesce_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        // Only "fallback" remains → unwrapped.
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - fallback
        ");
    }

    // ── in → true folding tests ──────────────────────────────────────

    #[test]
    fn in_folds_true_when_all_values_covered() {
        let (stats, info) = string_stats_with_values(&["a", "b"], 100);
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["in", ["get", "kind"], ["literal", ["a", "b", "c"]]]))
                .unwrap();
        // "c" not in stats → pruned. Then "a","b" cover all values and present==total → true.
        assert!(try_prune_in_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - true
        ");
    }

    #[test]
    fn in_folds_true_exact_coverage() {
        let (stats, info) = string_stats_with_values(&["x", "y"], 100);
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["in", ["get", "kind"], ["literal", ["x", "y"]]]))
                .unwrap();
        assert!(try_prune_in_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - literal
        - true
        ");
    }

    #[test]
    fn in_no_fold_true_when_not_all_present() {
        // present_count (50) < total_features (100) — can't fold to true.
        let mut vc = IndexMap::new();
        vc.insert("a".to_string(), 10u64);
        let mut props = BTreeMap::new();
        props.insert(
            "kind".to_string(),
            PropertyStats::String {
                present_count: 50,
                cardinality: 1,
                value_counts: Some(vc),
            },
        );
        let stats = make_stats(
            "lyr",
            LayerStats {
                total_features: 100,
                properties: props,
                ..Default::default()
            },
        );
        let info = make_layer_info();
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["in", ["get", "kind"], ["literal", ["a"]]])).unwrap();
        assert!(!try_prune_in_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
    }

    // ── data-driven ramp pruning tests ───────────────────────────────

    fn int_prop_stats(
        prop: &str,
        min: i64,
        max: i64,
        present: u64,
        total: u64,
    ) -> (TileStatistics, Vec<Option<VectorLayerInfo>>) {
        let mut props = BTreeMap::new();
        props.insert(
            prop.to_string(),
            PropertyStats::Integer {
                present_count: present,
                min,
                max,
                cardinality: 10,
                value_counts: None,
            },
        );
        let stats = make_stats(
            "lyr",
            LayerStats {
                total_features: total,
                properties: props,
                ..Default::default()
            },
        );
        (stats, make_layer_info())
    }

    #[test]
    fn step_prune_above_max() {
        let (stats, info) = int_prop_stats("rank", 1, 5, 100, 100);
        let mut arr: Vec<Value> = serde_json::from_value(json!([
            "step",
            ["get", "rank"],
            "small",
            3,
            "medium",
            7,
            "large"
        ]))
        .unwrap();
        assert!(try_prune_data_ramp_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        assert_yaml_snapshot!(Value::Array(arr), @r#"
        - step
        - - get
          - rank
        - small
        - 3
        - medium
        "#);
    }

    #[test]
    fn step_prune_below_min_when_all_present() {
        let (stats, info) = int_prop_stats("rank", 5, 10, 100, 100);
        let mut arr: Vec<Value> = serde_json::from_value(json!([
            "step",
            ["get", "rank"],
            "tiny",
            3,
            "small",
            5,
            "medium",
            8,
            "large"
        ]))
        .unwrap();
        assert!(try_prune_data_ramp_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        // Stops at 3 and 5 are ≤ min(5), so absorbed into default.
        assert_yaml_snapshot!(Value::Array(arr), @r#"
        - step
        - - get
          - rank
        - medium
        - 8
        - large
        "#);
    }

    #[test]
    fn step_no_prune_below_min_when_not_all_present() {
        let (stats, info) = int_prop_stats("rank", 5, 10, 80, 100);
        let mut arr: Vec<Value> = serde_json::from_value(json!([
            "step",
            ["get", "rank"],
            "tiny",
            3,
            "small",
            8,
            "large"
        ]))
        .unwrap();
        // threshold 3 < min=5, but not all present → below-min pruning unsafe.
        // threshold 8 ≤ max=10 → no above-max pruning either.
        assert!(!try_prune_data_ramp_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
    }

    #[test]
    fn step_collapses_when_all_pruned() {
        let (stats, info) = int_prop_stats("rank", 1, 2, 100, 100);
        let mut arr: Vec<Value> = serde_json::from_value(json!([
            "step",
            ["get", "rank"],
            "small",
            5,
            "medium",
            10,
            "large"
        ]))
        .unwrap();
        assert!(try_prune_data_ramp_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        // All stops > max(2) → removed → only default remains → collapse.
        assert_yaml_snapshot!(Value::Array(arr), @r#"
        - literal
        - small
        "#);
    }

    #[test]
    fn interpolate_prune_above_max() {
        let (stats, info) = int_prop_stats("pop", 0, 500, 100, 100);
        let mut arr: Vec<Value> = serde_json::from_value(json!([
            "interpolate",
            ["linear"],
            ["get", "pop"],
            0,
            1,
            200,
            5,
            500,
            10,
            1000,
            20
        ]))
        .unwrap();
        assert!(try_prune_data_ramp_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        // Keep boundary at 500 (≥ max=500), remove 1000.
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - interpolate
        - - linear
        - - get
          - pop
        - 0
        - 1
        - 200
        - 5
        - 500
        - 10
        ");
    }

    #[test]
    fn interpolate_prune_below_min() {
        let (stats, info) = int_prop_stats("pop", 500, 1000, 100, 100);
        let mut arr: Vec<Value> = serde_json::from_value(json!([
            "interpolate",
            ["linear"],
            ["get", "pop"],
            0,
            1,
            100,
            3,
            500,
            10,
            1000,
            20
        ]))
        .unwrap();
        assert!(try_prune_data_ramp_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
        // Keep boundary at 500 (last ≤ min=500), remove 0 and 100.
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - interpolate
        - - linear
        - - get
          - pop
        - 500
        - 10
        - 1000
        - 20
        ");
    }

    #[test]
    fn data_ramp_no_prune_when_zoom_driven() {
        let (stats, info) = int_prop_stats("x", 1, 5, 100, 100);
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["step", ["zoom"], "small", 3, "medium", 7, "large"]))
                .unwrap();
        // ["zoom"] is not ["get", prop] → no pruning.
        assert!(!try_prune_data_ramp_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
    }

    #[test]
    fn data_ramp_no_prune_when_sample_rate_below_1() {
        let (mut stats, info) = int_prop_stats("rank", 1, 5, 100, 100);
        stats.sample_rate = 0.5;
        let mut arr: Vec<Value> = serde_json::from_value(json!([
            "step",
            ["get", "rank"],
            "small",
            3,
            "medium",
            7,
            "large"
        ]))
        .unwrap();
        assert!(!try_prune_data_ramp_from_stats(
            &mut arr,
            Some(&stats),
            Some(&info),
            0
        ));
    }
}

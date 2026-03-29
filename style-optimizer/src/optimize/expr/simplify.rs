//! Expression simplification: interpolate/step collapsing, match dedup, any→in, case/coalesce.

use serde_json::Value;

use super::util::{extract_json_literal, replace_arr_with_value};

/// Simplify `interpolate`/`interpolate-hcl`/`interpolate-lab` and `step` expressions
/// when all output values are structurally equal.
pub(super) fn try_simplify_interpolate_or_step(arr: &mut Vec<Value>) -> bool {
    let Some(op) = arr[0].as_str() else {
        return false;
    };

    match op {
        "interpolate" | "interpolate-hcl" | "interpolate-lab" => {
            if arr.len() < 5 {
                return false;
            }
            let pairs_after_header = arr.len() - 3; // elements after ["op", method, input]
            if !pairs_after_header.is_multiple_of(2) {
                return false; // malformed
            }
            let first = &arr[4];
            if arr[4..].iter().step_by(2).all(|v| v == first) {
                let val = first.clone();
                *arr = vec![Value::String("literal".to_string()), val];
                return true;
            }
            // Prune intermediate stops between identical values.
            try_interpolate_stop_pruning(arr)
        }
        "step" => {
            if arr.len() < 3 {
                return false;
            }
            if arr.len() == 3 {
                let val = arr[2].clone();
                *arr = vec![Value::String("literal".to_string()), val];
                return true;
            }
            let pairs_after_default = arr.len() - 3;
            if !pairs_after_default.is_multiple_of(2) {
                return false; // malformed
            }
            let default_val = &arr[2];
            if arr[4..].iter().step_by(2).all(|v| v == default_val) {
                let val = default_val.clone();
                *arr = vec![Value::String("literal".to_string()), val];
                return true;
            }
            // Default == first stop output → first stop is redundant.
            if arr.len() >= 5 && arr[2] == arr.last().unwrap() {
                arr.remove(2); // stop output
                arr.remove(1); // stop threshold
                return true;
            }
            // Dedup adjacent same-output stops.
            try_step_stop_dedup(arr)
        }
        _ => false,
    }
}

/// Merge `match` expression arms that produce the same output value.
///
/// - Multiple labels with same output → grouped label array.
/// - All arms (including fallback) produce same value → collapse to that value.
/// - Arms whose output equals the fallback → remove those arms.
pub(super) fn try_simplify_match(arr: &mut Vec<Value>) -> bool {
    if arr.first().and_then(Value::as_str) != Some("match") {
        return false;
    }
    // Match layout: ["match", input, label1, out1, ..., fallback] — always odd length ≥ 5.
    if arr.len() < 5 {
        return false;
    }
    if arr.len().is_multiple_of(2) {
        return false;
    }

    let input = arr[1].clone();
    let fallback = arr.last().unwrap().clone();

    let arm_count = (arr.len() - 3) / 2;
    let mut arms: Vec<(Vec<Value>, Value)> = Vec::with_capacity(arm_count);
    for i in 0..arm_count {
        let label_val = arr[2 + i * 2].clone();
        let output = arr[3 + i * 2].clone();
        let labels = match label_val {
            Value::Array(labels) => labels,
            single => vec![single],
        };
        arms.push((labels, output));
    }

    let all_same_as_fallback = arms.iter().all(|(_, out)| *out == fallback);
    if all_same_as_fallback {
        *arr = vec![Value::String("literal".to_string()), fallback];
        return true;
    }

    // Fallback-matching arms are redundant.
    let before = arms.len();
    arms.retain(|(_, out)| *out != fallback);
    let removed_fallback_arms = arms.len() < before;

    let mut grouped: Vec<(Vec<Value>, Value)> = Vec::new();
    'arm: for (labels, output) in arms {
        for (existing_labels, existing_out) in &mut grouped {
            if *existing_out == output {
                existing_labels.extend(labels);
                continue 'arm;
            }
        }
        grouped.push((labels, output));
    }

    let same_structure = !removed_fallback_arms
        && grouped.len() == arm_count
        && grouped
            .iter()
            .zip(arr[2..].chunks(2))
            .all(|((new_labels, new_out), chunk)| {
                let orig_label = match &chunk[0] {
                    Value::Array(a) => a.clone(),
                    single => vec![single.clone()],
                };
                *new_labels == orig_label && *new_out == chunk[1]
            });
    if same_structure {
        return false;
    }

    let mut new_arr = vec![Value::String("match".to_string()), input];
    for (labels, output) in grouped {
        let label_val = if labels.len() == 1 {
            labels.into_iter().next().unwrap()
        } else {
            Value::Array(labels)
        };
        new_arr.push(label_val);
        new_arr.push(output);
    }
    new_arr.push(fallback);

    // All arms were merged away — collapse to fallback.
    if new_arr.len() == 3 {
        *arr = vec![Value::String("literal".to_string()), new_arr.remove(2)];
    } else {
        *arr = new_arr;
    }
    true
}

/// Rewrite `["any", ["==", x, a], ["==", x, b], ...]` → `["in", x, ["literal", [a, b, ...]]]`.
///
/// Only applies when every predicate is `["==", same_expr, literal]` (or the commuted form).
/// Requires at least two predicates.
pub(super) fn try_rewrite_any_to_in(arr: &mut Vec<Value>) -> bool {
    if arr.first().and_then(Value::as_str) != Some("any") {
        return false;
    }
    if arr.len() < 3 {
        return false;
    }
    let Some((common_expr, values)) = extract_eq_chain(&arr[1..]) else {
        return false;
    };
    let literal_arr = Value::Array(vec![
        Value::String("literal".to_string()),
        Value::Array(values),
    ]);
    *arr = vec![Value::String("in".to_string()), common_expr, literal_arr];
    true
}

/// Extracts `(common_lhs_expr, [rhs_literals])` when all predicates are `["==", same_expr, lit]`.
fn extract_eq_chain(predicates: &[Value]) -> Option<(Value, Vec<Value>)> {
    let mut common_expr: Option<Value> = None;
    let mut values = Vec::with_capacity(predicates.len());
    for pred in predicates {
        let Value::Array(p) = pred else {
            return None;
        };
        if p.len() != 3 || p[0].as_str() != Some("==") {
            return None;
        }
        let (expr, val) = if let Some(lit) = extract_json_literal(&p[2]) {
            (p[1].clone(), lit)
        } else if let Some(lit) = extract_json_literal(&p[1]) {
            (p[2].clone(), lit)
        } else {
            return None;
        };
        match &common_expr {
            None => {
                common_expr = Some(expr);
            }
            Some(e) if *e == expr => {}
            _ => return None,
        }
        values.push(val);
    }
    common_expr.map(|e| (e, values))
}

/// Simplify `case` expressions by removing redundant trailing arms.
///
/// - All arms + fallback produce the same value → collapse to that value.
/// - Trailing arms whose output equals the fallback → remove them (they'd return
///   the fallback anyway, so they add no value).
///
/// Note: only trailing arms can be safely removed. Removing a middle arm would
/// change which subsequent condition is evaluated first.
pub(super) fn try_simplify_case(arr: &mut Vec<Value>) -> bool {
    if arr.first().and_then(Value::as_str) != Some("case") {
        return false;
    }
    if arr.len() < 4 || !arr.len().is_multiple_of(2) {
        return false;
    }
    let fallback = arr.last().unwrap().clone();
    let n_arms = (arr.len() - 2) / 2;
    if (0..n_arms).all(|i| arr[2 + 2 * i] == fallback) {
        replace_arr_with_value(arr, fallback);
        return true;
    }
    // Only trailing arms can be trimmed; removing middle arms changes eval order.
    let trim_count = (0..n_arms)
        .rev()
        .take_while(|&i| arr[2 + 2 * i] == fallback)
        .count();
    if trim_count > 0 {
        let fallback = arr.pop().unwrap(); // temporarily remove fallback
        arr.truncate(arr.len() - 2 * trim_count); // drop trailing arms
        arr.push(fallback); // re-attach fallback
        return true;
    }
    false
}

/// Simplify `coalesce` expressions:
///
/// - `["coalesce", x]` → `x` (single arg)
/// - Null literal args are removed (they always pass through to the next arg).
/// - After a non-null literal arg, all subsequent args are unreachable → truncate.
pub(super) fn try_simplify_coalesce(arr: &mut Vec<Value>) -> bool {
    if arr.first().and_then(Value::as_str) != Some("coalesce") {
        return false;
    }
    if arr.len() < 2 {
        return false;
    }
    if arr.len() == 2 {
        let x = arr[1].clone();
        replace_arr_with_value(arr, x);
        return true;
    }
    let mut i = 1;
    while i < arr.len() {
        match extract_json_literal(&arr[i]) {
            Some(Value::Null) => {
                arr.remove(i);
                return true;
            }
            Some(_) => {
                // coalesce short-circuits on first non-null.
                if i + 1 < arr.len() {
                    arr.truncate(i + 1);
                    return true;
                }
                break;
            }
            None => {
                i += 1;
            }
        }
    }
    false
}

/// Flatten nested boolean operators: `["all", ["all", A, B], C]` → `["all", A, B, C]`.
/// Also for `any`.
pub(super) fn try_boolean_flattening(arr: &mut Vec<Value>) -> bool {
    let op = match arr.first().and_then(Value::as_str) {
        Some(op @ ("any" | "all")) => op.to_string(),
        _ => return false,
    };
    if arr.len() < 2 {
        return false;
    }

    let mut flattened = vec![Value::String(op.clone())];
    let mut did_flatten = false;

    for child in arr.iter().skip(1) {
        if let Value::Array(inner) = child
            && !inner.is_empty()
            && inner[0].as_str() == Some(op.as_str())
        {
            flattened.extend(inner.iter().skip(1).cloned());
            did_flatten = true;
        } else {
            flattened.push(child.clone());
        }
    }

    if did_flatten {
        *arr = flattened;
    }
    did_flatten
}

/// De Morgan's law: `["!", ["any", A, B]]` → `["all", ["!", A], ["!", B]]`
/// and `["!", ["all", A, B]]` → `["any", ["!", A], ["!", B]]`.
pub(super) fn try_demorgan(arr: &mut Vec<Value>) -> bool {
    if arr.len() != 2 || arr[0].as_str() != Some("!") {
        return false;
    }
    let Value::Array(inner) = &arr[1] else {
        return false;
    };
    let inner_op = match inner.first().and_then(Value::as_str) {
        Some("any") => "all",
        Some("all") => "any",
        _ => return false,
    };
    if inner.len() < 3 {
        return false;
    }
    let mut result = vec![Value::String(inner_op.to_string())];
    for child in inner.iter().skip(1) {
        result.push(Value::Array(vec![
            Value::String("!".to_string()),
            child.clone(),
        ]));
    }
    *arr = result;
    true
}

/// Single-element `in` → `==`: `["in", x, ["literal", [v]]]` → `["==", x, v]`.
pub(super) fn try_simplify_single_in(arr: &mut Vec<Value>) -> bool {
    if arr.len() != 3 || arr[0].as_str() != Some("in") {
        return false;
    }
    let Value::Array(lit_arr) = &arr[2] else {
        return false;
    };
    if lit_arr.len() != 2 || lit_arr[0].as_str() != Some("literal") {
        return false;
    }
    let Value::Array(values) = &lit_arr[1] else {
        return false;
    };
    if values.len() != 1 {
        return false;
    }
    let expr = arr[1].clone();
    let val = values[0].clone();
    *arr = vec![Value::String("==".to_string()), expr, val];
    true
}

/// Merge multiple `in` expressions on the same expression inside `any`:
/// `["any", ["in", x, ["literal", [a]]], ["in", x, ["literal", [b]]]]`
/// → `["any", ["in", x, ["literal", [a, b]]]]`
/// (then unary simplification will unwrap the `any`)
pub(super) fn try_merge_in_expressions(arr: &mut Vec<Value>) -> bool {
    if arr.first().and_then(Value::as_str) != Some("any") {
        return false;
    }
    if arr.len() < 3 {
        return false;
    }

    // Check if we have at least two `in` expressions with the same LHS.
    let mut in_indices: Vec<(usize, Value, Vec<Value>)> = Vec::new();
    for (i, child) in arr.iter().skip(1).enumerate() {
        if let Some((expr, values)) = extract_in_expr(child) {
            in_indices.push((i + 1, expr, values));
        }
    }

    if in_indices.len() < 2 {
        return false;
    }

    // Group by common expression.
    let first_expr = &in_indices[0].1;
    let all_same = in_indices.iter().all(|(_, e, _)| e == first_expr);
    if !all_same {
        return false;
    }

    // Merge all value lists.
    let mut merged_values: Vec<Value> = Vec::new();
    let mut indices_to_remove: Vec<usize> = Vec::new();
    for (idx, _, values) in &in_indices {
        merged_values.extend(values.iter().cloned());
        indices_to_remove.push(*idx);
    }

    // Build the merged `in` expression.
    let merged_in = Value::Array(vec![
        Value::String("in".to_string()),
        first_expr.clone(),
        Value::Array(vec![
            Value::String("literal".to_string()),
            Value::Array(merged_values),
        ]),
    ]);

    // Remove old `in` entries (in reverse to preserve indices) and insert the merged one.
    let first_idx = indices_to_remove[0];
    for idx in indices_to_remove.into_iter().rev() {
        arr.remove(idx);
    }
    arr.insert(first_idx, merged_in);
    true
}

/// Extract the expression and values from an `["in", expr, ["literal", [values...]]]`.
fn extract_in_expr(v: &Value) -> Option<(Value, Vec<Value>)> {
    let Value::Array(arr) = v else {
        return None;
    };
    if arr.len() != 3 || arr[0].as_str() != Some("in") {
        return None;
    }
    let Value::Array(lit_arr) = &arr[2] else {
        return None;
    };
    if lit_arr.len() != 2 || lit_arr[0].as_str() != Some("literal") {
        return None;
    }
    let Value::Array(values) = &lit_arr[1] else {
        return None;
    };
    Some((arr[1].clone(), values.clone()))
}

/// Inline single-use `let`/`var` bindings.
///
/// `["let", "x", expr, ["var", "x"]]` → `expr`
/// Only inlines when the variable is used exactly once in the body.
pub(super) fn try_inline_let_var(arr: &mut Vec<Value>) -> bool {
    if arr.first().and_then(Value::as_str) != Some("let") {
        return false;
    }
    // ["let", name1, val1, ..., body] — at least 4 elements (1 binding + body).
    if arr.len() < 4 || arr.len().is_multiple_of(2) {
        return false;
    }

    let n_bindings = (arr.len() - 2) / 2;
    let body_idx = arr.len() - 1;

    // Find single-use bindings.
    for i in 0..n_bindings {
        let name_idx = 1 + i * 2;
        let val_idx = 2 + i * 2;
        let Some(name) = arr[name_idx].as_str() else {
            continue;
        };

        // Count uses of this variable in the body.
        let uses = count_var_uses(&arr[body_idx], name);
        if uses == 1 {
            let val = arr[val_idx].clone();
            let body = arr[body_idx].clone();
            let inlined = substitute_var(&body, name, &val);

            if n_bindings == 1 {
                // Only binding → result is just the body.
                replace_arr_with_value(arr, inlined);
            } else {
                // Remove this binding, keep others.
                arr.remove(val_idx);
                arr.remove(name_idx);
                *arr.last_mut().unwrap() = inlined;
            }
            return true;
        }
    }
    false
}

/// Count occurrences of `["var", name]` in a value tree.
fn count_var_uses(v: &Value, name: &str) -> usize {
    match v {
        Value::Array(arr) => {
            if arr.len() == 2 && arr[0].as_str() == Some("var") && arr[1].as_str() == Some(name) {
                1
            } else {
                arr.iter().map(|child| count_var_uses(child, name)).sum()
            }
        }
        Value::Object(map) => map.values().map(|child| count_var_uses(child, name)).sum(),
        _ => 0,
    }
}

/// Replace all `["var", name]` with `replacement` in a value tree.
fn substitute_var(v: &Value, name: &str, replacement: &Value) -> Value {
    match v {
        Value::Array(arr) => {
            if arr.len() == 2 && arr[0].as_str() == Some("var") && arr[1].as_str() == Some(name) {
                replacement.clone()
            } else {
                Value::Array(
                    arr.iter()
                        .map(|child| substitute_var(child, name, replacement))
                        .collect(),
                )
            }
        }
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(k, child)| (k.clone(), substitute_var(child, name, replacement)))
                .collect(),
        ),
        other => other.clone(),
    }
}

/// Remove intermediate stops in `interpolate` expressions where adjacent stops have
/// identical output values.
fn try_interpolate_stop_pruning(arr: &mut Vec<Value>) -> bool {
    // ["interpolate", method, input, z0, v0, z1, v1, z2, v2, ...]
    // Header is 3 elements, then pairs of (zoom, value).
    if arr.len() < 7 {
        return false; // Need at least 3 stops to prune
    }
    let n_stops = (arr.len() - 3) / 2;
    if n_stops < 3 {
        return false;
    }

    // Find interior stops where prev_val == this_val == next_val.
    let mut to_remove: Vec<usize> = Vec::new();
    for i in 1..(n_stops - 1) {
        let prev_val = &arr[3 + (i - 1) * 2 + 1];
        let this_val = &arr[3 + i * 2 + 1];
        let next_val = &arr[3 + (i + 1) * 2 + 1];
        if prev_val == this_val && this_val == next_val {
            to_remove.push(i);
        }
    }

    if to_remove.is_empty() {
        return false;
    }

    // Remove in reverse order (pairs: zoom_idx = 3 + i*2, val_idx = 3 + i*2 + 1).
    for &i in to_remove.iter().rev() {
        let zoom_idx = 3 + i * 2;
        arr.remove(zoom_idx + 1);
        arr.remove(zoom_idx);
    }
    true
}

/// Deduplicate adjacent step stops with the same output value.
/// `["step", input, default, z1, v1, z2, v1, z3, v2]`
/// → `["step", input, default, z1, v1, z3, v2]`
fn try_step_stop_dedup(arr: &mut Vec<Value>) -> bool {
    // ["step", input, default, z1, v1, z2, v2, ...]
    if arr.len() < 7 {
        return false; // Need at least 2 stops
    }
    let n_stops = (arr.len() - 3) / 2;
    if n_stops < 2 {
        return false;
    }

    let mut to_remove: Vec<usize> = Vec::new();
    for i in 1..n_stops {
        let prev_val = &arr[3 + (i - 1) * 2 + 1];
        let this_val = &arr[3 + i * 2 + 1];
        if prev_val == this_val {
            to_remove.push(i);
        }
    }

    if to_remove.is_empty() {
        return false;
    }

    for &i in to_remove.iter().rev() {
        let zoom_idx = 3 + i * 2;
        arr.remove(zoom_idx + 1);
        arr.remove(zoom_idx);
    }
    true
}

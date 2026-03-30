//! Expression simplification: interpolate/step collapsing, match dedup, any→in, case/coalesce.

use serde_json::Value;

use super::util::{extract_json_literal, replace_arr_with_value};

/// Canonicalize `["exponential", 1]` → `["linear"]` in interpolate expressions.
#[expect(clippy::ptr_arg, reason = "to make trait happy")]
pub(super) fn try_canonicalize_interpolation_curve(arr: &mut Vec<Value>) -> bool {
    let Some(op) = arr[0].as_str() else {
        return false;
    };
    if !matches!(op, "interpolate" | "interpolate-hcl" | "interpolate-lab") {
        return false;
    }
    let Some(method) = arr.get(1).and_then(Value::as_array) else {
        return false;
    };
    if method.len() == 2
        && method[0].as_str() == Some("exponential")
        && method[1].as_f64() == Some(1.0)
    {
        arr[1] = Value::Array(vec![Value::String("linear".into())]);
        return true;
    }
    false
}

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
                replace_arr_with_value(arr, val);
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
                replace_arr_with_value(arr, val);
                return true;
            }
            let pairs_after_default = arr.len() - 3;
            if !pairs_after_default.is_multiple_of(2) {
                return false; // malformed
            }
            let default_val = &arr[2];
            if arr[4..].iter().step_by(2).all(|v| v == default_val) {
                let val = default_val.clone();
                replace_arr_with_value(arr, val);
                return true;
            }
            // Default == first stop output → first stop is redundant.
            if arr.len() >= 5 && &arr[2] == arr.last().unwrap() {
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

/// Rewrite boolean match expressions to `in` or `==`/`!=`.
///
/// - `["match", input, labels, true, false]` → `["in", input, ["literal", labels]]`
///   (single-element labels will be further simplified to `==` by `try_simplify_single_in`)
/// - `["match", input, [single], false, true]` → `["!=", input, single]`
pub(super) fn try_rewrite_boolean_match(arr: &mut Vec<Value>) -> bool {
    // Layout: ["match", input, label1, out1, ..., fallback] — must have exactly one arm.
    // One arm means: ["match", input, labels, output, fallback] = 5 elements.
    if arr.len() != 5 || arr[0].as_str() != Some("match") {
        return false;
    }

    let (is_true_false, is_false_true) = match (&arr[3], &arr[4]) {
        (Value::Bool(true), Value::Bool(false)) => (true, false),
        (Value::Bool(false), Value::Bool(true)) => (false, true),
        _ => return false,
    };

    let labels = match &arr[2] {
        Value::Array(labels) => labels.clone(),
        single => vec![single.clone()],
    };

    if is_true_false {
        // ["match", input, labels, true, false] → ["in", input, ["literal", labels]]
        let input = arr[1].take();
        let literal_arr = Value::Array(vec![
            Value::String("literal".to_string()),
            Value::Array(labels),
        ]);
        *arr = vec![Value::String("in".to_string()), input, literal_arr];
        true
    } else if is_false_true && labels.len() == 1 {
        // ["match", input, [single], false, true] → ["!=", input, single]
        let input = arr[1].take();
        let val = labels.into_iter().next().unwrap();
        *arr = vec![Value::String("!=".to_string()), input, val];
        true
    } else {
        false
    }
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

/// Flatten nested `case` expressions: when the fallback of a `case` is itself
/// another `case`, inline the inner arms into the outer expression.
///
/// `["case", c1, v1, ["case", c2, v2, fb]]` → `["case", c1, v1, c2, v2, fb]`
pub(super) fn try_flatten_case(arr: &mut Vec<Value>) -> bool {
    if arr.first().and_then(Value::as_str) != Some("case") {
        return false;
    }
    if arr.len() < 4 || !arr.len().is_multiple_of(2) {
        return false;
    }

    let is_nested_case = arr.last().unwrap().as_array().is_some_and(|inner| {
        inner.first().and_then(Value::as_str) == Some("case")
            && inner.len() >= 4
            && inner.len().is_multiple_of(2)
    });
    if !is_nested_case {
        return false;
    }

    let Value::Array(inner_arr) = arr.pop().unwrap() else {
        unreachable!("checked above");
    };
    // Skip inner_arr[0] ("case" keyword), append arms + fallback.
    arr.extend(inner_arr.into_iter().skip(1));
    true
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

/// Rewrite `case` to `match` when all arms test `["==", same_expr, literal]`.
///
/// `match` uses hash-based O(1) dispatch vs `case`'s sequential O(n) evaluation,
/// providing both a size reduction and runtime speedup.
///
/// Requires at least 2 arms, and all labels must be strings or numbers (not
/// booleans or null, which `match` doesn't accept per the spec).
pub(super) fn try_rewrite_case_to_match(arr: &mut Vec<Value>) -> bool {
    if arr.first().and_then(Value::as_str) != Some("case") {
        return false;
    }
    // Need at least 2 arms: ["case", c1, v1, c2, v2, fallback] → len >= 6, even
    if arr.len() < 6 || !arr.len().is_multiple_of(2) {
        return false;
    }

    let n_arms = (arr.len() - 2) / 2;
    let conditions: Vec<Value> = (0..n_arms).map(|i| arr[1 + 2 * i].clone()).collect();

    let Some((common_expr, labels)) = extract_eq_chain(&conditions) else {
        return false;
    };

    // match labels must be string or number — not bool or null.
    if labels
        .iter()
        .any(|l| !matches!(l, Value::String(_) | Value::Number(_)))
    {
        return false;
    }

    let fallback = arr.last().unwrap().clone();
    let outputs: Vec<Value> = (0..n_arms).map(|i| arr[2 + 2 * i].clone()).collect();

    let mut result = Vec::with_capacity(2 + 2 * n_arms + 1);
    result.push(Value::String("match".to_string()));
    result.push(common_expr);
    for (label, output) in labels.into_iter().zip(outputs) {
        result.push(label);
        result.push(output);
    }
    result.push(fallback);

    *arr = result;
    true
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

/// Replace all occurrences of `target` (by structural equality) with `replacement` in a value tree.
pub(super) fn substitute_expr(v: &Value, target: &Value, replacement: &Value) -> Value {
    if v == target {
        return replacement.clone();
    }
    match v {
        Value::Array(arr) => Value::Array(
            arr.iter()
                .map(|child| substitute_expr(child, target, replacement))
                .collect(),
        ),
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(k, child)| (k.clone(), substitute_expr(child, target, replacement)))
                .collect(),
        ),
        other => other.clone(),
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

// ── Filter-to-property constraint application ────────────────────────────────

use maplibre_style_spec::spec::{self, Boolean, ExprOrLiteral};

/// Serialize an `ExprOrLiteral` to a JSON `Value`.
fn expr_to_value(e: &ExprOrLiteral) -> Value {
    serde_json::to_value(e).unwrap_or(Value::Null)
}

/// Apply a typed `Boolean` constraint to a JSON property value.
/// Returns `true` if the property was modified.
pub(super) fn apply_filter_constraint(value: &mut Value, constraint: &Boolean) -> bool {
    match constraint {
        Boolean::EqualEqual(lhs, rhs, None) => {
            let (target, lit) = if super::fold::is_literal(rhs) && !super::fold::is_literal(lhs) {
                (expr_to_value(lhs), expr_to_value(rhs))
            } else if super::fold::is_literal(lhs) && !super::fold::is_literal(rhs) {
                (expr_to_value(rhs), expr_to_value(lhs))
            } else {
                return false;
            };
            let substituted = substitute_expr(value, &target, &lit);
            if substituted != *value {
                *value = substituted;
                return true;
            }
            false
        }
        Boolean::Has(prop, None) => {
            let spec::String::Literal(s) = prop.as_ref() else {
                return false;
            };
            apply_has_constraint(value, s.as_str())
        }
        Boolean::In(needle, haystack) => {
            let domain = match haystack {
                ExprOrLiteral::ArrayExpr(arr) => {
                    if let spec::Array::Literal(lit) = arr.as_ref() {
                        &lit.0
                    } else {
                        return false;
                    }
                }
                ExprOrLiteral::JSONArrayLiteral(lit) => &lit.0,
                _ => return false,
            };
            let target = expr_to_value(needle);
            apply_domain_constraint(value, &target, domain)
        }
        Boolean::Less(lhs, rhs, None) => apply_typed_range(value, lhs, rhs, "<"),
        Boolean::LessEqual(lhs, rhs, None) => apply_typed_range(value, lhs, rhs, "<="),
        Boolean::Greater(lhs, rhs, None) => apply_typed_range(value, lhs, rhs, ">"),
        Boolean::GreaterEqual(lhs, rhs, None) => apply_typed_range(value, lhs, rhs, ">="),
        _ => false,
    }
}

/// Helper: normalise a typed range constraint to `[op, expr, lit]` form and apply.
fn apply_typed_range(
    value: &mut Value,
    lhs: &ExprOrLiteral,
    rhs: &ExprOrLiteral,
    op: &str,
) -> bool {
    let lhs_is_lit = super::fold::is_literal(lhs);
    let rhs_is_lit = super::fold::is_literal(rhs);
    let (target, bound, norm_op) = if rhs_is_lit && !lhs_is_lit {
        (expr_to_value(lhs), expr_to_value(rhs), op)
    } else if lhs_is_lit && !rhs_is_lit {
        // Commute: `[op, lit, expr]` → `[flipped_op, expr, lit]`.
        let flipped = match op {
            "<" => ">",
            "<=" => ">=",
            ">" => "<",
            ">=" => "<=",
            _ => return false,
        };
        (expr_to_value(rhs), expr_to_value(lhs), flipped)
    } else {
        return false;
    };
    apply_range_constraint(value, &target, norm_op, &bound)
}

/// `has` constraint: if a property is guaranteed to exist, `coalesce` with that
/// property as an argument can be truncated at that argument (it won't be null).
fn apply_has_constraint(v: &mut Value, property: &str) -> bool {
    let Value::Array(arr) = v else {
        return false;
    };
    // At coalesce nodes, truncate after the guaranteed-present `["get", property]`.
    if arr.first().and_then(Value::as_str) == Some("coalesce") {
        let get_target = Value::Array(vec![
            Value::String("get".to_string()),
            Value::String(property.to_string()),
        ]);
        if let Some(pos) = arr[1..].iter().position(|arg| *arg == get_target) {
            let idx = pos + 1; // offset for the "coalesce" keyword
            if idx + 1 < arr.len() {
                // Truncate everything after this arg — it's guaranteed non-null.
                arr.truncate(idx + 1);
                return true;
            }
        }
    }
    // Recurse into children.
    let mut changed = false;
    for child in arr.iter_mut() {
        changed |= apply_has_constraint(child, property);
    }
    changed
}

/// Domain (`in`) constraint: prune `match` arms whose labels are outside the domain.
fn apply_domain_constraint(v: &mut Value, target: &Value, domain: &[Value]) -> bool {
    let Value::Array(arr) = v else {
        return false;
    };
    if arr.first().and_then(Value::as_str) == Some("match")
        && arr.len() >= 5
        && !arr.len().is_multiple_of(2)
        && arr[1] == *target
    {
        return prune_match_arms_by_domain(arr, domain);
    }
    // Recurse into children.
    let mut changed = false;
    for child in arr.iter_mut() {
        changed |= apply_domain_constraint(child, target, domain);
    }
    changed
}

/// Remove match arms whose labels have no intersection with the domain.
/// Shrink grouped label arrays to only domain members.
fn prune_match_arms_by_domain(arr: &mut Vec<Value>, domain: &[Value]) -> bool {
    // ["match", input, label1, out1, ..., fallback]
    let input = arr[1].clone();
    let fallback = arr.last().unwrap().clone();
    let arm_count = (arr.len() - 3) / 2;

    let mut new_arr = vec![Value::String("match".to_string()), input];
    let mut any_pruned = false;

    for i in 0..arm_count {
        let label_val = &arr[2 + i * 2];
        let output = &arr[3 + i * 2];

        let labels: Vec<&Value> = match label_val {
            Value::Array(labels) => labels.iter().collect(),
            single => vec![single],
        };

        let kept: Vec<Value> = labels
            .into_iter()
            .filter(|l| domain.contains(l))
            .cloned()
            .collect();

        if kept.is_empty() {
            any_pruned = true;
            continue;
        }
        if kept.len()
            < match label_val {
                Value::Array(a) => a.len(),
                _ => 1,
            }
        {
            any_pruned = true;
        }

        let label = if kept.len() == 1 {
            kept.into_iter().next().unwrap()
        } else {
            Value::Array(kept)
        };
        new_arr.push(label);
        new_arr.push(output.clone());
    }

    if !any_pruned {
        return false;
    }

    new_arr.push(fallback);

    // All arms pruned → collapse to fallback.
    if new_arr.len() == 3 {
        *arr = vec![Value::String("literal".to_string()), new_arr.remove(2)];
    } else {
        *arr = new_arr;
    }
    true
}

/// Range constraint: fold comparisons that are implied true/false by the filter's range.
fn apply_range_constraint(
    v: &mut Value,
    filter_target: &Value,
    filter_op: &str,
    filter_bound: &Value,
) -> bool {
    let Value::Array(arr) = v else {
        return false;
    };

    if let Some(result) = try_fold_range_comparison(arr, filter_target, filter_op, filter_bound) {
        replace_arr_with_value(arr, Value::Bool(result));
        return true;
    }

    // Recurse into children.
    let mut changed = false;
    for child in arr.iter_mut() {
        changed |= apply_range_constraint(child, filter_target, filter_op, filter_bound);
    }
    changed
}

/// Check if a comparison node `[cmp_op, lhs, rhs]` is implied true or false
/// by the filter constraint `filter_op(filter_target, filter_bound)`.
///
/// Returns `Some(true)` / `Some(false)` when the implication is certain, `None` otherwise.
fn try_fold_range_comparison(
    arr: &[Value],
    filter_target: &Value,
    filter_op: &str,
    filter_bound: &Value,
) -> Option<bool> {
    use std::cmp::Ordering::{Equal, Greater, Less};

    use super::util::compare_json_values;

    let prop_op = arr.first().and_then(Value::as_str)?;
    if !matches!(prop_op, "<" | "<=" | ">" | ">=") || arr.len() != 3 {
        return None;
    }

    // Determine which side is the target and which is the property's bound.
    let (prop_op_norm, prop_bound) = if arr[1] == *filter_target {
        // [op, target, lit] — already normalised.
        (prop_op, &arr[2])
    } else if arr[2] == *filter_target {
        // [op, lit, target] — commute.
        let flipped = match prop_op {
            "<" => ">",
            "<=" => ">=",
            ">" => "<",
            ">=" => "<=",
            _ => return None,
        };
        (flipped, &arr[1])
    } else {
        return None;
    };

    // Both bounds must be numeric for comparison.
    filter_bound.as_f64().filter(|f| f.is_finite())?;
    prop_bound.as_f64().filter(|f| f.is_finite())?;

    // ordering = compare(F, P): Greater means F > P, Less means F < P.
    let ordering = compare_json_values(filter_bound, prop_bound)?;

    // Filter guarantees: target `filter_op` F.
    // Property tests:    target `prop_op_norm` P.
    // We ask: does the filter imply the property is always true or always false?
    match (filter_op, prop_op_norm) {
        // Filter: target >= F
        (">=", ">=") if matches!(ordering, Greater | Equal) => Some(true), // F >= P ⇒ target >= P
        (">=", ">") if matches!(ordering, Greater) => Some(true),          // F > P ⇒ target > P
        (">=", "<") if matches!(ordering, Greater | Equal) => Some(false), // F >= P ⇒ ¬(target < P)
        (">=", "<=") if matches!(ordering, Greater) => Some(false),        // F > P ⇒ ¬(target <= P)

        // Filter: target > F
        (">", ">=") if matches!(ordering, Greater | Equal) => Some(true), // F >= P ⇒ target >= P
        (">", ">") if matches!(ordering, Greater | Equal) => Some(true),  // F >= P ⇒ target > P
        (">", "<") if matches!(ordering, Greater | Equal) => Some(false), // F >= P ⇒ ¬(target < P)
        (">", "<=") if matches!(ordering, Greater | Equal) => Some(false), // F >= P ⇒ ¬(target <= P)

        // Filter: target <= F
        ("<=", "<=") if matches!(ordering, Less | Equal) => Some(true), // F <= P ⇒ target <= P
        ("<=", "<") if matches!(ordering, Less) => Some(true),          // F < P ⇒ target < P
        ("<=", ">") if matches!(ordering, Less | Equal) => Some(false), // F <= P ⇒ ¬(target > P)
        ("<=", ">=") if matches!(ordering, Less) => Some(false),        // F < P ⇒ ¬(target >= P)

        // Filter: target < F
        ("<", "<=") if matches!(ordering, Less | Equal) => Some(true), // F <= P ⇒ target <= P
        ("<", "<") if matches!(ordering, Less | Equal) => Some(true),  // F <= P ⇒ target < P
        ("<", ">") if matches!(ordering, Less | Equal) => Some(false), // F <= P ⇒ ¬(target > P)
        ("<", ">=") if matches!(ordering, Less | Equal) => Some(false), // F <= P ⇒ ¬(target >= P)

        _ => None,
    }
}

// ── Typed Boolean rules ──────────────────────────────────────────────────────

/// De Morgan's law on typed `Boolean`:
/// `Not(All([A, B, ...]))` → `Any([Not(A), Not(B), ...])`
/// `Not(Any([A, B, ...]))` → `All([Not(A), Not(B), ...])`
///
/// Only applied when there are ≥2 children (otherwise unary unwrap handles it).
pub(super) fn try_demorgan_typed(filter: &mut Boolean) -> bool {
    let Boolean::Not(_) = filter else {
        return false;
    };
    // Take ownership to avoid cloning children.
    let Boolean::Not(inner) = std::mem::replace(filter, Boolean::Literal(false)) else {
        unreachable!();
    };
    match *inner {
        Boolean::All(children) if children.len() >= 2 => {
            let negated = children
                .into_iter()
                .map(|c| Boolean::Not(Box::new(c)))
                .collect();
            *filter = Boolean::Any(negated);
            true
        }
        Boolean::Any(children) if children.len() >= 2 => {
            let negated = children
                .into_iter()
                .map(|c| Boolean::Not(Box::new(c)))
                .collect();
            *filter = Boolean::All(negated);
            true
        }
        other => {
            // Not applicable — put it back.
            *filter = Boolean::Not(Box::new(other));
            false
        }
    }
}

/// Flatten nested boolean operators:
/// `All([All([A, B]), C])` → `All([A, B, C])`, same for `Any`.
pub(super) fn try_boolean_flattening_typed(filter: &mut Boolean) -> bool {
    match filter {
        Boolean::All(children) => flatten_typed(children, true),
        Boolean::Any(children) => flatten_typed(children, false),
        _ => false,
    }
}

fn flatten_typed(children: &mut Vec<Boolean>, is_all: bool) -> bool {
    let needs_flatten = children.iter().any(|c| match c {
        Boolean::All(_) if is_all => true,
        Boolean::Any(_) if !is_all => true,
        _ => false,
    });
    if !needs_flatten {
        return false;
    }
    let old = std::mem::take(children);
    for child in old {
        match child {
            Boolean::All(grandchildren) if is_all => children.extend(grandchildren),
            Boolean::Any(grandchildren) if !is_all => children.extend(grandchildren),
            other => children.push(other),
        }
    }
    true
}

/// Rewrite `Any([EqualEqual(get, a), EqualEqual(get, b), ...])` → `In(get, literal([a, b, ...]))`.
///
/// Only when all children are `EqualEqual` with the same non-literal first operand.
pub(super) fn try_rewrite_any_to_in_typed(filter: &mut Boolean) -> bool {
    let Boolean::Any(children) = filter else {
        return false;
    };
    if children.len() < 2 {
        return false;
    }

    let mut common_expr: Option<&ExprOrLiteral> = None;
    let mut values: Vec<ExprOrLiteral> = Vec::with_capacity(children.len());

    for child in children.iter() {
        let Boolean::EqualEqual(lhs, rhs, None) = child else {
            return false;
        };
        // One side must be a literal, the other the common expression.
        let (expr, val) = if super::fold::is_literal(rhs) && !super::fold::is_literal(lhs) {
            (lhs, rhs)
        } else if super::fold::is_literal(lhs) && !super::fold::is_literal(rhs) {
            (rhs, lhs)
        } else {
            return false;
        };
        match common_expr {
            None => common_expr = Some(expr),
            Some(prev) if prev == expr => {}
            _ => return false,
        }
        values.push(val.clone());
    }

    let Some(expr) = common_expr else {
        return false;
    };
    let literal_array = ExprOrLiteral::JSONArrayLiteral(spec::JSONArrayLiteral(
        values.into_iter().map(expr_or_literal_to_json).collect(),
    ));
    *filter = Boolean::In(expr.clone(), literal_array);
    true
}

/// Convert a known-literal `ExprOrLiteral` to a `serde_json::Value` without serde.
fn expr_or_literal_to_json(v: ExprOrLiteral) -> Value {
    match v {
        ExprOrLiteral::Null => Value::Null,
        ExprOrLiteral::Bool(b) => Value::Bool(b),
        ExprOrLiteral::NumberLiteral(n) => Value::Number(n.as_number().clone()),
        ExprOrLiteral::StringLiteral(s) => Value::String(s.as_str().to_owned()),
        // Fallback to serde for complex types (shouldn't happen for literals).
        other => serde_json::to_value(&other).unwrap_or(Value::Null),
    }
}

/// Unwrap unary `All`/`Any` and eliminate double negation:
/// `All([x])` → `x`, `Any([x])` → `x`, `Not(Not(x))` → `x`.
pub(super) fn try_simplify_unary_typed(filter: &mut Boolean) -> bool {
    match filter {
        Boolean::All(children) | Boolean::Any(children) if children.len() == 1 => {
            *filter = children.pop().unwrap();
            true
        }
        Boolean::Not(inner) if matches!(inner.as_ref(), Boolean::Not(_)) => {
            // Take ownership to avoid cloning: Not(Not(x)) → x.
            let Boolean::Not(inner) = std::mem::replace(filter, Boolean::Literal(false)) else {
                unreachable!();
            };
            let Boolean::Not(grand) = *inner else {
                unreachable!();
            };
            *filter = *grand;
            true
        }
        _ => false,
    }
}

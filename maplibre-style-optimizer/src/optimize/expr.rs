//! Expression-tree passes: normalisation, constant folding, selectivity reordering.

use maplibre_style_spec::mir::IntermediateSpec;
use serde_json::Value;

use super::OptPasses;
use super::walk::{PropertyContext, StyleVisitor};

// ── Visitors ──────────────────────────────────────────────────────────────────

pub(crate) struct NormalizeFoldVisitor<'a> {
    pub mir: &'a IntermediateSpec,
    pub passes: &'a OptPasses,
    pub changed: bool,
}

impl StyleVisitor for NormalizeFoldVisitor<'_> {
    fn visit_filter(&mut self, _: usize, _: &str, filter: &mut Value) {
        normalize_and_fold(filter, self.mir, self.passes, &mut self.changed);
    }

    fn visit_property(&mut self, _: &PropertyContext<'_>, value: &mut Value) {
        normalize_and_fold(value, self.mir, self.passes, &mut self.changed);
    }
}

pub(crate) struct ReorderSelectivityVisitor<'a> {
    pub mir: &'a IntermediateSpec,
}

impl StyleVisitor for ReorderSelectivityVisitor<'_> {
    fn visit_filter(&mut self, _: usize, _: &str, filter: &mut Value) {
        reorder_selectivity(filter, self.mir);
    }

    fn visit_property(&mut self, _: &PropertyContext<'_>, value: &mut Value) {
        reorder_selectivity(value, self.mir);
    }
}

// ── Recursive walkers ─────────────────────────────────────────────────────────

fn normalize_and_fold(
    v: &mut Value,
    mir: &IntermediateSpec,
    passes: &OptPasses,
    changed: &mut bool,
) {
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
            if passes.simplify_unary && arr.len() == 2 {
                match arr.first().and_then(Value::as_str) {
                    Some("any" | "all") => {
                        let inner = arr[1].take();
                        *v = inner;
                        *changed = true;
                        normalize_and_fold(v, mir, passes, changed);
                    }
                    Some("!") => {
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

fn reorder_selectivity(v: &mut Value, mir: &IntermediateSpec) {
    match v {
        Value::Array(arr) => {
            for x in arr.iter_mut() {
                reorder_selectivity(x, mir);
            }
            maybe_reorder_any_all(arr, mir);
        }
        Value::Object(map) => {
            for x in map.values_mut() {
                reorder_selectivity(x, mir);
            }
        }
        _ => {}
    }
}

// ── Per-node rewriting ────────────────────────────────────────────────────────

fn rewrite_expression_array(
    arr: &mut Vec<Value>,
    mir: &IntermediateSpec,
    passes: &OptPasses,
    changed: &mut bool,
) {
    while apply_one_rewrite_pass(arr, mir, passes) {
        *changed = true;
    }
}

fn apply_one_rewrite_pass(
    arr: &mut Vec<Value>,
    mir: &IntermediateSpec,
    passes: &OptPasses,
) -> bool {
    if passes.expression_kind && try_negate_comparison(arr, mir) {
        return true;
    }
    if passes.constant_fold {
        if try_fold_boolean_algebra(arr) {
            return true;
        }
        if try_fold_not(arr) {
            return true;
        }
        if try_fold_comparison(arr) {
            return true;
        }
    }
    false
}

/// `["!", [op, a, b]]` → `[negation_of(op), a, b]` when the negated operator exists in MIR.
///
/// Handles `==`↔`!=`, `<`↔`>=`, `>`↔`<=` generically via `IntermediateExpressions::negation_of`.
fn try_negate_comparison(arr: &mut Vec<Value>, mir: &IntermediateSpec) -> bool {
    if arr.len() != 2 || arr[0].as_str() != Some("!") {
        return false;
    }
    let Value::Array(inner) = &arr[1] else {
        return false;
    };
    if inner.len() != 3 {
        return false;
    }
    let Some(inner_op) = inner[0].as_str() else {
        return false;
    };
    let Some(negated) = mir.expressions.negation_of(inner_op) else {
        return false;
    };
    let a = inner[1].clone();
    let b = inner[2].clone();
    *arr = vec![Value::String(negated.to_string()), a, b];
    true
}

pub(crate) fn bool_literal(v: &Value) -> Option<bool> {
    match v {
        Value::Bool(b) => Some(*b),
        Value::Array(a) if a.len() == 2 && a[0].as_str() == Some("literal") => a[1].as_bool(),
        _ => None,
    }
}

pub(crate) fn extract_json_literal(v: &Value) -> Option<Value> {
    match v {
        Value::Number(_) | Value::String(_) | Value::Bool(_) | Value::Null => Some(v.clone()),
        Value::Array(a) if a.len() == 2 && a[0].as_str() == Some("literal") => Some(a[1].clone()),
        _ => None,
    }
}

fn try_fold_not(arr: &mut Vec<Value>) -> bool {
    if arr.len() != 2 || arr.first().and_then(Value::as_str) != Some("!") {
        return false;
    }
    if let Some(b) = bool_literal(&arr[1]) {
        *arr = vec![Value::String("literal".to_string()), Value::Bool(!b)];
        return true;
    }
    false
}

fn try_fold_comparison(arr: &mut Vec<Value>) -> bool {
    if arr.len() != 3 {
        return false;
    }
    let Some(op @ ("==" | "!=" | "<" | "<=" | ">" | ">=")) = arr[0].as_str() else {
        return false;
    };
    let Some(x) = extract_json_literal(&arr[1]) else {
        return false;
    };
    let Some(y) = extract_json_literal(&arr[2]) else {
        return false;
    };
    let Some(ord) = compare_json_values(&x, &y) else {
        return false;
    };
    let result = match op {
        "==" => ord == std::cmp::Ordering::Equal,
        "!=" => ord != std::cmp::Ordering::Equal,
        "<" => ord == std::cmp::Ordering::Less,
        "<=" => ord != std::cmp::Ordering::Greater,
        ">" => ord == std::cmp::Ordering::Greater,
        ">=" => ord != std::cmp::Ordering::Less,
        _ => return false,
    };
    *arr = vec![Value::String("literal".to_string()), Value::Bool(result)];
    true
}

fn compare_json_values(x: &Value, y: &Value) -> Option<std::cmp::Ordering> {
    match (x, y) {
        (Value::Number(a), Value::Number(b)) => {
            let af = a.as_f64()?;
            let bf = b.as_f64()?;
            af.partial_cmp(&bf)
        }
        (Value::String(a), Value::String(b)) => Some(a.cmp(b)),
        (Value::Bool(a), Value::Bool(b)) => Some(a.cmp(b)),
        (Value::Null, Value::Null) => Some(std::cmp::Ordering::Equal),
        _ => None,
    }
}

fn try_fold_boolean_algebra(arr: &mut Vec<Value>) -> bool {
    let op = match arr.first().and_then(Value::as_str) {
        Some("any") => "any",
        Some("all") => "all",
        _ => return false,
    };
    if arr.len() < 2 {
        return false;
    }
    let mut kept: Vec<Value> = Vec::new();
    let mut saw_true = false;
    let mut saw_false = false;
    for x in arr.iter().skip(1) {
        match bool_literal(x) {
            Some(true) => {
                saw_true = true;
                if op == "any" {
                    *arr = vec![Value::String("literal".to_string()), Value::Bool(true)];
                    return true;
                }
            }
            Some(false) => {
                saw_false = true;
                if op == "all" {
                    *arr = vec![Value::String("literal".to_string()), Value::Bool(false)];
                    return true;
                }
            }
            None => kept.push(x.clone()),
        }
    }
    if op == "any" && saw_false && kept.is_empty() && !saw_true {
        *arr = vec![Value::String("literal".to_string()), Value::Bool(false)];
        return true;
    }
    if op == "all" && saw_true && kept.is_empty() && !saw_false {
        *arr = vec![Value::String("literal".to_string()), Value::Bool(true)];
        return true;
    }
    let new_len = 1 + kept.len();
    if new_len != arr.len() {
        let mut out = vec![Value::String(op.to_string())];
        out.extend(kept);
        *arr = out;
        return true;
    }
    false
}

fn maybe_reorder_any_all(arr: &mut Vec<Value>, mir: &IntermediateSpec) {
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
    let mut out = vec![head];
    out.extend(ops);
    *arr = out;
}

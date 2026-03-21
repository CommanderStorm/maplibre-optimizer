//! Expression-tree passes: normalisation, constant folding, selectivity reordering.

use maplibre_style_spec::mir::{ExpressionOperator, IntermediateSpec};
use serde_json::Value;

use super::OptPasses;
use super::selectivity::estimate_selectivity;
use super::source_util::VectorLayerInfo;
use super::walk::{PropertyContext, StyleVisitor};
use crate::stats::TileStatistics;

// ── Visitors ──────────────────────────────────────────────────────────────────

pub(crate) struct NormalizeFoldVisitor<'a> {
    pub mir: &'a IntermediateSpec,
    pub passes: &'a OptPasses,
    pub stats: Option<&'a TileStatistics>,
    pub layer_info: Option<&'a [Option<VectorLayerInfo>]>,
    pub changed: bool,
}

impl StyleVisitor for NormalizeFoldVisitor<'_> {
    fn visit_filter(&mut self, layer_index: usize, _: &str, filter: &mut Value) {
        // Stats-driven: fold ["id"] → ["literal", null] when no feature IDs.
        if self.passes.constant_fold
            && should_fold_id(self.stats, self.layer_info, layer_index)
            && fold_id_to_null(filter)
        {
            self.changed = true;
        }
        normalize_and_fold(filter, self.mir, self.passes, &mut self.changed);
    }

    fn visit_property(&mut self, _: &PropertyContext<'_>, value: &mut Value) {
        normalize_and_fold(value, self.mir, self.passes, &mut self.changed);
    }
}

fn should_fold_id(
    stats: Option<&TileStatistics>,
    layer_info: Option<&[Option<VectorLayerInfo>]>,
    layer_index: usize,
) -> bool {
    let Some(stats) = stats else { return false };
    let Some(infos) = layer_info else {
        return false;
    };
    let Some(Some(info)) = infos.get(layer_index) else {
        return false;
    };
    stats
        .layer_stats(&info.source, &info.source_layer)
        .is_some_and(|ls| !ls.has_feature_ids)
}

/// Recursively replace `["id"]` with `["literal", null]`.
fn fold_id_to_null(v: &mut Value) -> bool {
    let Value::Array(arr) = v else {
        return false;
    };
    if arr.len() == 1 && arr[0].as_str() == Some("id") {
        *arr = vec![Value::String("literal".to_string()), Value::Null];
        return true;
    }
    let mut changed = false;
    for child in arr.iter_mut() {
        changed |= fold_id_to_null(child);
    }
    changed
}

pub(crate) struct ReorderSelectivityVisitor<'a> {
    pub mir: &'a IntermediateSpec,
    pub stats: Option<&'a TileStatistics>,
    pub layer_info: Option<&'a [Option<VectorLayerInfo>]>,
}

struct LayerContext<'a> {
    source: &'a str,
    source_layer: &'a str,
}

impl ReorderSelectivityVisitor<'_> {
    fn layer_context(&self, layer_index: usize) -> Option<LayerContext<'_>> {
        let stats = self.stats?;
        let info = self.layer_info?.get(layer_index)?.as_ref()?;
        // Verify stats exist for this layer.
        stats.layer_stats(&info.source, &info.source_layer)?;
        Some(LayerContext {
            source: &info.source,
            source_layer: &info.source_layer,
        })
    }
}

impl StyleVisitor for ReorderSelectivityVisitor<'_> {
    fn visit_filter(&mut self, layer_index: usize, _: &str, filter: &mut Value) {
        let ctx = self.layer_context(layer_index);
        reorder_selectivity(filter, self.mir, self.stats, ctx.as_ref());
    }

    fn visit_property(&mut self, ctx: &PropertyContext<'_>, value: &mut Value) {
        let lctx = self.layer_context(ctx.layer_index);
        reorder_selectivity(value, self.mir, self.stats, lctx.as_ref());
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

fn reorder_selectivity(
    v: &mut Value,
    mir: &IntermediateSpec,
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
        if try_fold_pure_operator(arr, mir) {
            return true;
        }
        if try_algebraic_simplify(arr) {
            return true;
        }
        if try_dead_branch_case(arr) {
            return true;
        }
        if try_dead_branch_match_literal(arr) {
            return true;
        }
        if try_filter_contradiction(arr) {
            return true;
        }
    }
    if passes.simplify_expressions {
        if try_simplify_interpolate_or_step(arr) {
            return true;
        }
        if try_simplify_match(arr) {
            return true;
        }
        if try_rewrite_any_to_in(arr) {
            return true;
        }
        if try_simplify_case(arr) {
            return true;
        }
        if try_simplify_coalesce(arr) {
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

/// Returns `true` if `v` is the numeric literal `expected` (bare number or `["literal", n]`).
#[allow(clippy::float_cmp)]
fn is_num(v: &Value, expected: f64) -> bool {
    match v {
        Value::Number(n) => n.as_f64().is_some_and(|f| f == expected),
        Value::Array(a) if a.len() == 2 && a[0].as_str() == Some("literal") => {
            a[1].as_f64().is_some_and(|f| f == expected)
        }
        _ => false,
    }
}

/// Replace an expression array `arr` with a value `v`.
///
/// If `v` is a `Value::Array`, the array is used directly (it becomes the new expression).
/// If `v` is a scalar, it is wrapped in `["literal", v]`.
fn replace_arr_with_value(arr: &mut Vec<Value>, v: Value) {
    match v {
        Value::Array(inner) => *arr = inner,
        scalar => *arr = vec![Value::String("literal".to_string()), scalar],
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
    let result = if let Some(ord) = compare_json_values(&x, &y) {
        match op {
            "==" => ord == std::cmp::Ordering::Equal,
            "!=" => ord != std::cmp::Ordering::Equal,
            "<" => ord == std::cmp::Ordering::Less,
            "<=" => ord != std::cmp::Ordering::Greater,
            ">" => ord == std::cmp::Ordering::Greater,
            ">=" => ord != std::cmp::Ordering::Less,
            _ => return false,
        }
    } else {
        // Different types: == is false, != is true; relational ops can't be folded.
        match op {
            "==" => false,
            "!=" => true,
            _ => return false,
        }
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

// ── Pass 3: Arithmetic / String / Type / Color constant folding ────────────────

/// Try to evaluate a pure operator whose all arguments are literal values.
fn try_fold_pure_operator(arr: &mut Vec<Value>, mir: &IntermediateSpec) -> bool {
    if arr.is_empty() {
        return false;
    }
    let Some(op) = arr[0].as_str() else {
        return false;
    };

    // Purity check: either flagged pure in MIR, or one of the special foldable lookups.
    let pure_in_mir = mir
        .expressions
        .operators
        .get(op)
        .is_some_and(ExpressionOperator::is_pure);
    if !pure_in_mir && !matches!(op, "length" | "at") {
        return false;
    }

    // All arguments must be literals.
    let args: Option<Vec<Value>> = arr[1..].iter().map(extract_json_literal).collect();
    let Some(args) = args else {
        return false;
    };

    let Some(result) = evaluate_pure_operator(op, &args) else {
        return false;
    };

    // Replace the expression array with ["literal", result].
    *arr = vec![Value::String("literal".to_string()), result];
    true
}

#[allow(clippy::too_many_lines)]
fn evaluate_pure_operator(op: &str, args: &[Value]) -> Option<Value> {
    match op {
        // ── Math: unary ────────────────────────────────────────────────────────
        "abs" => finite(args.first()?.as_f64()?.abs()),
        "ceil" => finite(args.first()?.as_f64()?.ceil()),
        "floor" => finite(args.first()?.as_f64()?.floor()),
        "round" => finite(args.first()?.as_f64()?.round()),
        "sqrt" => {
            let n = args.first()?.as_f64()?;
            if n < 0.0 {
                return None;
            }
            finite(n.sqrt())
        }
        "ln" => {
            let n = args.first()?.as_f64()?;
            if n <= 0.0 {
                return None;
            }
            finite(n.ln())
        }
        "log2" => {
            let n = args.first()?.as_f64()?;
            if n <= 0.0 {
                return None;
            }
            finite(n.log2())
        }
        "log10" => {
            let n = args.first()?.as_f64()?;
            if n <= 0.0 {
                return None;
            }
            finite(n.log10())
        }
        "sin" => finite(args.first()?.as_f64()?.sin()),
        "cos" => finite(args.first()?.as_f64()?.cos()),
        "tan" => finite(args.first()?.as_f64()?.tan()),
        "asin" => {
            let n = args.first()?.as_f64()?;
            if !(-1.0..=1.0).contains(&n) {
                return None;
            }
            finite(n.asin())
        }
        "acos" => {
            let n = args.first()?.as_f64()?;
            if !(-1.0..=1.0).contains(&n) {
                return None;
            }
            finite(n.acos())
        }
        "atan" => finite(args.first()?.as_f64()?.atan()),

        // ── Math: nullary constants ────────────────────────────────────────────
        "e" if args.is_empty() => finite(std::f64::consts::E),
        "pi" if args.is_empty() => finite(std::f64::consts::PI),
        "ln2" if args.is_empty() => finite(std::f64::consts::LN_2),

        // ── Math: variadic ────────────────────────────────────────────────────
        "+" => {
            if args.is_empty() {
                return None;
            }
            let mut s = 0.0_f64;
            for a in args {
                s += a.as_f64()?;
            }
            finite(s)
        }
        "-" => match args.len() {
            1 => finite(-args[0].as_f64()?),
            2 => finite(args[0].as_f64()? - args[1].as_f64()?),
            _ => None,
        },
        "*" => {
            if args.is_empty() {
                return None;
            }
            let mut p = 1.0_f64;
            for a in args {
                p *= a.as_f64()?;
            }
            finite(p)
        }
        "/" => {
            if args.len() != 2 {
                return None;
            }
            let b = args[1].as_f64()?;
            if b == 0.0 {
                return None;
            }
            finite(args[0].as_f64()? / b)
        }
        "%" => {
            if args.len() != 2 {
                return None;
            }
            let b = args[1].as_f64()?;
            if b == 0.0 {
                return None;
            }
            finite(args[0].as_f64()? % b)
        }
        "^" => {
            if args.len() != 2 {
                return None;
            }
            finite(args[0].as_f64()?.powf(args[1].as_f64()?))
        }
        "min" => {
            if args.is_empty() {
                return None;
            }
            let mut m = f64::INFINITY;
            for a in args {
                m = m.min(a.as_f64()?);
            }
            finite(m)
        }
        "max" => {
            if args.is_empty() {
                return None;
            }
            let mut m = f64::NEG_INFINITY;
            for a in args {
                m = m.max(a.as_f64()?);
            }
            finite(m)
        }

        // ── String ────────────────────────────────────────────────────────────
        "concat" => {
            let mut s = String::new();
            for a in args {
                match a {
                    Value::String(t) => s.push_str(t),
                    Value::Number(n) => s.push_str(&n.to_string()),
                    Value::Bool(b) => s.push_str(if *b { "true" } else { "false" }),
                    Value::Null => {}
                    _ => return None,
                }
            }
            Some(Value::String(s))
        }
        "downcase" => Some(Value::String(args.first()?.as_str()?.to_lowercase())),
        "upcase" => Some(Value::String(args.first()?.as_str()?.to_uppercase())),

        // ── Type coercion ─────────────────────────────────────────────────────
        "to-number" => match args.first()? {
            Value::Number(n) => Some(Value::Number(n.clone())),
            Value::String(s) => s.parse::<f64>().ok().and_then(finite),
            Value::Bool(b) => finite(if *b { 1.0 } else { 0.0 }),
            Value::Null => finite(0.0),
            _ => None,
        },
        "to-string" => match args.first()? {
            Value::String(s) => Some(Value::String(s.clone())),
            Value::Number(n) => Some(Value::String(n.to_string())),
            Value::Bool(b) => Some(Value::String(b.to_string())),
            Value::Null => Some(Value::String(String::new())),
            _ => None,
        },
        "to-boolean" => {
            let b = match args.first()? {
                Value::Bool(b) => *b,
                Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
                Value::String(s) => !s.is_empty(),
                Value::Null => false,
                Value::Array(_) | Value::Object(_) => true,
            };
            Some(Value::Bool(b))
        }
        "typeof" => {
            let t = match args.first()? {
                Value::Bool(_) => "boolean",
                Value::Number(_) => "number",
                Value::String(_) => "string",
                Value::Null => "null",
                Value::Array(_) => "array",
                Value::Object(_) => "object",
            };
            Some(Value::String(t.to_string()))
        }

        // ── Color ─────────────────────────────────────────────────────────────
        "rgb" => {
            if args.len() != 3 {
                return None;
            }
            let r = clamp_channel(args[0].as_f64()?);
            let g = clamp_channel(args[1].as_f64()?);
            let b = clamp_channel(args[2].as_f64()?);
            Some(Value::String(format!("rgba({r},{g},{b},1)")))
        }
        "rgba" => {
            if args.len() != 4 {
                return None;
            }
            let r = clamp_channel(args[0].as_f64()?);
            let g = clamp_channel(args[1].as_f64()?);
            let b = clamp_channel(args[2].as_f64()?);
            let a = args[3].as_f64()?;
            if !(0.0..=1.0).contains(&a) {
                return None;
            }
            Some(Value::String(format!("rgba({r},{g},{b},{a})")))
        }

        // ── Lookup on literal arrays/strings ──────────────────────────────────
        "length" => match args.first()? {
            Value::String(s) => Some(Value::Number(s.chars().count().into())),
            Value::Array(a) => Some(Value::Number(a.len().into())),
            _ => None,
        },
        "at" => {
            if args.len() != 2 {
                return None;
            }
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let idx = args[0].as_f64()? as usize;
            args[1].as_array()?.get(idx).cloned()
        }

        _ => None,
    }
}

fn finite(n: f64) -> Option<Value> {
    if n.is_finite() {
        serde_json::Number::from_f64(n).map(Value::Number)
    } else {
        None
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn clamp_channel(v: f64) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

// ── Pass 2: Interpolate / Step simplification ─────────────────────────────────

/// Simplify `interpolate`/`interpolate-hcl`/`interpolate-lab` and `step` expressions
/// when all output values are structurally equal.
fn try_simplify_interpolate_or_step(arr: &mut Vec<Value>) -> bool {
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
            false
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
            false
        }
        _ => false,
    }
}

// ── Pass 4: Match arm deduplication ──────────────────────────────────────────

/// Merge `match` expression arms that produce the same output value.
///
/// - Multiple labels with same output → grouped label array.
/// - All arms (including fallback) produce same value → collapse to that value.
/// - Arms whose output equals the fallback → remove those arms.
fn try_simplify_match(arr: &mut Vec<Value>) -> bool {
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

// ── Algebraic identity simplification ─────────────────────────────────────────

/// Eliminate identity-element operands in binary arithmetic (e.g. `x * 1 → x`).
///
/// Complements `try_fold_pure_operator` which handles all-literal expressions;
/// this covers the mixed literal+variable case.
fn try_algebraic_simplify(arr: &mut Vec<Value>) -> bool {
    if arr.len() != 3 {
        return false;
    }
    let Some(op) = arr[0].as_str() else {
        return false;
    };
    match op {
        "*" => {
            if is_num(&arr[2], 1.0) {
                let x = arr[1].clone();
                replace_arr_with_value(arr, x);
                return true;
            }
            if is_num(&arr[1], 1.0) {
                let x = arr[2].clone();
                replace_arr_with_value(arr, x);
                return true;
            }
        }
        "+" => {
            if is_num(&arr[2], 0.0) {
                let x = arr[1].clone();
                replace_arr_with_value(arr, x);
                return true;
            }
            if is_num(&arr[1], 0.0) {
                let x = arr[2].clone();
                replace_arr_with_value(arr, x);
                return true;
            }
        }
        "-" => {
            if is_num(&arr[2], 0.0) {
                let x = arr[1].clone();
                replace_arr_with_value(arr, x);
                return true;
            }
        }
        "/" | "^" => {
            if is_num(&arr[2], 1.0) {
                let x = arr[1].clone();
                replace_arr_with_value(arr, x);
                return true;
            }
        }
        _ => {}
    }
    false
}

// ── Dead branch elimination: case ─────────────────────────────────────────────

/// Resolve `case` arms with known-at-compile-time boolean conditions.
fn try_dead_branch_case(arr: &mut Vec<Value>) -> bool {
    if arr.first().and_then(Value::as_str) != Some("case") {
        return false;
    }
    if arr.len() < 2 || !arr.len().is_multiple_of(2) {
        return false;
    }
    if arr.len() == 2 {
        let fallback = arr[1].clone();
        replace_arr_with_value(arr, fallback);
        return true;
    }
    let n_arms = (arr.len() - 2) / 2;
    for i in 0..n_arms {
        let cond_idx = 1 + 2 * i;
        let out_idx = 2 + 2 * i;
        match bool_literal(&arr[cond_idx]) {
            Some(true) => {
                let out = arr[out_idx].clone();
                replace_arr_with_value(arr, out);
                return true;
            }
            Some(false) => {
                // Higher index first so lower index stays valid.
                arr.remove(out_idx);
                arr.remove(cond_idx);
                return true;
            }
            None => {}
        }
    }
    false
}

// ── Dead branch elimination: match with literal input ─────────────────────────

/// Resolve `match` expressions when the input is a known literal value.
///
/// Scans arms for a matching label; replaces with the arm's output, or with the
/// fallback if no arm matches.
fn try_dead_branch_match_literal(arr: &mut Vec<Value>) -> bool {
    if arr.first().and_then(Value::as_str) != Some("match") {
        return false;
    }
    if arr.len() < 5 || arr.len().is_multiple_of(2) {
        return false;
    }
    let Some(input_lit) = extract_json_literal(&arr[1]) else {
        return false;
    };
    let fallback = arr.last().unwrap().clone();
    let arm_count = (arr.len() - 3) / 2;
    for i in 0..arm_count {
        let label_val = &arr[2 + i * 2];
        let output = arr[3 + i * 2].clone();
        let matched = match label_val {
            Value::Array(labels) => labels.contains(&input_lit),
            single => *single == input_lit,
        };
        if matched {
            replace_arr_with_value(arr, output);
            return true;
        }
    }
    replace_arr_with_value(arr, fallback);
    true
}

// ── Filter contradiction detection ────────────────────────────────────────────

/// Detect contradictory `==`/`!=` predicates inside `["all", ...]` and fold to false.
fn try_filter_contradiction(arr: &mut Vec<Value>) -> bool {
    if arr.first().and_then(Value::as_str) != Some("all") {
        return false;
    }
    let predicates: Vec<&Value> = arr.iter().skip(1).collect();
    for i in 0..predicates.len() {
        for j in (i + 1)..predicates.len() {
            if predicates_contradict(predicates[i], predicates[j]) {
                *arr = vec![Value::String("literal".to_string()), Value::Bool(false)];
                return true;
            }
        }
    }
    false
}

fn predicates_contradict(a: &Value, b: &Value) -> bool {
    let (Some((a_op, a_lhs, a_rhs)), Some((b_op, b_lhs, b_rhs))) =
        (extract_eq_predicate(a), extract_eq_predicate(b))
    else {
        return false;
    };
    if a_lhs != b_lhs {
        return false;
    }
    if a_op == "==" && b_op == "==" && a_rhs != b_rhs {
        return true;
    }
    if ((a_op == "==" && b_op == "!=") || (a_op == "!=" && b_op == "==")) && a_rhs == b_rhs {
        return true;
    }
    false
}

/// Extracts `(op, lhs_expr, rhs_literal)` from `["=="|"!=", expr, lit]` or `["==", lit, expr]`.
fn extract_eq_predicate(v: &Value) -> Option<(String, Value, Value)> {
    let Value::Array(arr) = v else {
        return None;
    };
    if arr.len() != 3 {
        return None;
    }
    let op = arr[0].as_str()?.to_string();
    if !matches!(op.as_str(), "==" | "!=") {
        return None;
    }
    // ["op", expr, literal]
    if let Some(lit) = extract_json_literal(&arr[2]) {
        return Some((op, arr[1].clone(), lit));
    }
    // ["==", literal, expr] — commutative form (not valid for !=)
    if op == "=="
        && let Some(lit) = extract_json_literal(&arr[1])
    {
        return Some((op, arr[2].clone(), lit));
    }
    None
}

// ── any → in rewriting ────────────────────────────────────────────────────────

/// Rewrite `["any", ["==", x, a], ["==", x, b], ...]` → `["in", x, ["literal", [a, b, ...]]]`.
///
/// Only applies when every predicate is `["==", same_expr, literal]` (or the commuted form).
/// Requires at least two predicates.
fn try_rewrite_any_to_in(arr: &mut Vec<Value>) -> bool {
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

// ── case simplification ───────────────────────────────────────────────────────

/// Simplify `case` expressions by removing redundant trailing arms.
///
/// - All arms + fallback produce the same value → collapse to that value.
/// - Trailing arms whose output equals the fallback → remove them (they'd return
///   the fallback anyway, so they add no value).
///
/// Note: only trailing arms can be safely removed. Removing a middle arm would
/// change which subsequent condition is evaluated first.
fn try_simplify_case(arr: &mut Vec<Value>) -> bool {
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

// ── coalesce simplification ───────────────────────────────────────────────────

/// Simplify `coalesce` expressions:
///
/// - `["coalesce", x]` → `x` (single arg)
/// - Null literal args are removed (they always pass through to the next arg).
/// - After a non-null literal arg, all subsequent args are unreachable → truncate.
fn try_simplify_coalesce(arr: &mut Vec<Value>) -> bool {
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

// ── Selectivity reordering ────────────────────────────────────────────────────

fn maybe_reorder_any_all(
    arr: &mut Vec<Value>,
    mir: &IntermediateSpec,
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

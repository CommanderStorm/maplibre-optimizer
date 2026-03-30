//! Constant folding passes: boolean algebra, comparison, arithmetic, dead branches, contradictions.

use maplibre_style_spec::mir::{MirExpressionOperator, MirSpec};
use serde_json::Value;

use super::util::{
    bool_literal, clamp_channel, compare_json_values, extract_json_literal, finite, is_num,
    replace_arr_with_value,
};

/// `["!", [op, a, b]]` → `[negation_of(op), a, b]` when the negated operator exists in MIR.
///
/// Handles `==` <-> `!=`, `<` <-> `>=`, `>` <-> `<=` generically via `MirExpressions::negation_of`.
pub(super) fn try_negate_comparison(arr: &mut Vec<Value>, mir: &MirSpec) -> bool {
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

pub(super) fn try_fold_not(arr: &mut Vec<Value>) -> bool {
    if arr.len() != 2 || arr.first().and_then(Value::as_str) != Some("!") {
        return false;
    }
    if let Some(b) = bool_literal(&arr[1]) {
        *arr = vec![Value::String("literal".to_string()), Value::Bool(!b)];
        return true;
    }
    false
}

pub(super) fn try_fold_comparison(arr: &mut Vec<Value>) -> bool {
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

pub(super) fn try_fold_boolean_algebra(arr: &mut Vec<Value>) -> bool {
    let op = match arr.first().and_then(Value::as_str) {
        Some("any") => "any",
        Some("all") => "all",
        _ => return false,
    };
    // Vacuous: ["all"] → true, ["any"] → false.
    if arr.len() == 1 {
        let result = op == "all"; // vacuous truth / vacuous disjunction
        *arr = vec![Value::String("literal".to_string()), Value::Bool(result)];
        return true;
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

/// Try to evaluate a pure operator whose all arguments are literal values.
pub(super) fn try_fold_pure_operator(arr: &mut Vec<Value>, mir: &MirSpec) -> bool {
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
        .is_some_and(MirExpressionOperator::is_pure);
    if !pure_in_mir && !matches!(op, "length" | "at" | "in") {
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

        // ── Membership ───────────────────────────────────────────────────────
        "in" => {
            if args.len() != 2 {
                return None;
            }
            let needle = &args[0];
            match &args[1] {
                Value::Array(haystack) => Some(Value::Bool(haystack.contains(needle))),
                Value::String(haystack) => {
                    let needle_str = needle.as_str()?;
                    Some(Value::Bool(haystack.contains(needle_str)))
                }
                _ => None,
            }
        }

        _ => None,
    }
}

/// Eliminate identity-element operands in binary arithmetic (e.g. `x * 1 → x`).
///
/// Complements `try_fold_pure_operator` which handles all-literal expressions;
/// this covers the mixed literal+variable case.
pub(super) fn try_algebraic_simplify(arr: &mut Vec<Value>) -> bool {
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
            // NOTE: `x * 0 → 0` is unsound because x may be NaN at runtime
            // (0 * NaN = NaN, not 0). Same for `0 * x`.
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
        "-" if is_num(&arr[2], 0.0) => {
            let x = arr[1].clone();
            replace_arr_with_value(arr, x);
            return true;
        }
        // NOTE: `0 / x → 0` is unsound (0 / 0 = NaN in IEEE 754, not 0).
        "/" if is_num(&arr[2], 1.0) => {
            let x = arr[1].clone();
            replace_arr_with_value(arr, x);
            return true;
        }
        "^" => {
            if is_num(&arr[2], 1.0) {
                let x = arr[1].clone();
                replace_arr_with_value(arr, x);
                return true;
            }
            // x ^ 0 → 1
            if is_num(&arr[2], 0.0) {
                *arr = vec![Value::String("literal".to_string()), Value::from(1.0)];
                return true;
            }
        }
        _ => {}
    }
    false
}

/// Resolve `case` arms with known-at-compile-time boolean conditions.
pub(super) fn try_dead_branch_case(arr: &mut Vec<Value>) -> bool {
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

/// Resolve `match` expressions when the input is a known literal value.
///
/// Scans arms for a matching label; replaces with the arm's output, or with the
/// fallback if no arm matches.
pub(super) fn try_dead_branch_match_literal(arr: &mut Vec<Value>) -> bool {
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

/// Sparse conditional constant propagation for `case` expressions.
///
/// When a `case` arm condition is `["==", A, B]` with one side being a literal,
/// substitute the non-literal expression with the literal value inside the arm body.
#[expect(
    clippy::ptr_arg,
    reason = "fn pointer signature requires &mut Vec<Value>"
)]
pub(super) fn try_sccp_case(arr: &mut Vec<Value>) -> bool {
    if arr.first().and_then(Value::as_str) != Some("case") {
        return false;
    }
    // Need at least: ["case", cond, out, fallback] = 4 elements, even length
    if arr.len() < 4 || !arr.len().is_multiple_of(2) {
        return false;
    }
    let mut changed = false;
    let n_arms = (arr.len() - 2) / 2;
    for i in 0..n_arms {
        let cond_idx = 1 + 2 * i;
        let out_idx = 2 + 2 * i;
        let Value::Array(cond) = &arr[cond_idx] else {
            continue;
        };
        // Must be ["==", A, B] with exactly 3 elements (no collator)
        if cond.len() != 3 || cond[0].as_str() != Some("==") {
            continue;
        }
        let (target, literal) = if let Some(lit) = extract_json_literal(&cond[2]) {
            (&cond[1], lit)
        } else if let Some(lit) = extract_json_literal(&cond[1]) {
            (&cond[2], lit)
        } else {
            continue;
        };
        let target = target.clone();
        let replacement = super::simplify::substitute_expr(&arr[out_idx], &target, &literal);
        if replacement != arr[out_idx] {
            arr[out_idx] = replacement;
            changed = true;
        }
    }
    changed
}

/// Sparse conditional constant propagation for `match` expressions.
///
/// When a `match` arm has a single (non-array) label, the input expression equals
/// that label inside the arm body — substitute it.
#[expect(
    clippy::ptr_arg,
    reason = "fn pointer signature requires &mut Vec<Value>"
)]
pub(super) fn try_sccp_match(arr: &mut Vec<Value>) -> bool {
    if arr.first().and_then(Value::as_str) != Some("match") {
        return false;
    }
    // ["match", input, label, output, ..., fallback] — len >= 5, odd length
    if arr.len() < 5 || arr.len().is_multiple_of(2) {
        return false;
    }
    // Skip if input is already a literal — `try_dead_branch_match_literal` handles that.
    if extract_json_literal(&arr[1]).is_some() {
        return false;
    }
    let input_expr = arr[1].clone();
    let mut changed = false;
    let arm_count = (arr.len() - 3) / 2;
    for i in 0..arm_count {
        let label_idx = 2 + 2 * i;
        let out_idx = 3 + 2 * i;
        // Skip multi-label arms — we don't know which value matched.
        if arr[label_idx].is_array() {
            continue;
        }
        let label = arr[label_idx].clone();
        let replacement = super::simplify::substitute_expr(&arr[out_idx], &input_expr, &label);
        if replacement != arr[out_idx] {
            arr[out_idx] = replacement;
            changed = true;
        }
    }
    changed
}

/// Detect contradictory `==`/`!=` predicates inside `["all", ...]` and fold to false.
pub(super) fn try_filter_contradiction(arr: &mut Vec<Value>) -> bool {
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
    // Range contradiction: [">=", x, hi] + ["<", x, lo] where lo <= hi → false
    if is_range_contradiction(&a_op, &a_rhs, &b_op, &b_rhs)
        || is_range_contradiction(&b_op, &b_rhs, &a_op, &a_rhs)
    {
        return true;
    }
    false
}

/// Check if two range predicates on the same LHS contradict:
/// `[op_a, x, a_val]` and `[op_b, x, b_val]` where the ranges are disjoint.
fn is_range_contradiction(op_a: &str, a_val: &Value, op_b: &str, b_val: &Value) -> bool {
    let (Some(a_num), Some(b_num)) = (a_val.as_f64(), b_val.as_f64()) else {
        return false;
    };
    // ">=" lo_bound + "<" hi_bound where hi_bound <= lo_bound → empty range
    if op_a == ">=" && op_b == "<" && b_num <= a_num {
        return true;
    }
    // ">" lo_bound + "<=" hi_bound where hi_bound <= lo_bound → empty range
    if op_a == ">" && op_b == "<=" && b_num <= a_num {
        return true;
    }
    // ">" lo_bound + "<" hi_bound where hi_bound <= lo_bound → empty range
    if op_a == ">" && op_b == "<" && b_num <= a_num {
        return true;
    }
    // ">=" lo_bound + "<=" hi_bound where hi_bound < lo_bound → empty range
    if op_a == ">=" && op_b == "<=" && b_num < a_num {
        return true;
    }
    false
}

/// Extracts `(op, lhs_expr, rhs_literal)` from `["=="|"!="|"<"|"<="|">"|">=", expr, lit]`
/// or `["==", lit, expr]`.
pub(super) fn extract_eq_predicate(v: &Value) -> Option<(String, Value, Value)> {
    let Value::Array(arr) = v else {
        return None;
    };
    if arr.len() != 3 {
        return None;
    }
    let op = arr[0].as_str()?.to_string();
    if !matches!(op.as_str(), "==" | "!=" | "<" | "<=" | ">" | ">=") {
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

/// Substitute equality bindings into siblings within `["all", ...]`.
///
/// For each `["==", expr, literal]` predicate (or commuted form), replaces occurrences of
/// `expr` with `literal` in all other sibling predicates. This enables downstream folds
/// (e.g. `try_fold_comparison`, `try_fold_pure_operator`) to constant-fold the substituted
/// expressions.
///
/// Only substitutes into direct siblings — does not recurse into nested `all`/`any`.
#[expect(
    clippy::ptr_arg,
    reason = "fn pointer signature requires &mut Vec<Value>"
)]
pub(super) fn try_equivalence_substitution(arr: &mut Vec<Value>) -> bool {
    if arr.first().and_then(Value::as_str) != Some("all") {
        return false;
    }
    if arr.len() < 3 {
        return false;
    }

    // Collect equality bindings: (index, target_expr, literal_value).
    let mut bindings: Vec<(usize, Value, Value)> = Vec::new();
    for (i, elem) in arr.iter().enumerate().skip(1) {
        if let Some((op, target, lit)) = extract_eq_predicate(elem)
            && op == "=="
        {
            bindings.push((i, target, lit));
        }
    }
    if bindings.is_empty() {
        return false;
    }

    // Substitute bindings into non-equality siblings.
    let mut changed = false;
    for (idx, elem) in arr.iter_mut().enumerate().skip(1) {
        // Skip the equality predicates themselves.
        if bindings.iter().any(|(i, _, _)| *i == idx) {
            continue;
        }
        for (_, target, replacement) in &bindings {
            let substituted = super::simplify::substitute_expr(elem, target, replacement);
            if substituted != *elem {
                *elem = substituted;
                changed = true;
            }
        }
    }
    changed
}

/// Detect range tightening inside `["all", ...]`:
/// e.g. `["all", [">=", x, 2], [">=", x, 4]]` → `["all", [">=", x, 4]]`
/// Subsumes the weaker bound.
pub(super) fn try_range_tightening(arr: &mut Vec<Value>) -> bool {
    if arr.first().and_then(Value::as_str) != Some("all") {
        return false;
    }
    if arr.len() < 3 {
        return false;
    }

    // Collect all predicates with their parsed forms.
    let preds: Vec<Option<(String, Value, Value)>> =
        arr.iter().skip(1).map(extract_eq_predicate).collect();

    for i in 0..preds.len() {
        for j in (i + 1)..preds.len() {
            let (Some((a_op, a_lhs, a_rhs)), Some((b_op, b_lhs, b_rhs))) = (&preds[i], &preds[j])
            else {
                continue;
            };
            if a_lhs != b_lhs {
                continue;
            }
            let (Some(a_num), Some(b_num)) = (a_rhs.as_f64(), b_rhs.as_f64()) else {
                continue;
            };

            // Two lower bounds: keep the larger.
            if matches!((a_op.as_str(), b_op.as_str()), (">=" | ">", ">=" | ">")) {
                let drop_idx = if a_num >= b_num {
                    // a is tighter or equal → drop b
                    j + 1
                } else {
                    i + 1
                };
                arr.remove(drop_idx);
                return true;
            }
            // Two upper bounds: keep the smaller.
            if matches!((a_op.as_str(), b_op.as_str()), ("<=" | "<", "<=" | "<")) {
                let drop_idx = if a_num <= b_num {
                    // a is tighter or equal → drop b
                    j + 1
                } else {
                    i + 1
                };
                arr.remove(drop_idx);
                return true;
            }
        }
    }
    false
}

/// Predicate subsumption: `["all", ["==", x, "a"], ["has", "class"]]` → `["all", ["==", x, "a"]]`
/// because `==` on a property implies the property exists.
pub(super) fn try_predicate_subsumption(arr: &mut Vec<Value>) -> bool {
    if arr.first().and_then(Value::as_str) != Some("all") {
        return false;
    }
    if arr.len() < 3 {
        return false;
    }

    // Find `["has", prop]` predicates that are subsumed by an `["=="|"!="|"<"|"<="|">"|">=", ["get", prop], ...]`.
    let mut to_remove: Vec<usize> = Vec::new();
    for (i, pred) in arr.iter().skip(1).enumerate() {
        let Value::Array(has_arr) = pred else {
            continue;
        };
        if has_arr.len() != 2 || has_arr[0].as_str() != Some("has") {
            continue;
        }
        let Some(prop_name) = has_arr[1].as_str() else {
            continue;
        };

        // Check if any other predicate implies this `has`.
        for (j, other) in arr.iter().skip(1).enumerate() {
            if i == j {
                continue;
            }
            let Value::Array(other_arr) = other else {
                continue;
            };
            if other_arr.len() != 3 {
                continue;
            }
            if !matches!(
                other_arr[0].as_str(),
                Some("==" | "!=" | "<" | "<=" | ">" | ">=")
            ) {
                continue;
            }
            // Check if the LHS is ["get", prop_name]
            if let Value::Array(get_arr) = &other_arr[1]
                && get_arr.len() == 2
                && get_arr[0].as_str() == Some("get")
                && get_arr[1].as_str() == Some(prop_name)
            {
                to_remove.push(i + 1); // +1 because we skip(1) for the "all" operator
                break;
            }
        }
    }

    if to_remove.is_empty() {
        return false;
    }

    // Remove in reverse order to preserve indices.
    to_remove.sort_unstable();
    to_remove.dedup();
    for idx in to_remove.into_iter().rev() {
        arr.remove(idx);
    }
    true
}

/// Boolean absorption: `["all", A, ["any", A, B, ...]]` → remove the `any` child,
/// and dually `["any", A, ["all", A, B, ...]]` → remove the `all` child.
pub(super) fn try_boolean_absorption(arr: &mut Vec<Value>) -> bool {
    let dual = match arr.first().and_then(Value::as_str) {
        Some("all") => "any",
        Some("any") => "all",
        _ => return false,
    };
    if arr.len() < 3 {
        return false;
    }

    // Collect indices of top-level atoms (children that are NOT the dual operator).
    let atoms: Vec<&Value> = arr[1..]
        .iter()
        .filter(|child| {
            !(child.is_array()
                && child.as_array().unwrap().first().and_then(Value::as_str) == Some(dual))
        })
        .collect();

    if atoms.is_empty() {
        return false;
    }

    // For each child that IS the dual operator, check if any atom appears among its children.
    let mut to_remove: Vec<usize> = Vec::new();
    for (i, child) in arr[1..].iter().enumerate() {
        let Value::Array(inner) = child else {
            continue;
        };
        if inner.first().and_then(Value::as_str) != Some(dual) {
            continue;
        }
        // Check if any top-level atom appears inside this dual-operator child.
        let absorbed = atoms
            .iter()
            .any(|atom| inner[1..].iter().any(|sub| sub == *atom));
        if absorbed {
            to_remove.push(i + 1); // +1 because arr[0] is the operator
        }
    }

    if to_remove.is_empty() {
        return false;
    }

    for idx in to_remove.into_iter().rev() {
        arr.remove(idx);
    }
    true
}

/// Fold redundant coercions:
/// - `["to-string", "hello"]` → `["literal", "hello"]` (already a string)
/// - `["to-number", 42]` → `["literal", 42]` (already a number)
/// - `["to-boolean", true]` → `["literal", true]` (already a boolean)
/// - Idempotent: `["to-string", ["to-string", X]]` → `["to-string", X]`
pub(super) fn try_fold_redundant_coercion(arr: &mut Vec<Value>) -> bool {
    if arr.len() != 2 {
        return false;
    }
    let Some(op) = arr[0].as_str() else {
        return false;
    };

    match op {
        "to-string" => {
            // Already a string literal?
            if arr[1].is_string() {
                let val = arr[1].clone();
                *arr = vec![Value::String("literal".to_string()), val];
                return true;
            }
            // Idempotent: ["to-string", ["to-string", X]] → ["to-string", X]
            if let Value::Array(inner) = &arr[1]
                && inner.len() == 2
                && inner[0].as_str() == Some("to-string")
            {
                let inner_arg = inner[1].clone();
                arr[1] = inner_arg;
                return true;
            }
        }
        "to-number" => {
            if arr[1].is_number() {
                let val = arr[1].clone();
                *arr = vec![Value::String("literal".to_string()), val];
                return true;
            }
            if let Value::Array(inner) = &arr[1]
                && inner.len() == 2
                && inner[0].as_str() == Some("to-number")
            {
                let inner_arg = inner[1].clone();
                arr[1] = inner_arg;
                return true;
            }
        }
        "to-boolean" => {
            if arr[1].is_boolean() {
                let val = arr[1].clone();
                *arr = vec![Value::String("literal".to_string()), val];
                return true;
            }
            if let Value::Array(inner) = &arr[1]
                && inner.len() == 2
                && inner[0].as_str() == Some("to-boolean")
            {
                let inner_arg = inner[1].clone();
                arr[1] = inner_arg;
                return true;
            }
        }
        _ => {}
    }
    false
}

/// Rewrite `["get", key, ["properties"]]` → `["get", key]` and
/// `["has", key, ["properties"]]` → `["has", key]`.
///
/// The `["properties"]` object is the current feature's property bag — the default context
/// for `get`/`has` — so the explicit argument is redundant.
pub(super) fn try_fold_redundant_properties(arr: &mut Vec<Value>) -> bool {
    if arr.len() != 3 {
        return false;
    }
    let Some("get" | "has") = arr[0].as_str() else {
        return false;
    };
    if !arr[1].is_string() {
        return false;
    }
    let Value::Array(obj) = &arr[2] else {
        return false;
    };
    if obj.len() == 1 && obj[0].as_str() == Some("properties") {
        arr.truncate(2);
        return true;
    }
    false
}

// ── Typed Boolean rules ──────────────────────────────────────────────────────
//
// Typed equivalents of the JSON peephole rules above, operating directly on
// `Boolean` enum variants.  Used by the typed filter walker.

use maplibre_style_spec::spec::{Boolean, ExprOrLiteral};

/// `Not(Literal(b))` → `Literal(!b)`
pub(super) fn try_fold_not_typed(filter: &mut Boolean) -> bool {
    if let Boolean::Not(inner) = filter
        && let Boolean::Literal(b) = **inner
    {
        *filter = Boolean::Literal(!b);
        return true;
    }
    false
}

/// `Not(EqualEqual(a, b, c))` → `NotEqual(a, b, c)` and vice versa for all
/// comparison operator pairs.
pub(super) fn try_negate_comparison_typed(filter: &mut Boolean) -> bool {
    let Boolean::Not(_) = filter else {
        return false;
    };
    // Take ownership of the inner expression to avoid cloning.
    let Boolean::Not(inner) = std::mem::replace(filter, Boolean::Literal(false)) else {
        unreachable!();
    };
    let negated = match *inner {
        Boolean::EqualEqual(a, b, c) => Boolean::NotEqual(a, b, c),
        Boolean::NotEqual(a, b, c) => Boolean::EqualEqual(a, b, c),
        Boolean::Less(a, b, c) => Boolean::GreaterEqual(a, b, c),
        Boolean::GreaterEqual(a, b, c) => Boolean::Less(a, b, c),
        Boolean::Greater(a, b, c) => Boolean::LessEqual(a, b, c),
        Boolean::LessEqual(a, b, c) => Boolean::Greater(a, b, c),
        other => {
            // Not a comparison — put it back.
            *filter = Boolean::Not(Box::new(other));
            return false;
        }
    };
    *filter = negated;
    true
}

/// Fold comparisons of two identical literals:
/// `EqualEqual(x, x)` → `Literal(true)`, `NotEqual(x, x)` → `Literal(false)`, etc.
pub(super) fn try_fold_comparison_typed(filter: &mut Boolean) -> bool {
    let result = match filter {
        Boolean::EqualEqual(a, b, None) if a == b && is_literal(a) => true,
        Boolean::NotEqual(a, b, None) if a == b && is_literal(a) => false,
        // For ordering comparisons with equal operands: < and > are false, <= and >= are true.
        Boolean::Less(a, b, None) if a == b && is_literal(a) => false,
        Boolean::Greater(a, b, None) if a == b && is_literal(a) => false,
        Boolean::LessEqual(a, b, None) if a == b && is_literal(a) => true,
        Boolean::GreaterEqual(a, b, None) if a == b && is_literal(a) => true,
        _ => return false,
    };
    *filter = Boolean::Literal(result);
    true
}

/// Boolean algebra simplification for `All` and `Any`:
/// - `All([])` → `Literal(true)`, `Any([])` → `Literal(false)` (vacuous)
/// - `All` short-circuits on `Literal(false)`, `Any` short-circuits on `Literal(true)`
/// - Removes absorbed literals (`true` from `All`, `false` from `Any`)
pub(super) fn try_fold_boolean_algebra_typed(filter: &mut Boolean) -> bool {
    match filter {
        Boolean::All(_) => fold_all_any_typed(filter, true),
        Boolean::Any(_) => fold_all_any_typed(filter, false),
        _ => false,
    }
}

fn fold_all_any_typed(filter: &mut Boolean, is_all: bool) -> bool {
    let (Boolean::All(children) | Boolean::Any(children)) = filter else {
        return false;
    };

    // Vacuous: All([]) → true, Any([]) → false.
    if children.is_empty() {
        *filter = Boolean::Literal(is_all);
        return true;
    }

    let absorbing = !is_all; // true absorbs Any, false absorbs All
    let identity = is_all; // true is identity for All, false is identity for Any

    // Short-circuit on absorbing element.
    if children
        .iter()
        .any(|c| matches!(c, Boolean::Literal(b) if *b == absorbing))
    {
        *filter = Boolean::Literal(absorbing);
        return true;
    }

    // Filter out identity elements.
    let before = children.len();
    children.retain(|c| !matches!(c, Boolean::Literal(b) if *b == identity));

    if children.is_empty() {
        // All elements were identity → result is identity.
        *filter = Boolean::Literal(identity);
        return true;
    }

    children.len() != before
}

/// Detect contradictory `==`/`!=` predicates inside `All(children)` → `Literal(false)`.
pub(super) fn try_filter_contradiction_typed(filter: &mut Boolean) -> bool {
    let Boolean::All(children) = filter else {
        return false;
    };
    for i in 0..children.len() {
        for j in (i + 1)..children.len() {
            if typed_predicates_contradict(&children[i], &children[j]) {
                *filter = Boolean::Literal(false);
                return true;
            }
        }
    }
    false
}

/// Check if two typed boolean predicates contradict each other.
fn typed_predicates_contradict(a: &Boolean, b: &Boolean) -> bool {
    // Extract (lhs, rhs) from equality/inequality comparisons.
    let (a_eq, a_lhs, a_rhs) = match a {
        Boolean::EqualEqual(l, r, None) => (true, l, r),
        Boolean::NotEqual(l, r, None) => (false, l, r),
        _ => return false,
    };
    let (b_eq, b_lhs, b_rhs) = match b {
        Boolean::EqualEqual(l, r, None) => (true, l, r),
        Boolean::NotEqual(l, r, None) => (false, l, r),
        _ => return false,
    };
    if a_lhs != b_lhs {
        return false;
    }
    // Two ==s with different RHS: x == a ∧ x == b where a ≠ b → contradiction.
    if a_eq && b_eq && a_rhs != b_rhs {
        return true;
    }
    // == and != with same value: x == a ∧ x != a → contradiction.
    if a_eq != b_eq && a_rhs == b_rhs {
        return true;
    }
    false
}

/// Boolean absorption on typed `Boolean`: `All([A, Any([A, B, ...])])` → remove the `Any` child,
/// and dually `Any([A, All([A, B, ...])])` → remove the `All` child.
pub(super) fn try_boolean_absorption_typed(filter: &mut Boolean) -> bool {
    let is_all = match filter {
        Boolean::All(_) => true,
        Boolean::Any(_) => false,
        _ => return false,
    };

    let (Boolean::All(children) | Boolean::Any(children)) = filter else {
        return false;
    };

    if children.len() < 2 {
        return false;
    }

    // Collect top-level atoms: children that are NOT the dual variant.
    let atom_indices: Vec<usize> = children
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            if is_all {
                !matches!(c, Boolean::Any(_))
            } else {
                !matches!(c, Boolean::All(_))
            }
        })
        .map(|(i, _)| i)
        .collect();

    if atom_indices.is_empty() {
        return false;
    }

    // For each dual-variant child, check if any atom appears among its sub-children.
    let mut to_remove: Vec<usize> = Vec::new();
    for (i, child) in children.iter().enumerate() {
        let inner = if is_all {
            if let Boolean::Any(inner) = child {
                inner
            } else {
                continue;
            }
        } else if let Boolean::All(inner) = child {
            inner
        } else {
            continue;
        };

        let absorbed = atom_indices
            .iter()
            .any(|&ai| inner.iter().any(|sub| sub == &children[ai]));
        if absorbed {
            to_remove.push(i);
        }
    }

    if to_remove.is_empty() {
        return false;
    }

    for idx in to_remove.into_iter().rev() {
        children.remove(idx);
    }
    true
}

/// Distributive factoring on JSON expression arrays.
///
/// `["any", ["all", A, B], ["all", A, C]]` → `["all", A, ["any", B, C]]`
/// `["all", ["any", A, B], ["any", A, C]]` → `["any", A, ["all", B, C]]`
///
/// Fires when **every** child of the outer operator is the dual operator and all
/// children share at least one common operand.
pub(super) fn try_distributive_factoring(arr: &mut Vec<Value>) -> bool {
    let (outer, dual) = match arr.first().and_then(Value::as_str) {
        Some("any") => ("any", "all"),
        Some("all") => ("all", "any"),
        _ => return false,
    };

    // Need at least 2 children.
    if arr.len() < 3 {
        return false;
    }

    // Every child must be the dual operator.
    let children: Vec<&Vec<Value>> = arr[1..]
        .iter()
        .filter_map(|child| {
            let Value::Array(inner) = child else {
                return None;
            };
            if inner.first().and_then(Value::as_str) == Some(dual) && inner.len() >= 2 {
                Some(inner)
            } else {
                None
            }
        })
        .collect();

    if children.len() != arr.len() - 1 {
        return false;
    }

    // Find common operands across all children (intersection).
    // Start with operands of the first child, keep only those present in all others.
    let first_operands: Vec<&Value> = children[0][1..].iter().collect();
    let common: Vec<&Value> = first_operands
        .into_iter()
        .filter(|op| {
            children[1..]
                .iter()
                .all(|child| child[1..].iter().any(|v| v == *op))
        })
        .collect();

    if common.is_empty() {
        return false;
    }

    // Build remainders for each child (operands not in the common set).
    let remainders: Vec<Vec<Value>> = children
        .iter()
        .map(|child| {
            child[1..]
                .iter()
                .filter(|v| !common.iter().any(|c| c == v))
                .cloned()
                .collect()
        })
        .collect();

    // Build the inner expression from remainders wrapped in the outer operator.
    // Each remainder with 0 elements → identity (true for all, false for any).
    // Each remainder with 1 element → unwrap.
    let inner_children: Vec<Value> = remainders
        .into_iter()
        .map(|mut rem| {
            if rem.is_empty() {
                // Empty remainder → vacuous dual: all()=true, any()=false.
                // Other passes will simplify the resulting expression.
                Value::Bool(dual == "all")
            } else if rem.len() == 1 {
                rem.remove(0)
            } else {
                let mut inner = vec![Value::String(dual.to_string())];
                inner.extend(rem);
                Value::Array(inner)
            }
        })
        .collect();

    // Build result: [dual, common..., [outer, remainders...]]
    let mut result = vec![Value::String(dual.to_string())];
    result.extend(common.into_iter().cloned());

    if inner_children.len() == 1 {
        result.push(inner_children.into_iter().next().unwrap());
    } else {
        let mut wrapper = vec![Value::String(outer.to_string())];
        wrapper.extend(inner_children);
        result.push(Value::Array(wrapper));
    }

    *arr = result;
    true
}

/// Distributive factoring on typed `Boolean`:
///
/// `Any([All([A, B]), All([A, C])])` → `All([A, Any([B, C])])`
/// `All([Any([A, B]), Any([A, C])])` → `Any([A, All([B, C])])`
pub(super) fn try_distributive_factoring_typed(filter: &mut Boolean) -> bool {
    let is_all = match filter {
        Boolean::All(_) => true,
        Boolean::Any(_) => false,
        _ => return false,
    };

    let (Boolean::All(children) | Boolean::Any(children)) = filter else {
        return false;
    };

    if children.len() < 2 {
        return false;
    }

    // Every child must be the dual operator.
    let all_dual = children.iter().all(|c| {
        if is_all {
            matches!(c, Boolean::Any(_))
        } else {
            matches!(c, Boolean::All(_))
        }
    });
    if !all_dual {
        return false;
    }

    // Extract inner children of each dual-op child.
    let inner_vecs: Vec<&Vec<Boolean>> = children
        .iter()
        .map(|c| match c {
            Boolean::All(v) | Boolean::Any(v) => v,
            _ => unreachable!(),
        })
        .collect();

    // Find common operands: present in every child.
    let common: Vec<Boolean> = inner_vecs[0]
        .iter()
        .filter(|op| inner_vecs[1..].iter().all(|v| v.contains(op)))
        .cloned()
        .collect();

    if common.is_empty() {
        return false;
    }

    // Build remainders.
    let remainders: Vec<Vec<Boolean>> = inner_vecs
        .iter()
        .map(|v| v.iter().filter(|x| !common.contains(x)).cloned().collect())
        .collect();

    // Build inner children for the outer-op wrapper.
    let inner_children: Vec<Boolean> = remainders
        .into_iter()
        .map(|mut rem| {
            if rem.is_empty() {
                // Vacuous dual: all()=true, any()=false
                Boolean::Literal(!is_all)
            } else if rem.len() == 1 {
                rem.remove(0)
            } else if is_all {
                // Outer is all → dual is any → remainders wrap in any
                Boolean::Any(rem)
            } else {
                // Outer is any → dual is all → remainders wrap in all
                Boolean::All(rem)
            }
        })
        .collect();

    // Build result: dual(common..., outer(inner_children...))
    let mut result_children = common;
    if inner_children.len() == 1 {
        result_children.push(inner_children.into_iter().next().unwrap());
    } else {
        let wrapper = if is_all {
            Boolean::All(inner_children)
        } else {
            Boolean::Any(inner_children)
        };
        result_children.push(wrapper);
    }

    *filter = if is_all {
        Boolean::Any(result_children)
    } else {
        Boolean::All(result_children)
    };
    true
}

/// Check if an `ExprOrLiteral` is a literal (not a computed expression).
pub(super) fn is_literal(v: &ExprOrLiteral) -> bool {
    matches!(
        v,
        ExprOrLiteral::Null
            | ExprOrLiteral::Bool(_)
            | ExprOrLiteral::NumberLiteral(_)
            | ExprOrLiteral::StringLiteral(_)
    )
}

// ── Geometry-type fold from layer type ───────────────────────────────────────

/// Returns the geometry-type string that `["geometry-type"]` always evaluates to
/// for a given layer type, or `None` if multiple geometry types are possible.
fn layer_type_geometry(layer_type: &str) -> Option<&'static str> {
    match layer_type {
        "fill" | "fill-extrusion" => Some("Polygon"),
        "line" => Some("LineString"),
        "circle" | "heatmap" => Some("Point"),
        // symbol layers accept both Point and LineString placement.
        _ => None,
    }
}

/// Fold `["geometry-type"]` comparisons when the layer type constrains the result.
///
/// Handles `==`, `!=`, and `in` patterns. Recurses into sub-expressions.
pub(super) fn fold_geometry_type_from_layer(v: &mut Value, layer_type: &str) -> bool {
    let Some(known_geom) = layer_type_geometry(layer_type) else {
        return false;
    };

    let Value::Array(arr) = v else {
        return false;
    };

    if try_fold_geom_node(arr, known_geom) {
        return true;
    }

    let mut changed = false;
    for child in arr.iter_mut() {
        changed |= fold_geometry_type_from_layer(child, layer_type);
    }
    changed
}

/// Try to fold a single expression node that compares `["geometry-type"]`.
fn try_fold_geom_node(arr: &mut Vec<Value>, known_geom: &str) -> bool {
    let Some(op) = arr.first().and_then(Value::as_str) else {
        return false;
    };

    match op {
        // ["==", ["geometry-type"], "X"] or ["!=", ...]
        "==" | "!=" if arr.len() == 3 => {
            let (geom_str, _) = if is_geometry_type_expr(&arr[1]) && arr[2].is_string() {
                (arr[2].as_str().unwrap(), 1)
            } else if is_geometry_type_expr(&arr[2]) && arr[1].is_string() {
                (arr[1].as_str().unwrap(), 2)
            } else {
                return false;
            };

            let matches = geom_str == known_geom;
            let result = if op == "==" { matches } else { !matches };
            *arr = vec![Value::String("literal".into()), Value::Bool(result)];
            true
        }
        // ["in", ["geometry-type"], ["literal", [...]]]
        "in" if arr.len() == 3 && is_geometry_type_expr(&arr[1]) => {
            let Value::Array(lit_arr) = &arr[2] else {
                return false;
            };
            if lit_arr.len() != 2 || lit_arr[0].as_str() != Some("literal") {
                return false;
            }
            let Value::Array(values) = &lit_arr[1] else {
                return false;
            };
            let result = values.iter().any(|v| v.as_str() == Some(known_geom));
            *arr = vec![Value::String("literal".into()), Value::Bool(result)];
            true
        }
        _ => false,
    }
}

/// Check if a value is `["geometry-type"]`.
fn is_geometry_type_expr(v: &Value) -> bool {
    matches!(v, Value::Array(a) if a.len() == 1 && a[0].as_str() == Some("geometry-type"))
}

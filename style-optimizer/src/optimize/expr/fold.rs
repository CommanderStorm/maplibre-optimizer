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

    let Some(stats) = stats else { return false };
    if (stats.sample_rate - 1.0).abs() > f64::EPSILON {
        return false;
    }
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

    // If prop is unknown, all values are dead.
    let prop_stats = layer_stats.properties.get(prop);

    let kept: Vec<Value> = values
        .iter()
        .filter(|v| value_exists_in_stats(v, prop_stats))
        .cloned()
        .collect();

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

    let Some(stats) = stats else { return false };
    if (stats.sample_rate - 1.0).abs() > f64::EPSILON {
        return false;
    }
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

    let Some(stats) = stats else { return false };
    if (stats.sample_rate - 1.0).abs() > f64::EPSILON {
        return false;
    }
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

    // Find the first ["get", prop] arm where prop is present on all features.
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
    false
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
        try_fold_geometry_type_from_stats, try_fold_has_from_stats, try_fold_redundant_properties,
    };

    #[test]
    fn get_with_properties_is_simplified() {
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["get", "name", ["properties"]])).unwrap();
        assert!(try_fold_redundant_properties(&mut arr));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - get
        - name
        ");
    }

    #[test]
    fn has_with_properties_is_simplified() {
        let mut arr: Vec<Value> =
            serde_json::from_value(json!(["has", "name", ["properties"]])).unwrap();
        assert!(try_fold_redundant_properties(&mut arr));
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - has
        - name
        ");
    }

    #[test]
    fn get_without_properties_unchanged() {
        let mut arr: Vec<Value> = serde_json::from_value(json!(["get", "name"])).unwrap();
        assert!(!try_fold_redundant_properties(&mut arr));
        assert_eq!(Value::Array(arr), json!(["get", "name"]));
    }

    #[test]
    fn has_without_properties_unchanged() {
        let mut arr: Vec<Value> = serde_json::from_value(json!(["has", "name"])).unwrap();
        assert!(!try_fold_redundant_properties(&mut arr));
        assert_eq!(Value::Array(arr), json!(["has", "name"]));
    }

    #[test]
    fn get_with_non_properties_object_unchanged() {
        let original = json!(["get", "name", ["literal", {"a": 1}]]);
        let mut arr: Vec<Value> = serde_json::from_value(original.clone()).unwrap();
        assert!(!try_fold_redundant_properties(&mut arr));
        assert_eq!(Value::Array(arr), original);
    }

    #[test]
    fn get_with_other_object_expr_unchanged() {
        let original = json!(["get", "name", ["object-expr"]]);
        let mut arr: Vec<Value> = serde_json::from_value(original.clone()).unwrap();
        assert!(!try_fold_redundant_properties(&mut arr));
        assert_eq!(Value::Array(arr), original);
    }

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

    use super::try_fold_comparison_from_stats;

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

    use super::{
        try_fold_coalesce_from_stats, try_prune_in_from_stats, try_prune_match_from_stats,
    };

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
        assert_yaml_snapshot!(Value::Array(arr), @r"
        - in
        - - get
          - kind
        - - literal
          - - a
            - b
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
}

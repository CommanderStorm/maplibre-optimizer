//! Shared helpers for expression passes.

use serde_json::Value;

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
pub(super) fn is_num(v: &Value, expected: f64) -> bool {
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
/// If `v` is a non-empty `Value::Array`, the array is used directly (it becomes the new
/// expression).  An empty array can't be a valid expression, so it is wrapped in
/// `["literal", []]`.  Scalars are wrapped in `["literal", scalar]`.
pub(super) fn replace_arr_with_value(arr: &mut Vec<Value>, v: Value) {
    match v {
        Value::Array(inner) if !inner.is_empty() => *arr = inner,
        other => {
            *arr = vec![Value::String("literal".to_string()), other];
        }
    }
}

pub(super) fn compare_json_values(x: &Value, y: &Value) -> Option<std::cmp::Ordering> {
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

pub(super) fn finite(n: f64) -> Option<Value> {
    if n.is_finite() {
        serde_json::Number::from_f64(n).map(Value::Number)
    } else {
        None
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(super) fn clamp_channel(v: f64) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

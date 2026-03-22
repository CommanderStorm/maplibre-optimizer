//! Shared expression pattern-matching helpers.
//!
//! Extracted from `selectivity.rs` and `expr.rs` for reuse across field analysis,
//! advisory generation, and expression rewriting.

use serde_json::Value;

/// Returns `true` if `v` is a `["get", prop_name]` expression (exactly 2 elements).
pub(crate) fn is_get_expr(v: &Value) -> bool {
    matches!(v, Value::Array(a) if a.len() == 2 && a[0].as_str() == Some("get"))
}

/// Extract the property name from a `["get", prop_name]` expression.
pub(crate) fn get_prop_name(v: &Value) -> Option<&str> {
    let Value::Array(a) = v else { return None };
    if a.len() == 2 && a[0].as_str() == Some("get") {
        a[1].as_str()
    } else {
        None
    }
}

/// Given two sides of a binary expression, match one as `["get", prop]` and the other as a
/// JSON literal. Returns `(prop_name, literal_value)`.
pub(crate) fn extract_get_and_literal<'a>(
    lhs: &'a Value,
    rhs: &'a Value,
) -> Option<(&'a str, Value)> {
    if let Some(prop) = get_prop_name(lhs) {
        let lit = extract_json_literal(rhs)?;
        return Some((prop, lit));
    }
    if let Some(prop) = get_prop_name(rhs) {
        let lit = extract_json_literal(lhs)?;
        return Some((prop, lit));
    }
    None
}

/// Extract a JSON literal from a bare value or a `["literal", val]` wrapper.
pub(crate) fn extract_json_literal(v: &Value) -> Option<Value> {
    match v {
        Value::Number(_) | Value::String(_) | Value::Bool(_) | Value::Null => Some(v.clone()),
        Value::Array(a) if a.len() == 2 && a[0].as_str() == Some("literal") => Some(a[1].clone()),
        _ => None,
    }
}

/// Returns `true` if `v` is `["geometry-type"]`.
pub(crate) fn is_geometry_type_expr(v: &Value) -> bool {
    matches!(v, Value::Array(a) if a.len() == 1 && a[0].as_str() == Some("geometry-type"))
}

/// Returns `true` if `v` is `["id"]`.
pub(crate) fn is_id_expr(v: &Value) -> bool {
    matches!(v, Value::Array(a) if a.len() == 1 && a[0].as_str() == Some("id"))
}

/// Coerce a JSON value to `i64` (tries `as_i64`, then `as_f64` cast).
#[expect(clippy::cast_possible_truncation)]
pub(crate) fn json_as_i64(v: &Value) -> Option<i64> {
    v.as_i64().or_else(|| v.as_f64().map(|f| f as i64))
}

/// Coerce a JSON value to `u64` (tries `as_u64`, then `as_f64` cast).
#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn json_as_u64(v: &Value) -> Option<u64> {
    v.as_u64().or_else(|| v.as_f64().map(|f| f as u64))
}

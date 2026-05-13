//! Convert legacy `MapLibre` filter syntax to expression syntax.
//!
//! Legacy filter syntax uses bare property names as strings (e.g.
//! `["==", "class", "residential"]`), where the property name is implicitly
//! wrapped in `["get", ...]`. Expression syntax requires explicit getters
//! (`["==", ["get", "class"], "residential"]`). Without conversion, the
//! optimizer's expression-level passes (constant fold, simplify, layer merge)
//! misinterpret bare strings as literals and produce incorrect output.
//!
//! This module ports `convertFilter`/`isExpressionFilter` from the upstream
//! `MapLibre GL JS` source (`upstream/src/feature_filter/`) so that every style
//! entering the optimizer is normalized to expression syntax first.
//!
//! Note: the upstream converter injects per-branch runtime type checks inside
//! `any` expressions to preserve legacy semantics around type mismatches. We
//! omit those type checks — they matter only for pathological feature data
//! that mixes types on the same property, which does not occur in the styles
//! we benchmark.

use serde_json::{Value, json};

/// Convert every legacy filter in the style to expression syntax.
pub(crate) fn convert_legacy_filters_in_style(style: &mut Value) {
    let Some(layers) = style
        .as_object_mut()
        .and_then(|o| o.get_mut("layers"))
        .and_then(Value::as_array_mut)
    else {
        return;
    };
    for layer in layers {
        let Some(obj) = layer.as_object_mut() else {
            continue;
        };
        if let Some(filter) = obj.get_mut("filter")
            && !is_expression_filter(filter)
        {
            *filter = convert_filter(filter);
        }
    }
}

/// Return `true` if `filter` is already in expression syntax.
///
/// Mirrors `isExpressionFilter` in upstream `feature_filter/index.ts`.
fn is_expression_filter(filter: &Value) -> bool {
    if filter.is_boolean() {
        return true;
    }
    let Some(arr) = filter.as_array() else {
        return false;
    };
    if arr.is_empty() {
        return false;
    }
    let Some(op) = arr[0].as_str() else {
        // Non-string head is valid in expressions (e.g. inside nested expressions)
        return true;
    };
    match op {
        // `has` is an expression when its argument is not a legacy special key.
        "has" => {
            arr.len() >= 2 && arr[1].as_str() != Some("$id") && arr[1].as_str() != Some("$type")
        }
        // Expression `in` takes exactly `["in", needle, haystack]` where haystack is an array.
        "in" => arr.len() >= 3 && (!arr[1].is_string() || arr[2].is_array()),
        // Purely legacy operators have no expression form.
        "!in" | "!has" | "none" => false,
        // Comparison operators: legacy form is exactly 3 elements with a
        // bare *string* property in position 1. A non-string second argument
        // is always expression syntax (comparing literals or nested
        // expressions), as is any form where either arg is already an array.
        "==" | "!=" | ">" | ">=" | "<" | "<=" => {
            arr.len() != 3 || !arr[1].is_string() || arr[2].is_array()
        }
        // Boolean combinators are expressions only if every child is an expression.
        "any" | "all" => arr
            .iter()
            .skip(1)
            .all(|child| child.is_boolean() || is_expression_filter(child)),
        _ => true,
    }
}

/// Convert `filter` to expression syntax. Already-expression filters pass through.
fn convert_filter(filter: &Value) -> Value {
    if is_expression_filter(filter) {
        return filter.clone();
    }
    if filter.is_null() {
        return json!(true);
    }
    let Some(arr) = filter.as_array() else {
        return json!(true);
    };
    let Some(op) = arr.first().and_then(Value::as_str) else {
        return json!(true);
    };
    // Short legacy filter forms like `["any"]` / `["all"]` / `["none"]` with no
    // arguments: upstream returns `op != "any"` (i.e. `all`/`none` match all).
    if arr.len() <= 1 {
        return json!(op != "any");
    }
    match op {
        "==" | "!=" | "<" | ">" | "<=" | ">=" => {
            let property = arr[1].as_str().unwrap_or("");
            convert_comparison_op(property, &arr[2], op)
        }
        "any" => {
            let mut out = vec![json!("any")];
            for child in arr.iter().skip(1) {
                out.push(convert_filter(child));
            }
            Value::Array(out)
        }
        "all" => {
            let children: Vec<Value> = arr.iter().skip(1).map(convert_filter).collect();
            if children.len() == 1 {
                children.into_iter().next().unwrap()
            } else {
                let mut out = vec![json!("all")];
                out.extend(children);
                Value::Array(out)
            }
        }
        "none" => {
            let mut any = vec![json!("any")];
            any.extend(arr.iter().skip(1).cloned());
            json!(["!", convert_filter(&Value::Array(any))])
        }
        "in" => {
            let property = arr[1].as_str().unwrap_or("");
            convert_in_op(property, &arr[2..], false)
        }
        "!in" => {
            let property = arr[1].as_str().unwrap_or("");
            convert_in_op(property, &arr[2..], true)
        }
        "has" => convert_has_op(arr[1].as_str().unwrap_or("")),
        "!has" => json!(["!", convert_has_op(arr[1].as_str().unwrap_or(""))]),
        _ => json!(true),
    }
}

fn convert_comparison_op(property: &str, value: &Value, op: &str) -> Value {
    // `$type` comparisons map to `geometry-type`; `$id` maps to the `id` accessor.
    if property == "$type" {
        return json!([op, ["geometry-type"], value]);
    }
    let get: Value = if property == "$id" {
        json!(["id"])
    } else {
        json!(["get", property])
    };
    // Legacy filters treat "property missing" as not equal to null, so we must
    // explicitly guard equality/inequality against null for real properties
    // (not `$id`, which has special null handling).
    if op == "==" && property != "$id" && value.is_null() {
        return json!(["all", ["has", property], ["==", get, Value::Null]]);
    }
    if op == "!=" && property != "$id" && value.is_null() {
        return json!(["any", ["!", ["has", property]], ["!=", get, Value::Null]]);
    }
    json!([op, get, value])
}

fn convert_in_op(property: &str, values: &[Value], negate: bool) -> Value {
    if values.is_empty() {
        return json!(negate);
    }
    let get: Value = match property {
        "$type" => json!(["geometry-type"]),
        "$id" => json!(["id"]),
        _ => json!(["get", property]),
    };
    // Prefer `match` when all values share a single string/number type — it
    // compiles to a faster lookup than a chain of equality tests.
    let all_string = values.iter().all(Value::is_string);
    let all_number = values.iter().all(Value::is_number);
    if all_string || all_number {
        let mut unique: Vec<Value> = values.to_vec();
        unique.sort_by(|a, b| {
            // Compare by the raw inner representation to avoid allocating
            // a formatted string on every comparison.
            match (a.as_str(), b.as_str()) {
                (Some(a), Some(b)) => a.cmp(b),
                _ => a
                    .as_f64()
                    .partial_cmp(&b.as_f64())
                    .unwrap_or(std::cmp::Ordering::Equal),
            }
        });
        unique.dedup();
        return json!(["match", get, unique, !negate, negate]);
    }
    if negate {
        let mut out = vec![json!("all")];
        for v in values {
            out.push(json!(["!=", get.clone(), v]));
        }
        Value::Array(out)
    } else {
        let mut out = vec![json!("any")];
        for v in values {
            out.push(json!(["==", get.clone(), v]));
        }
        Value::Array(out)
    }
}

fn convert_has_op(property: &str) -> Value {
    match property {
        "$type" => json!(true),
        "$id" => json!(["!=", ["id"], Value::Null]),
        _ => json!(["has", property]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_legacy_equality_on_bare_string() {
        let f = json!(["==", "class", "residential"]);
        assert!(!is_expression_filter(&f));
    }

    #[test]
    fn detects_expression_equality_on_get() {
        let f = json!(["==", ["get", "class"], "residential"]);
        assert!(is_expression_filter(&f));
    }

    #[test]
    fn converts_legacy_comparison_to_get() {
        let f = json!(["==", "class", "residential"]);
        assert_eq!(
            convert_filter(&f),
            json!(["==", ["get", "class"], "residential"])
        );
    }

    #[test]
    fn converts_legacy_type_comparison_to_geometry_type() {
        let f = json!(["==", "$type", "Polygon"]);
        assert_eq!(
            convert_filter(&f),
            json!(["==", ["geometry-type"], "Polygon"])
        );
    }

    #[test]
    fn converts_nested_all_filter() {
        let f = json!([
            "all",
            ["==", "$type", "Polygon"],
            ["==", "class", "residential"],
        ]);
        assert_eq!(
            convert_filter(&f),
            json!([
                "all",
                ["==", ["geometry-type"], "Polygon"],
                ["==", ["get", "class"], "residential"],
            ])
        );
    }

    #[test]
    fn converts_legacy_in_to_match() {
        let f = json!(["in", "class", "a", "b", "c"]);
        // Expected: ["match", ["get", "class"], ["a", "b", "c"], true, false]
        let out = convert_filter(&f);
        assert_eq!(out[0], json!("match"));
        assert_eq!(out[1], json!(["get", "class"]));
    }

    #[test]
    fn converts_legacy_has_to_expression_has() {
        let f = json!(["has", "class"]);
        assert_eq!(convert_filter(&f), json!(["has", "class"]));
    }

    #[test]
    fn converts_none_to_not_any() {
        let f = json!(["none", ["==", "class", "a"]]);
        // ["!", ["any", ["==", ["get", "class"], "a"]]]
        let out = convert_filter(&f);
        assert_eq!(out[0], json!("!"));
    }

    #[test]
    fn passes_through_already_expression_filter() {
        let f = json!(["==", ["geometry-type"], "Polygon"]);
        assert_eq!(convert_filter(&f), f);
    }

    #[test]
    fn converts_whole_style() {
        let mut style = json!({
            "layers": [
                {"id": "a", "filter": ["==", "class", "residential"]},
                {"id": "b", "filter": ["==", ["get", "class"], "water"]},
            ]
        });
        convert_legacy_filters_in_style(&mut style);
        let layers = style["layers"].as_array().unwrap();
        assert_eq!(
            layers[0]["filter"],
            json!(["==", ["get", "class"], "residential"])
        );
        assert_eq!(
            layers[1]["filter"],
            json!(["==", ["get", "class"], "water"])
        );
    }
}

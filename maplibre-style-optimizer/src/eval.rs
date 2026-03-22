//! Minimal `MapLibre` expression evaluator for testing advisory correctness.
//!
//! This evaluator supports a subset of the `MapLibre` expression language sufficient to verify
//! that `render(original_style, original_data) ≡ render(rewritten_style, rewritten_data)`.

#![cfg(test)]

use std::collections::BTreeMap;

use serde_json::Value;

/// A synthetic tile feature for evaluation.
#[derive(Debug, Clone)]
pub struct Feature {
    pub properties: BTreeMap<String, Value>,
    pub geometry_type: String,
    pub id: Option<u64>,
    pub zoom: f64,
}

/// Evaluate a `MapLibre` expression against a feature.
///
/// Returns `None` for unsupported operators or malformed expressions.
#[expect(clippy::too_many_lines)]
pub fn eval_expression(expr: &Value, feature: &Feature) -> Option<Value> {
    match expr {
        // Bare literals pass through
        Value::Number(_) | Value::String(_) | Value::Bool(_) | Value::Null => Some(expr.clone()),
        Value::Object(_) => Some(expr.clone()),
        Value::Array(arr) => {
            if arr.is_empty() {
                return None;
            }
            let op = arr[0].as_str()?;

            match op {
                "literal" if arr.len() == 2 => Some(arr[1].clone()),

                "get" if arr.len() == 2 => {
                    let prop = arr[1].as_str()?;
                    Some(feature.properties.get(prop).cloned().unwrap_or(Value::Null))
                }

                "has" if arr.len() == 2 => {
                    let prop = arr[1].as_str()?;
                    Some(Value::Bool(feature.properties.contains_key(prop)))
                }

                "geometry-type" if arr.len() == 1 => {
                    Some(Value::String(feature.geometry_type.clone()))
                }

                "id" if arr.len() == 1 => Some(match feature.id {
                    Some(id) => Value::from(id),
                    None => Value::Null,
                }),

                "zoom" if arr.len() == 1 => Some(Value::from(feature.zoom)),

                "==" if arr.len() == 3 => {
                    let lhs = eval_expression(&arr[1], feature)?;
                    let rhs = eval_expression(&arr[2], feature)?;
                    Some(Value::Bool(values_equal(&lhs, &rhs)))
                }

                "!=" if arr.len() == 3 => {
                    let lhs = eval_expression(&arr[1], feature)?;
                    let rhs = eval_expression(&arr[2], feature)?;
                    Some(Value::Bool(!values_equal(&lhs, &rhs)))
                }

                "<" if arr.len() == 3 => {
                    let lhs = eval_expression(&arr[1], feature)?;
                    let rhs = eval_expression(&arr[2], feature)?;
                    Some(Value::Bool(compare_values(&lhs, &rhs)? < 0))
                }

                "<=" if arr.len() == 3 => {
                    let lhs = eval_expression(&arr[1], feature)?;
                    let rhs = eval_expression(&arr[2], feature)?;
                    Some(Value::Bool(compare_values(&lhs, &rhs)? <= 0))
                }

                ">" if arr.len() == 3 => {
                    let lhs = eval_expression(&arr[1], feature)?;
                    let rhs = eval_expression(&arr[2], feature)?;
                    Some(Value::Bool(compare_values(&lhs, &rhs)? > 0))
                }

                ">=" if arr.len() == 3 => {
                    let lhs = eval_expression(&arr[1], feature)?;
                    let rhs = eval_expression(&arr[2], feature)?;
                    Some(Value::Bool(compare_values(&lhs, &rhs)? >= 0))
                }

                "all" => {
                    for child in &arr[1..] {
                        let val = eval_expression(child, feature)?;
                        if !is_truthy(&val) {
                            return Some(Value::Bool(false));
                        }
                    }
                    Some(Value::Bool(true))
                }

                "any" => {
                    for child in &arr[1..] {
                        let val = eval_expression(child, feature)?;
                        if is_truthy(&val) {
                            return Some(Value::Bool(true));
                        }
                    }
                    Some(Value::Bool(false))
                }

                "!" if arr.len() == 2 => {
                    let val = eval_expression(&arr[1], feature)?;
                    Some(Value::Bool(!is_truthy(&val)))
                }

                "match" if arr.len() >= 4 => eval_match(arr, feature),

                "in" if arr.len() == 3 => {
                    let input = eval_expression(&arr[1], feature)?;
                    let values = eval_expression(&arr[2], feature)?;
                    let Value::Array(members) = values else {
                        return None;
                    };
                    Some(Value::Bool(members.iter().any(|m| values_equal(&input, m))))
                }

                "case" if arr.len() >= 3 => eval_case(arr, feature),

                "coalesce" => {
                    for child in &arr[1..] {
                        let val = eval_expression(child, feature)?;
                        if !val.is_null() {
                            return Some(val);
                        }
                    }
                    Some(Value::Null)
                }

                "step" if arr.len() >= 4 => eval_step(arr, feature),

                "interpolate" if arr.len() >= 6 => eval_interpolate(arr, feature),

                "concat" => {
                    let mut result = String::new();
                    for child in &arr[1..] {
                        let val = eval_expression(child, feature)?;
                        result.push_str(&value_to_string(&val));
                    }
                    Some(Value::String(result))
                }

                "to-string" if arr.len() == 2 => {
                    let val = eval_expression(&arr[1], feature)?;
                    Some(Value::String(value_to_string(&val)))
                }

                "to-number" if arr.len() >= 2 => {
                    for child in &arr[1..] {
                        let val = eval_expression(child, feature)?;
                        if let Some(n) = value_to_number(&val) {
                            return Some(Value::from(n));
                        }
                    }
                    Some(Value::from(0.0))
                }

                "to-boolean" if arr.len() == 2 => {
                    let val = eval_expression(&arr[1], feature)?;
                    Some(Value::Bool(is_truthy(&val)))
                }

                "+" => {
                    let mut sum = 0.0f64;
                    for child in &arr[1..] {
                        let val = eval_expression(child, feature)?;
                        sum += val.as_f64()?;
                    }
                    Some(Value::from(sum))
                }

                "-" if arr.len() == 2 => {
                    let val = eval_expression(&arr[1], feature)?.as_f64()?;
                    Some(Value::from(-val))
                }

                "-" if arr.len() == 3 => {
                    let a = eval_expression(&arr[1], feature)?.as_f64()?;
                    let b = eval_expression(&arr[2], feature)?.as_f64()?;
                    Some(Value::from(a - b))
                }

                "*" => {
                    let mut product = 1.0f64;
                    for child in &arr[1..] {
                        let val = eval_expression(child, feature)?;
                        product *= val.as_f64()?;
                    }
                    Some(Value::from(product))
                }

                "/" if arr.len() == 3 => {
                    let a = eval_expression(&arr[1], feature)?.as_f64()?;
                    let b = eval_expression(&arr[2], feature)?.as_f64()?;
                    Some(Value::from(a / b))
                }

                "%" if arr.len() == 3 => {
                    let a = eval_expression(&arr[1], feature)?.as_f64()?;
                    let b = eval_expression(&arr[2], feature)?.as_f64()?;
                    Some(Value::from(a % b))
                }

                "^" if arr.len() == 3 => {
                    let a = eval_expression(&arr[1], feature)?.as_f64()?;
                    let b = eval_expression(&arr[2], feature)?.as_f64()?;
                    Some(Value::from(a.powf(b)))
                }

                "abs" if arr.len() == 2 => {
                    let val = eval_expression(&arr[1], feature)?.as_f64()?;
                    Some(Value::from(val.abs()))
                }

                "ceil" if arr.len() == 2 => {
                    let val = eval_expression(&arr[1], feature)?.as_f64()?;
                    Some(Value::from(val.ceil()))
                }

                "floor" if arr.len() == 2 => {
                    let val = eval_expression(&arr[1], feature)?.as_f64()?;
                    Some(Value::from(val.floor()))
                }

                "round" if arr.len() == 2 => {
                    let val = eval_expression(&arr[1], feature)?.as_f64()?;
                    Some(Value::from(val.round()))
                }

                "min" => {
                    let mut min = f64::INFINITY;
                    for child in &arr[1..] {
                        let val = eval_expression(child, feature)?.as_f64()?;
                        if val < min {
                            min = val;
                        }
                    }
                    Some(Value::from(min))
                }

                "max" => {
                    let mut max = f64::NEG_INFINITY;
                    for child in &arr[1..] {
                        let val = eval_expression(child, feature)?.as_f64()?;
                        if val > max {
                            max = val;
                        }
                    }
                    Some(Value::from(max))
                }

                "typeof" if arr.len() == 2 => {
                    let val = eval_expression(&arr[1], feature)?;
                    let type_str = match val {
                        Value::Null => "null",
                        Value::Bool(_) => "boolean",
                        Value::Number(_) => "number",
                        Value::String(_) => "string",
                        Value::Array(_) => "array",
                        Value::Object(_) => "object",
                    };
                    Some(Value::String(type_str.to_string()))
                }

                "length" if arr.len() == 2 => {
                    let val = eval_expression(&arr[1], feature)?;
                    match val {
                        Value::String(s) => Some(Value::from(s.len() as u64)),
                        Value::Array(a) => Some(Value::from(a.len() as u64)),
                        _ => None,
                    }
                }

                _ => None,
            }
        }
    }
}

fn eval_match(arr: &[Value], feature: &Feature) -> Option<Value> {
    let input = eval_expression(&arr[1], feature)?;
    let pair_count = (arr.len() - 3) / 2;

    for i in 0..pair_count {
        let label_idx = 2 + i * 2;
        let output_idx = label_idx + 1;
        let label = &arr[label_idx];

        let matches = match label {
            Value::Array(labels) => labels.iter().any(|l| values_equal(&input, l)),
            _ => values_equal(&input, label),
        };

        if matches {
            return eval_expression(&arr[output_idx], feature);
        }
    }

    // Fallback
    eval_expression(arr.last()?, feature)
}

fn eval_case(arr: &[Value], feature: &Feature) -> Option<Value> {
    let pair_count = (arr.len() - 2) / 2;

    for i in 0..pair_count {
        let cond_idx = 1 + i * 2;
        let output_idx = cond_idx + 1;
        let cond = eval_expression(&arr[cond_idx], feature)?;
        if is_truthy(&cond) {
            return eval_expression(&arr[output_idx], feature);
        }
    }

    // Fallback
    eval_expression(arr.last()?, feature)
}

fn eval_step(arr: &[Value], feature: &Feature) -> Option<Value> {
    // ["step", input, default, stop1, out1, stop2, out2, ...]
    let input = eval_expression(&arr[1], feature)?.as_f64()?;
    let default = &arr[2];
    let stop_count = (arr.len() - 3) / 2;

    let mut result = default;
    for i in 0..stop_count {
        let stop_idx = 3 + i * 2;
        let output_idx = stop_idx + 1;
        let stop = arr[stop_idx].as_f64()?;
        if input >= stop {
            result = &arr[output_idx];
        } else {
            break;
        }
    }

    eval_expression(result, feature)
}

fn eval_interpolate(arr: &[Value], feature: &Feature) -> Option<Value> {
    // Only support linear interpolation: ["interpolate", ["linear"], input, stop1, out1, ...]
    let interp_type = arr[1].as_array()?;
    if interp_type.first()?.as_str()? != "linear" {
        return None;
    }

    let input = eval_expression(&arr[2], feature)?.as_f64()?;
    let stop_count = (arr.len() - 3) / 2;

    if stop_count == 0 {
        return None;
    }

    // Collect stops
    let mut stops: Vec<(f64, f64)> = Vec::with_capacity(stop_count);
    for i in 0..stop_count {
        let stop_idx = 3 + i * 2;
        let output_idx = stop_idx + 1;
        let stop = arr[stop_idx].as_f64()?;
        let output = eval_expression(&arr[output_idx], feature)?.as_f64()?;
        stops.push((stop, output));
    }

    // Clamp or interpolate
    if input <= stops[0].0 {
        return Some(Value::from(stops[0].1));
    }
    if input >= stops[stops.len() - 1].0 {
        return Some(Value::from(stops[stops.len() - 1].1));
    }

    for i in 0..stops.len() - 1 {
        let (s0, o0) = stops[i];
        let (s1, o1) = stops[i + 1];
        if input >= s0 && input <= s1 {
            let t = (input - s0) / (s1 - s0);
            return Some(Value::from(o0 + t * (o1 - o0)));
        }
    }

    None
}

#[expect(clippy::float_cmp)]
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => {
            // Compare as f64 for cross-type number equality
            a.as_f64() == b.as_f64()
        }
        (Value::String(a), Value::String(b)) => a == b,
        _ => false,
    }
}

fn compare_values(a: &Value, b: &Value) -> Option<i8> {
    let a = a.as_f64()?;
    let b = b.as_f64()?;
    Some(if a < b {
        -1
    } else if a > b {
        1
    } else {
        0
    })
}

fn is_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(_) | Value::Object(_) => String::new(),
    }
}

fn value_to_number(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse().ok(),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        Value::Null => Some(0.0),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn basic_feature() -> Feature {
        let mut properties = BTreeMap::new();
        properties.insert("class".to_string(), json!("water"));
        properties.insert("rank".to_string(), json!(3));
        Feature {
            properties,
            geometry_type: "Polygon".to_string(),
            id: Some(42),
            zoom: 10.0,
        }
    }

    #[test]
    fn eval_get() {
        let f = basic_feature();
        assert_eq!(
            eval_expression(&json!(["get", "class"]), &f),
            Some(json!("water"))
        );
        assert_eq!(
            eval_expression(&json!(["get", "missing"]), &f),
            Some(Value::Null)
        );
    }

    #[test]
    fn eval_has() {
        let f = basic_feature();
        assert_eq!(
            eval_expression(&json!(["has", "class"]), &f),
            Some(json!(true))
        );
        assert_eq!(
            eval_expression(&json!(["has", "missing"]), &f),
            Some(json!(false))
        );
    }

    #[test]
    fn eval_eq() {
        let f = basic_feature();
        assert_eq!(
            eval_expression(&json!(["==", ["get", "class"], "water"]), &f),
            Some(json!(true))
        );
        assert_eq!(
            eval_expression(&json!(["==", ["get", "class"], "forest"]), &f),
            Some(json!(false))
        );
    }

    #[test]
    fn eval_neq() {
        let f = basic_feature();
        assert_eq!(
            eval_expression(&json!(["!=", ["get", "class"], "water"]), &f),
            Some(json!(false))
        );
    }

    #[test]
    fn eval_comparison() {
        let f = basic_feature();
        assert_eq!(
            eval_expression(&json!(["<", ["get", "rank"], 5]), &f),
            Some(json!(true))
        );
        assert_eq!(
            eval_expression(&json!([">=", ["get", "rank"], 3]), &f),
            Some(json!(true))
        );
    }

    #[test]
    fn eval_all_any() {
        let f = basic_feature();
        assert_eq!(
            eval_expression(
                &json!([
                    "all",
                    ["==", ["get", "class"], "water"],
                    ["<", ["get", "rank"], 5]
                ]),
                &f
            ),
            Some(json!(true))
        );
        assert_eq!(
            eval_expression(
                &json!([
                    "any",
                    ["==", ["get", "class"], "forest"],
                    ["<", ["get", "rank"], 5]
                ]),
                &f
            ),
            Some(json!(true))
        );
    }

    #[test]
    fn eval_not() {
        let f = basic_feature();
        assert_eq!(
            eval_expression(&json!(["!", ["==", ["get", "class"], "forest"]]), &f),
            Some(json!(true))
        );
    }

    #[test]
    fn eval_match() {
        let f = basic_feature();
        assert_eq!(
            eval_expression(
                &json!([
                    "match",
                    ["get", "class"],
                    "water",
                    "#00f",
                    "forest",
                    "#0f0",
                    "#ccc"
                ]),
                &f
            ),
            Some(json!("#00f"))
        );
        // Fallback
        let mut f2 = f.clone();
        f2.properties.insert("class".to_string(), json!("unknown"));
        assert_eq!(
            eval_expression(
                &json!([
                    "match",
                    ["get", "class"],
                    "water",
                    "#00f",
                    "forest",
                    "#0f0",
                    "#ccc"
                ]),
                &f2
            ),
            Some(json!("#ccc"))
        );
    }

    #[test]
    fn eval_match_array_labels() {
        let f = basic_feature();
        assert_eq!(
            eval_expression(
                &json!(["match", ["get", "class"], ["water", "river"], true, false]),
                &f
            ),
            Some(json!(true))
        );
    }

    #[test]
    fn eval_in() {
        let f = basic_feature();
        assert_eq!(
            eval_expression(
                &json!(["in", ["get", "class"], ["literal", ["water", "river"]]]),
                &f
            ),
            Some(json!(true))
        );
        assert_eq!(
            eval_expression(
                &json!(["in", ["get", "class"], ["literal", ["forest", "river"]]]),
                &f
            ),
            Some(json!(false))
        );
    }

    #[test]
    fn eval_case() {
        let f = basic_feature();
        assert_eq!(
            eval_expression(
                &json!(["case", ["==", ["get", "class"], "water"], "#00f", "#ccc"]),
                &f
            ),
            Some(json!("#00f"))
        );
    }

    #[test]
    fn eval_coalesce() {
        let f = basic_feature();
        assert_eq!(
            eval_expression(&json!(["coalesce", ["get", "missing"], "default"]), &f),
            Some(json!("default"))
        );
        assert_eq!(
            eval_expression(&json!(["coalesce", ["get", "class"], "default"]), &f),
            Some(json!("water"))
        );
    }

    #[test]
    fn eval_step() {
        let f = basic_feature();
        assert_eq!(
            eval_expression(
                &json!(["step", ["get", "rank"], "low", 5, "medium", 10, "high"]),
                &f
            ),
            Some(json!("low"))
        );
    }

    #[test]
    fn eval_interpolate_linear() {
        let f = basic_feature();
        let result = eval_expression(
            &json!(["interpolate", ["linear"], ["zoom"], 5, 0.0, 15, 1.0]),
            &f,
        );
        let val = result.unwrap().as_f64().unwrap();
        assert!((val - 0.5).abs() < 1e-9);
    }

    #[test]
    fn eval_concat() {
        let f = basic_feature();
        assert_eq!(
            eval_expression(&json!(["concat", "hello ", ["get", "class"]]), &f),
            Some(json!("hello water"))
        );
    }

    #[test]
    fn eval_arithmetic() {
        let f = basic_feature();
        assert_eq!(
            eval_expression(&json!(["+", 1, 2, 3]), &f),
            Some(json!(6.0))
        );
        assert_eq!(eval_expression(&json!(["*", 2, 3]), &f), Some(json!(6.0)));
        assert_eq!(eval_expression(&json!(["-", 10, 3]), &f), Some(json!(7.0)));
        assert_eq!(eval_expression(&json!(["/", 10, 2]), &f), Some(json!(5.0)));
    }

    #[test]
    fn eval_geometry_type() {
        let f = basic_feature();
        assert_eq!(
            eval_expression(&json!(["geometry-type"]), &f),
            Some(json!("Polygon"))
        );
    }

    #[test]
    fn eval_zoom() {
        let f = basic_feature();
        assert_eq!(eval_expression(&json!(["zoom"]), &f), Some(json!(10.0)));
    }

    #[test]
    fn eval_id() {
        let f = basic_feature();
        assert_eq!(eval_expression(&json!(["id"]), &f), Some(json!(42)));
    }

    #[test]
    fn eval_typeof() {
        let f = basic_feature();
        assert_eq!(
            eval_expression(&json!(["typeof", ["get", "class"]]), &f),
            Some(json!("string"))
        );
        assert_eq!(
            eval_expression(&json!(["typeof", ["get", "rank"]]), &f),
            Some(json!("number"))
        );
    }

    #[test]
    fn eval_to_string() {
        let f = basic_feature();
        assert_eq!(
            eval_expression(&json!(["to-string", 42]), &f),
            Some(json!("42"))
        );
    }

    #[test]
    fn advisory_equivalence_simple() {
        // Simulate: original style has ["==", ["get", "class"], "water"]
        // Advisory encodes "water" → 0
        // Rewritten style has ["==", ["get", "class"], 0]
        // Original feature has class = "water", transformed feature has class = 0

        let original_feature = Feature {
            properties: BTreeMap::from([("class".to_string(), json!("water"))]),
            geometry_type: "Polygon".to_string(),
            id: None,
            zoom: 10.0,
        };

        let transformed_feature = Feature {
            properties: BTreeMap::from([("class".to_string(), json!(0))]),
            geometry_type: "Polygon".to_string(),
            id: None,
            zoom: 10.0,
        };

        let original_expr = json!(["==", ["get", "class"], "water"]);
        let rewritten_expr = json!(["==", ["get", "class"], 0]);

        let original_result = eval_expression(&original_expr, &original_feature);
        let rewritten_result = eval_expression(&rewritten_expr, &transformed_feature);

        assert_eq!(original_result, rewritten_result);
    }

    #[test]
    fn advisory_equivalence_match_fallback() {
        // Original: feature with class="grass" hits fallback
        // After encoding with sentinel: class=2 (sentinel) also hits fallback

        let original_feature = Feature {
            properties: BTreeMap::from([("class".to_string(), json!("grass"))]),
            geometry_type: "Polygon".to_string(),
            id: None,
            zoom: 10.0,
        };

        let transformed_feature = Feature {
            properties: BTreeMap::from([("class".to_string(), json!(2))]),
            geometry_type: "Polygon".to_string(),
            id: None,
            zoom: 10.0,
        };

        let original_expr = json!([
            "match",
            ["get", "class"],
            "water",
            "#00f",
            "forest",
            "#0f0",
            "#ccc"
        ]);
        let rewritten_expr = json!(["match", ["get", "class"], 0, "#00f", 1, "#0f0", "#ccc"]);

        let original_result = eval_expression(&original_expr, &original_feature);
        let rewritten_result = eval_expression(&rewritten_expr, &transformed_feature);

        assert_eq!(original_result, rewritten_result);
        assert_eq!(original_result, Some(json!("#ccc")));
    }
}

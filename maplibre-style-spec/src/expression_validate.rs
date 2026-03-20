//! Expression validation: recursive walk + deserialize against generated syntax enums where
//! they match upstream. Some generated `Vec<Any>` shapes cannot represent nested operators
//! (e.g. `image` inside `coalesce`); those paths rely on recursion instead of serde.

use std::collections::{HashMap, HashSet};

use serde_json::Value;

use crate::mir::Expressions;
use crate::spec::{
    self, Any, Array, ArrayLessTypeLengthGreater, ArrayOfType, Boolean, Collator, Color,
    ColorOrArrayOfColor, Formatted, Image, InterpolationName, Number,
    NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection, Object,
};

/// Type-coercion / assertion operators (`value_i` are arbitrary sub-expressions).
const ASSERT_LIKE: &[&str] = &[
    "boolean",
    "number",
    "string",
    "object",
    "array",
    "to-boolean",
    "to-number",
    "to-string",
    "to-color",
    "typeof",
];

const COMPARISON_OPS: &[&str] = &["<", "<=", ">", ">="];

/// Build `operator -> output-type groups` from the expression preprocessor (same as codegen).
fn operator_to_output_groups(ex: &Expressions) -> HashMap<String, Vec<String>> {
    let mut m: HashMap<String, Vec<String>> = HashMap::new();
    for (output_key, group) in &ex.by_output_type {
        for op in group.variants.keys() {
            m.entry(op.clone()).or_default().push(output_key.clone());
        }
    }
    m
}

/// Convenience: build the operator → groups map once per [`Expressions`] snapshot.
pub fn operator_groups_map(ex: &Expressions) -> HashMap<String, Vec<String>> {
    operator_to_output_groups(ex)
}

fn looks_like_operator_call(v: &Value, known_ops: &HashSet<String>) -> bool {
    let Value::Array(a) = v else {
        return false;
    };
    a.first()
        .and_then(Value::as_str)
        .is_some_and(|h| known_ops.contains(h))
}

fn try_deserialize_output_group(key: &str, expr: &Value) -> Result<(), String> {
    let r: Result<(), serde_json::Error> = match key {
        "Any" => serde_json::from_value::<Any>(expr.clone()).map(|_| ()),
        "Array" => serde_json::from_value::<Array>(expr.clone()).map(|_| ()),
        "ArrayLessTypeLengthGreater" => {
            serde_json::from_value::<ArrayLessTypeLengthGreater>(expr.clone()).map(|_| ())
        }
        // Legacy / alias: typed `array<T>` expressions share the `Array` syntax enum shape.
        "ArrayOfT" => serde_json::from_value::<Array>(expr.clone()).map(|_| ()),
        "ArrayOfType" => serde_json::from_value::<ArrayOfType>(expr.clone()).map(|_| ()),
        "Boolean" => serde_json::from_value::<Boolean>(expr.clone()).map(|_| ()),
        "Collator" => serde_json::from_value::<Collator>(expr.clone()).map(|_| ()),
        "Color" => serde_json::from_value::<Color>(expr.clone()).map(|_| ()),
        "ColorOrArrayOfColor" => {
            serde_json::from_value::<ColorOrArrayOfColor>(expr.clone()).map(|_| ())
        }
        "String" => serde_json::from_value::<spec::String>(expr.clone()).map(|_| ()),
        "Formatted" => serde_json::from_value::<Formatted>(expr.clone()).map(|_| ()),
        "Image" => serde_json::from_value::<Image>(expr.clone()).map(|_| ()),
        "InterpolationName" => {
            serde_json::from_value::<InterpolationName>(expr.clone()).map(|_| ())
        }
        "Number" => serde_json::from_value::<Number>(expr.clone()).map(|_| ()),
        "NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection" => serde_json::from_value::<
            NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection,
        >(expr.clone())
        .map(|_| ()),
        "Object" => serde_json::from_value::<Object>(expr.clone()).map(|_| ()),
        _ => return Err(format!("unmapped expression output group {key:?}")),
    };
    r.map_err(|e| e.to_string())
}

fn try_typed_for_op(
    expr: &Value,
    op: &str,
    op_to_groups: &HashMap<String, Vec<String>>,
) -> Result<(), String> {
    let Some(groups) = op_to_groups.get(op) else {
        return Err(format!("operator {op:?} not in expression spec groups"));
    };
    let mut last_err = String::new();
    for g in groups {
        match try_deserialize_output_group(g, expr) {
            Ok(()) => return Ok(()),
            Err(e) => last_err = e,
        }
    }
    if last_err.is_empty() {
        Err("expression rejected".into())
    } else {
        Err(last_err)
    }
}

fn walk_nested_expr_operand(
    v: &Value,
    op_to_groups: &HashMap<String, Vec<String>>,
    known_ops: &HashSet<String>,
) -> Result<(), String> {
    match v {
        Value::Array(items) => {
            if items.is_empty() {
                return Err("empty sub-array in expression context".into());
            }
            if looks_like_operator_call(v, known_ops) {
                return validate_expression_with_spec(v, op_to_groups, known_ops);
            }
            for item in items {
                walk_nested_expr_operand(item, op_to_groups, known_ops)?;
            }
            Ok(())
        }
        Value::Object(_) => Err("bare JSON object in expression (use [\"literal\", {...}])".into()),
        Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Null => Ok(()),
    }
}

fn is_interpolate_op(op: &str) -> bool {
    op == "interpolate" || op == "interpolate-hcl" || op == "interpolate-lab"
}

/// Loose interpolate check (stops are numeric literals in fixtures; nested expressions recursed).
fn validate_interpolate_chain(
    args: &[Value],
    op_to_groups: &HashMap<String, Vec<String>>,
    known_ops: &HashSet<String>,
) -> Result<(), String> {
    if args.len() < 4 {
        return Err("interpolate: too few arguments".into());
    }
    let n_stops_vals = args.len().saturating_sub(3);
    if !n_stops_vals.is_multiple_of(2) {
        return Err("interpolate: malformed stop/value pairs".into());
    }
    for a in args.iter().skip(1) {
        if looks_like_operator_call(a, known_ops) {
            validate_expression_with_spec(a, op_to_groups, known_ops)?;
        }
    }
    Ok(())
}

/// Deserialize `expr` for upstream compile parity: recurse through assertions and Decisions,
/// use typed serde where it matches JS, and avoid known-broken comparison serde (see `COMPARISON_OPS`).
pub fn validate_expression_with_spec(
    expr: &Value,
    op_to_groups: &HashMap<String, Vec<String>>,
    known_ops: &HashSet<String>,
) -> Result<(), String> {
    match expr {
        Value::String(_) | Value::Number(_) | Value::Bool(_) => Ok(()),
        Value::Object(_) => Ok(()),
        Value::Null => Ok(()),
        Value::Array(args) => {
            if args.is_empty() {
                return Err(
                    "expression array must be non-empty (use [\"literal\", []] for empty array)"
                        .into(),
                );
            }
            let op = args
                .first()
                .and_then(Value::as_str)
                .ok_or_else(|| "expression operator must be a string".to_string())?;

            if !known_ops.contains(op) {
                return Err(format!("unknown expression operator {op:?}"));
            }

            if op == "literal" {
                return if args.len() < 2 {
                    Err("\"literal\" requires at least one value".into())
                } else {
                    Ok(())
                };
            }

            if op == "error" {
                return if args.len() >= 2 {
                    Ok(())
                } else {
                    Err("\"error\" requires a message".into())
                };
            }

            if op == "var" {
                return if args.len() == 2 {
                    Ok(())
                } else {
                    Err("\"var\" expects a single variable name".into())
                };
            }

            if ASSERT_LIKE.contains(&op) {
                if args.len() < 2 {
                    return Err(format!("{op}: expected at least one argument"));
                }
                for a in args.iter().skip(1) {
                    validate_expression_with_spec(a, op_to_groups, known_ops)?;
                }
                return Ok(());
            }

            if is_interpolate_op(op) {
                return validate_interpolate_chain(args, op_to_groups, known_ops);
            }

            if matches!(op, "all" | "any") {
                if args.len() == 1 {
                    return Ok(());
                }
                for a in args.iter().skip(1) {
                    validate_expression_with_spec(a, op_to_groups, known_ops)?;
                }
                return Ok(());
            }

            if op == "coalesce" {
                if args.len() < 2 {
                    return Err("coalesce: requires at least one argument".into());
                }
                for a in args.iter().skip(1) {
                    validate_expression_with_spec(a, op_to_groups, known_ops)?;
                }
                return Ok(());
            }

            if op == "concat" {
                if args.len() == 1 {
                    return Ok(());
                }
                for a in args.iter().skip(1) {
                    walk_nested_expr_operand(a, op_to_groups, known_ops)?;
                }
                return Ok(());
            }

            if op == "index-of" {
                if !(3..=4).contains(&args.len()) {
                    return Err("index-of: expects 2–3 operands plus optional from-index".into());
                }
                for a in args.iter().skip(1) {
                    walk_nested_expr_operand(a, op_to_groups, known_ops)?;
                }
                return Ok(());
            }

            if op == "in" {
                if args.len() < 3 {
                    return Err("\"in\": expects at least needle and haystack".into());
                }
                for a in args.iter().skip(1) {
                    walk_nested_expr_operand(a, op_to_groups, known_ops)?;
                }
                return Ok(());
            }

            if op == "case" {
                if args.len() < 4 {
                    return Err("case: too few arguments".into());
                }
                let tail = &args[1..];
                if tail.len() % 2 == 0 {
                    return Err("case: expected condition/output pairs plus fallback".into());
                }
                let fallback = tail.last().expect("len checked");
                let pairs = &tail[..tail.len() - 1];
                for i in (0..pairs.len()).step_by(2) {
                    validate_expression_with_spec(&pairs[i], op_to_groups, known_ops)?;
                    validate_expression_with_spec(&pairs[i + 1], op_to_groups, known_ops)?;
                }
                validate_expression_with_spec(fallback, op_to_groups, known_ops)?;
                return Ok(());
            }

            if op == "let" {
                if args.len() < 4 {
                    return Err("let: too few arguments".into());
                }
                let tail = &args[1..];
                if tail.len() % 2 == 0 {
                    return Err("let: expected name/value pairs plus body".into());
                }
                let body = tail.last().expect("len checked");
                let binds = &tail[..tail.len() - 1];
                for i in (0..binds.len()).step_by(2) {
                    if !binds[i].is_string() {
                        return Err("let: binding name must be a string".into());
                    }
                    validate_expression_with_spec(&binds[i + 1], op_to_groups, known_ops)?;
                }
                validate_expression_with_spec(body, op_to_groups, known_ops)?;
                return Ok(());
            }

            if op == "match" {
                if args.len() < 4 {
                    return Err("match: too few arguments".into());
                }
                validate_expression_with_spec(&args[1], op_to_groups, known_ops)?;
                let rest = &args[2..];
                let default = rest
                    .last()
                    .ok_or_else(|| "match: missing fallback".to_string())?;
                let pairs = &rest[..rest.len() - 1];
                if pairs.len() % 2 != 0 {
                    return Err("match: label/output pairs must precede fallback".into());
                }
                for i in (0..pairs.len()).step_by(2) {
                    walk_nested_expr_operand(&pairs[i], op_to_groups, known_ops)?;
                    validate_expression_with_spec(&pairs[i + 1], op_to_groups, known_ops)?;
                }
                validate_expression_with_spec(default, op_to_groups, known_ops)?;
                return Ok(());
            }

            if op == "step" {
                if args.len() < 5 {
                    return Err("step: too few arguments".into());
                }
                validate_expression_with_spec(&args[1], op_to_groups, known_ops)?;
                validate_expression_with_spec(&args[2], op_to_groups, known_ops)?;
                let stops = &args[3..];
                if stops.len() % 2 != 0 {
                    return Err("step: stop inputs and outputs must come in pairs".into());
                }
                for i in (0..stops.len()).step_by(2) {
                    walk_nested_expr_operand(&stops[i], op_to_groups, known_ops)?;
                    validate_expression_with_spec(&stops[i + 1], op_to_groups, known_ops)?;
                }
                return Ok(());
            }

            if op == "number-format" {
                if args.len() < 2 {
                    return Err("number-format: too few arguments".into());
                }
                validate_expression_with_spec(&args[1], op_to_groups, known_ops)?;
                if let Some(opts) = args.get(2)
                    && !opts.is_object() {
                        validate_expression_with_spec(opts, op_to_groups, known_ops)?;
                    }
                return Ok(());
            }

            if COMPARISON_OPS.contains(&op) {
                let tail = &args[1..];
                if tail.len() < 2 {
                    return Err(format!("{op}: requires two comparands"));
                }
                for v in tail {
                    walk_nested_expr_operand(v, op_to_groups, known_ops)?;
                }
                return Ok(());
            }

            match op {
                "collator" => {
                    let tail = &args[1..];
                    if tail.len() == 1 && tail[0].is_object() {
                        return Ok(());
                    }
                    for v in tail {
                        walk_nested_expr_operand(v, op_to_groups, known_ops)?;
                    }
                    Ok(())
                }
                "distance" | "within" => {
                    for v in args.iter().skip(1) {
                        if v.is_object() {
                            continue;
                        }
                        walk_nested_expr_operand(v, op_to_groups, known_ops)?;
                    }
                    Ok(())
                }
                "format" => {
                    for v in args.iter().skip(1) {
                        if v.is_object() {
                            continue;
                        }
                        walk_nested_expr_operand(v, op_to_groups, known_ops)?;
                    }
                    Ok(())
                }
                _ => {
                    try_typed_for_op(expr, op, op_to_groups)?;
                    for v in args.iter().skip(1) {
                        walk_nested_expr_operand(v, op_to_groups, known_ops)?;
                    }
                    Ok(())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use std::collections::BTreeMap;

    use super::{operator_groups_map, validate_expression_with_spec};

    use crate::decoder::StyleReference;
    use crate::mir::IntermediateSpec;

    fn setup() -> (HashMap<String, Vec<String>>, HashSet<String>) {
        let v8 = include_str!("../../upstream/src/reference/v8.json");
        let reference: StyleReference = serde_json::from_str(v8).expect("v8.json should parse");
        let spec = IntermediateSpec::from(reference);
        let op_to_groups = operator_groups_map(&spec.expressions);
        let known_ops: HashSet<String> = spec.expressions.operators.keys().cloned().collect();
        (op_to_groups, known_ops)
    }

    #[test]
    fn dump_expression_output_groups() {
        let v8 = include_str!("../../upstream/src/reference/v8.json");
        let reference: StyleReference = serde_json::from_str(v8).expect("v8");
        let spec = IntermediateSpec::from(reference);
        let keys: Vec<_> = spec.expressions.by_output_type.keys().cloned().collect();
        assert!(
            !keys.is_empty(),
            "expected non-empty by_output_type (check v8.json / preprocessor)"
        );
        let mut unknown: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let known: std::collections::HashSet<&'static str> = [
            "Any",
            "Array",
            "ArrayLessTypeLengthGreater",
            "ArrayOfType",
            "Boolean",
            "Collator",
            "Color",
            "ColorOrArrayOfColor",
            "String",
            "Formatted",
            "Image",
            "InterpolationName",
            "Number",
            "NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection",
            "Object",
        ]
        .into_iter()
        .collect();

        for k in &keys {
            if !known.contains(k.as_str()) {
                unknown.insert(k.clone(), vec![]);
            }
        }
        assert!(
            unknown.is_empty(),
            "add try_deserialize arms for: {:?}",
            unknown.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    fn validates_any_case_fallback_structure() {
        let (op_to_groups, known_ops) = setup();
        let expr = serde_json::json!(["case", ["boolean", ["feature-state","hover"], false], 1, 0.5]);
        assert!(validate_expression_with_spec(&expr, &op_to_groups, &known_ops).is_ok());
    }

    #[test]
    fn validates_any_let_binding_structure() {
        let (op_to_groups, known_ops) = setup();
        let expr = serde_json::json!(["let","someNumber",500,["interpolate",["linear"],["var","someNumber"],274,"#edf8e9",1551,"#006d2c"]]);
        assert!(validate_expression_with_spec(&expr, &op_to_groups, &known_ops).is_ok());
    }

    #[test]
    fn validates_any_step_input_can_be_expression() {
        let (op_to_groups, known_ops) = setup();
        let expr = serde_json::json!(["step",["get","point_count"],20,100,30,750,40]);
        assert!(validate_expression_with_spec(&expr, &op_to_groups, &known_ops).is_ok());
    }

    #[test]
    fn validates_comparison_ops_with_nested_get() {
        let (op_to_groups, known_ops) = setup();
        let expr = serde_json::json!(["<",["get","mag"],2]);
        assert!(validate_expression_with_spec(&expr, &op_to_groups, &known_ops).is_ok());
    }

    #[test]
    fn validates_match_labels_can_be_arrays() {
        let (op_to_groups, known_ops) = setup();
        let expr = serde_json::json!(["match",["get","x"],[-2,-1],"negative",0,"zero",[1,2],"positive","otherwise"]);
        assert!(validate_expression_with_spec(&expr, &op_to_groups, &known_ops).is_ok());
    }

    #[test]
    fn validates_number_format_with_bare_options_object() {
        let (op_to_groups, known_ops) = setup();
        let expr = serde_json::json!(["number-format",["get","mag"],{}]);
        assert!(validate_expression_with_spec(&expr, &op_to_groups, &known_ops).is_ok());
    }

    #[test]
    fn validates_number_plus_allows_var_operands() {
        let (op_to_groups, known_ops) = setup();
        let expr = serde_json::json!(["+",["var","x"],2]);
        assert!(validate_expression_with_spec(&expr, &op_to_groups, &known_ops).is_ok());
    }

    #[test]
    fn validates_empty_max_min() {
        let (op_to_groups, known_ops) = setup();
        let max_expr = serde_json::json!(["max"]);
        let min_expr = serde_json::json!(["min"]);
        assert!(validate_expression_with_spec(&max_expr, &op_to_groups, &known_ops).is_ok());
        assert!(validate_expression_with_spec(&min_expr, &op_to_groups, &known_ops).is_ok());
    }
}

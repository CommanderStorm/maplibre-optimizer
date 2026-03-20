//! Expression validation: recursive walk + deserialize against generated syntax enums where
//! they match upstream. Some generated `Vec<Any>` shapes cannot represent nested operators
//! (e.g. `image` inside `coalesce`); those paths rely on recursion instead of serde.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use serde_json::Value;

use crate::decoder::StyleReference;
use crate::mir::{ExprParamType, ExprType, Expressions, IntermediateSpec, LiteralKind};
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

static MIR_SPEC: OnceLock<IntermediateSpec> = OnceLock::new();

fn mir_spec() -> &'static IntermediateSpec {
    MIR_SPEC.get_or_init(|| {
        let v8 = include_str!("../../upstream/src/reference/v8.json");
        let reference: StyleReference =
            serde_json::from_str(v8).expect("v8.json should parse as StyleReference");
        IntermediateSpec::from(reference)
    })
}

#[derive(Debug, Clone, Default)]
struct TypeEnv {
    bindings: HashMap<String, ExprType>,
}

fn resolve_type(t: &ExprType, env: &TypeEnv) -> ExprType {
    match t {
        ExprType::TypeVar(name) => env.bindings.get(name).cloned().unwrap_or_else(|| t.clone()),
        other => other.clone(),
    }
}

fn unify_types(actual: &ExprType, expected: &ExprType, env: &mut TypeEnv) -> Result<(), String> {
    // Treat `any` as a permissive supertype: upstream often succeeds by inserting coercions/assertions.
    if matches!(actual, ExprType::Any) || matches!(expected, ExprType::Any) {
        return Ok(());
    }

    let actual = resolve_type(actual, env);
    let expected = resolve_type(expected, env);

    // Upstream supports coercions between some expression outputs.
    //
    // In particular (see upstream parsing_context.ts):
    // - a `color`/`formatted` expected type can be produced from a `string` value.
    // For parity we treat these as compatible at compile-time.
    match (&actual, &expected) {
        (ExprType::String, ExprType::Color) => return Ok(()),
        (ExprType::String, ExprType::Formatted) => return Ok(()),
        _ => {}
    }

    match (&actual, &expected) {
        (ExprType::TypeVar(tv), other) | (other, ExprType::TypeVar(tv)) => {
            env.bindings.insert(tv.clone(), other.clone());
            Ok(())
        }
        (ExprType::Boolean, ExprType::Boolean)
        | (ExprType::Number, ExprType::Number)
        | (ExprType::String, ExprType::String)
        | (ExprType::Collator, ExprType::Collator)
        | (ExprType::Formatted, ExprType::Formatted)
        | (ExprType::Image, ExprType::Image)
        | (ExprType::Object, ExprType::Object)
        | (ExprType::Color, ExprType::Color)
        | (ExprType::Interpolation, ExprType::Interpolation) => Ok(()),
        (
            ExprType::Array {
                element: a_el,
                length: a_len,
            },
            ExprType::Array {
                element: e_el,
                length: e_len,
            },
        ) => {
            if let Some(required_len) = e_len
                && a_len != &Some(*required_len)
            {
                return Err(format!(
                    "array length mismatch: expected {required_len}, actual {a_len:?}"
                ));
            }

            match (e_el.as_deref(), a_el.as_deref()) {
                (None, _) => Ok(()),
                (Some(e), None) => {
                    if let ExprParamType::TypeVar(tv) = e {
                        env.bindings.insert(tv.clone(), ExprType::Any);
                        Ok(())
                    } else {
                        Err("array element constraint missing".into())
                    }
                }
                (Some(e), Some(a)) => {
                    if a == e {
                        Ok(())
                    } else if let (
                        ExprParamType::Expression(e_ty),
                        ExprParamType::Expression(a_ty),
                    ) = (e, a)
                    {
                        // Propagate scalar coercions down into array element constraints.
                        // Example: `interpolate-*-color-array` passes `["literal", ["white", "black"]]`,
                        // which we infer as `array<string>` but needs to unify with `array<color>`.
                        match (a_ty, e_ty) {
                            (ExprType::String, ExprType::Color) => Ok(()),
                            (ExprType::String, ExprType::Formatted) => Ok(()),
                            _ => Err(format!(
                                "array element constraint mismatch: expected {e:?}, actual {a:?}"
                            )),
                        }
                    } else if let ExprParamType::TypeVar(tv) = e {
                        env.bindings.insert(tv.clone(), param_type_to_expr_type(a));
                        Ok(())
                    } else if let ExprParamType::TypeVar(tv) = a {
                        env.bindings.insert(tv.clone(), param_type_to_expr_type(e));
                        Ok(())
                    } else {
                        Err(format!(
                            "array element constraint mismatch: expected {e:?}, actual {a:?}"
                        ))
                    }
                }
            }
        }
        _ => Err(format!(
            "type mismatch: expected {expected:?}, actual {actual:?}"
        )),
    }
}

fn param_type_to_expr_type(pt: &ExprParamType) -> ExprType {
    match pt {
        ExprParamType::Literal(kind) => match kind {
            LiteralKind::Number => ExprType::Number,
            LiteralKind::String => ExprType::String,
            LiteralKind::GeoJSONObject | LiteralKind::JSONObject => ExprType::Object,
            LiteralKind::JSONArray => ExprType::Array {
                element: None,
                length: None,
            },
        },
        ExprParamType::LiteralAnyOf(_) => ExprType::Any,
        ExprParamType::Expression(t) => t.clone(),
        ExprParamType::ExpressionAnyOf(_) => ExprType::Any,
        ExprParamType::InlineObject(_) => ExprType::Object,
        ExprParamType::TypeVar(tv) => ExprType::TypeVar(tv.clone()),
    }
}

fn validate_literal_kind_value(kind: &LiteralKind, v: &Value) -> Result<(), String> {
    match kind {
        LiteralKind::Number => v
            .is_number()
            .then_some(())
            .ok_or_else(|| "expected number literal".to_string()),
        LiteralKind::String => v
            .is_string()
            .then_some(())
            .ok_or_else(|| "expected string literal".to_string()),
        LiteralKind::GeoJSONObject | LiteralKind::JSONObject => v
            .is_object()
            .then_some(())
            .ok_or_else(|| "expected JSON object literal".to_string()),
        LiteralKind::JSONArray => v
            .is_array()
            .then_some(())
            .ok_or_else(|| "expected JSON array literal".to_string()),
    }
}

fn validate_param_type_value(
    pt: &ExprParamType,
    v: &Value,
    spec: &IntermediateSpec,
    env: &mut TypeEnv,
) -> Result<(), String> {
    match pt {
        ExprParamType::Literal(kind) => validate_literal_kind_value(kind, v),
        ExprParamType::LiteralAnyOf(kinds) => {
            if kinds
                .iter()
                .any(|k| validate_literal_kind_value(k, v).is_ok())
            {
                Ok(())
            } else {
                Err("literal kind mismatch".into())
            }
        }
        ExprParamType::Expression(expected_ty) => {
            let _ = validate_expression_with_mir(v, expected_ty, spec, env)?;
            Ok(())
        }
        ExprParamType::ExpressionAnyOf(options) => {
            for opt in options {
                if validate_param_type_value(opt, v, spec, env).is_ok() {
                    return Ok(());
                }
            }
            Err("expression any-of mismatch".into())
        }
        ExprParamType::InlineObject(schema) => {
            let Value::Object(map) = v else {
                return Err("inline object must be a JSON object".into());
            };
            // Be tolerant of missing fields (defaults) and extra keys (upstream doesn't reject them).
            for (k, field_pt) in schema {
                if let Some(field_val) = map.get(k) {
                    // Upstream `format` treats `text-font` as an array-of-strings font stack,
                    // but v8.json (and thus MIR) models it as a plain `string` override.
                    // Accept literal arrays of strings here to match upstream's compile-time behavior.
                    if k == "text-font"
                        && matches!(field_pt, ExprParamType::Expression(ExprType::String))
                        && matches!(field_val, Value::Array(_))
                    {
                        // If it's the typical `["literal", [...]]` shape, validate element kinds.
                        if let Value::Array(parts) = field_val {
                            let is_literal_call = parts.len() == 2
                                && parts[0].as_str().is_some_and(|op| op == "literal");
                            if is_literal_call
                                && let Value::Array(elems) = &parts[1]
                                && elems.iter().any(|x| !x.is_string())
                            {
                                return Err("text-font must be an array of strings".into());
                            }
                        }
                        validate_expression_with_mir(field_val, &ExprType::Any, spec, env)?;
                        continue;
                    }
                    validate_param_type_value(field_pt, field_val, spec, env)?;
                }
            }
            Ok(())
        }
        ExprParamType::TypeVar(tv) => {
            let expected = ExprType::TypeVar(tv.clone());
            let _ = validate_expression_with_mir(v, &expected, spec, env)?;
            Ok(())
        }
    }
}

fn validate_array_literal_of_typed_values(
    arr: &[Value],
    expected: &ExprType,
    spec: &IntermediateSpec,
    env: &mut TypeEnv,
) -> Result<ExprType, String> {
    let ExprType::Array { element, length } = expected else {
        return Err(
            "internal error: validate_array_literal_of_typed_values called with non-array expected"
                .into(),
        );
    };
    let Some(element) = element else {
        return Err("internal error: validate_array_literal_of_typed_values called without element constraint".into());
    };

    // Only treat arrays as literal values in contexts where v8.json declared `array<... literal>`.
    match element.as_ref() {
        ExprParamType::Literal(_) | ExprParamType::LiteralAnyOf(_) => {}
        _ => return Err("array literal allowed only for array<... literal> contexts".into()),
    }

    if let Some(required_len) = length
        && arr.len() != *required_len
    {
        return Err(format!(
            "array length mismatch: expected {required_len}, actual {}",
            arr.len()
        ));
    }

    for v in arr {
        validate_param_type_value(element.as_ref(), v, spec, env)?;
    }

    let actual = ExprType::Array {
        element: Some(element.clone()),
        length: *length,
    };
    unify_types(&actual, expected, env)?;
    Ok(actual)
}

fn validate_interpolation_type(
    arr: &[Value],
    expected: &ExprType,
    _env: &mut TypeEnv,
) -> Result<ExprType, String> {
    if !matches!(expected, ExprType::Interpolation) {
        return Err(
            "internal error: validate_interpolation_type called with non-interpolation expected"
                .into(),
        );
    }
    if arr.is_empty() {
        return Err("interpolation type must be a non-empty array".into());
    }
    let t0 = arr[0]
        .as_str()
        .ok_or_else(|| "interpolation type discriminator must be a string".to_string())?;

    match t0 {
        "linear" => Ok(ExprType::Interpolation),
        "exponential" => {
            if arr.len() < 2 {
                return Err("exponential interpolation requires a numeric base".into());
            }
            if !arr[1].is_number() {
                return Err("exponential interpolation base must be a number".into());
            }
            Ok(ExprType::Interpolation)
        }
        "cubic-bezier" => {
            if arr.len() != 5 {
                return Err(
                    "cubic-bezier interpolation requires four control point numbers".into(),
                );
            }
            for (i, v) in arr[1..].iter().enumerate() {
                let n = v
                    .as_f64()
                    .ok_or_else(|| format!("cubic-bezier control point #{i} must be a number"))?;
                if !(0.0..=1.0).contains(&n) {
                    return Err(format!(
                        "cubic-bezier control point #{i} must be in [0,1], got {n}"
                    ));
                }
            }
            Ok(ExprType::Interpolation)
        }
        other => Err(format!("unknown interpolation type {other:?}")),
    }
}

fn validate_literal_operator(
    args: &[Value],
    expected: &ExprType,
    env: &mut TypeEnv,
) -> Result<ExprType, String> {
    if args.len() != 1 {
        return Err(format!(
            "\"literal\" requires exactly one argument, got {}",
            args.len()
        ));
    }

    let v = &args[0];
    let actual = match v {
        Value::Null => ExprType::Any,
        Value::Bool(_) => ExprType::Boolean,
        Value::Number(_) => ExprType::Number,
        Value::String(_) => ExprType::String,
        Value::Array(a) => {
            // Infer element type for literal array values so generic operators like
            // `at` can propagate the right type variable.
            let element = if a.is_empty() {
                None
            } else if a.iter().all(|x| x.is_string()) {
                Some(Box::new(ExprParamType::Expression(ExprType::String)))
            } else if a.iter().all(|x| x.is_number()) {
                Some(Box::new(ExprParamType::Expression(ExprType::Number)))
            } else if a.iter().all(|x| x.is_boolean()) {
                Some(Box::new(ExprParamType::Expression(ExprType::Boolean)))
            } else {
                None
            };

            ExprType::Array {
                element,
                length: Some(a.len()),
            }
        }
        Value::Object(_) => ExprType::Object,
    };

    unify_types(&actual, expected, env)?;
    Ok(actual)
}

fn match_overload_params(
    params: &crate::mir::OverloadParams,
    args: &[Value],
    spec: &IntermediateSpec,
    env: &mut TypeEnv,
) -> Result<(), String> {
    match params {
        crate::mir::OverloadParams::Fixed(ps) => {
            if args.len() != ps.len() {
                return Err(format!(
                    "arity mismatch: expected {} args, got {}",
                    ps.len(),
                    args.len()
                ));
            }
            for (p, a) in ps.iter().zip(args.iter()) {
                validate_param_type_value(&p.r#type, a, spec, env)?;
            }
            Ok(())
        }
        crate::mir::OverloadParams::WithOptional { required, optional } => {
            if args.len() < required.len() || args.len() > required.len() + optional.len() {
                return Err("optional arity mismatch".into());
            }

            for (p, a) in required.iter().zip(args.iter().take(required.len())) {
                validate_param_type_value(&p.r#type, a, spec, env)?;
            }
            let provided_opt = args.len() - required.len();
            for (p, a) in optional
                .iter()
                .take(provided_opt)
                .zip(args[required.len()..].iter())
            {
                validate_param_type_value(&p.r#type, a, spec, env)?;
            }
            Ok(())
        }
        crate::mir::OverloadParams::Variadic {
            prefix,
            repeating,
            suffix,
        } => {
            if repeating.is_empty() {
                return Err("internal error: variadic repeating unit is empty".into());
            }
            if args.len() < prefix.len() {
                return Err("variadic arity too small".into());
            }

            let unit = repeating.len();
            let min_with_suffix = prefix.len() + suffix.len();

            // Layout A: suffix always present.
            if args.len() >= min_with_suffix {
                let rem = args.len() - prefix.len() - suffix.len();
                if !rem.is_multiple_of(unit) {
                    return Err("variadic arity mismatch".into());
                }
                let repeats = rem / unit;

                for (p, a) in prefix.iter().zip(args.iter().take(prefix.len())) {
                    validate_param_type_value(&p.r#type, a, spec, env)?;
                }

                for rep_idx in 0..repeats {
                    let base = prefix.len() + rep_idx * unit;
                    for (p, a) in repeating.iter().zip(args[base..base + unit].iter()) {
                        validate_param_type_value(&p.r#type, a, spec, env)?;
                    }
                }

                let suffix_start = prefix.len() + repeats * unit;
                for (p, a) in suffix.iter().zip(args[suffix_start..].iter()) {
                    validate_param_type_value(&p.r#type, a, spec, env)?;
                }
                return Ok(());
            }

            // Layout B: suffix omitted (only valid when we don't have enough args to fill it).
            let rem = args.len() - prefix.len();
            if !rem.is_multiple_of(unit) {
                return Err("variadic arity mismatch".into());
            }
            let repeats = rem / unit;

            for (p, a) in prefix.iter().zip(args.iter().take(prefix.len())) {
                validate_param_type_value(&p.r#type, a, spec, env)?;
            }

            for rep_idx in 0..repeats {
                let base = prefix.len() + rep_idx * unit;
                for (p, a) in repeating.iter().zip(args[base..base + unit].iter()) {
                    validate_param_type_value(&p.r#type, a, spec, env)?;
                }
            }

            Ok(())
        }
    }
}

fn validate_operator_call(
    operator: &crate::mir::ExpressionOperator,
    args: &[Value],
    expected: &ExprType,
    spec: &IntermediateSpec,
    env: &mut TypeEnv,
) -> Result<ExprType, String> {
    let mut last_err = String::new();

    for overload in &operator.overloads {
        let mut attempt_env = env.clone();
        match match_overload_params(&overload.params, args, spec, &mut attempt_env) {
            Ok(()) => {
                let out = overload.output.clone();
                if unify_types(&out, expected, &mut attempt_env).is_ok() {
                    *env = attempt_env;
                    return Ok(resolve_type(&out, env));
                }
                last_err = "operator overload output type mismatch".into();
            }
            Err(e) => last_err = e,
        }
    }

    if last_err.is_empty() {
        Err("expression rejected".into())
    } else {
        Err(last_err)
    }
}

fn validate_expression_with_mir(
    expr: &Value,
    expected: &ExprType,
    spec: &IntermediateSpec,
    env: &mut TypeEnv,
) -> Result<ExprType, String> {
    match expr {
        Value::Null => {
            unify_types(&ExprType::Any, expected, env)?;
            Ok(ExprType::Any)
        }
        Value::Bool(_) => {
            let actual = ExprType::Boolean;
            unify_types(&actual, expected, env)?;
            Ok(resolve_type(&actual, env))
        }
        Value::Number(_) => {
            let actual = ExprType::Number;
            unify_types(&actual, expected, env)?;
            Ok(resolve_type(&actual, env))
        }
        Value::String(_) => {
            let actual = ExprType::String;
            unify_types(&actual, expected, env)?;
            Ok(resolve_type(&actual, env))
        }
        Value::Object(_) => {
            // Some v8.json parameters are modeled as expression-output `object` even
            // though upstream treats bare objects as raw values for those operators.
            // Allow bare objects when the expected type can accept them.
            let allow = matches!(resolve_type(expected, env), ExprType::Object)
                || matches!(expected, ExprType::Object)
                || matches!(expected, ExprType::Any)
                || matches!(expected, ExprType::TypeVar(_));

            if !allow {
                return Err(
                    "Bare JSON object invalid in expression. Use [\"literal\", {...}].".into(),
                );
            }

            let actual = ExprType::Object;
            unify_types(&actual, expected, env)?;
            Ok(actual)
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                return Err("expression array must be non-empty".into());
            }

            let resolved_expected = resolve_type(expected, env);
            let is_interpolation_expected = matches!(resolved_expected, ExprType::Interpolation)
                || matches!(expected, ExprType::Interpolation)
                || matches!(expected, ExprType::TypeVar(tv) if tv == "interpolation");
            if is_interpolation_expected {
                let actual = validate_interpolation_type(arr, &ExprType::Interpolation, env)?;
                unify_types(&actual, expected, env)?;
                return Ok(actual);
            }

            // Typed raw literal arrays used by `match` labels: v8.json declares them as `array<... literal>`.
            if let ExprType::Array { element, .. } = expected
                && let Some(element) = element
                && matches!(
                    element.as_ref(),
                    ExprParamType::Literal(_) | ExprParamType::LiteralAnyOf(_)
                )
            {
                return validate_array_literal_of_typed_values(arr, expected, spec, env);
            }

            // Expression operator call.
            let op = arr[0]
                .as_str()
                .ok_or_else(|| "expression operator must be a string".to_string())?;
            let args = &arr[1..];

            if op == "array" {
                // Upstream `["array", ...]` is an *assertion with fallback candidates*:
                // - `["array", <value>]`
                // - `["array", <type>, <value>]`
                // - `["array", <type>, <length>, <value> ...]`
                // where `<value> ...` are runtime candidates; compile-time only validates
                // the optional item-type / length shape.
                if args.is_empty() {
                    return Err("\"array\" requires at least one argument".into());
                }

                // Parse `itemType` + optional `length` exactly like upstream.
                let mut idx = 0usize;
                let element: Option<Box<ExprParamType>> = if args.len() > 1 {
                    let item_type = args[0]
                        .as_str()
                        .ok_or_else(|| "array item type must be a string literal".to_string())?;
                    let item_expr_ty = match item_type {
                        "string" => ExprType::String,
                        "number" => ExprType::Number,
                        "boolean" => ExprType::Boolean,
                        _ => {
                            return Err(
                                "The item type argument of \"array\" must be one of string, number, boolean"
                                    .into(),
                            );
                        }
                    };
                    idx = 1;
                    Some(Box::new(ExprParamType::Expression(item_expr_ty)))
                } else {
                    // Model the omitted item type (`ValueType`) as a fresh type variable
                    // so it can unify with any expected `array<...>` element type.
                    Some(Box::new(ExprParamType::TypeVar("__array_item".into())))
                };

                let mut length: Option<usize> = None;
                if args.len() > 2 {
                    let len_v = &args[1];
                    if !len_v.is_null() {
                        let n = len_v.as_f64().ok_or_else(|| {
                            "The length argument to \"array\" must be a number literal".to_string()
                        })?;
                        if n < 0.0 || n != n.floor() {
                            return Err(
                                "The length argument to \"array\" must be a positive integer literal"
                                    .into(),
                            );
                        }
                        length = Some(n as usize);
                    }
                    idx = 2;
                }

                let candidates = &args[idx..];
                if candidates.is_empty() {
                    return Err("\"array\" requires at least one value candidate".into());
                }

                // Upstream parses each candidate as a generic `ValueType` (compile-time),
                // and only applies subtype checks at evaluation time with fallback.
                for c in candidates {
                    validate_expression_with_mir(c, &ExprType::Any, spec, env)?;
                }

                let actual = ExprType::Array { element, length };
                unify_types(&actual, expected, env)?;
                return Ok(actual);
            }

            if op == "error" {
                // Upstream uses `["error", ...]` in short-circuiting tests to ensure
                // that the presence of an error node doesn't prevent compilation.
                if args.is_empty() {
                    return Err("\"error\" requires at least one argument".into());
                }
                let actual = ExprType::Any;
                unify_types(&actual, expected, env)?;
                return Ok(actual);
            }

            if op == "format" {
                // `format` uses a bespoke repeating structure:
                //   (input, style_overrides?)*   with style-override represented as a bare JSON object.
                //
                // v8.json models this via variadic overloads with optional parameters, but our generic
                // variadic matcher doesn't implement the same optionality semantics. Validate the
                // upstream sequence shape directly instead.
                let operator = spec
                    .expressions
                    .operators
                    .get(op)
                    .ok_or_else(|| format!("unknown expression operator {op:?}"))?;

                let input_pt = operator
                    .parameters
                    .iter()
                    .find(|p| p.name == "input_i")
                    .map(|p| &p.r#type)
                    .ok_or_else(|| "\"format\" missing input_i param type".to_string())?;
                let style_overrides_pt = operator
                    .parameters
                    .iter()
                    .find(|p| p.name == "style_overrides_i")
                    .map(|p| &p.r#type)
                    .ok_or_else(|| "\"format\" missing style_overrides_i param type".to_string())?;

                if args.is_empty() {
                    return Err("\"format\" requires at least one input".into());
                }

                let mut next_token_may_be_style_obj = false;
                for a in args {
                    if next_token_may_be_style_obj && matches!(a, Value::Object(_)) {
                        validate_param_type_value(style_overrides_pt, a, spec, env)?;
                        next_token_may_be_style_obj = false;
                    } else {
                        validate_param_type_value(input_pt, a, spec, env)?;
                        next_token_may_be_style_obj = true;
                    }
                }

                let actual = ExprType::Formatted;
                unify_types(&actual, expected, env)?;
                return Ok(actual);
            }

            if op == "literal" {
                let actual = validate_literal_operator(args, expected, env)?;
                return Ok(actual);
            }

            let Some(operator) = spec.expressions.operators.get(op) else {
                return Err(format!("unknown expression operator {op:?}"));
            };
            validate_operator_call(operator, args, expected, spec, env)
        }
    }
}

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
    _op_to_groups: &HashMap<String, Vec<String>>,
    _known_ops: &HashSet<String>,
) -> Result<(), String> {
    let spec = mir_spec();
    let mut env = TypeEnv::default();
    validate_expression_with_mir(expr, &ExprType::Any, spec, &mut env).map(|_| ())
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap, HashSet};

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
        let expr = serde_json::json!([
            "case",
            ["boolean", ["feature-state", "hover"], false],
            1,
            0.5
        ]);
        assert!(validate_expression_with_spec(&expr, &op_to_groups, &known_ops).is_ok());
    }

    #[test]
    fn validates_any_let_binding_structure() {
        let (op_to_groups, known_ops) = setup();
        let expr = serde_json::json!([
            "let",
            "someNumber",
            500,
            [
                "interpolate",
                ["linear"],
                ["var", "someNumber"],
                274,
                "#edf8e9",
                1551,
                "#006d2c"
            ]
        ]);
        assert!(validate_expression_with_spec(&expr, &op_to_groups, &known_ops).is_ok());
    }

    #[test]
    fn validates_any_step_input_can_be_expression() {
        let (op_to_groups, known_ops) = setup();
        let expr = serde_json::json!(["step", ["get", "point_count"], 20, 100, 30, 750, 40]);
        assert!(validate_expression_with_spec(&expr, &op_to_groups, &known_ops).is_ok());
    }

    #[test]
    fn validates_comparison_ops_with_nested_get() {
        let (op_to_groups, known_ops) = setup();
        let expr = serde_json::json!(["<", ["get", "mag"], 2]);
        assert!(validate_expression_with_spec(&expr, &op_to_groups, &known_ops).is_ok());
    }

    #[test]
    fn validates_match_labels_can_be_arrays() {
        let (op_to_groups, known_ops) = setup();
        let expr = serde_json::json!([
            "match",
            ["get", "x"],
            [-2, -1],
            "negative",
            0,
            "zero",
            [1, 2],
            "positive",
            "otherwise"
        ]);
        assert!(validate_expression_with_spec(&expr, &op_to_groups, &known_ops).is_ok());
    }

    #[test]
    fn validates_number_format_with_bare_options_object() {
        let (op_to_groups, known_ops) = setup();
        let expr = serde_json::json!(["number-format", ["get", "mag"], {}]);
        assert!(validate_expression_with_spec(&expr, &op_to_groups, &known_ops).is_ok());
    }

    #[test]
    fn validates_number_plus_allows_var_operands() {
        let (op_to_groups, known_ops) = setup();
        let expr = serde_json::json!(["+", ["var", "x"], 2]);
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

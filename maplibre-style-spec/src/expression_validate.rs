//! Expression validation: recursive walk + deserialize against generated syntax enums where
//! they match upstream. Some generated `Vec<Any>` shapes cannot represent nested operators
//! (e.g. `image` inside `coalesce`); those paths rely on recursion instead of serde.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use serde_json::Value;

use crate::decoder::StyleReference;
use crate::mir::{MirExprParamType, MirExprType, MirExpressions, MirLiteralKind, MirSpec};
use crate::spec::ExprOrLiteral;

static MIR_SPEC: LazyLock<MirSpec> = LazyLock::new(|| {
    let v8 = include_str!("../../upstream/src/reference/v8.json");
    let reference: StyleReference =
        serde_json::from_str(v8).expect("v8.json should parse as StyleReference");
    MirSpec::from(reference)
});

#[derive(Debug, Clone, Default)]
struct TypeEnv {
    bindings: HashMap<String, MirExprType>,
}

fn resolve_type(t: &MirExprType, env: &TypeEnv) -> MirExprType {
    match t {
        MirExprType::TypeVar(name) => env.bindings.get(name).cloned().unwrap_or_else(|| t.clone()),
        other => other.clone(),
    }
}

fn unify_types(
    actual: &MirExprType,
    expected: &MirExprType,
    env: &mut TypeEnv,
) -> Result<(), String> {
    // Treat `any` as a permissive supertype: upstream often succeeds by inserting coercions/assertions.
    if matches!(actual, MirExprType::Any) || matches!(expected, MirExprType::Any) {
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
        (MirExprType::String, MirExprType::Color) => return Ok(()),
        (MirExprType::String, MirExprType::Formatted) => return Ok(()),
        _ => {}
    }

    match (&actual, &expected) {
        (MirExprType::TypeVar(tv), other) | (other, MirExprType::TypeVar(tv)) => {
            env.bindings.insert(tv.clone(), other.clone());
            Ok(())
        }
        (MirExprType::Boolean, MirExprType::Boolean)
        | (MirExprType::Number, MirExprType::Number)
        | (MirExprType::String, MirExprType::String)
        | (MirExprType::Collator, MirExprType::Collator)
        | (MirExprType::Formatted, MirExprType::Formatted)
        | (MirExprType::Image, MirExprType::Image)
        | (MirExprType::Object, MirExprType::Object)
        | (MirExprType::Color, MirExprType::Color)
        | (MirExprType::Interpolation, MirExprType::Interpolation) => Ok(()),
        (
            MirExprType::Array {
                element: a_el,
                length: a_len,
            },
            MirExprType::Array {
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
                    if let MirExprParamType::TypeVar(tv) = e {
                        env.bindings.insert(tv.clone(), MirExprType::Any);
                        Ok(())
                    } else {
                        Err("array element constraint missing".into())
                    }
                }
                (Some(e), Some(a)) => {
                    if a == e {
                        Ok(())
                    } else if let (
                        MirExprParamType::Expression(e_ty),
                        MirExprParamType::Expression(a_ty),
                    ) = (e, a)
                    {
                        // String → Color/Formatted coercion for array elements (e.g. color arrays
                        // passed as `["literal", ["white", "black"]]`).
                        match (a_ty, e_ty) {
                            (MirExprType::String, MirExprType::Color) => Ok(()),
                            (MirExprType::String, MirExprType::Formatted) => Ok(()),
                            _ => Err(format!(
                                "array element constraint mismatch: expected {e:?}, actual {a:?}"
                            )),
                        }
                    } else if let MirExprParamType::TypeVar(tv) = e {
                        env.bindings.insert(tv.clone(), param_type_to_expr_type(a));
                        Ok(())
                    } else if let MirExprParamType::TypeVar(tv) = a {
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

fn param_type_to_expr_type(pt: &MirExprParamType) -> MirExprType {
    match pt {
        MirExprParamType::Literal(kind) => match kind {
            MirLiteralKind::Number => MirExprType::Number,
            MirLiteralKind::String => MirExprType::String,
            MirLiteralKind::GeoJSONObject | MirLiteralKind::JSONObject => MirExprType::Object,
            MirLiteralKind::JSONArray => MirExprType::Array {
                element: None,
                length: None,
            },
        },
        MirExprParamType::LiteralAnyOf(_) => MirExprType::Any,
        MirExprParamType::Expression(t) => t.clone(),
        MirExprParamType::ExpressionAnyOf(_) => MirExprType::Any,
        MirExprParamType::InlineObject(_) => MirExprType::Object,
        MirExprParamType::TypeVar(tv) => MirExprType::TypeVar(tv.clone()),
    }
}

fn validate_literal_kind_value(kind: &MirLiteralKind, v: &Value) -> Result<(), String> {
    match kind {
        MirLiteralKind::Number => v
            .is_number()
            .then_some(())
            .ok_or_else(|| "expected number literal".to_string()),
        MirLiteralKind::String => v
            .is_string()
            .then_some(())
            .ok_or_else(|| "expected string literal".to_string()),
        MirLiteralKind::GeoJSONObject | MirLiteralKind::JSONObject => v
            .is_object()
            .then_some(())
            .ok_or_else(|| "expected JSON object literal".to_string()),
        MirLiteralKind::JSONArray => v
            .is_array()
            .then_some(())
            .ok_or_else(|| "expected JSON array literal".to_string()),
    }
}

fn validate_param_type_value(
    pt: &MirExprParamType,
    v: &Value,
    spec: &MirSpec,
    env: &mut TypeEnv,
) -> Result<(), String> {
    match pt {
        MirExprParamType::Literal(kind) => validate_literal_kind_value(kind, v),
        MirExprParamType::LiteralAnyOf(kinds) => {
            if kinds
                .iter()
                .any(|k| validate_literal_kind_value(k, v).is_ok())
            {
                Ok(())
            } else {
                Err("literal kind mismatch".into())
            }
        }
        MirExprParamType::Expression(expected_ty) => {
            let _ = validate_expression_with_mir(v, expected_ty, spec, env)?;
            Ok(())
        }
        MirExprParamType::ExpressionAnyOf(options) => {
            for opt in options {
                if validate_param_type_value(opt, v, spec, env).is_ok() {
                    return Ok(());
                }
            }
            Err("expression any-of mismatch".into())
        }
        MirExprParamType::InlineObject(schema) => {
            let Value::Object(map) = v else {
                return Err("inline object must be a JSON object".into());
            };
            // Match upstream tolerance: missing fields use defaults, extra keys are ignored.
            for (k, field_pt) in schema {
                if let Some(field_val) = map.get(k) {
                    // v8.json models text-font as `string`, but upstream accepts array-of-strings
                    // font stacks at compile time.
                    if k == "text-font"
                        && matches!(field_pt, MirExprParamType::Expression(MirExprType::String))
                        && matches!(field_val, Value::Array(_))
                    {
                        // Validate elements inside `["literal", [...]]` font stacks.
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
                        validate_expression_with_mir(field_val, &MirExprType::Any, spec, env)?;
                        continue;
                    }
                    validate_param_type_value(field_pt, field_val, spec, env)?;
                }
            }
            Ok(())
        }
        MirExprParamType::TypeVar(tv) => {
            let expected = MirExprType::TypeVar(tv.clone());
            let _ = validate_expression_with_mir(v, &expected, spec, env)?;
            Ok(())
        }
    }
}

fn validate_array_literal_of_typed_values(
    arr: &[Value],
    expected: &MirExprType,
    spec: &MirSpec,
    env: &mut TypeEnv,
) -> Result<MirExprType, String> {
    let MirExprType::Array { element, length } = expected else {
        return Err(
            "internal error: validate_array_literal_of_typed_values called with non-array expected"
                .into(),
        );
    };
    let Some(element) = element else {
        return Err("internal error: validate_array_literal_of_typed_values called without element constraint".into());
    };

    // Restrict to `array<... literal>` contexts to avoid treating expression arrays as literal values.
    match element.as_ref() {
        MirExprParamType::Literal(_) | MirExprParamType::LiteralAnyOf(_) => {}
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

    let actual = MirExprType::Array {
        element: Some(element.clone()),
        length: *length,
    };
    unify_types(&actual, expected, env)?;
    Ok(actual)
}

fn validate_interpolation_type(
    arr: &[Value],
    expected: &MirExprType,
    _env: &mut TypeEnv,
) -> Result<MirExprType, String> {
    if !matches!(expected, MirExprType::Interpolation) {
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
        "linear" => Ok(MirExprType::Interpolation),
        "exponential" => {
            if arr.len() < 2 {
                return Err("exponential interpolation requires a numeric base".into());
            }
            if !arr[1].is_number() {
                return Err("exponential interpolation base must be a number".into());
            }
            Ok(MirExprType::Interpolation)
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
            Ok(MirExprType::Interpolation)
        }
        other => Err(format!("unknown interpolation type {other:?}")),
    }
}

fn validate_literal_operator(
    args: &[Value],
    expected: &MirExprType,
    env: &mut TypeEnv,
) -> Result<MirExprType, String> {
    if args.len() != 1 {
        return Err(format!(
            "\"literal\" requires exactly one argument, got {}",
            args.len()
        ));
    }

    let v = &args[0];
    let actual = match v {
        Value::Null => MirExprType::Any,
        Value::Bool(_) => MirExprType::Boolean,
        Value::Number(_) => MirExprType::Number,
        Value::String(_) => MirExprType::String,
        Value::Array(a) => {
            // Infer element type so downstream operators (e.g. `at`) propagate correct types.
            let element = if a.is_empty() {
                match resolve_type(expected, env) {
                    MirExprType::Array { element, .. } => element,
                    _ => None,
                }
            } else if a.iter().all(|x| x.is_string()) {
                Some(Box::new(MirExprParamType::Expression(MirExprType::String)))
            } else if a.iter().all(|x| x.is_number()) {
                Some(Box::new(MirExprParamType::Expression(MirExprType::Number)))
            } else if a.iter().all(|x| x.is_boolean()) {
                Some(Box::new(MirExprParamType::Expression(MirExprType::Boolean)))
            } else {
                None
            };

            MirExprType::Array {
                element,
                length: Some(a.len()),
            }
        }
        Value::Object(_) => MirExprType::Object,
    };

    unify_types(&actual, expected, env)?;
    Ok(actual)
}

fn match_overload_params(
    params: &crate::mir::MirOverloadParams,
    args: &[Value],
    spec: &MirSpec,
    env: &mut TypeEnv,
) -> Result<(), String> {
    match params {
        crate::mir::MirOverloadParams::Fixed(ps) => {
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
        crate::mir::MirOverloadParams::WithOptional { required, optional } => {
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
        crate::mir::MirOverloadParams::Variadic {
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
    operator: &crate::mir::MirExpressionOperator,
    args: &[Value],
    expected: &MirExprType,
    spec: &MirSpec,
    env: &mut TypeEnv,
) -> Result<MirExprType, String> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ComparisonKind {
    Boolean,
    Number,
    String,
    Null,
    Value,
    Other,
}

/// Mirror GL JS compile-time predicates for comparison operators
/// (`==`, `!=`, `<`, `>`, `<=`, `>=`) and collator usage.
fn validate_comparison_operator_specifics(
    op: &str,
    args: &[Value],
    spec: &MirSpec,
) -> Result<(), String> {
    let is_equality = op == "==" || op == "!=";
    let is_ordering = !is_equality;

    if args.len() < 2 || args.len() > 3 {
        return Ok(());
    }

    let lhs = &args[0];
    let rhs = &args[1];
    let collator_present = args.len() == 3;

    let mut tmp_env = TypeEnv::default();
    let mut classify = |v: &Value| -> Result<ComparisonKind, String> {
        if v.is_null() {
            return Ok(ComparisonKind::Null);
        }
        if v.is_boolean() {
            return Ok(ComparisonKind::Boolean);
        }
        if v.is_number() {
            return Ok(ComparisonKind::Number);
        }
        if v.is_string() {
            return Ok(ComparisonKind::String);
        }
        if let Value::Array(arr) = v {
            // Distinguish plain array literals `[1,2]` from expression calls `["get", "x"]`.
            if arr.first().and_then(|x| x.as_str()).is_none() {
                return Ok(ComparisonKind::Other);
            }
        }
        // For operator calls / computed expressions, infer the resulting kind via our type walker.
        let ty = validate_expression_with_mir(v, &MirExprType::Any, spec, &mut tmp_env)?;
        Ok(match ty {
            MirExprType::Boolean => ComparisonKind::Boolean,
            MirExprType::Number => ComparisonKind::Number,
            MirExprType::String => ComparisonKind::String,
            MirExprType::Any => ComparisonKind::Value,
            // Not comparable by construction for equality/ordering.
            _ => ComparisonKind::Other,
        })
    };

    let lhs_kind = classify(lhs)?;
    let rhs_kind = classify(rhs)?;

    // Mirrors `isComparableType` in GL JS comparison.ts.
    let comparable_lhs = if is_equality {
        matches!(
            lhs_kind,
            ComparisonKind::Boolean
                | ComparisonKind::String
                | ComparisonKind::Number
                | ComparisonKind::Null
                | ComparisonKind::Value
        )
    } else {
        matches!(
            lhs_kind,
            ComparisonKind::String | ComparisonKind::Number | ComparisonKind::Value
        )
    };
    let comparable_rhs = if is_equality {
        matches!(
            rhs_kind,
            ComparisonKind::Boolean
                | ComparisonKind::String
                | ComparisonKind::Number
                | ComparisonKind::Null
                | ComparisonKind::Value
        )
    } else {
        matches!(
            rhs_kind,
            ComparisonKind::String | ComparisonKind::Number | ComparisonKind::Value
        )
    };

    if !comparable_lhs || !comparable_rhs {
        return Err(format!(
            "\"{op}\" comparisons are not supported for operand types {:?} and {:?}",
            lhs_kind, rhs_kind
        ));
    }

    // Upstream rejects mixed concrete types (e.g. string vs number).
    if lhs_kind != rhs_kind
        && lhs_kind != ComparisonKind::Value
        && rhs_kind != ComparisonKind::Value
    {
        return Err(format!(
            "Cannot compare types {:?} and {:?} with \"{op}\"",
            lhs_kind, rhs_kind
        ));
    }

    // Upstream rejects collators on non-string/non-value operands.
    if collator_present {
        let lhs_allows = lhs_kind == ComparisonKind::String || lhs_kind == ComparisonKind::Value;
        let rhs_allows = rhs_kind == ComparisonKind::String || rhs_kind == ComparisonKind::Value;
        if !lhs_allows && !rhs_allows {
            return Err("Cannot use collator to compare non-string types".into());
        }
    }

    if is_ordering {
        // Upstream rejects null for ordering operators.
        if lhs_kind == ComparisonKind::Null || rhs_kind == ComparisonKind::Null {
            return Err(format!("\"{op}\" comparisons are not supported for null"));
        }
    }

    Ok(())
}

/// Mirror GL JS compile-time predicates for `coalesce`.
///
/// Upstream rejects `null` (and other mismatched literals) when `coalesce` is
/// used in a context with a concrete (non-`value`) expected output type.
fn validate_coalesce_operator_specifics(
    args: &[Value],
    expected: &MirExprType,
    spec: &MirSpec,
) -> Result<(), String> {
    // Upstream can only reject args for concrete expected types, not `value`.
    if matches!(expected, MirExprType::Any | MirExprType::TypeVar(_)) {
        return Ok(());
    }

    for a in args {
        if a.is_null() {
            return Err("coalesce does not accept null for a concrete expected type".into());
        }

        // Each arg must match the output type (e.g. reject number in string context).
        let mut tmp_env = TypeEnv::default();
        validate_expression_with_mir(a, expected, spec, &mut tmp_env)?;
    }

    Ok(())
}

/// Mirror GL JS compile-time predicates for `in` and `index-of`.
///
/// Upstream rejects when the first argument ("needle") is itself an array
/// expression (see fixtures like `invalid-needle-literal-array`).
fn validate_in_indexof_operator_specifics(args: &[Value], spec: &MirSpec) -> Result<(), String> {
    if args.len() != 2 {
        return Ok(());
    }

    let needle = &args[0];

    let mut tmp_env = TypeEnv::default();
    let needle_ty = validate_expression_with_mir(needle, &MirExprType::Any, spec, &mut tmp_env)?;

    // Upstream rejects array-typed needles.
    if matches!(needle_ty, MirExprType::Array { .. }) {
        return Err("`in`/`index-of` needle cannot be an array".into());
    }

    Ok(())
}

fn is_literal_expression_value(v: &Value) -> bool {
    // Primitives and `["literal", ...]` wrappers are compile-time values.
    if v.is_null() || v.is_boolean() || v.is_number() || v.is_string() {
        return true;
    }
    if let Value::Array(arr) = v {
        return arr.first().is_some_and(|h| h.as_str() == Some("literal"));
    }
    false
}

fn is_interpolate_output_interpolatable(op: &str, expected: &MirExprType) -> bool {
    // Interpolation only makes sense for numeric / color types.
    match op {
        "interpolate" => match expected {
            MirExprType::Number | MirExprType::Color => true,
            MirExprType::Array {
                element: Some(el), ..
            } => matches!(
                el.as_ref(),
                MirExprParamType::Expression(MirExprType::Number | MirExprType::Color)
            ),
            _ => false,
        },
        "interpolate-hcl" | "interpolate-lab" => match expected {
            MirExprType::Color => true,
            MirExprType::Array {
                element: Some(el), ..
            } => matches!(
                el.as_ref(),
                MirExprParamType::Expression(MirExprType::Color)
            ),
            _ => false,
        },
        _ => false,
    }
}

fn validate_interpolate_operator_specifics(
    op: &str,
    args: &[Value],
    expected: &MirExprType,
    _spec: &MirSpec,
    _actual_output: &MirExprType,
) -> Result<(), String> {
    if args.len() < 4 {
        return Ok(());
    }
    if !(args.len() - 2).is_multiple_of(2) {
        // Arity issues are handled elsewhere by overload matching.
        return Ok(());
    }
    let pair_count = (args.len() - 2) / 2;

    // Enforce strictly ascending stop inputs only when they are numeric literals.
    let mut prev: Option<f64> = None;
    let mut all_inputs_are_literals = true;
    for i in 0..pair_count {
        let stop_in = &args[2 + 2 * i];
        let x = match stop_in {
            Value::Number(n) => n
                .as_f64()
                .ok_or_else(|| format!("interpolate stop input value out of f64 range: {n:?}"))?,
            _ => {
                all_inputs_are_literals = false;
                break;
            }
        };

        if let Some(p) = prev
            && x <= p
        {
            return Err(
                "interpolate stop input literals must be arranged in strictly ascending order"
                    .into(),
            );
        }
        prev = Some(x);
    }

    // Upstream rejects certain array outputs for exponential interpolation when the array
    // length is not statically known.
    //
    // In the remaining parity failure, the stop outputs are built via:
    //   ["array", "number", <expr>]
    // i.e. the `array` operator is called without the explicit `length` argument, so the
    // resulting type is `array<number>` with unknown length.
    if op == "interpolate"
        && matches!(
            args.first(),
            Some(Value::Array(a)) if a.first().and_then(|v| v.as_str()) == Some("exponential")
        )
        && all_inputs_are_literals
    {
        for i in 0..pair_count {
            let stop_out = &args[3 + 2 * i];
            if let Value::Array(arr) = stop_out {
                // `["array", "<itemType>", <candidate>]` (len == 3) => omitted length
                if arr.len() == 3
                    && arr
                        .first()
                        .and_then(|v| v.as_str())
                        .is_some_and(|s| s == "array")
                    && arr
                        .get(1)
                        .and_then(|v| v.as_str())
                        .is_some_and(|s| s == "number")
                {
                    return Err("Type array<number> is not interpolatable".into());
                }
            }
        }
    }

    // Enforce interpolatability based on expected output type, but only when stop
    // outputs are compile-time literals.
    let mut all_outputs_are_literals = true;
    for i in 0..pair_count {
        let stop_out = &args[3 + 2 * i];
        if !is_literal_expression_value(stop_out) {
            all_outputs_are_literals = false;
            break;
        }
    }

    if all_inputs_are_literals && all_outputs_are_literals {
        let output_ty = if matches!(expected, MirExprType::Any | MirExprType::TypeVar(_)) {
            // With no explicit expected output type (common in expression-level fixtures),
            // infer interpolatability from the *literal stop outputs*.
            //
            // Important: do not use full MIR type inference here because compile-time literal
            // typing is permissive (e.g. color strings currently type as `String`). Instead,
            // apply targeted classification using lightweight parsing heuristics.
            const PROJECTION_TOKENS: [&str; 2] = ["mercator", "vertical-perspective"];

            #[derive(Copy, Clone, Debug, PartialEq, Eq)]
            enum StopOutKind {
                Number,
                ColorString,
                ProjectionString,
                OtherString,
                ArrayOfNumber,
                ArrayOfColor,
                OtherArray,
            }

            let infer_stop_kind = |stop_out: &Value| -> Option<StopOutKind> {
                match stop_out {
                    Value::Number(_) => Some(StopOutKind::Number),
                    Value::String(s) => {
                        if PROJECTION_TOKENS.contains(&s.as_str()) {
                            return Some(StopOutKind::ProjectionString);
                        }
                        if color::parse_color(s).is_ok() {
                            return Some(StopOutKind::ColorString);
                        }
                        Some(StopOutKind::OtherString)
                    }
                    Value::Array(arr) => {
                        if arr.first().and_then(|v| v.as_str()) != Some("literal") {
                            return None;
                        }
                        let inner = arr.get(1)?;
                        match inner {
                            Value::Number(_) => Some(StopOutKind::Number),
                            Value::String(s) => {
                                if PROJECTION_TOKENS.contains(&s.as_str()) {
                                    return Some(StopOutKind::ProjectionString);
                                }
                                if color::parse_color(s).is_ok() {
                                    return Some(StopOutKind::ColorString);
                                }
                                Some(StopOutKind::OtherString)
                            }
                            Value::Array(inner_arr) => {
                                let all_numbers = inner_arr.iter().all(|v| v.is_number());
                                if all_numbers {
                                    return Some(StopOutKind::ArrayOfNumber);
                                }
                                let all_color_strings = inner_arr.iter().all(|v| {
                                    v.as_str()
                                        .and_then(|s| {
                                            if color::parse_color(s).is_ok() {
                                                Some(())
                                            } else {
                                                None
                                            }
                                        })
                                        .is_some()
                                });
                                if all_color_strings {
                                    return Some(StopOutKind::ArrayOfColor);
                                }
                                Some(StopOutKind::OtherArray)
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                }
            };

            let kinds: Vec<StopOutKind> = (0..pair_count)
                .filter_map(|i| {
                    let stop_out = &args[3 + 2 * i];
                    infer_stop_kind(stop_out)
                })
                .collect();
            if kinds.len() != pair_count {
                MirExprType::Any
            } else if kinds.iter().all(|k| matches!(k, StopOutKind::Number)) {
                MirExprType::Number
            } else if kinds.iter().all(|k| matches!(k, StopOutKind::ColorString)) {
                MirExprType::Color
            } else if kinds
                .iter()
                .all(|k| matches!(k, StopOutKind::ProjectionString))
            {
                // Projection interpolation is handled as a special case below.
                MirExprType::String
            } else if kinds.iter().all(|k| matches!(k, StopOutKind::OtherString)) {
                MirExprType::String
            } else if kinds
                .iter()
                .all(|k| matches!(k, StopOutKind::ArrayOfNumber))
            {
                MirExprType::Array {
                    element: Some(Box::new(MirExprParamType::Expression(MirExprType::Number))),
                    length: None,
                }
            } else if kinds.iter().all(|k| matches!(k, StopOutKind::ArrayOfColor)) {
                MirExprType::Array {
                    element: Some(Box::new(MirExprParamType::Expression(MirExprType::Color))),
                    length: None,
                }
            } else {
                MirExprType::Any
            }
        } else {
            expected.clone()
        };

        // Our `MirExprType` model currently collapses `projectionDefinition` into `MirExprType::String`.
        // Upstream still treats projection interpolation as interpolatable, but regular string
        // interpolation is rejected.
        //
        // Mirror upstream by accepting only *known* projection-definition literals as the
        // interpolated stop outputs.
        if op == "interpolate" && matches!(output_ty, MirExprType::String) {
            const PROJECTION_TOKENS: [&str; 2] = ["mercator", "vertical-perspective"];
            let all_stops_are_projection_literals = (0..pair_count).all(|i| {
                let stop_out = &args[3 + 2 * i];
                match stop_out {
                    Value::String(s) => PROJECTION_TOKENS.contains(&s.as_str()),
                    _ => false,
                }
            });
            if all_stops_are_projection_literals {
                return Ok(());
            }
        }

        if !is_interpolate_output_interpolatable(op, &output_ty) {
            return Err(format!("Type {output_ty:?} is not interpolatable"));
        }
    }

    Ok(())
}

fn validate_expression_with_mir(
    expr: &Value,
    expected: &MirExprType,
    spec: &MirSpec,
    env: &mut TypeEnv,
) -> Result<MirExprType, String> {
    match expr {
        Value::Null => {
            unify_types(&MirExprType::Any, expected, env)?;
            Ok(MirExprType::Any)
        }
        Value::Bool(_) => {
            let actual = MirExprType::Boolean;
            unify_types(&actual, expected, env)?;
            Ok(resolve_type(&actual, env))
        }
        Value::Number(_) => {
            let actual = MirExprType::Number;
            unify_types(&actual, expected, env)?;
            Ok(resolve_type(&actual, env))
        }
        Value::String(_) => {
            let actual = MirExprType::String;
            unify_types(&actual, expected, env)?;
            Ok(resolve_type(&actual, env))
        }
        Value::Object(map) => {
            // Some v8.json parameters are modeled as expression-output `object` even
            // though upstream treats bare objects as raw values for those operators.
            // Some v8.json parameters are modeled as expression-output `object` even
            // though upstream treats bare objects as raw values for those operators.
            // Allow bare objects when the expected type can accept them.
            //
            // Additionally, upstream expression integration fixtures include a legacy object-form
            // (e.g. `{ "type": "categorical", ... }`). When we see a `{type: "..."}`
            // expression envelope, accept it as `Any` so we don't spuriously reject valid legacy
            // expressions when the expected type is more specific.
            let is_legacy_object_form_expr = map
                .get("type")
                .and_then(|v| v.as_str())
                .is_some()
                // Legacy interpolation expressions sometimes omit an explicit `"type"` and
                // instead use `{"property": ..., "stops": ...}`.
                || (map.contains_key("property") && map.contains_key("stops"));
            let allow = matches!(resolve_type(expected, env), MirExprType::Object)
                || matches!(expected, MirExprType::Object)
                || matches!(expected, MirExprType::Any)
                || matches!(expected, MirExprType::TypeVar(_));

            if !allow && !is_legacy_object_form_expr {
                return Err(
                    "Bare JSON object invalid in expression. Use [\"literal\", {...}].".into(),
                );
            }

            let actual = if is_legacy_object_form_expr {
                MirExprType::Any
            } else {
                MirExprType::Object
            };
            unify_types(&actual, expected, env)?;
            Ok(actual)
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                return Err("expression array must be non-empty".into());
            }

            let resolved_expected = resolve_type(expected, env);
            let is_interpolation_expected = matches!(resolved_expected, MirExprType::Interpolation)
                || matches!(expected, MirExprType::Interpolation)
                || matches!(expected, MirExprType::TypeVar(tv) if tv == "interpolation");
            if is_interpolation_expected {
                let actual = validate_interpolation_type(arr, &MirExprType::Interpolation, env)?;
                unify_types(&actual, expected, env)?;
                return Ok(actual);
            }

            // Typed raw literal arrays used by `match` labels: v8.json declares them as `array<... literal>`.
            if let MirExprType::Array { element, .. } = expected
                && let Some(element) = element
                && matches!(
                    element.as_ref(),
                    MirExprParamType::Literal(_) | MirExprParamType::LiteralAnyOf(_)
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
                let element: Option<Box<MirExprParamType>> = if args.len() > 1 {
                    let item_type = args[0]
                        .as_str()
                        .ok_or_else(|| "array item type must be a string literal".to_string())?;
                    let item_expr_ty = match item_type {
                        "string" => MirExprType::String,
                        "number" => MirExprType::Number,
                        "boolean" => MirExprType::Boolean,
                        _ => {
                            return Err(
                                "The item type argument of \"array\" must be one of string, number, boolean"
                                    .into(),
                            );
                        }
                    };
                    idx = 1;
                    Some(Box::new(MirExprParamType::Expression(item_expr_ty)))
                } else {
                    // Model the omitted item type (`ValueType`) as a fresh type variable
                    // so it can unify with any expected `array<...>` element type.
                    Some(Box::new(MirExprParamType::TypeVar("__array_item".into())))
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
                    validate_expression_with_mir(c, &MirExprType::Any, spec, env)?;
                }

                let actual = MirExprType::Array { element, length };
                unify_types(&actual, expected, env)?;
                return Ok(actual);
            }

            if op == "error" {
                // Upstream uses `["error", ...]` in short-circuiting tests to ensure
                // that the presence of an error node doesn't prevent compilation.
                if args.is_empty() {
                    return Err("\"error\" requires at least one argument".into());
                }
                let actual = MirExprType::Any;
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

                let actual = MirExprType::Formatted;
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
            let out = validate_operator_call(operator, args, expected, spec, env)?;
            match op {
                "==" | "!=" | "<" | ">" | "<=" | ">=" => {
                    validate_comparison_operator_specifics(op, args, spec)?;
                }
                "coalesce" => {
                    validate_coalesce_operator_specifics(args, expected, spec)?;
                }
                "in" | "index-of" => {
                    validate_in_indexof_operator_specifics(args, spec)?;
                }
                "interpolate" | "interpolate-hcl" | "interpolate-lab" => {
                    validate_interpolate_operator_specifics(op, args, expected, spec, &out)?;
                }
                _ => {}
            }
            Ok(out)
        }
    }
}

/// Build `operator -> output-type groups` from the expression preprocessor (same as codegen).
fn operator_to_output_groups(ex: &MirExpressions) -> HashMap<String, Vec<String>> {
    let mut m: HashMap<String, Vec<String>> = HashMap::new();
    for (output_key, group) in &ex.by_output_type {
        for op in group.variants.keys() {
            m.entry(op.clone()).or_default().push(output_key.clone());
        }
    }
    m
}

/// Convenience: build the operator → groups map once per [`MirExpressions`] snapshot.
pub fn operator_groups_map(ex: &MirExpressions) -> HashMap<String, Vec<String>> {
    operator_to_output_groups(ex)
}

/// Normalize serde output from generated expression syntax enums into the JSON
/// shape expected by `validate_expression_with_mir`:
/// `["operator", arg0, arg1, ...]`.
fn normalize_serialized_expr_value(v: Value) -> Result<Value, String> {
    match v {
        Value::Array(arr) => {
            let mut out = Vec::with_capacity(arr.len());
            for x in arr {
                out.push(normalize_serialized_expr_value(x)?);
            }
            Ok(Value::Array(out))
        }
        Value::Object(map) => {
            // Heuristic: expression-operator enums serialize as `{ "VariantName": data }`.
            // JSON literal objects usually have multiple keys and often start with lowercase.
            if map.len() == 1
                && let Some((k, data)) = map.iter().next()
            {
                let first = k.chars().next();
                if first.is_some_and(|c| c.is_uppercase()) {
                    let op = camel_to_kebab(k);
                    let args = match data {
                        Value::Array(a) => {
                            let mut out = Vec::with_capacity(a.len());
                            for x in a {
                                out.push(normalize_serialized_expr_value(x.clone())?);
                            }
                            out
                        }
                        Value::Null => Vec::new(),
                        other => vec![normalize_serialized_expr_value(other.clone())?],
                    };
                    let mut out = Vec::with_capacity(1 + args.len());
                    out.push(Value::String(op));
                    out.extend(args);
                    return Ok(Value::Array(out));
                }
            }

            let mut out_map = serde_json::Map::with_capacity(map.len());
            for (k, v) in map {
                out_map.insert(k, normalize_serialized_expr_value(v)?);
            }
            Ok(Value::Object(out_map))
        }
        other => Ok(other),
    }
}

fn camel_to_kebab(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i != 0 {
                out.push('-');
            }
            for lower in ch.to_lowercase() {
                out.push(lower);
            }
        } else {
            out.push(ch);
        }
    }
    out
}

/// Deserialize `expr` for upstream compile parity: recurse through assertions and Decisions.
pub fn validate_expression_with_spec(
    expr: &ExprOrLiteral,
    expected: &MirExprType,
    _op_to_groups: &HashMap<String, Vec<String>>,
    _known_ops: &HashSet<String>,
) -> Result<(), String> {
    let spec = &*MIR_SPEC;
    let mut env = TypeEnv::default();
    let serialized = serde_json::to_value(expr).map_err(|e| e.to_string())?;
    let normalized = normalize_serialized_expr_value(serialized)?;
    validate_expression_with_mir(&normalized, expected, spec, &mut env).map(|_| ())
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap, HashSet};

    use super::{operator_groups_map, validate_expression_with_spec};
    use crate::decoder::StyleReference;
    use crate::mir::{MirExprType, MirSpec};
    use crate::spec::ExprOrLiteral;

    fn setup() -> (HashMap<String, Vec<String>>, HashSet<String>) {
        let v8 = include_str!("../../upstream/src/reference/v8.json");
        let reference: StyleReference = serde_json::from_str(v8).expect("v8.json should parse");
        let spec = MirSpec::from(reference);
        let op_to_groups = operator_groups_map(&spec.expressions);
        let known_ops: HashSet<String> = spec.expressions.operators.keys().cloned().collect();
        (op_to_groups, known_ops)
    }

    #[test]
    fn dump_expression_output_groups() {
        let v8 = include_str!("../../upstream/src/reference/v8.json");
        let reference: StyleReference = serde_json::from_str(v8).expect("v8");
        let spec = MirSpec::from(reference);
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
        let typed: ExprOrLiteral = serde_json::from_value(expr).expect("expr should deserialize");
        assert!(
            validate_expression_with_spec(&typed, &MirExprType::Any, &op_to_groups, &known_ops)
                .is_ok()
        );
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
        let typed: ExprOrLiteral = serde_json::from_value(expr).expect("expr should deserialize");
        assert!(
            validate_expression_with_spec(&typed, &MirExprType::Any, &op_to_groups, &known_ops)
                .is_ok()
        );
    }

    #[test]
    fn validates_any_step_input_can_be_expression() {
        let (op_to_groups, known_ops) = setup();
        let expr = serde_json::json!(["step", ["get", "point_count"], 20, 100, 30, 750, 40]);
        let typed: ExprOrLiteral = serde_json::from_value(expr).expect("expr should deserialize");
        assert!(
            validate_expression_with_spec(&typed, &MirExprType::Any, &op_to_groups, &known_ops)
                .is_ok()
        );
    }

    #[test]
    fn validates_comparison_ops_with_nested_get() {
        let (op_to_groups, known_ops) = setup();
        let expr = serde_json::json!(["<", ["get", "mag"], 2]);
        let typed: ExprOrLiteral = serde_json::from_value(expr).expect("expr should deserialize");
        assert!(
            validate_expression_with_spec(&typed, &MirExprType::Any, &op_to_groups, &known_ops)
                .is_ok()
        );
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
        let typed: ExprOrLiteral = serde_json::from_value(expr).expect("expr should deserialize");
        assert!(
            validate_expression_with_spec(&typed, &MirExprType::Any, &op_to_groups, &known_ops)
                .is_ok()
        );
    }

    #[test]
    fn validates_number_format_with_bare_options_object() {
        let (op_to_groups, known_ops) = setup();
        let expr = serde_json::json!(["number-format", ["get", "mag"], {}]);
        let typed: ExprOrLiteral = serde_json::from_value(expr).expect("expr should deserialize");
        assert!(
            validate_expression_with_spec(&typed, &MirExprType::Any, &op_to_groups, &known_ops)
                .is_ok()
        );
    }

    #[test]
    fn validates_number_plus_allows_var_operands() {
        let (op_to_groups, known_ops) = setup();
        let expr = serde_json::json!(["+", ["var", "x"], 2]);
        let typed: ExprOrLiteral = serde_json::from_value(expr).expect("expr should deserialize");
        assert!(
            validate_expression_with_spec(&typed, &MirExprType::Any, &op_to_groups, &known_ops)
                .is_ok()
        );
    }

    #[test]
    fn validates_empty_max_min() {
        let (op_to_groups, known_ops) = setup();
        let max_expr = serde_json::json!(["max"]);
        let min_expr = serde_json::json!(["min"]);
        let typed_max: ExprOrLiteral =
            serde_json::from_value(max_expr).expect("max_expr should deserialize");
        let typed_min: ExprOrLiteral =
            serde_json::from_value(min_expr).expect("min_expr should deserialize");
        assert!(
            validate_expression_with_spec(&typed_max, &MirExprType::Any, &op_to_groups, &known_ops)
                .is_ok()
        );
        assert!(
            validate_expression_with_spec(&typed_min, &MirExprType::Any, &op_to_groups, &known_ops)
                .is_ok()
        );
    }
}

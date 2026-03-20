use std::collections::BTreeMap;

use codegen2::{Function, Impl, Scope, Variant};
use serde_json::Value;

use crate::generator::autotest::generate_test_from_examples_if_present;
use crate::generator::formatter::{to_snake_case, to_upper_camel_case};
use crate::generator::fuzz;
use crate::mir::types::{
    MirExpression, MirLiteral as Literal, MirOverload as Overload, MirParameter as Parameter,
    MirParameterType as ParameterType, MirSyntax as Syntax, SyntaxVariantDef,
};

fn emit_tuple_slot(v: &mut Variant, rust_ty: &str) {
    match rust_ty {
        "serde_json::Value" => v.tuple_with_attrs([fuzz::ARB_JSON_VALUE], rust_ty),
        "Option<serde_json::Value>" => v.tuple_with_attrs([fuzz::ARB_OPTION_JSON_VALUE], rust_ty),
        fuzz::JSON_MAP_TY => v.tuple_with_attrs([fuzz::ARB_JSON_MAP], rust_ty),
        fuzz::OPTION_JSON_MAP_TY => v.tuple_with_attrs([fuzz::ARB_OPTION_JSON_MAP], rust_ty),
        _ => v.tuple(rust_ty),
    };
}

pub fn generate_syntax_enum(
    scope: &mut Scope,
    name: &str,
    doc: &str,
    values: &BTreeMap<String, SyntaxVariantDef>,
) {
    pregenerate_all_operator_parameter_types(scope, values);
    // pass 1: populate enum variants
    let variadic_row_struct_names = generate_syntax_enum_body(scope, name, doc, values);
    // pass 2: generate the previously referenced enum variants for overloaded variants
    generate_referenced_multi_overload_options_enums(scope, name, values);
    let examples: Vec<&serde_json::Value> = values
        .iter()
        .filter_map(|(operator_key, def)| {
            def.example.as_ref().filter(|ex| {
                example_json_operator_matches_variant_key(ex, operator_key.as_str())
                    && example_matches_fixed_arity_overload(def, ex)
            })
        })
        .collect();
    let example_for_visitor = examples
        .first()
        .copied()
        .or_else(|| values.values().find_map(|d| d.example.as_ref()))
        .expect("syntax enum must have at least one example");
    generate_syntax_enum_deserializer(
        scope,
        name,
        values,
        example_for_visitor,
        &variadic_row_struct_names,
    );

    generate_test_from_examples_if_present(scope, name, examples);
}

/// For generated `#[rstest]` decode checks, only attach an `example` to the variant for
/// which its JSON leading operator matches that variant's operator key.
fn example_json_operator_matches_variant_key(
    example: &serde_json::Value,
    operator_key: &str,
) -> bool {
    match example {
        serde_json::Value::Array(elems) => elems
            .first()
            .and_then(|v| v.as_str())
            .is_some_and(|head| head == operator_key),
        // Non-array examples (should be rare for syntax enums): accept only if they cannot be classified.
        _ => true,
    }
}

/// Drop examples whose argument count cannot match any **non-variadic** overload for this variant
/// (e.g. the global `array` example is 4 elements but the `array` *output* group only includes the
/// 1-arg `["array", value]` overload).
fn example_matches_fixed_arity_overload(
    def: &SyntaxVariantDef,
    example: &serde_json::Value,
) -> bool {
    let serde_json::Value::Array(elems) = example else {
        return true;
    };
    let nargs = elems.len().saturating_sub(1);
    let mut any_variadic = false;
    for ov in &def.syntax.overloads {
        if ov.parameters.iter().any(|p| p == "...") {
            any_variadic = true;
            continue;
        }
        let req = ov.parameters.iter().filter(|p| !p.ends_with('?')).count();
        let opt = ov.parameters.iter().filter(|p| p.ends_with('?')).count();
        if nargs >= req && nargs <= req + opt {
            return true;
        }
    }
    any_variadic
}

fn pregenerate_all_operator_parameter_types(
    scope: &mut Scope,
    values: &BTreeMap<String, SyntaxVariantDef>,
) {
    for def in values.values() {
        for overload in &def.syntax.overloads {
            for pname in &overload.parameters {
                if pname == "..." {
                    continue;
                }
                let Some(param) = def
                    .syntax
                    .parameters
                    .iter()
                    .find(|p| p.matches_overload_parameter_name(pname))
                else {
                    continue;
                };
                pregenerate_parameter_type(scope, &param.r#type);
            }
        }
    }
}

fn pregenerate_parameter_type(scope: &mut Scope, param: &ParameterType) {
    match param {
        ParameterType::LiteralAnyOf(ls) => {
            generate_any_of(scope, ls);
        }
        ParameterType::ExpressionAnyOf(es) => {
            generate_expression_any_of(scope, es);
            for e in es {
                pregenerate_parameter_type(scope, e);
            }
        }
        ParameterType::Expression(inner) => {
            pregenerate_mir_expression(scope, inner.as_ref());
        }
        _ => {}
    }
}

fn pregenerate_mir_expression(scope: &mut Scope, ex: &MirExpression) {
    if let MirExpression::Array {
        r#type: Some(et), ..
    } = ex
    {
        pregenerate_parameter_type(scope, et);
    }
}

/// `array<color>` / `array<number>` arms in an expression `any-of` refer to the recursive
/// interpolate stop types, not separate `ArrayOfColor` / `ArrayOfNumber` enums.
fn normalize_expression_union_component_type(component: &str) -> String {
    match component {
        "ArrayOfColor" => "ColorOrArrayOfColor".to_string(),
        // Avoid infinite `Number` ⇄ `MinusOptions` recursion; stops are validated structurally.
        "ArrayOfNumber" => "serde_json::Value".to_string(),
        // Interpolate `projection` stops are `ProjectionType` values; box breaks the cycle with `ProjectionType::Expr(...)`.
        "Projection" => "Box<ProjectionType>".to_string(),
        other => other.to_string(),
    }
}

/// Derive syntax enum variant identifier from operator keys by stripping the shared prefix
/// that equals the output-type enum name.
///
/// This directly targets `clippy::enum_variant_names` by ensuring the resulting identifier does
/// not start with the output-type enum name.
fn normalized_syntax_variant_ident(syntax_enum_name: &str, operator_key: &str) -> String {
    let mut base = to_upper_camel_case(operator_key);

    // Strip the entire shared prefix (repeat in case it shows up multiple times).
    while base.starts_with(syntax_enum_name) {
        base = base[syntax_enum_name.len()..].to_string();
    }

    // Also strip the shared suffix. Operators like `to-boolean` become `ToBoolean`, which ends
    // with the output-type enum name and triggers `clippy::enum_variant_names`.
    while base.ends_with(syntax_enum_name) {
        let new_len = base.len() - syntax_enum_name.len();
        base.truncate(new_len);
    }

    // If we stripped everything (e.g. operator key exactly equals the output-type enum name),
    // fall back to a stable non-empty identifier.
    if base.is_empty() {
        "Op".to_string()
    } else {
        base
    }
}

/// `#[serde(untagged)]` is required only when JSON shapes differ by variant (e.g. bare string vs
/// `["op", …]`). Using it for homogeneous expression unions breaks deserialization (ambiguous arms).
fn expression_union_needs_untagged(types: &[ParameterType]) -> bool {
    let mut has_literal = false;
    let mut has_non_literal = false;
    for p in types {
        match p {
            ParameterType::Literal(_) | ParameterType::LiteralAnyOf(_) => has_literal = true,
            ParameterType::Object(_) => has_non_literal = true,
            ParameterType::Expression(_)
            | ParameterType::ExpressionAnyOf(_)
            | ParameterType::Reference(_) => has_non_literal = true,
        }
    }
    has_literal && has_non_literal
}

fn generate_expression_any_of(scope: &mut Scope, types: &[ParameterType]) -> String {
    // `interpolate` / `step` stop outputs mix JSON literals (bare numbers) with expression arrays.
    // The spec encodes the numeric stop output as `expression(number)` which we normally lower to
    // the [`Number`] syntax enum — that rejects bare JSON numbers, so map that arm to
    // [`NumberLiteral`] only in unions that also include `projection` stops (the v-shaped output set).
    let has_projection_stop = types.iter().any(|p| {
        matches!(
            p,
            ParameterType::Reference(r) if r == "projection"
        )
    });
    let number_literal_with_number_expr = types
        .iter()
        .any(|p| matches!(p, ParameterType::Literal(Literal::Number)))
        && types.iter().any(|p| {
            matches!(
                p,
                ParameterType::Expression(e) if matches!(e.as_ref(), MirExpression::Number)
            )
        });
    let mut arms: Vec<(String, String)> = types
        .iter()
        .map(|p| {
            let label = p.to_upper_camel_case();
            let rust_ty = if has_projection_stop
                && matches!(
                    p,
                    ParameterType::Expression(e) if matches!(e.as_ref(), MirExpression::Number)
                ) {
                "NumberLiteral".to_string()
            } else if number_literal_with_number_expr
                && matches!(
                    p,
                    ParameterType::Expression(e) if matches!(e.as_ref(), MirExpression::Number)
                )
            {
                // Needs indirection: `Number` contains this union recursively (`+`, `*`, …).
                "Box<Number>".to_string()
            } else {
                normalize_expression_union_component_type(&label)
            };
            (label, rust_ty)
        })
        .collect();

    // `interpolate-hcl` / `interpolate-lab` accept CSS color strings (e.g. `"#f00"`) as stop outputs.
    let mut arm_labels: Vec<&str> = arms.iter().map(|(l, _)| l.as_str()).collect();
    arm_labels.sort_unstable();
    arm_labels.dedup();
    let mut extra_untagged = false;
    if arm_labels == ["ArrayOfColor", "Color"] {
        arms.insert(
            0,
            ("StringLiteral".to_string(), "StringLiteral".to_string()),
        );
        extra_untagged = true;
    }
    // `Number`↔`NumberLiteral` and projection↔string stops share one serde shape; without `untagged`
    // serde expects externally-tagged enums and rejects plain JSON scalars / arrays.
    if has_projection_stop {
        extra_untagged = true;
    }

    // `clippy::enum_variant_names`: avoid common suffixes like `Literal` on untagged literal unions.
    if arms.iter().all(|(label, _)| label.ends_with("Literal")) {
        for (label, _) in &mut arms {
            let stripped = label.trim_end_matches("Literal");
            *label = if stripped.is_empty() {
                label.clone()
            } else {
                stripped.to_string()
            };
        }
    }

    let any_of_type = format!(
        "{}AsUnion",
        arms.iter()
            .map(|(a, _)| a.as_str())
            .collect::<Vec<_>>()
            .join("Or")
    );
    if scope.get_enum_mut(&any_of_type).is_none() {
        let mut enu = scope
            .new_enum(&any_of_type)
            .doc("Either of the below variants")
            .vis("pub");
        if expression_union_needs_untagged(types) || extra_untagged {
            enu = enu.attr("serde(untagged)");
        }
        enu.derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
            .attr(fuzz::CFG_DERIVE_ARBITRARY);
        for (variant, rust_ty) in &arms {
            emit_tuple_slot(enu.new_variant(variant), rust_ty.as_str());
        }
    }
    any_of_type
}

fn generate_referenced_multi_overload_options_enums(
    scope: &mut Scope,
    name: &str,
    values: &BTreeMap<String, SyntaxVariantDef>,
) {
    for (key, value) in values {
        let var_name = normalized_syntax_variant_ident(name, key);
        let syntax = &value.syntax;
        let name_and_possibly_group = if let Some(group) = &value.group {
            format!("{name} (group={group})")
        } else {
            name.to_string()
        };
        assert!(
            !syntax.overloads.is_empty(),
            "{key} in {name_and_possibly_group} does not have a single overload"
        );
        if syntax.overloads.len() != 1 {
            let options_name = format!("{var_name}Options");
            // actually overloaded
            generate_multi_overload(scope, (name, &var_name, &options_name), syntax);
        }
    }
}

/// Variadic `interpolate` / `step` shape: one Rust tuple type `(headers..., Vec<(stop_in, stop_out)>)`.
///
/// Stored as a single [`codegen2`] tuple slot — the formatter only prints the first slot when
/// multiple tuple fields have no attributes.
#[derive(Debug, Clone)]
struct InterpolateStyleFields {
    /// e.g. `(Interpolation, Number, Vec<(NumberLiteral, U)>)`
    combined_tuple_type: String,
}

fn generate_syntax_enum_body(
    scope: &mut Scope,
    name: &str,
    doc: &str,
    values: &BTreeMap<String, SyntaxVariantDef>,
) -> BTreeMap<String, String> {
    // Variadic variants need `generate_parameter_variant`, which mutates `scope`. `new_enum` also
    // holds a `scope` borrow via its handle, so precompute tuple types before creating the enum.
    let mut variadic_tuple_types: BTreeMap<String, String> = BTreeMap::new();
    let mut variadic_interpolate_style: BTreeMap<String, InterpolateStyleFields> = BTreeMap::new();
    let mut variadic_row_struct_names: BTreeMap<String, String> = BTreeMap::new();
    for (key, value) in values {
        let syntax = &value.syntax;
        let name_and_possibly_group = if let Some(group) = &value.group {
            format!("{name} (group={group})")
        } else {
            name.to_string()
        };
        assert!(
            !syntax.overloads.is_empty(),
            "{key} in {name_and_possibly_group} does not have a single overload"
        );
        if syntax.overloads.len() == 1 && syntax.has_variadic_overload() {
            let overload = &syntax.overloads[0];
            let position_of_variadic_separator = overload.position_of_variadic_separator();
            let variant_name = normalized_syntax_variant_ident(name, key);

            if name == "Any" && key == "match" {
                let variant_name = normalized_syntax_variant_ident(name, key);
                let label_ty = generate_parameter_type(
                    scope,
                    (name, variant_name.as_str(), "label_i"),
                    &syntax.parameters,
                );
                let output_ty = generate_parameter_type(
                    scope,
                    (name, variant_name.as_str(), "output_i"),
                    &syntax.parameters,
                );
                let fallback_ty = generate_parameter_type(
                    scope,
                    (name, variant_name.as_str(), "fallback"),
                    &syntax.parameters,
                );
                let lt = box_recursive_types(name, &label_ty);
                let ot = box_recursive_types(name, &output_ty);
                let ft = box_recursive_types(name, &fallback_ty);
                variadic_tuple_types.insert(
                    key.clone(),
                    format!("(serde_json::Value, Vec<({lt},{ot})>, {ft})"),
                );
                continue;
            }

            if overload_uses_interpolate_style_variadic(overload) {
                let sep = position_of_variadic_separator;
                let header_params = &overload.parameters[0..sep.saturating_sub(2)];
                let header_rust_types: Vec<String> = header_params
                    .iter()
                    .map(|overload_param| {
                        let part = generate_parameter_type(
                            scope,
                            (name, variant_name.as_str(), overload_param.as_str()),
                            &syntax.parameters,
                        );
                        box_recursive_types(name, &part)
                    })
                    .collect();
                let pa = &overload.parameters[sep - 2];
                let pb = &overload.parameters[sep - 1];
                let rust_a = generate_parameter_type(
                    scope,
                    (name, variant_name.as_str(), pa.as_str()),
                    &syntax.parameters,
                );
                let rust_b = generate_parameter_type(
                    scope,
                    (name, variant_name.as_str(), pb.as_str()),
                    &syntax.parameters,
                );
                let pair_a = box_recursive_types(name, &rust_a);
                let pair_b = box_recursive_types(name, &rust_b);
                let stops_ty = format!("Vec<({pair_a},{pair_b})>");
                let combined = if header_rust_types.is_empty() {
                    stops_ty
                } else {
                    format!("({},{})", header_rust_types.join(","), stops_ty)
                };
                variadic_interpolate_style.insert(
                    key.clone(),
                    InterpolateStyleFields {
                        combined_tuple_type: combined,
                    },
                );
                continue;
            }

            let prefix = &overload.parameters[0..position_of_variadic_separator];
            let parts: Vec<String> = prefix
                .iter()
                .map(|overload_param| {
                    let part_ty = generate_parameter_type(
                        scope,
                        (name, variant_name.as_str(), overload_param.as_str()),
                        &syntax.parameters,
                    );
                    // If the repeating element type *is* the parent enum, `Vec<T>` already
                    // provides the necessary indirection; avoid `Box<T>` which triggers
                    // `clippy::vec_box` in generated `spec.rs`.
                    if part_ty == name {
                        part_ty
                    } else {
                        box_recursive_types(name, &part_ty)
                    }
                })
                .collect();
            let tuple_type_names = parts.join(",");
            let tuple = if parts.len() == 2
                && parts[1].contains("serde_json::Map")
                && parts[1].contains("Option<")
            {
                let row_name = format!("{name}{variant_name}VariadicRow");
                if scope.get_struct_mut(&row_name).is_none() {
                    scope
                        .new_struct(&row_name)
                        .vis("pub")
                        .doc("Tuple row for variadic (content, optional style object) pairs.")
                        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
                        .attr(fuzz::CFG_DERIVE_ARBITRARY)
                        .tuple_field(&parts[0])
                        .tuple_field_with_attrs([fuzz::ARB_OPTION_JSON_MAP], &parts[1]);
                }
                variadic_row_struct_names.insert(key.clone(), row_name.clone());
                format!("Vec<{row_name}>")
            } else if parts.len() > 1 {
                box_recursive_types(name, &format!("Vec<({tuple_type_names})>"))
            } else {
                box_recursive_types(name, &format!("Vec<{tuple_type_names}>"))
            };
            let suffix_params = variadic_non_template_suffix_parameters(overload);
            let suffix_types: Vec<String> = suffix_params
                .iter()
                .map(|overload_param| {
                    let part = generate_parameter_type(
                        scope,
                        (name, variant_name.as_str(), overload_param.as_str()),
                        &syntax.parameters,
                    );
                    box_recursive_types(name, &part)
                })
                .collect();
            let full_tuple = if suffix_types.is_empty() {
                tuple
            } else {
                format!("({},{})", tuple, suffix_types.join(","))
            };
            variadic_tuple_types.insert(key.clone(), full_tuple);
        }
    }

    let enu = scope
        .new_enum(name)
        .doc(doc)
        .vis("pub")
        .derive("serde::Serialize, PartialEq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY);
    for (key, value) in values {
        let var_name = normalized_syntax_variant_ident(name, key);
        let syntax = &value.syntax;
        let name_and_possibly_group = if let Some(group) = &value.group {
            format!("{name} (group={group})")
        } else {
            name.to_string()
        };
        assert!(
            !syntax.overloads.is_empty(),
            "{key} in {name_and_possibly_group} does not have a single overload"
        );
        if syntax.overloads.len() == 1 {
            let overload = &syntax.overloads[0];
            if syntax.has_variadic_overload() {
                let var = enu.new_variant(&var_name).doc(&value.doc);
                if let Some(shape) = variadic_interpolate_style.get(key) {
                    var.tuple(shape.combined_tuple_type.as_str());
                } else {
                    let tuple = variadic_tuple_types
                        .get(key)
                        .unwrap_or_else(|| panic!("variadic tuple missing for operator {key}"));
                    var.tuple(tuple);
                }
                continue;
            }
            let var = enu.new_variant(&var_name).doc(&value.doc);
            for p in &overload.parameters {
                let param = p.clone();
                // Most single-overload syntax enums model parameters as raw JSON values.
                // Collator is special: upstream defines `collator` as a typed expression, and
                // we want its operands/arguments to remain strongly typed.
                let tuple_identifier = if name == "Collator"
                    && key == "collator"
                    && param == "options"
                {
                    // Collator options are an object schema; represent them as a JSON object map.
                    "serde_json::Map<std::string::String, serde_json::Value>".to_string()
                } else if name == "String" && key == "resolved-locale" && param == "collator" {
                    // `resolved-locale` takes a `collator` expression, not an opaque JSON value.
                    "Collator".to_string()
                } else if name == "Boolean" && (key == "==" || key == "!=") && param == "collator?"
                {
                    // Equality operators inline `collator?` as an optional third argument.
                    "Option<Collator>".to_string()
                } else if param.strip_suffix('?').is_some() {
                    "Option<serde_json::Value>".to_string()
                } else {
                    "serde_json::Value".to_string()
                };

                emit_tuple_slot(var, &tuple_identifier);
            }
        } else {
            let options_name = format!("{var_name}Options");
            let var = enu.new_variant(&var_name).doc(&value.doc);
            var.tuple(&options_name);
        }
    }
    variadic_row_struct_names
}

/// Breaks `E`-recursive shapes like `Number::Minus`/`Number::Star` without infinite-sized types.
fn box_recursive_types(parent_syntax_enum: &str, rust_type: &str) -> String {
    let t = rust_type.trim();
    if let Some(rest) = t.strip_prefix("Option<").and_then(|s| s.strip_suffix('>')) {
        return format!(
            "Option<{}>",
            box_recursive_types(parent_syntax_enum, rest.trim())
        );
    }
    if t == parent_syntax_enum {
        return format!("Box<{t}>");
    }
    // `Number::IndexOf` carries `Any` operands; `Any` can nest `Number` again.
    if parent_syntax_enum == "Number" && t == "Any" {
        return "Box<Any>".to_string();
    }
    if let Some(rest) = t.strip_prefix("Vec<").and_then(|s| s.strip_suffix('>')) {
        let rest = rest.trim();
        if rest.starts_with('(') && rest.ends_with(')') {
            let inner = &rest[1..rest.len() - 1];
            let parts: Vec<String> = inner
                .split(',')
                .map(|p| {
                    let inner = p.trim();
                    // `Vec<T>` already provides heap indirection; avoid redundant `Box<T>` which
                    // triggers `clippy::vec_box` in generated `spec.rs`.
                    if inner == parent_syntax_enum {
                        inner.to_string()
                    } else {
                        box_recursive_types(parent_syntax_enum, inner)
                    }
                })
                .collect();
            return format!("Vec<({})>", parts.join(","));
        }
        if rest == parent_syntax_enum {
            return format!("Vec<{}>", rest);
        }
        return format!("Vec<{}>", box_recursive_types(parent_syntax_enum, rest));
    }
    t.to_string()
}

/// `<` / `<=` / … use `(string,string)` and `(number,number)` overloads. `untagged` +
/// `SeqAccessDeserializer` breaks when serde probes variants; merge to one `(Value, Value, …)`.
fn comparison_operands_merge_applies(syntax: &Syntax) -> bool {
    if syntax.overloads.len() != 2 {
        return false;
    }
    let a = &syntax.overloads[0].parameters;
    let b = &syntax.overloads[1].parameters;
    if a.len() != 3 || b.len() != 3 {
        return false;
    }
    if !a[2].ends_with("collator?") || !b[2].ends_with("collator?") {
        return false;
    }
    let a0 = a[0].as_str();
    let b0 = b[0].as_str();
    (a0.starts_with("string_") && b0.starts_with("number_"))
        || (a0.starts_with("number_") && b0.starts_with("string_"))
}

fn generate_multi_overload(
    scope: &mut Scope,
    (name, var_name, options_name): (&str, &str, &str),
    syntax: &Syntax,
) {
    if comparison_operands_merge_applies(syntax) {
        scope
            .new_enum(options_name)
            .doc(format!(
                "Options for deserializing the syntax enum variant [`{name}::{var_name}`]"
            ))
            .vis("pub")
            .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
            .attr(fuzz::CFG_DERIVE_ARBITRARY)
            .new_variant("Args")
            .tuple("(serde_json::Value, serde_json::Value, Option<Collator>)");
        return;
    }
    // because scope can only be owned by one owner, we first need to generate all tuples, then can add them
    let mut overloads_tuples = Vec::with_capacity(syntax.overloads.len());
    for overload in &syntax.overloads {
        if overload.is_variadic(&syntax.parameters) {
            overloads_tuples.push(generate_variadic_overload_tuples(
                scope,
                (name, var_name),
                syntax,
                overload,
            ));
        } else {
            let var_name = overload.output_type.to_upper_camel_case();
            let mut tuples = Vec::with_capacity(overload.parameters.len());
            for param in &overload.parameters {
                // Trailing optional tuple fields + untagged enums + `SeqAccessDeserializer` cannot
                // reliably rewind; accept JSON for common optional slots.
                if param.as_str() == "collator?" {
                    tuples.push("Option<Collator>".to_string());
                    continue;
                }
                if param.as_str() == "from_index?" {
                    tuples.push("Option<serde_json::Value>".to_string());
                    continue;
                }
                let param_name =
                    generate_parameter_type(scope, (name, &var_name, param), &syntax.parameters);
                tuples.push(box_recursive_types(name, &param_name));
            }
            overloads_tuples.push(tuples);
        }
    }

    let enu = scope
        .new_enum(options_name)
        .doc(format!(
            "Options for deserializing the syntax enum variant [`{name}::{var_name}`]"
        ))
        .vis("pub")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .attr("serde(untagged)")
        .attr(fuzz::CFG_DERIVE_ARBITRARY);
    let variant_naming_strat = OverloadVariantNamingStrategy::detect(&syntax.overloads);
    for (i, overload) in syntax.overloads.iter().enumerate() {
        let var_name = variant_naming_strat.var_name(overload, i);
        let var = enu.new_variant(&var_name);
        if overload.is_variadic(&syntax.parameters) {
            for t in &overloads_tuples[i] {
                emit_tuple_slot(var, t);
            }
        } else {
            for (pi, param_raw) in overload.parameters.iter().enumerate() {
                let t = &overloads_tuples[i][pi];
                if param_raw.ends_with('?') {
                    var.tuple_with_attrs(["#[serde(default)]"], t.as_str());
                } else {
                    emit_tuple_slot(var, t);
                }
            }
        }
    }
}

fn generate_variadic_overload_tuples(
    scope: &mut Scope,
    (name, var_name): (&str, &str),
    syntax: &Syntax,
    overload: &Overload,
) -> Vec<String> {
    let separator_idx = overload.position_of_variadic_separator();
    let prefix = &overload.parameters[..separator_idx];
    let after_separator = &overload.parameters[separator_idx + 1..];

    let mut repeating = Vec::new();
    let mut suffix = Vec::new();

    for param in after_separator {
        if let Some(mapped) = map_template_n_to_1(param) {
            repeating.push(mapped);
        } else {
            suffix.push(param.clone());
        }
    }

    if repeating.is_empty() {
        repeating = prefix.to_vec();
    }

    let mut tuple_types = Vec::new();
    let repeating_types = repeating
        .iter()
        .map(|param| {
            let s = generate_parameter_type(scope, (name, var_name, param), &syntax.parameters);
            box_recursive_types(name, &s)
        })
        .collect::<Vec<_>>();

    // When a repeating template is stored in a `Vec<T>`, `Vec` already provides the heap indirection
    // needed for recursion. Avoid redundant `Box<T>` inside `Vec`, which triggers `clippy::vec_box`.
    let parent_box = format!("Box<{name}>");
    let repeating_types = repeating_types
        .into_iter()
        .map(|t| if t == parent_box { name.to_string() } else { t })
        .collect::<Vec<_>>();

    if repeating_types.len() == 1 {
        tuple_types.push(format!("Vec<{}>", repeating_types[0]));
    } else {
        tuple_types.push(format!("Vec<({})>", repeating_types.join(",")));
    }

    tuple_types.extend(suffix.iter().map(|param| {
        let s = generate_parameter_type(scope, (name, var_name, param), &syntax.parameters);
        box_recursive_types(name, &s)
    }));

    tuple_types
}

fn map_template_n_to_1(param: &str) -> Option<String> {
    if let Some(base) = param.strip_suffix("_n?") {
        return Some(format!("{base}_1?"));
    }
    if let Some(base) = param.strip_suffix("_n") {
        return Some(format!("{base}_1"));
    }
    None
}

/// Parameters after the `...` separator that are **not** part of the repeating template (e.g.
/// `fallback` / `expression` on `case` / `let`).
fn variadic_non_template_suffix_parameters(overload: &Overload) -> Vec<String> {
    let sep = overload.position_of_variadic_separator();
    overload.parameters[sep + 1..]
        .iter()
        .filter(|p| map_template_n_to_1(p).is_none())
        .cloned()
        .collect()
}

enum OverloadVariantNamingStrategy {
    OutputType,
    NumberOptions(Vec<usize>),
    ConstantMapping(Vec<String>),
}

impl OverloadVariantNamingStrategy {
    fn detect(overloads: &Vec<Overload>) -> Self {
        assert!(
            overloads.len() > 1,
            "renaming detection does only make sense for more than one overload"
        );
        // case 1: the output type is different
        let mut output_types = overloads
            .iter()
            .map(|o| o.output_type.to_upper_camel_case())
            .collect::<Vec<_>>();
        output_types.sort_unstable();
        let all_output_types = output_types.len();
        output_types.dedup();
        if all_output_types == output_types.len() {
            return OverloadVariantNamingStrategy::OutputType;
        }

        // case 2: the parameter lengths are all different
        let mut parameter_lengths = overloads
            .iter()
            .map(|o| o.parameters.len())
            .collect::<Vec<_>>();
        let params_clone = parameter_lengths.clone();
        parameter_lengths.sort_unstable();
        let all_params = parameter_lengths.len();
        parameter_lengths.dedup();
        if all_params == parameter_lengths.len() {
            return OverloadVariantNamingStrategy::NumberOptions(params_clone);
        }

        // case 3: the first parameter is different
        let ordered_first: Vec<String> = overloads
            .iter()
            .map(|o| o.parameters.first().cloned().unwrap_or_default())
            .map(|name| {
                if name.ends_with('?') {
                    format!("Opt{}", name.replace('?', ""))
                } else {
                    name
                }
            })
            .map(|name| {
                if name.ends_with("_1") {
                    name.replace("_1", "")
                } else {
                    name
                }
            })
            .map(to_upper_camel_case)
            .collect();
        let mut sorted_for_uniq = ordered_first.clone();
        sorted_for_uniq.sort_unstable();
        let n = sorted_for_uniq.len();
        sorted_for_uniq.dedup();
        if n == sorted_for_uniq.len() {
            // IMPORTANT: preserve overload order — tuple types are built in the same order as
            // `overloads`; sorting names alone would pair the wrong arm with each overload
            // (e.g. `LessOptions::Number` getting `(String, String)`).
            return OverloadVariantNamingStrategy::ConstantMapping(ordered_first);
        }

        panic!("could not determine a good naming strategy for {overloads:?}");
    }
    fn var_name(&self, overload: &Overload, i: usize) -> String {
        match self {
            OverloadVariantNamingStrategy::OutputType => overload.output_type.to_upper_camel_case(),
            OverloadVariantNamingStrategy::NumberOptions(ns) => {
                format!("{}Params", to_upper_camel_case(ns[i].to_string()))
            }
            OverloadVariantNamingStrategy::ConstantMapping(ms) => ms[i].clone(),
        }
    }
}

fn generate_parameter_type(
    scope: &mut Scope,
    (name, var_name, param): (&str, &str, &str),
    parameters: &[Parameter],
) -> String {
    if let Some(param) = param.strip_suffix('?') {
        let param = parameters.iter()
            .find(|p| p.matches_overload_parameter_name(param))
            .unwrap_or_else(|| panic!("parameter {param} from the syntax overload of {name}::{var_name} does not have a syntax parameter"));
        let param_name = generate_parameter_variant(scope, &param.r#type);
        format!("Option<{param_name}>")
    } else {
        let param = parameters.iter()
            .find(|p| p.matches_overload_parameter_name(param))
            .unwrap_or_else(|| panic!("parameter {param} from the syntax overload of {name}::{var_name} does not have a syntax parameter"));
        generate_parameter_variant(scope, &param.r#type)
    }
}

fn generate_parameter_variant(scope: &mut Scope, param: &ParameterType) -> String {
    match &param {
        ParameterType::Literal(l) => l.to_upper_camel_case().to_string(),
        ParameterType::LiteralAnyOf(ls) => generate_any_of(scope, ls),
        ParameterType::Expression(e) => match e.as_ref() {
            // `any` in the style spec means "expression or JSON literal", not the [`Any`] syntax enum.
            MirExpression::Any => "serde_json::Value".to_string(),
            MirExpression::Number => generate_expression_any_of(
                scope,
                &[
                    ParameterType::Literal(Literal::Number),
                    ParameterType::Expression(Box::new(MirExpression::Number)),
                ],
            ),
            _ => e.to_upper_camel_case().to_string(),
        },
        ParameterType::ExpressionAnyOf(es) => generate_expression_any_of(scope, es),
        ParameterType::Object(_) => {
            "serde_json::Map<std::string::String, serde_json::Value>".to_string()
        }
        ParameterType::Reference(r) => {
            if r == "T" {
                // Type variable `T` permits literals or arbitrary nested expressions (e.g. `coalesce`
                // with `image`, `in` with feature properties, …).
                "serde_json::Value".to_string()
            } else if r == "projection" {
                // Projection definition strings / expressions — not the root `{ "type": … }` object (`struct Projection`).
                "ProjectionType".to_string()
            } else {
                to_upper_camel_case(r)
            }
        }
    }
}
fn generate_any_of(scope: &mut Scope, any_of: &[Literal]) -> String {
    let ts = any_of
        .iter()
        .map(|l| l.to_upper_camel_case())
        .collect::<Vec<_>>();
    // Suffix avoids clashing with real expression output-type enums (e.g. `ColorOrArrayOfColor`).
    let any_of_type = format!("{}AsUnion", ts.join("Or"));
    if scope.get_enum_mut(&any_of_type).is_none() {
        let enu = scope
            .new_enum(&any_of_type)
            .doc("Either of the below variants")
            .vis("pub")
            .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
            .attr(fuzz::CFG_DERIVE_ARBITRARY);
        for t in ts {
            enu.new_variant(t).tuple(t);
        }
    }
    any_of_type
}

fn overload_uses_interpolate_style_variadic(overload: &Overload) -> bool {
    let sep = overload.position_of_variadic_separator();
    if sep < 4 {
        return false;
    }
    let pa = &overload.parameters[sep - 2];
    let pb = &overload.parameters[sep - 1];
    pa.contains("stop_input") && pb.contains("stop_output")
}

/// Precomputed fragment for variadic operators whose overload uses the `step` / `interpolate`
/// layout: shared prefix once, then `(stop_input, stop_output)` pairs (`…` at index ≥ 4).
///
/// Built before `generate_visitor` so `generate_parameter_type` can use `scope` without
/// conflicting with the visitor impl borrow.
#[derive(Debug, Clone)]
struct VariadicSep4Plan {
    header_lines: Vec<String>,
    header_bind_names: Vec<String>,
    pair_bind_a: String,
    pair_bind_b: String,
    pair_ty_a: String,
    pair_ty_b: String,
    pair_b_optional: bool,
}

fn precompute_variadic_sep4_plans(
    scope: &mut Scope,
    syntax_enum_name: &str,
    values: &BTreeMap<String, SyntaxVariantDef>,
) -> BTreeMap<String, VariadicSep4Plan> {
    let mut out = BTreeMap::new();
    for (key, syntax_docs) in values {
        let syntax = &syntax_docs.syntax;
        if syntax.overloads.len() != 1 || !syntax.has_variadic_overload() {
            continue;
        }
        let overload = &syntax.overloads[0];
        if !overload_uses_interpolate_style_variadic(overload) {
            continue;
        }
        let sep = overload.position_of_variadic_separator();
        let variant_name = normalized_syntax_variant_ident(syntax_enum_name, key);
        let fixed_header_len = sep - 2;
        let mut header_lines = Vec::new();
        let mut header_bind_names = Vec::new();
        for param in overload.parameters.iter().take(fixed_header_len) {
            if param.ends_with('?') {
                let rust_ty = generate_parameter_type(
                    scope,
                    (syntax_enum_name, variant_name.as_str(), param.as_str()),
                    &syntax.parameters,
                );
                let rust_ty = box_recursive_types(syntax_enum_name, &rust_ty);
                let p = param.strip_suffix('?').expect("checked ends_with ?");
                header_lines.push(format!("let {p}: {rust_ty} = seq.next_element()?;"));
                header_bind_names.push(p.to_string());
            } else if param == "type" {
                let rust_ty = generate_parameter_type(
                    scope,
                    (syntax_enum_name, variant_name.as_str(), param.as_str()),
                    &syntax.parameters,
                );
                let rust_ty = box_recursive_types(syntax_enum_name, &rust_ty);
                header_lines.push(format!(
                    "let r#type: {rust_ty} = visit_seq_field(&mut seq, \"type\")?;"
                ));
                header_bind_names.push("r#type".to_string());
            } else {
                let rust_ty = generate_parameter_type(
                    scope,
                    (syntax_enum_name, variant_name.as_str(), param.as_str()),
                    &syntax.parameters,
                );
                let rust_ty = box_recursive_types(syntax_enum_name, &rust_ty);
                header_lines.push(format!(
                    "let {param}: {rust_ty} = visit_seq_field(&mut seq, \"{param}\")?;"
                ));
                header_bind_names.push(param.to_string());
            }
        }
        let pa = &overload.parameters[sep - 2];
        let pb = &overload.parameters[sep - 1];
        let rust_a = generate_parameter_type(
            scope,
            (syntax_enum_name, variant_name.as_str(), pa.as_str()),
            &syntax.parameters,
        );
        let rust_b = generate_parameter_type(
            scope,
            (syntax_enum_name, variant_name.as_str(), pb.as_str()),
            &syntax.parameters,
        );
        let pair_ty_a = box_recursive_types(syntax_enum_name, &rust_a);
        let pair_ty_b = box_recursive_types(syntax_enum_name, &rust_b);
        out.insert(
            key.clone(),
            VariadicSep4Plan {
                header_lines,
                header_bind_names,
                pair_bind_a: to_snake_case(pa).replace("_1", "_i"),
                pair_bind_b: to_snake_case(pb).replace("_1", "_i"),
                pair_ty_a,
                pair_ty_b,
                pair_b_optional: pb.ends_with('?'),
            },
        );
    }
    out
}

fn emit_any_match_deserializer_arm(visit_seq: &mut Function, (lt, ot, ft): (&str, &str, &str)) {
    visit_seq.line("let mut rest: Vec<serde_json::Value> = Vec::new();");
    visit_seq.line("while let Some(v) = seq.next_element()? { rest.push(v); }");
    visit_seq.line("if rest.len() < 2 {");
    visit_seq.line("return Err(serde::de::Error::custom(\"Any::Match: too few arguments\"));");
    visit_seq.line("}");
    visit_seq.line("if !rest.len().is_multiple_of(2) {");
    visit_seq.line("return Err(serde::de::Error::custom(\"Any::Match: expected an even number of arguments after operator (input + label/output pairs + fallback)\"));");
    visit_seq.line("}");
    visit_seq.line("let fallback_v = rest.pop().unwrap();");
    visit_seq.line("let input = rest.remove(0);");
    visit_seq.line("let mut pairs = Vec::new();");
    visit_seq.line("for chunk in rest.chunks_exact(2) {");
    visit_seq.line(format!(
        "let label_i: {lt} = serde_json::from_value(chunk[0].clone()).map_err(serde::de::Error::custom)?;"
    ));
    visit_seq.line(format!(
        "let output_i: {ot} = serde_json::from_value(chunk[1].clone()).map_err(serde::de::Error::custom)?;"
    ));
    visit_seq.line("pairs.push((label_i, output_i));");
    visit_seq.line("}");
    visit_seq.line("if pairs.is_empty() {");
    visit_seq
        .line("return Err(serde::de::Error::custom(\"Any::Match: missing label/output pairs\"));");
    visit_seq.line("}");
    visit_seq.line(format!(
        "let fallback: {ft} = serde_json::from_value(fallback_v).map_err(serde::de::Error::custom)?;"
    ));
    visit_seq.line("Ok(Any::Match((input, pairs, fallback)))");
}

fn emit_number_minus_deserializer_arm(visit_seq: &mut Function, union_ty: &str) {
    visit_seq.line("let mut rest: Vec<serde_json::Value> = Vec::new();");
    visit_seq.line("while let Some(v) = seq.next_element()? { rest.push(v); }");
    visit_seq.line("match rest.len() {");
    visit_seq.line(format!(
        "2 => Ok(Number::Minus(MinusOptions::TwoParams(serde_json::from_value::<{union_ty}>(rest[0].clone()).map_err(serde::de::Error::custom)?, serde_json::from_value::<{union_ty}>(rest[1].clone()).map_err(serde::de::Error::custom)?))),"
    ));
    visit_seq.line(format!(
        "1 => Ok(Number::Minus(MinusOptions::OneParams(serde_json::from_value::<{union_ty}>(rest[0].clone()).map_err(serde::de::Error::custom)?))),"
    ));
    visit_seq.line("len => Err(serde::de::Error::custom(format!(\"'-': expected 1 or 2 arguments, got {len}\"))),");
    visit_seq.line("}");
}

fn generate_syntax_enum_deserializer(
    scope: &mut Scope,
    name: &str,
    values: &BTreeMap<String, SyntaxVariantDef>,
    example: &serde_json::Value,
    variadic_row_struct_names: &BTreeMap<String, String>,
) {
    let sep4_plans = precompute_variadic_sep4_plans(scope, name, values);
    let any_match_types = if name == "Any" {
        values.get("match").map(|def| {
            let syntax = &def.syntax;
            let variant_name = "Match";
            let label_ty = generate_parameter_type(
                scope,
                ("Any", variant_name, "label_i"),
                &syntax.parameters,
            );
            let output_ty = generate_parameter_type(
                scope,
                ("Any", variant_name, "output_i"),
                &syntax.parameters,
            );
            let fallback_ty = generate_parameter_type(
                scope,
                ("Any", variant_name, "fallback"),
                &syntax.parameters,
            );
            (
                box_recursive_types("Any", &label_ty),
                box_recursive_types("Any", &output_ty),
                box_recursive_types("Any", &fallback_ty),
            )
        })
    } else {
        None
    };
    let number_minus_union_ty = if name == "Number" {
        Some(generate_expression_any_of(
            scope,
            &[
                ParameterType::Literal(Literal::Number),
                ParameterType::Expression(Box::new(MirExpression::Number)),
            ],
        ))
    } else {
        None
    };
    let vis = generate_visitor(scope, name, example);

    let visit_seq = vis
        .new_fn("visit_seq")
        .generic("A: serde::de::SeqAccess<'de>")
        .arg_self()
        .arg("mut seq", "A")
        .ret("Result<Self::Value, A::Error>");
    generate_visit_seq_field(visit_seq);
    // operator decoding
    visit_seq.line("// First element: operator string");
    visit_seq.line("let op: std::string::String = seq.next_element()?.ok_or_else(|| serde::de::Error::custom(\"missing operator\"))?;");
    visit_seq.line("match op.as_str() {");
    for (key, syntax_docs) in values {
        let syntax = &syntax_docs.syntax;
        let variant_name = normalized_syntax_variant_ident(name, key);

        visit_seq.line(format!("\"{key}\" => {{"));
        if syntax.overloads.len() == 1
            && let Some(overload) = syntax.overloads.first()
        {
            if syntax.has_variadic_overload() {
                let row_struct = variadic_row_struct_names
                    .get(key.as_str())
                    .map(String::as_str);
                if name == "Any" && key == "match" {
                    let (lt, ot, ft) = any_match_types
                        .as_ref()
                        .expect("Any::Match precomputed types");
                    emit_any_match_deserializer_arm(
                        visit_seq,
                        (lt.as_str(), ot.as_str(), ft.as_str()),
                    );
                } else {
                    generate_syntax_enum_deserializer_regular_variadic_variant(
                        visit_seq,
                        (name, &variant_name),
                        overload,
                        row_struct,
                        sep4_plans.get(key.as_str()),
                    );
                }
            } else {
                generate_syntax_enum_deserializer_regular_variant(
                    visit_seq,
                    (name, &variant_name),
                    overload,
                );
            }
        } else {
            if syntax.has_variadic_overload() {
                unreachable!(
                    "{name}::{variant_name} needs multiple variadic overloads, i.e. {variant_name}Options implemented"
                );
            } else {
                if name == "Number" && key == "-" {
                    let u = number_minus_union_ty
                        .as_ref()
                        .expect("Number minus operand union");
                    emit_number_minus_deserializer_arm(visit_seq, u.as_str());
                } else {
                    generate_syntax_enum_deserializer_multi_overload_variant(
                        visit_seq,
                        (name, &variant_name),
                    );
                }
            }
        }
        visit_seq.line("},");
    }

    let variants = values.keys().cloned().collect::<Vec<_>>();
    visit_seq.line(format!(
        "_ => Err(serde::de::Error::unknown_variant(&op, &[\"{}\"]))",
        variants.join("\", \"")
    ));
    visit_seq.line("}");
}

fn generate_visitor<'a>(scope: &'a mut Scope, name: &str, example: &Value) -> &'a mut Impl {
    let visitor_name = format!("{name}Visitor");
    scope
        .new_impl(name)
        .generic("'de")
        .impl_trait("serde::Deserialize<'de>")
        .new_fn("deserialize")
        .arg("deserializer", "D")
        .generic("D")
        .bound("D", "serde::Deserializer<'de>")
        .ret("Result<Self, D::Error>")
        .line(format!("deserializer.deserialize_seq({visitor_name})"));

    scope.new_struct(&visitor_name).doc(format!(
        "Visitor for deserializing the syntax enum [`{name}`]"
    ));

    let vis = scope
        .new_impl(&visitor_name)
        .generic("'de")
        .impl_trait("serde::de::Visitor<'de>")
        .associate_type("Value", name);
    let example_compact = serde_json::to_string(example).unwrap_or_default();
    let expecting_msg = format!("an {name} expression (example: {example_compact})");
    vis.new_fn("expecting")
        .arg_ref_self()
        .arg("formatter", "&mut std::fmt::Formatter")
        .ret("std::fmt::Result")
        .line(format!("formatter.write_str({expecting_msg:?})"));
    vis
}

/// generates a helper function for visiting a field
fn generate_visit_seq_field(visit_seq: &mut Function) {
    visit_seq
        .line("/// Reads the next element from the sequence or reports a missing field error.");
    visit_seq.line("#[allow(dead_code)]");
    visit_seq.line(
        "fn visit_seq_field<'de, A, T>(seq: &mut A, name: &'static str) -> Result<T, A::Error>",
    );
    visit_seq.line("where A: serde::de::SeqAccess<'de>, T: serde::Deserialize<'de> {");
    visit_seq.line("seq.next_element()?.ok_or_else(|| serde::de::Error::missing_field(name))");
    visit_seq.line("}");
    visit_seq.line("");
}

fn generate_syntax_enum_deserializer_multi_overload_variant(
    visit_seq: &mut Function,
    (name, variant_name): (&str, &str),
) {
    let options_name = format!("{variant_name}Options");

    visit_seq.line(format!(
        "// Delegate the remainder of the sequence to {options_name} deserialization"
    ));
    visit_seq
        .line("let remainder_of_sequence = serde::de::value::SeqAccessDeserializer::new(seq);");
    visit_seq.line(format!(
        "let options = <{options_name} as serde::Deserialize>::deserialize(remainder_of_sequence)?;"
    ));
    visit_seq.line(format!("Ok({name}::{variant_name}(options))"));
}

fn generate_syntax_enum_deserializer_regular_variadic_variant(
    visit_seq: &mut Function,
    (name, variant_name): (&str, &str),
    overload: &Overload,
    row_struct_name: Option<&str>,
    sep4_plan: Option<&VariadicSep4Plan>,
) {
    let position_of_variadic_separator = overload.position_of_variadic_separator();
    assert_ne!(position_of_variadic_separator, 0);

    if overload_uses_interpolate_style_variadic(overload) {
        let plan =
            sep4_plan.expect("interpolate-style variadic must have a precomputed VariadicSep4Plan");
        for line in &plan.header_lines {
            visit_seq.line(line.clone());
        }
        visit_seq.line("let mut stops = Vec::new();");
        let bind_a = &plan.pair_bind_a;
        let bind_b = &plan.pair_bind_b;
        let ty_a = &plan.pair_ty_a;
        let ty_b = &plan.pair_ty_b;
        visit_seq.line(format!(
            "while let Some({bind_a}) = seq.next_element::<{ty_a}>()? {{"
        ));
        if plan.pair_b_optional {
            visit_seq.line(format!(
                "let {bind_b}: {ty_b} = seq.next_element()?; // optional param"
            ));
        } else {
            visit_seq.line(format!(
                "let {bind_b}: {ty_b} = seq.next_element()?.ok_or_else(|| serde::de::Error::custom(\"expected {bind_b} in {name}::{variant_name}\"))?;"
            ));
        }

        visit_seq.line(format!("stops.push(({bind_a}, {bind_b}));"));
        visit_seq.line("}");
        visit_seq.line("if stops.is_empty() {".to_string());
        visit_seq.line(format!("return Err(serde::de::Error::custom(\"{name}::{variant_name} requires at least one stop pair\"));"));
        visit_seq.line("}");
        let hdr = plan.header_bind_names.join(", ");
        visit_seq.line(format!("Ok({name}::{variant_name}(({hdr}, stops)))"));
        return;
    }

    visit_seq.line("let mut inputs = Vec::new();");
    if position_of_variadic_separator == 1 {
        visit_seq.line("while let Some(element) = seq.next_element()? {");
    } else {
        let base_name = to_snake_case(&overload.parameters[0]).replace("_1", "_i");
        visit_seq.line(format!(
            "while let Some({base_name}) = seq.next_element()? {{"
        ));
        let non_base_parameters = &overload.parameters[1..position_of_variadic_separator]
            .iter()
            .map(|p| (to_snake_case(p).replace("_1", "_i"), p.ends_with('?')))
            .collect::<Vec<_>>();
        for (param_name, is_optional) in non_base_parameters {
            if *is_optional {
                visit_seq.line(format!(
                    "let {param_name} = seq.next_element()?; // optional param"
                ));
            } else {
                visit_seq.line(format!("let {param_name} = seq.next_element()?.ok_or_else(|| serde::de::Error::custom(\"expected {param_name} in {name}::{variant_name}\"))?;"));
            }
        }
        let tuple_inner = non_base_parameters
            .iter()
            .map(|(p, _)| p.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        if let Some(row) = row_struct_name {
            visit_seq.line(format!("let element = {row}({base_name},{tuple_inner});"));
        } else {
            visit_seq.line(format!("let element = ({base_name},{tuple_inner});"));
        }
    }
    visit_seq.line("inputs.push(element);");
    visit_seq.line("}");
    visit_seq.line("if inputs.is_empty() {".to_string());
    visit_seq.line(format!("return Err(serde::de::Error::custom(\"{name}::{variant_name} requires at least one argument\"));"));
    visit_seq.line("}");
    let suffix_params = variadic_non_template_suffix_parameters(overload);
    let mut suffix_binds: Vec<String> = Vec::new();
    for pname in &suffix_params {
        let is_opt = pname.ends_with('?');
        let lookup = pname.strip_suffix('?').unwrap_or(pname);
        let bind = if lookup == "type" {
            "r#type".to_string()
        } else {
            lookup.to_string()
        };
        if is_opt {
            visit_seq.line(format!("let {bind} = seq.next_element()?;"));
        } else if lookup == "type" {
            visit_seq.line("let r#type = visit_seq_field(&mut seq, \"type\")?;".to_string());
        } else {
            visit_seq.line(format!(
                "let {bind} = visit_seq_field(&mut seq, \"{lookup}\")?;"
            ));
        }
        suffix_binds.push(bind);
    }
    if suffix_params.is_empty() {
        visit_seq.line(format!("Ok({name}::{variant_name}(inputs))"));
    } else {
        visit_seq.line(format!(
            "Ok({name}::{variant_name}((inputs, {})))",
            suffix_binds.join(", ")
        ));
    }
}
fn generate_syntax_enum_deserializer_regular_variant(
    visit_seq: &mut Function,
    (name, variant_name): (&str, &str),
    overload: &Overload,
) {
    for param in &overload.parameters {
        if let Some(param) = param.strip_suffix('?') {
            visit_seq.line(format!("let {param} = seq.next_element()?;"));
        } else if param == "type" {
            visit_seq.line(format!(
                "let r#{param} = visit_seq_field(&mut seq, \"{param}\")?;"
            ));
        } else {
            visit_seq.line(format!(
                "let {param} = visit_seq_field(&mut seq, \"{param}\")?;"
            ));
        };
    }
    if overload.parameters.is_empty() {
        visit_seq.line(format!("Ok({name}::{variant_name})"));
    } else {
        let parameters = overload
            .parameters
            .iter()
            .map(|p| p.strip_suffix('?').unwrap_or(p))
            .map(|p| {
                if p == "type" {
                    format!("r#{}", p)
                } else {
                    p.to_string()
                }
            })
            .collect::<Vec<_>>();
        visit_seq.line(format!(
            "Ok({name}::{variant_name}({params}))",
            params = parameters.join(", ")
        ));
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::decoder::StyleReference;
    use crate::mir::IntermediateSpec;

    #[test]
    fn test_generate_spec_expressions() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "expression": {
          "type": "array",
          "value": "expression_name",
          "minimum": 1,
          "doc": "An expression defines a function that can be used for data-driven style properties or feature filters."
        },
        "expression_name": {
          "doc": "",
          "type": "enum",
          "values": {
            "let": {
              "doc": "Binds expressions to named variables.",
              "syntax": {
                "overloads": [
                  {
                    "parameters": ["var_name_1", "var_value_1", "...", "var_name_n", "var_value_n", "expression"],
                    "output-type": "any"
                  }
                ],
                "parameters": [
                  { "name": "var_name_i", "type": "string literal", "doc": "Variable name." },
                  { "name": "var_value_i", "type": "any", "doc": "Variable value." },
                  { "name": "expression", "type": "any", "doc": "Result expression." }
                ]
              },
              "example": ["let", "someNumber", 500, ["interpolate", ["linear"], ["var", "someNumber"], 274, "#edf8e9", 1551, "#006d2c"]],
              "group": "Variable binding",
              "sdk-support": {}
            }
          }
        }
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        let spec = IntermediateSpec::from(reference);
        insta::assert_snapshot!(crate::generator::generate_spec_scope(&spec));
    }

    #[test]
    fn test_generate_spec_interpolation() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "interpolation_name": {
          "doc": "First element in an interpolation array. May be followed by a number of arguments.",
          "type": "enum",
          "values": {
            "linear": {
              "doc": "Interpolates linearly between the pair of stops just less than and just greater than the input",
              "syntax": {
                "overloads": [{ "parameters": [], "output-type": "interpolation" }],
                "parameters": []
              },
              "example": ["linear"],
              "sdk-support": {}
            }
          }
        }
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        let spec = IntermediateSpec::from(reference);
        insta::assert_snapshot!(crate::generator::generate_spec_scope(&spec));
    }

    #[test]
    fn test_generate_spec_fmt() {
        let reference = json!({
          "$version": 8,
          "$root": {},
          "expression_name": {
            "doc": "First element in an expression array. May be followed by a number of arguments.",
            "type": "enum",
            "values": {
              "format": {
                "doc": "Returns a `formatted` string for displaying mixed-format text in the `text-field` property.",
                "syntax": {
                  "overloads": [
                    {
                      "parameters": ["input_1", "style_overrides_1?", "...", "input_n", "style_overrides_n?"],
                      "output-type": "formatted"
                    }
                  ],
                  "parameters": [
                    { "name": "input_i", "type": ["string", "image"] },
                    { "name": "style_overrides_i", "type": { "text-font": { "type": "string", "doc": "Font override." } } }
                  ]
                },
                "example": ["format", ["upcase", ["get", "FacilityName"]], {"font-scale": 0.8}, "\n\n", {}, ["downcase", ["get", "Comments"]], {"font-scale": 0.6}],
                "sdk-support": {}
              }
            }
          }
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        let spec = IntermediateSpec::from(reference);
        insta::assert_snapshot!(crate::generator::generate_spec_scope(&spec));
    }
}

use std::collections::BTreeMap;

use crate::decoder::DecodedTopLevelItem;
use crate::mir::expressions::{MirExpressionGroup, MirExpressions};
use crate::mir::types::{MirParameterType, MirSyntax, MirSyntaxVariantDef};

/// Remove and process `expression_name` and `expression` from the top-level fields,
/// returning the fully constructed [`MirExpressions`].
///
/// Two complementary representations are built from the same source data:
///
/// - `by_output_type` — the code-generator view (T expanded into each concrete type).
/// - `operators` — the optimizer view (flat map, T-polymorphism preserved).
pub fn preprocess_expression(fields: &mut BTreeMap<String, DecodedTopLevelItem>) -> MirExpressions {
    // not used in the generator or MIR
    let _ = fields.remove("filter_operator");
    let Some(_) = fields.remove("expression") else {
        // Not present in test fixtures — return empty.
        return MirExpressions {
            by_output_type: BTreeMap::new(),
            operators: BTreeMap::new(),
        };
    };

    let expression_name = fields
        .remove("expression_name")
        .expect("expression_name to be a top level item");

    // ── Build operators (optimizer view) ─────────────────────────────────────
    //
    // All operators are included with their T-polymorphic output preserved.
    let operators = MirExpressions::build_operators(&expression_name);

    // ── Build by_output_type (generator view) ────────────────────────────────
    //
    // This mirrors the generator's `reorder_expressions`: operators are grouped
    // by output-type name and T is expanded into every concrete output type.
    let by_output_type = build_by_output_type(&expression_name);

    MirExpressions {
        by_output_type,
        operators,
    }
}

// ── Generator view construction ───────────────────────────────────────────────

/// Group all operators from `expression_name` by output-type name, expanding T.
///
/// This exactly mirrors the generator's `reorder_expressions` but operates on
/// the already-decoded types rather than raw JSON.
fn build_by_output_type(
    expression_name: &DecodedTopLevelItem,
) -> BTreeMap<String, MirExpressionGroup> {
    let syntax_enum_values = {
        let (values, _common, default) = expression_name.as_item().as_primitive().as_enum();
        assert_eq!(
            default, None,
            "expression_name must not have a default value"
        );
        values.as_syntax_enum().clone()
    };

    let mut by_output_type: BTreeMap<String, MirExpressionGroup> = BTreeMap::new();
    // Tracks output type name → its ParameterType (needed to specialise T below).
    let mut possible_expressions: BTreeMap<String, MirParameterType> = BTreeMap::new();

    for (expr_key, syntax_enum) in &syntax_enum_values {
        for overload in &syntax_enum.syntax.overloads {
            let mir_overload: crate::mir::types::MirOverload = overload.clone().into();
            let output_type_name = mir_overload.output_type.to_upper_camel_case();
            possible_expressions.insert(output_type_name.clone(), mir_overload.output_type.clone());

            let group = by_output_type
                .entry(output_type_name.clone())
                .or_insert_with(|| MirExpressionGroup {
                    variants: BTreeMap::new(),
                });

            group
                .variants
                .entry(expr_key.clone())
                .and_modify(|def: &mut MirSyntaxVariantDef| {
                    // For `in`, the `substring/string` overload serialises identically to
                    // `item/array`, making `#[serde(untagged)]` non-deterministic on
                    // deserialisation. Drop it; `Item(ExprOrLiteral, ExprOrLiteral)` covers both.
                    let skip = expr_key == "in"
                        && mir_overload
                            .parameters
                            .first()
                            .is_some_and(|p| p == "substring");
                    // Comparison operators: merge string/number overloads into one
                    // with `any`-typed params (like == / !=).
                    let skip = skip || matches!(expr_key.as_str(), "<" | "<=" | ">" | ">=");
                    if !skip {
                        def.syntax.overloads.push(mir_overload.clone());
                    }
                })
                .or_insert_with(|| {
                    let mut parameters: Vec<_> = syntax_enum
                        .syntax
                        .parameters
                        .clone()
                        .into_iter()
                        .map(Into::into)
                        .collect();
                    MirSyntax::patch_expression_parameters(expr_key, &mut parameters);
                    MirSyntaxVariantDef {
                        doc: syntax_enum.doc.clone(),
                        syntax: MirSyntax {
                            overloads: vec![mir_overload],
                            parameters,
                        },
                        example: syntax_enum.example.clone(),
                        group: syntax_enum.group.clone(),
                    }
                });
        }
    }

    // Merge comparison overloads: rename params to input_1/input_2 with `any` type.
    if let Some(boolean_group) = by_output_type.get_mut("Boolean") {
        for (expr_key, def) in &mut boolean_group.variants {
            if matches!(expr_key.as_str(), "<" | "<=" | ">" | ">=") {
                MirSyntax::merge_comparison_overloads(expr_key, &mut def.syntax);
            }
        }
    }

    // Expand the T group into every concrete output type.
    if let Some(t_group) = by_output_type.remove("T") {
        let t_param = possible_expressions
            .remove("T")
            .expect("T must be in possible_expressions");
        assert_eq!(
            t_param,
            MirParameterType::Reference("T".to_string()),
            "T must be a reference parameter"
        );

        // Collect first to avoid borrow-checker conflict while mutating by_output_type.
        let real_output_types: Vec<(String, MirParameterType)> = possible_expressions
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        for (output_type_name, output_type_param) in real_output_types {
            let group = by_output_type
                .get_mut(&output_type_name)
                .expect("output type group must exist");

            for (k, v) in &t_group.variants {
                let mut specialised = v.clone();
                for o in specialised.syntax.overloads.iter_mut() {
                    o.output_type = output_type_param.clone();
                }
                group.variants.insert(k.clone(), specialised);
            }
        }
    }

    // `Any`-output operators (`get`, `let`, `case`, `match`, …) are intentionally
    // kept only in the `Any` group. They are polymorphic: their concrete return type is
    // determined by context, not by the operator itself. Typed expression enums (`Number`,
    // `String`, `Formatted`, …) represent operators with a FIXED output type.
    //
    // In typed parameter positions, an explicit `Any(Box<Any>)` union arm (generated by
    // `generate_expression_any_of`) handles polymorphic expressions. In generic positions
    // (`ExprOrLiteral`), the `AnyExpr(Box<Any>)` variant covers them.
    //
    // Exception: `step` is expanded into per-output-type ramp groups (like `interpolate`).
    // In typed paint/layout positions, the output type IS known, so a typed `Step` variant
    // enables structural passes (e.g. zoom-bounded stop pruning) on the typed representation.
    expand_step_into_ramp_groups(&mut by_output_type);

    // Merge typed array groups (e.g. `ArrayOfString`) into the generic `Array` group.
    // `array<string>` is-a `array`, so `Array` must accept operators like `split` that
    // return a typed array subtype.
    merge_typed_arrays_into_array(&mut by_output_type);

    // Handle the case where the expression_name uses `DecodedEnumValues::Version` — not
    // present in production but guards test fixtures that lack DecodedSyntaxEnum values.
    by_output_type
}

/// Merge operators from typed array groups (e.g. `ArrayOfString`) into the
/// generic `Array` group, since `array<T>` is a subtype of `array`.
fn merge_typed_arrays_into_array(by_output_type: &mut BTreeMap<String, MirExpressionGroup>) {
    let typed_array_keys: Vec<String> = by_output_type
        .keys()
        .filter(|k| k.starts_with("ArrayOf") && *k != "ArrayOfType")
        .cloned()
        .collect();

    if typed_array_keys.is_empty() {
        return;
    }

    let mut to_merge: Vec<(String, MirSyntaxVariantDef)> = Vec::new();
    for key in &typed_array_keys {
        if let Some(group) = by_output_type.get(key) {
            for (op_name, variant) in &group.variants {
                to_merge.push((op_name.clone(), variant.clone()));
            }
        }
    }

    if let Some(array_group) = by_output_type.get_mut("Array") {
        for (op_name, variant) in to_merge {
            array_group.variants.entry(op_name).or_insert(variant);
        }
    }
}

/// Expand `step` from the `Any` group into per-output-type ramp groups.
///
/// `step` uses `output-type: any` in v8.json, so it normally lives only in `Any`.
/// However, in typed paint/layout positions the output type is known. We clone
/// `step` into every group that already contains an `interpolate`-family operator,
/// rewriting the `output_0` and `stop_output_i` parameter types to match that
/// group's output union type.
fn expand_step_into_ramp_groups(by_output_type: &mut BTreeMap<String, MirExpressionGroup>) {
    let step_def = by_output_type
        .get("Any")
        .and_then(|g| g.variants.get("step"))
        .cloned();
    let Some(step_def) = step_def else { return };

    // Collect (group_name, stop_output_i parameter type) from groups that have
    // interpolate-family operators. These are the ramp groups where step should also live.
    let ramp_targets: Vec<(String, MirParameterType)> = by_output_type
        .iter()
        .filter_map(|(group_name, group)| {
            let interp_def = group
                .variants
                .iter()
                .find(|(k, _)| k.starts_with("interpolate"))?;
            let output_param = interp_def
                .1
                .syntax
                .parameters
                .iter()
                .find(|p| p.name == "stop_output_i")?;
            Some((group_name.clone(), output_param.r#type.clone()))
        })
        .collect();

    for (group_name, output_param_type) in ramp_targets {
        let mut typed_step = step_def.clone();
        // The v8.json example uses numeric outputs; clear it to avoid auto-generated
        // tests that try to deserialize numbers as colors/projections.
        typed_step.example = None;
        // Rewrite output_0 and stop_output_i from `any` to the group's output type.
        for param in &mut typed_step.syntax.parameters {
            if param.name == "output_0" || param.name == "stop_output_i" {
                param.r#type = output_param_type.clone();
            }
        }
        for overload in &mut typed_step.syntax.overloads {
            overload.output_type = output_param_type.clone();
        }
        by_output_type
            .get_mut(&group_name)
            .expect("group must exist")
            .variants
            .insert("step".to_string(), typed_step);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::StyleReference;

    #[test]
    fn test_expression_name_renaming() {
        let mut reference: StyleReference =
            serde_json::from_str(include_str!("../../fixture/expression_name_renaming.json"))
                .unwrap();
        let exprs = preprocess_expression(&mut reference.fields);
        // Both keys consumed from fields.
        assert!(!reference.fields.contains_key("expression_name"));
        assert!(!reference.fields.contains_key("expression"));
        // The operators map must be populated.
        assert!(
            !exprs.operators.is_empty() || exprs.by_output_type.is_empty(),
            "operators must be populated when by_output_type is non-empty"
        );
    }

    #[test]
    fn test_operators_not_expanded() {
        // Verify that polymorphic T operators appear with TypeVar output,
        // not expanded into concrete types.
        use serde_json::json;

        use crate::mir::expressions::MirExprType;

        let fixture = json!({
            "$version": 8,
            "$root": {},
            "expression": {
                "type": "array",
                "value": "expression_name",
                "minimum": 1,
                "doc": ""
            },
            "expression_name": {
                "doc": "",
                "type": "enum",
                "values": {
                    "literal": {
                        "doc": "Returns the input value as-is.",
                        "syntax": {
                            "overloads": [{"parameters": ["value"], "output-type": "T"}],
                            "parameters": [{"name": "value", "type": "any", "doc": "Any value."}]
                        },
                        "sdk-support": {}
                    }
                }
            }
        });
        let reference: StyleReference = serde_json::from_value(fixture).unwrap();
        // Call the preprocessor directly to avoid needing a full spec with layers.
        let mut fields = reference.fields;
        let exprs = preprocess_expression(&mut fields);

        let literal = exprs
            .operators
            .get("literal")
            .expect("literal operator must exist");
        assert_eq!(
            literal.overloads.len(),
            1,
            "literal must have exactly one overload"
        );
        assert_eq!(
            literal.overloads[0].output,
            MirExprType::TypeVar("T".to_string()),
            "literal's output must be TypeVar(T), not expanded"
        );
    }
}

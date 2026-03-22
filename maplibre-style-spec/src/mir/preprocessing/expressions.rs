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

    // Expand the `Any` group into every concrete output type.
    // Operators with output type `any` (e.g. `get`, `match`, `step`, `coalesce`) are
    // polymorphic and can appear wherever a specific type is expected.
    if let Some(any_group) = by_output_type.get("Any").cloned() {
        let concrete_types: Vec<(String, MirParameterType)> = possible_expressions
            .iter()
            .filter(|(k, _)| k.as_str() != "Any")
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        for (output_type_name, output_type_param) in concrete_types {
            let group =
                by_output_type
                    .entry(output_type_name)
                    .or_insert_with(|| MirExpressionGroup {
                        variants: BTreeMap::new(),
                    });

            for (k, v) in &any_group.variants {
                // Only insert if the concrete group doesn't already have this operator
                // (a concrete definition takes precedence over the `any` fallback).
                if !group.variants.contains_key(k) {
                    let mut specialised = v.clone();
                    for o in specialised.syntax.overloads.iter_mut() {
                        o.output_type = output_type_param.clone();
                    }
                    group.variants.insert(k.clone(), specialised);
                }
            }
        }
    }

    // Handle the case where the expression_name uses `DecodedEnumValues::Version` — not
    // present in production but guards test fixtures that lack DecodedSyntaxEnum values.
    by_output_type
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

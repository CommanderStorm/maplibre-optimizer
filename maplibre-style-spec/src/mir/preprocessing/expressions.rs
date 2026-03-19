use std::collections::BTreeMap;

use crate::decoder::r#enum::{EnumValues, ParameterType, Syntax};
use crate::decoder::TopLevelItem;
use crate::mir::expressions::{ExpressionGroup, IntermediateExpressions};
use crate::mir::types::SyntaxVariantDef;

/// Remove and process `expression_name` and `expression` from the top-level fields,
/// returning grouped expressions keyed by output type.
///
/// This is the MIR equivalent of the generator's `reorder_expressions`.
pub fn preprocess_expression(
    fields: &mut BTreeMap<String, TopLevelItem>,
) -> IntermediateExpressions {
    let Some(_) = fields.remove("expression") else {
        // Not present in test fixtures — return empty.
        return IntermediateExpressions {
            by_output_type: BTreeMap::new(),
        };
    };

    let expression_name = fields
        .remove("expression_name")
        .expect("expression_name to be a top level item");

    let syntax_enum_values = {
        let (values, _common, default) = expression_name
            .as_item()
            .as_primitive()
            .as_enum();
        assert_eq!(
            default, None,
            "expression_name must not have a default value"
        );
        values
            .as_syntax_enum()
            .clone()
    };

    let mut by_output_type: BTreeMap<String, ExpressionGroup> = BTreeMap::new();
    // Maps output type name → ParameterType (needed for T specialization)
    let mut possible_expressions: BTreeMap<String, ParameterType> = BTreeMap::new();

    for (expr_key, syntax_enum) in &syntax_enum_values {
        for overload in &syntax_enum.syntax.overloads {
            let output_type_name = overload.output_type.to_upper_camel_case();
            possible_expressions.insert(output_type_name.clone(), overload.output_type.clone());

            let group = by_output_type
                .entry(output_type_name.clone())
                .or_insert_with(|| ExpressionGroup {
                    variants: BTreeMap::new(),
                });

            group
                .variants
                .entry(expr_key.clone())
                .and_modify(|def: &mut SyntaxVariantDef| def.syntax.overloads.push(overload.clone()))
                .or_insert_with(|| SyntaxVariantDef {
                    doc: syntax_enum.doc.clone(),
                    syntax: Syntax {
                        overloads: vec![overload.clone()],
                        parameters: syntax_enum.syntax.parameters.clone(),
                    },
                    example: syntax_enum.example.clone(),
                    group: syntax_enum.group.clone(),
                });
        }
    }

    // Handle the generic "T" type: specialise its expressions into each real output type.
    if let Some(t_group) = by_output_type.remove("T") {
        // Verify T is a ParameterType::Reference("T") as expected
        let t_param = possible_expressions
            .remove("T")
            .expect("T must be in possible_expressions");
        assert_eq!(
            t_param,
            ParameterType::Reference("T".to_string()),
            "T must be a reference parameter"
        );

        let real_output_types: Vec<(String, ParameterType)> = possible_expressions
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

    IntermediateExpressions { by_output_type }
}

#[test]
fn test_expression_name_renaming() {
    use crate::decoder::StyleReference;
    let mut reference: StyleReference =
        serde_json::from_str(include_str!("../../fixture/expression_name_renaming.json")).unwrap();
    let _exprs = preprocess_expression(&mut reference.fields);
    // Verify that expression_name and expression have been consumed from fields
    assert!(!reference.fields.contains_key("expression_name"));
    assert!(!reference.fields.contains_key("expression"));
}

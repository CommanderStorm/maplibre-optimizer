use std::collections::{BTreeMap, HashMap};

use codegen2::Scope;

use crate::decoder::r#enum::{EnumValues, ParameterType};
use crate::decoder::{Fields, ParsedItem, PrimitiveType, StyleReference, TopLevelItem};
use crate::generator::formatter::{to_snake_case, to_upper_camel_case};
use crate::generator::literals::generate_literals;

mod autotest;
pub mod formatter;
mod items;
mod literals;

pub fn generate_spec_scope(mut reference: StyleReference) -> String {
    let mut scope = Scope::new();
    assert_eq!(reference.version, 8);
    reorder_expressions(&mut reference.fields);

    generate_spec(&mut scope, &reference.root);
    generate_literals(&mut scope);
    let discriminants = extract_and_remove_discriminants(&mut reference.fields);

    for (key, item) in &reference.fields {
        let name = to_upper_camel_case(key);
        generate_top_level_item(&mut scope, item, &name, &discriminants)
    }
    scope
        .get_or_new_module("test")
        .attr("cfg(test)")
        .import("super", "*");

    scope.to_string()
}

fn reorder_expressions(fields: &mut BTreeMap<String, TopLevelItem>) {
    let Some(_) = fields.remove("expression") else {
        return; // we are in a testcase
    };
    let mut possible_expressions = HashMap::new();
    let expression_name = fields
        .remove("expression_name")
        .expect("expression_name to be a top level item");
    let expr_name_values = {
        let (values, _common, default) = expression_name
            .as_item()
            .expect("expression_name must be an item")
            .as_primitive()
            .expect("expression_name must be a primitive")
            .as_enum()
            .expect("expression_name must be an enum");
        assert_eq!(
            default, None,
            "expression_name must not have a default value.. effects are a little unclear"
        );
        let values = values
            .as_syntax_enum()
            .expect("expression_name must be syntax enum");
        values
    };

    for (key, syntax_enum) in expr_name_values {
        for overload in &syntax_enum.syntax.overloads {
            let output_type_name = overload.output_type.to_upper_camel_case();
            possible_expressions.insert(output_type_name.clone(), overload.output_type.clone());

            fields
                .entry(output_type_name.clone())
                .and_modify(|f| {
                    f.as_item_mut()
                        .expect("is a syntax enum")
                        .as_primitive_mut()
                        .expect("is a syntax enum")
                        .enum_values_mut()
                        .expect("is a syntax enum")
                        .as_syntax_enum_mut()
                        .expect("is a syntax enum")
                        .entry(key.clone())
                        .and_modify(|e| e.syntax.overloads.push(overload.clone()))
                        .or_insert_with(|| {
                            let mut s = syntax_enum.clone();
                            s.syntax.overloads = vec![overload.clone()];
                            s
                        });
                })
                .or_insert_with(|| {
                    let mut tl_common = Fields::default();
                    tl_common.doc = format!("{output_type_name:?}");
                    TopLevelItem::Item(Box::new(ParsedItem::Primitive(PrimitiveType::Enum {
                        common: tl_common,
                        default: None,
                        values: EnumValues::SyntaxEnum([(key.clone(), syntax_enum.clone())].into()),
                    })))
                });
        }
    }

    // because we are funny, t values can be any value :tada:
    // -> for correct codegen, we need to insert them anywhere
    // for tests, this is not enforced to be present :)
    if let Some(param_t) = possible_expressions.remove("T") {
        assert_eq!(param_t, ParameterType::Reference("T".to_string()));
        let t = fields.remove("T").expect("T must be a top level item");
        let t_values = t
            .as_item()
            .expect("T must be an item")
            .as_primitive()
            .expect("T must be a primitive")
            .as_enum()
            .expect("T must be an enum")
            .0
            .as_syntax_enum()
            .expect("T must be a syntax enum");
        for (expr_type_name, more_specific_t_type) in &possible_expressions {
            let expr_type_values = fields
                .get_mut(expr_type_name)
                .unwrap()
                .as_item_mut()
                .expect("expr_type must be an item")
                .as_primitive_mut()
                .expect("expr_type must be a primitive")
                .enum_values_mut()
                .expect("expr_type must be an enum")
                .as_syntax_enum_mut()
                .expect("expr_type must be a syntax enum");
            for (k, v) in t_values {
                let mut specialised_v = v.clone();
                for o in specialised_v.syntax.overloads.iter_mut() {
                    o.output_type = more_specific_t_type.clone();
                }
                expr_type_values.insert(k.clone(), specialised_v);
            }
        }
    }

    let mut keys = possible_expressions.into_keys().collect::<Vec<_>>();
    keys.sort_unstable();
    fields.insert("expression".to_string(), TopLevelItem::OneOf(keys));
}

fn extract_and_remove_discriminants(
    fields: &mut BTreeMap<String, TopLevelItem>,
) -> Vec<(String, String, String, String)> {
    // collect where to search
    let anyof_top_level_fields = fields.iter().filter_map(|f| match f {
        (k, TopLevelItem::OneOf(i)) => Some((k.clone(), i.clone())),
        _ => None,
    });

    // collect discriminants at the search places
    let mut discriminants = Vec::new();
    for (top_name, join_keys) in anyof_top_level_fields {
        let variant_name = to_upper_camel_case(&top_name);
        for join_key in join_keys {
            let joined_top_level = fields.get(&join_key).unwrap_or_else(|| panic!("anyof {top_name} does refer to its variant {join_key}, but there is no join partner"));
            let TopLevelItem::Group(btree_map) = joined_top_level else {
                // cannot possibly contain a discriminant
                continue;
            };
            for (descriminant, item) in btree_map {
                if let ParsedItem::Primitive(PrimitiveType::Enum { values, .. }) = item
                    && values.len() == 1
                {
                    let EnumValues::Enum(enum_values) = values else {
                        unreachable!("the version is not referenced in an anyof")
                    };
                    let value = enum_values.keys().next().expect("value length is 1");
                    discriminants.push((
                        variant_name.clone(),
                        join_key.clone(),
                        descriminant.clone(),
                        value.clone(),
                    ));
                }
            }
        }
    }

    // now remove the discriminants
    for (k, v) in fields.iter_mut() {
        if let TopLevelItem::Group(i) = v {
            for (_, k2, tag, _) in &discriminants {
                if k == k2 {
                    let _ = i.remove(tag);
                }
            }
        }
    }
    discriminants
}

fn generate_spec(scope: &mut Scope, root: &BTreeMap<String, ParsedItem>) {
    let spec = scope
        .new_struct("MaplibreStyleSpecification")
        .doc("This is a Maplibre Style Specification")
        .vis("pub")
        .derive("serde::Deserialize, PartialEq, Debug, Clone");
    for (key, field) in root {
        let mut field_type_name = to_upper_camel_case(&format!("root {key}"));
        if field.optional() {
            field_type_name = format!("Option<{field_type_name}>");
        }

        let fields = spec
            .new_field(to_snake_case(key), field_type_name)
            .vis("pub")
            .doc(field.doc());
        if &to_snake_case(key) != key {
            fields.annotation(format!("#[serde(rename=\"{key}\")]"));
        }
    }
    for (key, field) in root {
        generate_parsed_item(scope, field, &to_upper_camel_case(&format!("root {key}")));
    }
}

fn generate_top_level_item(
    scope: &mut Scope,
    item: &TopLevelItem,
    name: &str,
    discriminants: &[(String, String, String, String)],
) {
    if name == "PropertyType" {
        return; // bogus, not a style spec item, rather metadata about the item
    }
    match item {
        TopLevelItem::Item(item) => generate_parsed_item(scope, item, name),
        TopLevelItem::Group(items) => generate_top_level_group(scope, items, name),
        TopLevelItem::OneOf(items) => generate_top_level_oneof(scope, items, name, discriminants),
    }
}

fn generate_top_level_group(scope: &mut Scope, items: &BTreeMap<String, ParsedItem>, name: &str) {
    // special case for *:
    if items.len() == 1
        && let Some(item) = items.get("*")
    {
        let field_type_name = to_upper_camel_case(&format!("Inner {name}"));
        scope
            .new_struct(name)
            .vis("pub")
            .derive("serde::Deserialize, PartialEq, Debug, Clone")
            .tuple_field(format!(
                "std::collections::BTreeMap<String,{field_type_name}>"
            ));
        generate_parsed_item(scope, item, &field_type_name);

        return;
    }

    let group = scope
        .new_struct(name)
        .vis("pub")
        .derive("serde::Deserialize, PartialEq, Debug, Clone");
    for (key, item) in items {
        let mut field_type_name = to_upper_camel_case(&format!("{name} {key}"));
        if key == "*" {
            field_type_name = format!("std::collections::BTreeMap<String,{field_type_name}>");
        }
        if item.optional() {
            field_type_name = format!("Option<{field_type_name}>");
        }
        let field = group
            .new_field(to_snake_case(key), field_type_name)
            .doc(item.doc())
            .vis("pub");
        if key == "*" {
            field.annotation("#[serde(flatten)]");
        } else if &to_snake_case(key) != key {
            field.annotation(format!("#[serde(rename=\"{key}\")]"));
        }
    }

    for (key, item) in items {
        generate_parsed_item(scope, item, &to_upper_camel_case(&format!("{name} {key}")));
    }
}

fn generate_top_level_oneof(
    scope: &mut Scope,
    items: &Vec<String>,
    name: &str,
    discriminants: &[(String, String, String, String)],
) {
    let enu = scope
        .new_enum(name)
        .vis("pub")
        .derive("serde::Deserialize, PartialEq, Debug, Clone");

    let enum_variant_key = discriminants
        .iter()
        .find(|d| d.0 == name)
        .map(|d| d.2.clone());
    let descriminants = discriminants
        .iter()
        .filter(|d| d.0 == name)
        .cloned()
        .map(|d| (d.1, d.3))
        .collect::<HashMap<_, _>>();

    if let Some(enum_variant_key) = enum_variant_key {
        enu.attr(format!("serde(tag=\"{enum_variant_key}\")"));
    } else {
        enu.attr("serde(untagged)");
    }
    for key in items {
        let var_name = to_upper_camel_case(key);
        let var = enu.new_variant(&var_name).tuple(&var_name);
        if &var_name != key
            && let Some(descriminant_key) = descriminants.get(key)
        {
            var.annotation(format!("#[serde(rename=\"{descriminant_key}\")]"));
        }
    }
}

fn generate_parsed_item(scope: &mut Scope, item: &ParsedItem, name: &str) {
    match item {
        ParsedItem::Primitive(p) => match p {
            PrimitiveType::Number {
                common,
                default,
                maximum,
                minimum,
                period,
            } => items::number::generate(
                scope,
                name,
                common,
                default.as_ref(),
                maximum.as_ref(),
                minimum.as_ref(),
                period.as_ref(),
            ),
            PrimitiveType::Enum {
                common,
                default,
                values,
            } => items::r#enum::generate(scope, name, common, default.as_ref(), values),
            PrimitiveType::Array {
                common,
                default,
                value,
                values,
                minimum,
                maximum,
                length,
            } => items::array::generate(
                scope,
                name,
                common,
                default.as_ref(),
                value,
                values.as_ref(),
                minimum.as_ref(),
                maximum.as_ref(),
                length.as_ref(),
            ),
            PrimitiveType::Color { common, default } => {
                items::color::generate(scope, name, common, default.as_ref())
            }
            PrimitiveType::String { common, default } => {
                items::string::generate(scope, name, common, default.as_deref())
            }
            PrimitiveType::Boolean { common, default } => {
                items::boolean::generate(scope, name, common, default.as_ref())
            }
            PrimitiveType::ResolvedImage { common, tokens } => {
                items::resolved_image::generate(scope, name, common, *tokens)
            }
            PrimitiveType::NumberArray {
                common,
                default,
                minimum,
                maximum,
            } => items::number_array::generate(
                scope,
                name,
                common,
                default.as_ref(),
                minimum.as_ref(),
                maximum.as_ref(),
            ),
            PrimitiveType::ColorArray { common, default } => {
                items::color_array::generate(scope, name, common, default.as_deref())
            }

            PrimitiveType::Padding { common, default } => {
                items::padding::generate(scope, name, common, default.as_ref())
            }
            PrimitiveType::Formatted {
                common,
                tokens,
                default,
            } => items::formatted::generate(scope, name, common, default, *tokens),

            // meta-types, not something proper but still useful to handle explicitly
            PrimitiveType::Star(fields) => items::star::generate(scope, name, fields),
            PrimitiveType::State { common, default } => {
                items::state::generate(scope, name, common, default)
            }
            PrimitiveType::PropertyType(_) => {}

            // below are types which are only primitives due to bad spec work upstream
            PrimitiveType::ProjectionDefinition { common, default } => {
                items::projection_definition::generate(scope, name, common, default.as_str())
            }
            PrimitiveType::VariableAnchorOffsetCollection(fields) => {
                items::variable_anchor_offset_collection::generate(scope, name, fields)
            }
            PrimitiveType::Sprite(fields) => items::sprite::generate(scope, name, fields),
            PrimitiveType::PromoteId(fields) => items::promote_id::generate(scope, name, fields),
        },
        ParsedItem::Reference { references, common } => {
            items::reference::generate(scope, name, references, common)
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    #[test]
    fn test_generate_spec_items() {
        let reference = json!({
            "$version": 8,
            "$root": {},
            "number_one": {
              "doc": "A number between 0 and 10.",
              "type": "number",
              "default": 0
            }
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(generate_spec_scope(reference));
    }

    #[test]
    fn test_generate_spec_groups() {
        let reference = json!({
            "$version": 8,
            "$root": {},
            "names": {
              "name_one": {
                "type": "number",
                "doc": "A number between 0 and 10.",
                "default": 1.0
              }
            }
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        let spec = generate_spec_scope(reference);
        insta::assert_snapshot!(&spec);
    }

    #[test]
    fn test_generate_spec_oneof() {
        let reference = json!({
            "$version": 8,
            "$root": {},
            "number_one": {
              "type": "number",
              "doc": "A number between 0 and 20.",
              "default": 1.0,
              "minimum": 0.0,
              "maximum": 10.0
            },
            "number_two": {
              "type": "number",
              "doc": "Another number"
            },
            "numbers": ["number_one", "number_two"]
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(generate_spec_scope(reference));
    }

    #[test]
    fn test_expression_name_renaming() {
        let mut reference: StyleReference =
            serde_json::from_str(include_str!("../fixture/expression_name_renaming.json")).unwrap();
        reorder_expressions(&mut reference.fields);
        insta::assert_json_snapshot!(reference);
    }
}

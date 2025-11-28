use std::collections::{BTreeMap, HashMap};

use codegen::Scope;

use crate::decoder::{EnumValues, ParsedItem, StyleReference, TopLevelItem};
use crate::generator::formatter::{to_snake_case, to_upper_camel_case};

mod autotest;
mod formatter;
mod items;

pub fn generate_spec_scope(mut reference: StyleReference) -> String {
    let mut scope = Scope::new();

    assert_eq!(reference.version, 8);

    generate_spec(&mut scope, &reference.root);
    let discriminants = extract_and_remove_discriminants(&mut reference.fields);

    for (key, item) in &reference.fields {
        let name = to_upper_camel_case(&key);
        generate_top_level_item(&mut scope, &item, &name, &discriminants)
    }
    scope
        .get_or_new_module("test")
        .attr("cfg(test)")
        .import("super", "*");

    scope.to_string()
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
            let joined_top_level =fields.get(&join_key).expect(&format!("anyof {top_name} does refer to its variant {join_key}, but there is no join partner"));
            let TopLevelItem::Group(btree_map) = joined_top_level else {
                // cannot possibly contain a discriminant
                continue;
            };
            for (descriminant, item) in btree_map {
                if let ParsedItem::Enum { values, .. } = item
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
        generate_parsed_item(scope, &field, &to_upper_camel_case(&format!("root {key}")));
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
        TopLevelItem::Item(item) => generate_parsed_item(scope, &item, name),
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
        generate_parsed_item(scope, &item, &field_type_name);

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
        } else {
            if &to_snake_case(key) != key {
                field.annotation(format!("#[serde(rename=\"{key}\")]"));
            }
        }
    }

    for (key, item) in items {
        generate_parsed_item(scope, &item, &to_upper_camel_case(&format!("{name} {key}")));
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
        .filter(|d| &d.0 == name)
        .next()
        .map(|d| d.2.clone());
    let descriminants = discriminants
        .iter()
        .filter(|d| &d.0 == name)
        .cloned()
        .map(|d| (d.1, d.3))
        .collect::<HashMap<_, _>>();

    if let Some(enum_variant_key) = enum_variant_key {
        enu.attr(format!("serde(tag=\"{enum_variant_key}\")"));
    } else {
        enu.attr("serde(untagged)");
    }
    for key in items {
        let var_name = to_upper_camel_case(&key);
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
        ParsedItem::Number {
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
        ParsedItem::Enum {
            common,
            default,
            values,
        } => items::r#enum::generate(scope, name, common, default.as_ref(), values),
        ParsedItem::Array {
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
        ParsedItem::Color { common, default } => {
            items::color::generate(scope, name, common, default.as_ref())
        }
        ParsedItem::String { common, default } => {
            items::string::generate(scope, name, common, default.as_deref())
        }
        ParsedItem::Boolean { common, default } => {
            items::boolean::generate(scope, name, common, default.as_ref())
        }
        ParsedItem::Star(fields) => items::star::generate(scope, name, fields),
        ParsedItem::PropertyType(_) => {} // a meta-type, not something to have in the decoding code
        ParsedItem::ResolvedImage { common, tokens } => {
            items::resolved_image::generate(scope, name, common, *tokens)
        }
        ParsedItem::PromoteId(fields) => items::promote_id::generate(scope, name, fields),
        ParsedItem::NumberArray {
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
        ParsedItem::ColorArray { common, default } => {
            items::color_array::generate(scope, name, common, default.as_deref())
        }
        ParsedItem::VariableAnchorOffsetCollection(fields) => {
            items::variable_anchor_offset_collection::generate(scope, name, fields)
        }
        ParsedItem::Transition(fields) => items::transition::generate(scope, name, fields),
        ParsedItem::Terrain(fields) => items::terrain::generate(scope, name, fields),
        ParsedItem::State { common, default } => {
            items::state::generate(scope, name, common, default)
        }
        ParsedItem::Sprite(fields) => items::sprite::generate(scope, name, fields),
        ParsedItem::Sources(fields) => items::sources::generate(scope, name, fields),
        ParsedItem::Source(fields) => items::source::generate(scope, name, fields),
        ParsedItem::Sky(fields) => items::sky::generate(scope, name, fields),
        ParsedItem::ProjectionDefinition { common, default } => {
            items::projection_definition::generate(scope, name, common, default.as_str())
        }
        ParsedItem::Projection(fields) => items::projection::generate(scope, name, fields),
        ParsedItem::Paint(fields) => items::paint::generate(scope, name, fields),
        ParsedItem::Padding { common, default } => {
            items::padding::generate(scope, name, common, default.as_ref())
        }
        ParsedItem::Light(fields) => items::light::generate(scope, name, fields),
        ParsedItem::Layout(fields) => items::layout::generate(scope, name, fields),
        ParsedItem::Formatted {
            common,
            tokens,
            default,
        } => items::formatted::generate(scope, name, common, default, *tokens),
        ParsedItem::Filter(fields) => items::filter::generate(scope, name, fields),
        ParsedItem::Expression(fields) => items::expression::generate(scope, name, fields),
        ParsedItem::FontFaces(fields) => items::font_faces::generate(scope, name, fields),
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
        insta::assert_snapshot!(generate_spec_scope(reference), @r#"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        /// A number between 0 and 10.
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct NumberOne(serde_json::Number);

        impl Default for NumberOne {
            fn default() -> Self {
                Self(serde_json::Number::from_i128(0).expect("the number is serialised from a number and is thus always valid"))
            }
        }

        #[cfg(test)] 
        mod test {
            use super::*;

        }
        "#);
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
        insta::assert_snapshot!(&spec, @r#"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct Names {
            /// A number between 0 and 10.
            pub name_one: Option<NamesNameOne>,
        }

        /// A number between 0 and 10.
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct NamesNameOne(serde_json::Number);

        impl Default for NamesNameOne {
            fn default() -> Self {
                Self(serde_json::Number::from_f64(1.0).expect("the number is serialised from a number and is thus always valid"))
            }
        }

        #[cfg(test)] 
        mod test {
            use super::*;

        }
        "#);
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
        insta::assert_snapshot!(generate_spec_scope(reference), @r#"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        /// A number between 0 and 20.
        ///
        /// Range: 0.0..=10.0
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct NumberOne(serde_json::Number);

        impl Default for NumberOne {
            fn default() -> Self {
                Self(serde_json::Number::from_f64(1.0).expect("the number is serialised from a number and is thus always valid"))
            }
        }

        /// Another number
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct NumberTwo(serde_json::Number);

        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        #[serde(untagged)]
        pub enum Numbers {
            NumberOne(NumberOne),
            NumberTwo(NumberTwo),
        }

        #[cfg(test)] 
        mod test {
            use super::*;

        }
        "#);
    }
}

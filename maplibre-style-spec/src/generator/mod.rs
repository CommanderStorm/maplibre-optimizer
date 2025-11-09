use std::collections::BTreeMap;

use codegen::Scope;

use crate::decoder::{ParsedItem, StyleReference, TopLevelItem};
use crate::generator::formatter::{to_snake_case, to_upper_camel_case};

mod autotest;
mod formatter;
mod items;

pub fn generate_spec_scope(reference: StyleReference) -> String {
    let mut scope = Scope::new();

    assert_eq!(reference.version, 8);

    generate_spec(&mut scope, &reference.root);
    for (key, item) in reference.fields.into_iter() {
        generate_top_level_item(&mut scope, item, &to_upper_camel_case(&key))
    }
    scope
        .get_or_new_module("test")
        .attr("cfg(test)")
        .import("super", "*");

    scope.to_string()
}

fn generate_spec(scope: &mut Scope, root: &BTreeMap<String, ParsedItem>) {
    let spec = scope
        .new_struct("MaplibreStyleSpecification")
        .doc("This is a Maplibre Style Specification")
        .vis("pub")
        .derive("serde::Deserialize, PartialEq, Debug, Clone");
    for (key, field) in root {
        let fields = spec
            .new_field(
                to_snake_case(key),
                to_upper_camel_case(&format!("root {key}")),
            )
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

fn generate_top_level_item(scope: &mut Scope, item: TopLevelItem, name: &str) {
    if name == "PropertyType" {
        return; // bogus, not a style spec item, rather metadata about the item
    }
    match item {
        TopLevelItem::Item(item) => generate_parsed_item(scope, &item, name),
        TopLevelItem::Group(items) => {
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
            {
                let group = scope
                    .new_struct(name)
                    .vis("pub")
                    .derive("serde::Deserialize, PartialEq, Debug, Clone");
                for (key, item) in &items {
                    let mut field_type_name = to_upper_camel_case(&format!("{name} {key}"));
                    if key == "*" {
                        field_type_name =
                            format!("std::collections::BTreeMap<String,{field_type_name}>");
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
            }
            for (key, item) in items {
                generate_parsed_item(scope, &item, &to_upper_camel_case(&format!("{name} {key}")));
            }
        }
        TopLevelItem::OneOf(items) => {
            let enu = scope
                .new_enum(name)
                .attr("serde(untagged)")
                .vis("pub")
                .derive("serde::Deserialize, PartialEq, Debug, Clone");
            for key in items {
                let var = to_upper_camel_case(&key);
                enu.new_variant(&var).tuple(&var);
            }
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
            pub name_one: NamesNameOne,
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

use std::collections::HashMap;

use codegen::Scope;

use crate::decoder::{ ParsedItem, StyleReference, TopLevelItem};
mod items;

pub fn generate_spec_scope(reference: StyleReference) -> String {
    let mut scope = Scope::new();

    assert_eq!(reference.version, 8);

    generate_spec(&mut scope, &reference.root);
    for (key, item) in reference.fields.into_iter() {
        generate_top_level_item(&mut scope, item, &to_upper_camel_case(&key))
    }
    scope.to_string()
}

fn generate_spec(scope: &mut Scope, root: &HashMap<String, ParsedItem>) {
    let spec = scope
        .new_struct("MaplibreStyleSpecification")
        .doc("This is a Maplibre Style Specification")
        .vis("pub")
        .derive("serde::Deserialize, PartialEq, Debug, Clone");
    for (key, field) in root {
        spec.new_field(key, to_upper_camel_case(key))
            .vis("pub")
            .doc(field.doc());
    }
}

fn generate_top_level_item(scope: &mut Scope, item: TopLevelItem, name: &str) {
    match item {
        TopLevelItem::Item(item) => generate_parsed_item(scope, &item, &name),
        TopLevelItem::Group(items) => {
            {
                let group = scope
                    .new_struct(&name)
                    .vis("pub")
                    .derive("serde::Deserialize, PartialEq, Debug, Clone");
                for (key, item) in &items {
                    group
                        .new_field(key, to_upper_camel_case(key))
                        .doc(item.doc())
                        .vis("pub");
                }
            }
            for (key, item) in items {
                generate_parsed_item(scope, &item, &to_upper_camel_case(&key));
            }
        }
        TopLevelItem::OneOf(items) => {
            let enu = scope
                .new_enum(name)
                .vis("pub")
                .derive("serde::Deserialize, PartialEq, Debug, Clone");
            for key in items {
                enu.new_variant(to_upper_camel_case(&key))
                    .annotation(format!("#[serde(rename=\"{key}\")]"))
                    .tuple(&to_upper_camel_case(&key));
                // todo: is this correct or does this need an untagged variant?
            }
        }
    }
}

#[allow(unused_variables)]
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
        ParsedItem::PropertyType(fields) => items::property_type::generate(scope, name, fields),
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
        } => items::formatted::generate(scope, name, common, &default, *tokens),
        ParsedItem::Filter(fields) => items::filter::generate(scope, name, fields),
        ParsedItem::Expression(fields) => items::expression::generate(scope, name, fields),
    }
}

/// Converts a string to a valid Rust struct name (UpperCamelCase)
fn to_upper_camel_case(name: &str) -> String {
    name.split(|c: char| !c.is_alphanumeric()) // split on non-alphanumeric
        .filter(|s| !s.is_empty()) // skip empty parts
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
                None => String::new(),
            }
        })
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_to_upper_camel_case() {
        assert_eq!(to_upper_camel_case("my_struct_name"), "MyStructName");
        assert_eq!(to_upper_camel_case("hello world"), "HelloWorld");
        assert_eq!(to_upper_camel_case("123abc"), "123abc");
        assert_eq!(to_upper_camel_case("__weird__name__"), "WeirdName");
        assert_eq!(to_upper_camel_case("alreadyCamel"), "Alreadycamel");
    }

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
        insta::assert_snapshot!(generate_spec_scope(reference), @r"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        /// A number between 0 and 10.
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct NumberOne(serde_json::Number);

        impl Default for NumberOne {
            fn default() {
                0
            }
        }
        ");
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
        insta::assert_snapshot!(&spec, @r"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct Names {
            /// A number between 0 and 10.
            pub name_one: NameOne,
        }

        /// A number between 0 and 10.
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct NameOne(serde_json::Number);

        impl Default for NameOne {
            fn default() {
                1.0
            }
        }
        ");
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

        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub enum Numbers {
            #[serde(rename="number_one")]
            NumberOne(NumberOne),
            #[serde(rename="number_two")]
            NumberTwo(NumberTwo),
        }

        /// Another number
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct NumberTwo(serde_json::Number);

        /// A number between 0 and 20.
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct NumberOne(serde_json::Number);

        impl Default for NumberOne {
            fn default() {
                1.0
            }
        }
        "#);
    }
}

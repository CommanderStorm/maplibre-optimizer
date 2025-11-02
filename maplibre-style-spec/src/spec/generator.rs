use std::collections::HashMap;

use codegen::Scope;

use crate::spec::decoder::{EnumValues, ParsedItem, StyleReference, TopLevelItem};

pub fn generate_spec_scope(reference: StyleReference) -> String {
    let mut scope = Scope::new();

    assert_eq!(reference.version, 8);

    generate_spec(&mut scope, &reference.root);
    for (key, item) in reference.fields.into_iter() {
        generate_top_level_item(&mut scope, item, to_upper_camel_case(&key))
    }
    scope.to_string()
}

fn generate_spec(scope: &mut Scope, root: &HashMap<String, ParsedItem>) {
    let spec = scope
        .new_struct("MaplibreStyleSpecification")
        .doc("This is a Maplibre Style Specification")
        .vis("pub")
        .derive("serde::Deserialise, PartialEq, Debug, Clone");
    for (key, field) in root {
        spec.new_field(key, to_upper_camel_case(key))
            .vis("pub")
            .doc(field.doc());
    }
}

fn generate_top_level_item(scope: &mut Scope, item: TopLevelItem, name: String) {
    match item {
        TopLevelItem::Item(item) => generate_parsed_item(scope, &item, name),
        TopLevelItem::Group(items) => {
            {
                let group = scope
                    .new_struct(&name)
                    .vis("pub")
                    .derive("serde::Deserialise, PartialEq, Debug, Clone");
                for (key, item) in &items {
                    group
                        .new_field(key, to_upper_camel_case(key))
                        .doc(item.doc())
                        .vis("pub");
                }
            }
            for (key, item) in items {
                generate_parsed_item(scope, &item, to_upper_camel_case(&key));
            }
        }
        TopLevelItem::OneOf(items) => {
            let enu = scope
                .new_enum(&name)
                .vis("pub")
                .derive("serde::Deserialise, PartialEq, Debug, Clone")
                ;
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
fn generate_parsed_item(scope: &mut Scope, item: &ParsedItem, name: String) {
    match item {
        ParsedItem::Number {
            common,
            default,
            maximum: _maximum,
            minimum: _minimum,
            period: _period,
        } => {
            scope
                .new_struct(&name)
                .doc(&common.doc)
                .vis("pub")
                .derive("serde::Deserialise, PartialEq, Debug, Clone")
                .tuple_field("serde_json::Number");
            if let Some(default) = default {
                scope
                    .new_impl(&name)
                    .impl_trait("Default")
                    .new_fn("default")
                    .line(default);
            }
        }
        ParsedItem::Enum {
            common,
            default,
            values,
        } => {
            match values {
                EnumValues::Simple(values) => {
                    let enu = scope
                        .new_enum(&name)
                        .doc(&common.doc)
                        .vis("pub")
                        .derive("serde::Deserialise, PartialEq, Eq, Debug, Clone, Copy");
                    for value in values {
                        enu.new_variant(to_upper_camel_case(value))
                            .annotation(format!("serde(rename=\"{value}\")"));
                    }
                }
                EnumValues::Numeric(values) => {
                    scope
                        .new_struct(&name)
                        .doc(&common.doc)
                        .vis("pub")
                        .derive("serde::Deserialise, PartialEq, Eq, Debug, Clone, Copy")
                        .tuple_field("u8");
                    assert!(values.len() <= u8::MAX as usize);
                    assert!(
                        values
                            .iter()
                            .all(|v| v.as_u64().is_some_and(|v| v <= u8::MAX as u64))
                    );
                    // todo: contribute proper repr(u8) variant support
                }
                EnumValues::Complex(values) => {
                    let enu = scope
                        .new_enum(&name)
                        .doc(&common.doc)
                        .vis("pub")
                        .derive("serde::Deserialise, PartialEq, Eq, Debug, Clone, Copy");
                    for (key, value) in values {
                        enu.new_variant(to_upper_camel_case(key))
                            .annotation(format!("#[serde(rename=\"{key}\")]"))
                            .annotation(format!("/// {}", value.doc));
                        // todo: this is sort of a hack, but it works for now
                        // upstream a proprer .doc() method
                    }
                }
            }

            if let Some(default) = default {
                scope
                    .new_impl(&name)
                    .impl_trait("Default")
                    .new_fn("default")
                    .line(default);
            }
        }
        ParsedItem::Array {
            common,
            default,
            value,
            values,
            minimum,
            maximum,
            length,
        } => todo!(),
        ParsedItem::Color { common, default } => todo!(),
        ParsedItem::String { common, default } => todo!(),
        ParsedItem::Boolean { common, default } => todo!(),
        ParsedItem::Star(fields) => todo!(),
        ParsedItem::PropertyType(fields) => todo!(),
        ParsedItem::ResolvedImage { common, tokens } => todo!(),
        ParsedItem::PromoteId(fields) => todo!(),
        ParsedItem::NumberArray {
            common,
            default,
            minimum: _minimum,
            maximum: _maximum,
        } => todo!(),
        ParsedItem::ColorArray { common, default } => todo!(),
        ParsedItem::VariableAnchorOffsetCollection(fields) => todo!(),
        ParsedItem::Transition(fields) => todo!(),
        ParsedItem::Terrain(fields) => todo!(),
        ParsedItem::State { common, default } => todo!(),
        ParsedItem::Sprite(fields) => todo!(),
        ParsedItem::Sources(fields) => todo!(),
        ParsedItem::Source(fields) => todo!(),
        ParsedItem::Sky(fields) => todo!(),
        ParsedItem::ProjectionDefinition { common, default } => todo!(),
        ParsedItem::Projection(fields) => todo!(),
        ParsedItem::Paint(fields) => todo!(),
        ParsedItem::Padding { common, default } => todo!(),
        ParsedItem::Light(fields) => todo!(),
        ParsedItem::Layout(fields) => todo!(),
        ParsedItem::Formatted {
            common,
            tokens: _tokens,
            default,
        } => todo!(),
        ParsedItem::Filter(fields) => todo!(),
        ParsedItem::Expression(fields) => todo!(),
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
        #[derive(serde::Deserialise, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        /// A number between 0 and 10.
        #[derive(serde::Deserialise, PartialEq, Debug, Clone)]
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
                "type": "string",
                "default": "default_name"
              }
            }
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(generate_spec_scope(reference), @"");
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
              "default": 2.0,
              "minimum": 0.0,
              "maximum": 10.0
            },
            "number_two": {
              "type": "number",
              "doc": "A number between 0 and 20.",
              "default": 2.0,
              "minimum": 0.0,
              "maximum": 20.0
            },
            "numbers": ["number_one", "number_two"]
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(generate_spec_scope(reference), @"");
    }
}

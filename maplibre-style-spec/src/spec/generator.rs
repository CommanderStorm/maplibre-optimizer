use codegen::Scope;

use crate::spec::decoder::{StyleReference, TopLevelItem};

pub fn generate_spec_scope(reference: StyleReference) -> String {
    let mut scope = Scope::new();

    generate_spec(&mut scope, &reference);
    for (key, item) in reference.fields.into_iter() {
        generate_top_level_item(&mut scope, item, to_upper_camel_case(&key))
    }
    scope.to_string()
}

fn generate_spec(scope: &mut Scope, reference: &StyleReference) {
    assert_eq!(reference.version, 8);
    let spec = scope
        .new_struct("MaplibreStyleSpecification")
        .vis("pub")
        .doc("This is a Maplibre Style Specification")
        .derive("serde::Deserialise")
        .field("$version", "u8");
    for key in reference.fields.keys() {
        spec.field(&key, to_upper_camel_case(&key));
    }
}

fn generate_top_level_item(_scope: &mut Scope, _item: TopLevelItem, _name: String) {
    todo!()
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
    use std::collections::HashMap;

    use crate::spec::decoder::{Fields, ParsedItem};

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
        let reference = StyleReference {
            version: 8,
            fields: HashMap::from([(
                "number_one".to_string(),
                TopLevelItem::Item(ParsedItem::Number {
                    common: Fields {
                        doc: "A number between 0 and 10.".to_string(),
                        ..Default::default()
                    },
                    default: Some(0.into()),
                    maximum: Some(10.into()),
                    minimum: Some(0.into()),
                    period: Some(10.into()),
                }),
            )]),
        };
        insta::assert_snapshot!(generate_spec_scope(reference), @"");
    }

    #[test]
    fn test_generate_spec_groups() {
        let reference = StyleReference {
            version: 8,
            fields: HashMap::from([
                (
                    "names".to_string(),
                    TopLevelItem::Group(HashMap::from([(
                        "name_one".to_string(),
                        ParsedItem::String {
                            common: Fields {
                                doc: "A string.".to_string(),
                                ..Default::default()
                            },
                            default: Some("default_name".to_string()),
                        },
                    )])),
                ),
                (
                    "one".to_string(),
                    TopLevelItem::OneOf(vec!["number_one".to_string()]),
                ),
            ]),
        };
        insta::assert_snapshot!(generate_spec_scope(reference), @"");
    }

    #[test]
    fn test_generate_spec_oneof() {
        let reference = StyleReference {
            version: 8,
            fields: HashMap::from([
                (
                    "number_one".to_string(),
                    TopLevelItem::Item(ParsedItem::Number {
                        common: Fields {
                            doc: "A number between 0 and 10.".to_string(),
                            ..Default::default()
                        },
                        default: Some(0.into()),
                        maximum: Some(10.into()),
                        minimum: Some(0.into()),
                        period: Some(10.into()),
                    }),
                ),
                (
                    "number_two".to_string(),
                    TopLevelItem::Item(ParsedItem::Number {
                        common: Fields {
                            doc: "A number between 0 and 10.".to_string(),
                            ..Default::default()
                        },
                        default: Some(0.into()),
                        maximum: Some(10.into()),
                        minimum: Some(0.into()),
                        period: Some(10.into()),
                    }),
                ),
                (
                    "one".to_string(),
                    TopLevelItem::OneOf(vec!["number_one".to_string(), "number_two".to_string()]),
                ),
            ]),
        };
        insta::assert_snapshot!(generate_spec_scope(reference), @"");
    }
}

use codegen::Scope;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields, default: &str, _tokens: bool) {
    scope
        .new_struct(name)
        .doc(&common.doc)
        .derive("serde::Deserialize, PartialEq, Debug, Clone")
        .tuple_field("String");

    scope
        .new_impl(name)
        .impl_trait("Default")
        .new_fn("default")
        .ret("Self")
        .line(format!("Self(\"{default}\".to_string())"));
    generate_test_from_example_if_present(scope, name, common.example.as_ref());
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::decoder::StyleReference;
    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(&mut scope, "Foo", &Fields::default(), "some", false);
        insta::assert_snapshot!(scope.to_string(), @r#"
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        struct Foo(String);

        impl Default for Foo {
            fn default() -> Self {
                Self("some".to_string())
            }
        }
        "#)
    }

    #[test]
    fn test_generate_spec() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "text-field": {
          "type": "formatted",
          "default": "",
          "tokens": true,
          "doc": "Value to use for a text label. If a plain `string` is provided, it will be treated as a `formatted` with default/inherited formatting options.",
          "sdk-support": {
            "basic functionality": {
              "js": "0.10.0",
              "android": "2.0.1",
              "ios": "2.0.0"
            },
            "data-driven styling": {
              "js": "0.33.0",
              "android": "5.0.0",
              "ios": "3.5.0"
            }
          },
          "expression": {
            "interpolated": false,
            "parameters": [
              "zoom",
              "feature"
            ]
          },
          "property-type": "data-driven"
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r#"
            /// This is a Maplibre Style Specification
            #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
            pub struct MaplibreStyleSpecification;

            /// Value to use for a text label. If a plain `string` is provided, it will be treated as a `formatted` with default/inherited formatting options.
            #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
            struct TextField(String);

            impl Default for TextField {
                fn default() -> Self {
                    Self("".to_string())
                }
            }

            #[cfg(test)] 
            mod test {
                use super::*;

            }
            "#);
    }
}

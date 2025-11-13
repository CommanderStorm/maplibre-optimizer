use codegen::Scope;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields, default: Option<&bool>) {
    scope
        .new_struct(name)
        .doc(&common.doc)
        .derive("serde::Deserialize, PartialEq, Debug, Clone, Copy")
        .tuple_field("bool");

    if let Some(default) = default {
        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(format!("Self({default})"));
    }
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
        generate(&mut scope, "Foo", &Fields::default(), None);
        insta::assert_snapshot!(scope.to_string(), @r"
        #[derive(serde::Deserialize, PartialEq, Debug, Clone, Copy)]
        struct Foo(bool);
        ")
    }

    #[test]
    fn test_generate_spec() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "fill-antialias": {
          "type": "boolean",
          "default": true,
          "doc": "Whether or not the fill should be antialiased.",
          "sdk-support": {
            "basic functionality": {
              "js": "0.10.0",
              "android": "2.0.1",
              "ios": "2.0.0"
            }
          },
          "expression": {
            "interpolated": false,
            "parameters": [
              "zoom"
            ]
          },
          "property-type": "data-constant"
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r"
            /// This is a Maplibre Style Specification
            #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
            pub struct MaplibreStyleSpecification;

            /// Whether or not the fill should be antialiased.
            #[derive(serde::Deserialize, PartialEq, Debug, Clone, Copy)]
            struct FillAntialias(bool);

            impl Default for FillAntialias {
                fn default() -> Self {
                    Self(true)
                }
            }

            #[cfg(test)] 
            mod test {
                use super::*;

            }
            ");
    }
}

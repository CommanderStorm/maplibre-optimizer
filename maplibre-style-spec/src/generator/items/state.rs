use codegen2::Scope;
use serde_json::Value;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields, default: &Value) {
    scope
        .new_struct(name)
        .doc(&common.doc)
        .derive("serde::Deserialize, PartialEq, Debug, Clone")
        .tuple_field("serde_json::Value");

    scope
        .new_impl(name)
        .impl_trait("Default")
        .new_fn("default")
        .ret("Self")
        .line(format!("Self(serde_json::json!({default}))"));
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
        generate(
            &mut scope,
            "Foo",
            &Fields::default(),
            &Value::String("hello_world".to_string()),
        );
        insta::assert_snapshot!(scope.to_string(), @r#"
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        struct Foo(serde_json::Value);

        impl Default for Foo {
            fn default() -> Self {
                Self(serde_json::json!("hello_world"))
            }
        }
        "#)
    }

    #[test]
    fn test_generate_spec() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "state": {
          "type": "state",
          "default": {},
          "doc": "An object used to define default values when using the [`global-state`](https://maplibre.org/maplibre-style-spec/expressions/#global-state) expression.",
          "example": {
            "chargerType": {
              "default": ["CCS", "CHAdeMO", "Type2"]
            },
            "minPreferredChargingSpeed": {
              "default": 50
            }
          },
          "sdk-support": {
            "basic functionality": {
              "js": "5.6.0",
              "android": "https://github.com/maplibre/maplibre-native/issues/3302",
              "ios": "https://github.com/maplibre/maplibre-native/issues/3302"
            }
          }
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r#"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        /// An object used to define default values when using the [`global-state`](https://maplibre.org/maplibre-style-spec/expressions/#global-state) expression.
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        struct State(serde_json::Value);

        impl Default for State {
            fn default() -> Self {
                Self(serde_json::json!({}))
            }
        }

        #[cfg(test)] 
        mod test {
            use super::*;

            #[test]
            fn test_example_state_decodes() {
                let example = serde_json::json!({"chargerType":{"default":["CCS","CHAdeMO","Type2"]},"minPreferredChargingSpeed":{"default":50}});
                let _ = serde_json::from_value::<State>(example).expect("example should decode");
            }
        }
        "#);
    }
}

use codegen2::Scope;
use serde_json::Value;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields, default: Option<&Value>) {
    scope
        .new_struct(name)
        .doc(&common.doc)
        .derive("serde::Deserialize, PartialEq, Debug, Clone")
        .tuple_field("color::DynamicColor");

    if let Some(default) = default {
        let fun = scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self");
        if let Value::String(default) = default {
            fun.line(format!("Self(color::parse_color(\"{default}\").expect(\"Invalid color specified as the default value\"))"));
        } else {
            fun.line(format!("let default = serde_json::json!({default});"));
            fun.line("let default = serde_json::from_value(default).expect(\"Invalid color specified as the default value\");");
            fun.line("Self(default)");
        }
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
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        struct Foo(color::DynamicColor);
        ")
    }

    #[test]
    fn test_generate_spec() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "color-relief-color": {
          "type": "color",
          "doc": "Defines the color of each pixel based on its elevation. Should be an expression that uses `[\"elevation\"]` as input.",
          "example": [
            "interpolate",
            ["linear"],
            ["elevation"],
            0, "black",
            8849, "white"
          ],
          "transition": false,
          "sdk-support": {
            "basic functionality": {
              "js": "5.6.0",
              "android": "https://github.com/maplibre/maplibre-native/issues/3408",
              "ios": "https://github.com/maplibre/maplibre-native/issues/3408"
            },
            "data-driven styling": {}
          },
          "expression": {
            "interpolated": true,
            "parameters": [
              "elevation"
            ]
          },
          "property-type": "color-ramp"
        }
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r#"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        /// Defines the color of each pixel based on its elevation. Should be an expression that uses `["elevation"]` as input.
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        struct ColorReliefColor(color::DynamicColor);

        #[cfg(test)]
        mod test {
            use super::*;

            #[test]
            fn test_example_color_relief_color_decodes() {
                let example = serde_json::json!(["interpolate",["linear"],["elevation"],0,"black",8849,"white"]);
                let _ = serde_json::from_value::<ColorReliefColor>(example).expect("example should decode");
            }
        }
        "#);
    }
}

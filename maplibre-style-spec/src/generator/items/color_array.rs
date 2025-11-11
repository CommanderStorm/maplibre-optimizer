use codegen::Scope;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields, default: Option<&str>) {
    let enu = scope
        .new_enum(name)
        .vis("pub")
        .attr("serde(untagged)")
        .doc(&common.doc)
        .derive("serde::Deserialize, PartialEq, Debug, Clone");
    enu.new_variant("One")
        .tuple("color::DynamicColor")
        .doc("A color");
    enu.new_variant("Multiple")
        .tuple("Vec<color::DynamicColor>")
        .doc("A set of colors");

    if let Some(default) = default {
        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(format!("Self::One(color::parse_color(\"{default}\").expect(\"Invalid color specified as the default value\"))"));
    }
    generate_test_from_example_if_present(scope, name, common);
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
        #[serde(untagged)]
        pub enum Foo {
            /// A color
            One(color::DynamicColor),
            /// A set of colors
            Multiple(Vec<color::DynamicColor>),
        }
        ")
    }

    #[test]
    fn test_generate_spec() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "hillshade-shadow-color": {
          "type": "colorArray",
          "default": "#000000",
          "doc": "The shading color of areas that face away from the light source(s). Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.",
          "transition": true,
          "sdk-support": {
            "basic functionality": {
              "js": "0.43.0",
              "android": "6.0.0",
              "ios": "4.0.0"
            },
            "multidirectional": {
              "js": "5.5.0",
              "android": "https://github.com/maplibre/maplibre-native/issues/3396",
              "ios": "https://github.com/maplibre/maplibre-native/issues/3396"
            }
          },
          "expression": {
            "interpolated": true,
            "parameters": [
              "zoom"
            ]
          },
          "property-type": "data-constant"
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r##"
            /// This is a Maplibre Style Specification
            #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
            pub struct MaplibreStyleSpecification;

            /// The shading color of areas that face away from the light source(s). Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
            #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
            #[serde(untagged)]
            pub enum HillshadeShadowColor {
                /// A color
                One(color::DynamicColor),
                /// A set of colors
                Multiple(Vec<color::DynamicColor>),
            }

            impl Default for HillshadeShadowColor {
                fn default() -> Self {
                    Self::One(color::parse_color("#000000").expect("Invalid color specified as the default value"))
                }
            }

            #[cfg(test)] 
            mod test {
                use super::*;

            }
            "##);
    }
}

use codegen::Scope;
use serde_json::Number;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::items::number::generate_number_default;

pub fn generate(
    scope: &mut Scope,
    name: &str,
    common: &Fields,
    default: Option<&Number>,
    min: Option<&Number>,
    max: Option<&Number>,
) {
    let enu = scope
        .new_enum(name)
        .vis("pub")
        .attr("serde(untagged)")
        .doc(&common.doc_with_range(max, min, None))
        .derive("serde::Deserialize, PartialEq, Debug, Clone");
    enu.new_variant("One").tuple("serde_json::Number");
    enu.new_variant("Many").tuple("Vec<serde_json::Number>");

    if let Some(default) = default {
        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(format!("Self::One({})", generate_number_default(default)));
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
        generate(&mut scope, "Foo", &Fields::default(), None, None, None);
        insta::assert_snapshot!(scope.to_string(), @r"
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        #[serde(untagged)]
        pub enum Foo {
            One(serde_json::Number),
            Many(Vec<serde_json::Number>),
        }
        ")
    }

    #[test]
    fn test_generate_spec() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "paint_hillshade": {
          "hillshade-illumination-direction": {
            "type": "numberArray",
            "default": 335,
            "minimum": 0,
            "maximum": 359,
            "doc": "The direction of the light source(s) used to generate the hillshading with 0 as the top of the viewport if `hillshade-illumination-anchor` is set to `viewport` and due north if `hillshade-illumination-anchor` is set to `map`. Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.",
            "transition": false,
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
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r#"
            /// This is a Maplibre Style Specification
            #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
            pub struct MaplibreStyleSpecification;

            #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
            pub struct PaintHillshade {
                /// The direction of the light source(s) used to generate the hillshading with 0 as the top of the viewport if `hillshade-illumination-anchor` is set to `viewport` and due north if `hillshade-illumination-anchor` is set to `map`. Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
                #[serde(rename="hillshade-illumination-direction")]
                pub hillshade_illumination_direction: Option<PaintHillshadeHillshadeIlluminationDirection>,
            }

            /// The direction of the light source(s) used to generate the hillshading with 0 as the top of the viewport if `hillshade-illumination-anchor` is set to `viewport` and due north if `hillshade-illumination-anchor` is set to `map`. Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
            ///
            /// Range: 0..=359
            #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
            #[serde(untagged)]
            pub enum PaintHillshadeHillshadeIlluminationDirection {
                One(serde_json::Number),
                Many(Vec<serde_json::Number>),
            }

            impl Default for PaintHillshadeHillshadeIlluminationDirection {
                fn default() -> Self {
                    Self::One(serde_json::Number::from_i128(335).expect("the number is serialised from a number and is thus always valid"))
                }
            }

            #[cfg(test)] 
            mod test {
                use super::*;

            }
            "#);
    }
}

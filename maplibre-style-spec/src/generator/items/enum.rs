use std::collections::BTreeMap;

use codegen::Scope;
use serde_json::{Number, Value};

use crate::decoder::{EnumDocs, EnumValues, Fields};
use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::formatter::to_upper_camel_case;

pub fn generate(
    scope: &mut Scope,
    name: &str,
    common: &Fields,
    default: Option<&Value>,
    values: &EnumValues,
) {
    match values {
        EnumValues::Version(values) => {
            generate_version(scope, name, &common, values);
        }
        EnumValues::Enum(values) => {
            generate_regular_enum(scope, name, &common, values);
        }
    }

    if let Some(default) = default {
        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(format!(
                "Self::{}",
                to_upper_camel_case(&default.to_string())
            ));
    }
    generate_test_from_example_if_present(scope, name, common);
}

fn generate_regular_enum(
    scope: &mut Scope,
    name: &str,
    common: &&Fields,
    values: &BTreeMap<String, EnumDocs>,
) {
    let enu = scope
        .new_enum(name)
        .doc(&common.doc)
        .vis("pub")
        .derive("serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy");
    for (key, value) in values {
        let var_name = to_upper_camel_case(key);
        let var = enu.new_variant(&var_name).doc(&value.doc);
        if key != &var_name {
            var.annotation(format!("#[serde(rename=\"{key}\")]"));
        }
    }
}

fn generate_version(scope: &mut Scope, name: &str, common: &&Fields, values: &Vec<Number>) {
    assert!(values.len() <= u8::MAX as usize);

    let enu = scope
        .new_enum(name)
        .doc(&common.doc)
        .vis("pub")
        .repr("u8")
        .derive("serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy");
    for v in values {
        enu.new_variant(to_upper_camel_case(&v.to_string()))
            .discriminant(v.to_string());
    }
    assert!(
        values
            .iter()
            .all(|v| v.as_u64().is_some_and(|v| v <= u8::MAX as u64))
    );
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::decoder::StyleReference;

    #[test]
    fn test_generate_spec() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "colorSpace": {
          "type": "enum",
          "values": {
            "rgb": {
              "doc": "Use the RGB color space to interpolate color values"
            },
            "lab": {
              "doc": "Use the LAB color space to interpolate color values."
            },
            "hcl": {
              "doc": "Use the HCL color space to interpolate color values, interpolating the Hue, Chroma, and Luminance channels individually."
            }
          },
          "doc": "The color space in which colors interpolated. Interpolating colors in perceptual color spaces like LAB and HCL tend to produce color ramps that look more consistent and produce colors that can be differentiated more easily than those interpolated in RGB space.",
          "default": "rgb"
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r#"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        /// The color space in which colors interpolated. Interpolating colors in perceptual color spaces like LAB and HCL tend to produce color ramps that look more consistent and produce colors that can be differentiated more easily than those interpolated in RGB space.
        #[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
        pub enum ColorSpace {
            /// Use the HCL color space to interpolate color values, interpolating the Hue, Chroma, and Luminance channels individually.
            #[serde(rename="hcl")]
            Hcl,
            /// Use the LAB color space to interpolate color values.
            #[serde(rename="lab")]
            Lab,
            /// Use the RGB color space to interpolate color values
            #[serde(rename="rgb")]
            Rgb,
        }

        impl Default for ColorSpace {
            fn default() -> Self {
                Self::Rgb
            }
        }

        #[cfg(test)] 
        mod test {
            use super::*;

        }
        "#);
    }

    #[test]
    fn test_generate_spec_weird_keys() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "filter_operator": {
          "type": "enum",
          "values": {
            "==": {
              "doc": "`[\"==\", key, value]` equality: `feature[key] = value`"
            },
            "!=": {
              "doc": "`[\"!=\", key, value]` inequality: `feature[key] ≠ value`"
            },
            ">": {
              "doc": "`[\">\", key, value]` greater than: `feature[key] > value`"
            },
            ">=": {
              "doc": "`[\">=\", key, value]` greater than or equal: `feature[key] ≥ value`"
            },
          },
          "doc": "The filter operator."
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r#"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        /// The filter operator.
        #[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
        pub enum FilterOperator {
            /// `["!=", key, value]` inequality: `feature[key] ≠ value`
            #[serde(rename="!=")]
            NotEqual,
            /// `["==", key, value]` equality: `feature[key] = value`
            #[serde(rename="==")]
            EqualEqual,
            /// `[">", key, value]` greater than: `feature[key] > value`
            #[serde(rename=">")]
            Greater,
            /// `[">=", key, value]` greater than or equal: `feature[key] ≥ value`
            #[serde(rename=">=")]
            GreaterEqual,
        }

        #[cfg(test)] 
        mod test {
            use super::*;

        }
        "#);
    }

    #[test]
    fn test_generate_spec_version() {
        let reference = json!({
        "$version": 8,
        "$root": {
          "version": {
            "required": true,
            "type": "enum",
            "values": [
              8
            ],
            "doc": "Style specification version number. Must be 8.",
            "example": 8
          },
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r#"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification {
            /// Style specification version number. Must be 8.
            pub version: RootVersion,
        }

        /// Style specification version number. Must be 8.
        #[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
        #[repr(u8)]
        pub enum RootVersion {
            Eight = 8,
        }

        #[cfg(test)] 
        mod test {
            use super::*;

            #[test]
            fn test_example_root_version_decodes() {
                let example = serde_json::json!(8);
                let _ = serde_json::from_value::<RootVersion>(example).expect("example should decode");
            }
        }
        "#);
    }

    #[test]
    fn test_generate_spec_expressions() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "expression_name": {
          "doc": "",
          "type": "enum",
          "values": {
            "let": {
              "doc": "Binds expressions to named variables, which can then be referenced in the result expression using `[\"var\", \"variable_name\"]`.\n\n - [Visualize population density](https://maplibre.org/maplibre-gl-js/docs/examples/visualize-population-density/)",
              "syntax": {
                "overloads": [
                  {
                    "parameters": ["var_1_name", "var_1_value", "...", "var_n_name", "var_n_value", "expression"],
                    "output-type": "any"
                  }
                ],
                "parameters": [
                  {
                    "name": "var_i_name",
                    "type": "string literal",
                    "description": "The name of the i-th variable."
                  },
                  {
                    "name": "var_i_value",
                    "type": "any",
                    "description": "The value of the i-th variable."
                  },
                  {
                    "name": "expression",
                    "type": "any",
                    "description": "The expression within which the named variables can be referenced."
                  }
                ]
              },
              "example": ["let", "someNumber", 500, ["interpolate", ["linear"], ["var", "someNumber"], 274, "#edf8e9", 1551, "#006d2c"]],
              "group": "Variable binding",
              "sdk-support": {
                "basic functionality": {
                  "js": "0.41.0",
                  "android": "6.0.0",
                  "ios": "4.0.0"
                }
              }
            },
          },
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r#"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        #[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
        pub enum ExpressionName {
            /// Binds expressions to named variables, which can then be referenced in the result expression using `["var", "variable_name"]`.
            /// 
            ///  - [Visualize population density](https://maplibre.org/maplibre-gl-js/docs/examples/visualize-population-density/)
            #[serde(rename="let")]
            Let,
        }

        #[cfg(test)] 
        mod test {
            use super::*;

        }
        "#);
    }
}

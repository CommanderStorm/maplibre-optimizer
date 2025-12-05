use std::collections::BTreeMap;

use codegen2::Scope;

use crate::decoder::Fields;
use crate::decoder::r#enum::EnumDocs;
use crate::generator::formatter::to_upper_camel_case;

pub fn generate_regular_enum(
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
}

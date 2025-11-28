use std::collections::BTreeMap;

use codegen2::Scope;
use serde_json::{Number, Value};

use crate::decoder::{EnumDocs, EnumValues, Fields};
use crate::generator::autotest::{
    generate_test_from_example_if_present, generate_test_from_examples_if_present,
};
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
            let has_syntax = values.values().any(|v| v.syntax.is_some());
            if has_syntax {
                generate_syntax_enum(scope, name, &common, values);
            } else {
                generate_regular_enum(scope, name, &common, values);
            }
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
    generate_test_from_example_if_present(scope, name, common.example.as_ref());
}

fn generate_syntax_enum(
    scope: &mut Scope,
    name: &str,
    common: &&Fields,
    values: &BTreeMap<String, EnumDocs>,
) {
    let enu = scope
        .new_enum(name)
        .doc(&common.doc)
        .vis("pub")
        .derive("PartialEq, Eq, Debug, Clone");
    for (key, value) in values {
        let var_name = to_upper_camel_case(key);
        let var = enu.new_variant(&var_name).doc(&value.doc);
        let group = value.group.as_ref().expect(&format!(
            "syntax enum should have a group, but {key} does not have one"
        ));
        let syntax = value.syntax.as_ref().expect(&format!(
            "syntax enum should have a syntax, but {key} does not have one"
        ));
        assert!(
            !syntax.overloads.is_empty(),
            "{key} in {name} (group={group}) does not have a single overload"
        );
        if syntax.overloads.len() == 1 {
            // not overloaded, above the Option<T> level
            let overload = &syntax.overloads[0];
            if overload.parameters.iter().any(|p| p == "...") {
                var.tuple("Vec<serde_json::Value>");
                continue;
            }
            for p in &overload.parameters {
                let param = p.clone();
                let tuple_identifier = if let Some(_) = param.strip_suffix('?') {
                    format!("Option<serde_json::Value>")
                } else {
                    "serde_json::Value".to_string()
                };

                var.tuple(tuple_identifier);
            }
        } else {
            // actually overloaded
            let options_name = format!("{var_name}Options");
            var.tuple(&options_name);
        }
    }
    for (key, value) in values {
        let var_name = to_upper_camel_case(key);
        let group = value.group.as_ref().expect(&format!(
            "syntax enum should have a group, but {key} does not have one"
        ));
        let syntax = value.syntax.as_ref().expect(&format!(
            "syntax enum should have a syntax, but {key} does not have one"
        ));
        assert!(
            !syntax.overloads.is_empty(),
            "{key} in {name} (group={group}) does not have a single overload"
        );
        if syntax.overloads.len() != 1 {
            // actually overloaded
            let options_name = format!("{var_name}Options");

            let _enu = scope
                .new_enum(&options_name)
                .doc(format!(
                    "Options for deserializing the syntax enum variant [`{name}::{var_name}`]"
                ))
                .vis("pub")
                .derive("serde::Deserialize, PartialEq, Eq, Debug, Clone");
            // todo: enumerate options
        }
    }

    let visitor_name = format!("{name}Visitor");
    scope
        .new_impl(name)
        .generic("'de")
        .impl_trait("serde::Deserialize<'de>")
        .new_fn("deserialize")
        .arg("deserializer", "D")
        .generic("D")
        .bound("D", "serde::Deserializer<'de>")
        .ret("Result<Self, D::Error>")
        .line(format!("deserializer.deserialize_any({visitor_name})"));

    scope
        .new_struct(&visitor_name)
        .doc("Visitor for deserializing the syntax enum [`{name}`]");

    let vis = scope
        .new_impl(&visitor_name)
        .generic("'de")
        .impl_trait("serde::de::Visitor<'de>")
        .associate_type("Value", &name);
    vis.new_fn("expecting")
        .arg_ref_self()
        .arg("formatter", "&mut std::fmt::Formatter")
        .ret("std::fmt::Result")
        .line("formatter.write_str(r#\"an expression array like [\"==\", 1, 2]\"#)");

    let visit_seq = vis
        .new_fn("visit_seq")
        .generic("A: serde::de::SeqAccess<'de>")
        .arg_self()
        .arg("mut seq", "A")
        .ret("Result<Self::Value, A::Error>");
    visit_seq.line("// First element: operator string");
    visit_seq.line("let op: String = seq.next_element()?.ok_or_else(|| serde::de::Error::custom(\"missing operator\"))?;");
    visit_seq.line("match op.as_str() {");
    for (key, _value) in values {
        let variant_name = to_upper_camel_case(key);
        visit_seq.line(format!(
            "\"{key}\" => todo!(\"{name}::{variant_name} decoding is not currently implemented\"),"
        ));
    }

    visit_seq.line("_ => Err(serde::de::Error::custom(&format!(\"unknown operator {op} in expression. Please check the documentation for the avaliable expressions.\")))");
    visit_seq.line("}");

    let examples = values.values().filter_map(|e| e.example.as_ref()).collect();
    generate_test_from_examples_if_present(scope, &name, examples);
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
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r##"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        #[derive(PartialEq, Eq, Debug, Clone)]
        pub enum ExpressionName {
            /// Binds expressions to named variables, which can then be referenced in the result expression using `["var", "variable_name"]`.
            ///
            ///  - [Visualize population density](https://maplibre.org/maplibre-gl-js/docs/examples/visualize-population-density/)
            Let(Vec<serde_json::Value>),
        }

        impl<'de> serde::Deserialize<'de> for ExpressionName {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_any(ExpressionNameVisitor)
            }
        }

        /// Visitor for deserializing the syntax enum [`{name}`]
        struct ExpressionNameVisitor;

        impl<'de> serde::de::Visitor<'de> for ExpressionNameVisitor {
            type Value = ExpressionName;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(r#"an expression array like ["==", 1, 2]"#)
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                // First element: operator string
                let op: String = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("missing operator"))?;
                match op.as_str() {
                "let" => todo!("ExpressionName::Let decoding is not currently implemented"),
                _ => Err(serde::de::Error::custom(&format!("unknown operator {op} in expression. Please check the documentation for the avaliable expressions.")))
                }
            }
        }

        #[cfg(test)]
        mod test {
            use super::*;

            #[rstest::rstest]
            #[case(serde_json::json!(["let","someNumber",500,["interpolate",["linear"],["var","someNumber"],274,"#edf8e9",1551,"#006d2c"]]))]
            fn test_example_expression_name_decodes(#[case] example: serde_json::Value) {
                let _ = serde_json::from_value::<ExpressionName>(example).expect("example should decode");
            }
        }
        "##);
    }
}

use std::collections::BTreeMap;

use codegen2::Scope;

use crate::decoder::Fields;
use crate::decoder::enum_decoder::SyntaxEnum;
use crate::generator::autotest::generate_test_from_examples_if_present;
use crate::generator::formatter::to_upper_camel_case;

pub fn generate_syntax_enum(
    scope: &mut Scope,
    name: &str,
    common: &&Fields,
    values: &BTreeMap<String, SyntaxEnum>,
) {
    let enu = scope
        .new_enum(name)
        .doc(&common.doc)
        .vis("pub")
        .derive("PartialEq, Eq, Debug, Clone");
    for (key, value) in values {
        let var_name = to_upper_camel_case(key);
        let var = enu.new_variant(&var_name).doc(&value.doc);
        let syntax = &value.syntax;
        assert!(
            !syntax.overloads.is_empty(),
            "{key} in {name} (group={group}) does not have a single overload",
            group = value.group
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
                let tuple_identifier = if param.strip_suffix('?').is_some() {
                    "Option<serde_json::Value>".to_string()
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
        let syntax = &value.syntax;
        assert!(
            !syntax.overloads.is_empty(),
            "{key} in {name} (group={group}) does not have a single overload",
            group = value.group
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

    generate_syntax_enum_deserializer(scope, &name, values);

    let examples = values.values().filter_map(|e| e.example.as_ref()).collect();
    generate_test_from_examples_if_present(scope, name, examples);
}

fn generate_syntax_enum_deserializer(
    scope: &mut Scope,
    name: &&str,
    values: &BTreeMap<String, SyntaxEnum>,
) {
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
        .associate_type("Value", name);
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
    for key in values.keys() {
        let variant_name = to_upper_camel_case(key);
        visit_seq.line(format!(
            "\"{key}\" => todo!(\"{name}::{variant_name} decoding is not currently implemented\"),"
        ));
    }

    visit_seq.line("_ => Err(serde::de::Error::custom(&format!(\"unknown operator {op} in expression. Please check the documentation for the avaliable expressions.\")))");
    visit_seq.line("}");
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::decoder::StyleReference;
    #[test]
    fn test_generate_spec_expressions() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "expression": {
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
        pub enum Expression {
            /// Binds expressions to named variables, which can then be referenced in the result expression using `["var", "variable_name"]`.
            ///
            ///  - [Visualize population density](https://maplibre.org/maplibre-gl-js/docs/examples/visualize-population-density/)
            Let(Vec<serde_json::Value>),
        }

        impl<'de> serde::Deserialize<'de> for Expression {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_any(ExpressionVisitor)
            }
        }

        /// Visitor for deserializing the syntax enum [`{name}`]
        struct ExpressionVisitor;

        impl<'de> serde::de::Visitor<'de> for ExpressionVisitor {
            type Value = Expression;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(r#"an expression array like ["==", 1, 2]"#)
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                // First element: operator string
                let op: String = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("missing operator"))?;
                match op.as_str() {
                "let" => todo!("Expression::Let decoding is not currently implemented"),
                _ => Err(serde::de::Error::custom(&format!("unknown operator {op} in expression. Please check the documentation for the avaliable expressions.")))
                }
            }
        }

        #[cfg(test)]
        mod test {
            use super::*;

            #[rstest::rstest]
            #[case(serde_json::json!(["let","someNumber",500,["interpolate",["linear"],["var","someNumber"],274,"#edf8e9",1551,"#006d2c"]]))]
            fn test_example_expression_decodes(#[case] example: serde_json::Value) {
                let _ = serde_json::from_value::<Expression>(example).expect("example should decode");
            }
        }
        "##);
    }
}

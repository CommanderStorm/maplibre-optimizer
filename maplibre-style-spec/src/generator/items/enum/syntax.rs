use std::collections::BTreeMap;

use codegen2::{Function, Impl, Scope};
use serde_json::Value;

use crate::decoder::r#enum::{Literal, Overload, Parameter, ParameterType, Syntax, SyntaxEnum};
use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_examples_if_present;
use crate::generator::formatter::{to_snake_case, to_upper_camel_case};

pub fn generate_syntax_enum(
    scope: &mut Scope,
    name: &str,
    common: &&Fields,
    values: &BTreeMap<String, SyntaxEnum>,
) {
    // pass 1: populate enum variants
    generate_syntax_enum_body(scope, name, &common, values);
    // pass 2: generate the previously referenced enum variants for overloaded variants
    generate_referenced_multi_overload_options_enums(scope, name, values);
    let examples = values
        .values()
        .filter_map(|e| e.example.as_ref())
        .collect::<Vec<_>>();
    generate_syntax_enum_deserializer(scope, name, values, examples[0]);

    generate_test_from_examples_if_present(scope, name, examples);
}

fn generate_referenced_multi_overload_options_enums(
    scope: &mut Scope,
    name: &str,
    values: &BTreeMap<String, SyntaxEnum>,
) {
    for (key, value) in values {
        let var_name = to_upper_camel_case(key);
        let syntax = &value.syntax;
        let name_and_possibly_group = if let Some(group) = &value.group {
            format!("{name} (group={group})")
        } else {
            name.to_string()
        };
        assert!(
            !syntax.overloads.is_empty(),
            "{key} in {name_and_possibly_group} does not have a single overload"
        );
        if syntax.overloads.len() != 1 {
            let options_name = format!("{var_name}Options");
            // actually overloaded
            generate_multi_overload(scope, (name, &var_name, &options_name), syntax);
        }
    }
}

fn generate_syntax_enum_body(
    scope: &mut Scope,
    name: &str,
    common: &&&Fields,
    values: &BTreeMap<String, SyntaxEnum>,
) {
    let enu = scope
        .new_enum(name)
        .doc(&common.doc)
        .vis("pub")
        .derive("PartialEq, Debug, Clone");
    for (key, value) in values {
        let var_name = to_upper_camel_case(key);
        let var = enu.new_variant(&var_name).doc(&value.doc);
        let syntax = &value.syntax;
        let name_and_possibly_group = if let Some(group) = &value.group {
            format!("{name} (group={group})")
        } else {
            name.to_string()
        };
        assert!(
            !syntax.overloads.is_empty(),
            "{key} in {name_and_possibly_group} does not have a single overload"
        );
        if syntax.overloads.len() == 1 {
            // not overloaded, above the Option<T> level
            let overload = &syntax.overloads[0];
            if syntax.has_variadic_overload() {
                let position_of_variadic_separator = overload.position_of_variadic_separator();

                let params = &overload.parameters[0..position_of_variadic_separator]
                    .iter()
                    .map(|o| {
                        syntax
                            .parameters
                            .iter()
                            .find(|p| p.matches_overload_parameter_name(o))
                            .expect("overload parameter not found in syntax parameters")
                    })
                    .collect::<Vec<_>>();
                let mut tuple_type_names = params[0].r#type.to_upper_camel_case();
                for p in &params[1..] {
                    tuple_type_names.push_str(",");
                    tuple_type_names.push_str(p.r#type.to_upper_camel_case().as_str());
                }
                if params.len() > 1 {
                    var.tuple(format!("Vec<({tuple_type_names})>"));
                } else {
                    var.tuple(format!("Vec<{tuple_type_names}>"));
                }
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
}

fn generate_multi_overload(
    scope: &mut Scope,
    (name, var_name, options_name): (&str, &str, &str),
    syntax: &Syntax,
) {
    // because scope can only be owned by one owner, we first need to generate all tuples, then can add them
    let mut overloads_tuples = Vec::with_capacity(syntax.overloads.len());
    for overload in &syntax.overloads {
        if overload.is_variadic(&syntax.parameters) {
            // todo: needs proper variadic codegen
            overloads_tuples.push(vec!["Vec<serde_json::Value>".to_string()]);
        } else {
            let var_name = overload.output_type.to_upper_camel_case();
            let mut tuples = Vec::with_capacity(overload.parameters.len());
            for param in &overload.parameters {
                let param_name =
                    generate_parameter_type(scope, (name, &var_name, param), &syntax.parameters);
                tuples.push(param_name);
            }
            overloads_tuples.push(tuples);
        }
    }

    let enu = scope
        .new_enum(&options_name)
        .doc(format!(
            "Options for deserializing the syntax enum variant [`{name}::{var_name}`]"
        ))
        .vis("pub")
        .derive("serde::Deserialize, PartialEq, Debug, Clone")
        .attr("serde(untagged)");
    let variant_naming_strat = OverloadVariantNamingStrategy::detect(&syntax.overloads);
    for (i, overload) in syntax.overloads.iter().enumerate() {
        let var_name = variant_naming_strat.var_name(overload, i);
        let var = enu.new_variant(&var_name);
        for t in &overloads_tuples[i] {
            var.tuple(t);
        }
    }
}

enum OverloadVariantNamingStrategy {
    OutputType,
    NumberOptions(Vec<usize>),
    ConstantMapping(Vec<String>),
}

impl OverloadVariantNamingStrategy {
    fn detect(overloads: &Vec<Overload>) -> Self {
        assert!(
            overloads.len() > 1,
            "renaming detection does only make sense for more than one overload"
        );
        // case 1: the output type is different
        let mut output_types = overloads
            .iter()
            .map(|o| o.output_type.to_upper_camel_case())
            .collect::<Vec<_>>();
        output_types.sort_unstable();
        let all_output_types = output_types.len();
        output_types.dedup();
        if all_output_types == output_types.len() {
            return OverloadVariantNamingStrategy::OutputType;
        }

        // case 2: the parameter lengths are all different
        let mut parameter_lengths = overloads
            .iter()
            .map(|o| o.parameters.len())
            .collect::<Vec<_>>();
        let params_clone = parameter_lengths.clone();
        parameter_lengths.sort_unstable();
        let all_params = parameter_lengths.len();
        parameter_lengths.dedup();
        if all_params == parameter_lengths.len() {
            return OverloadVariantNamingStrategy::NumberOptions(params_clone);
        }

        // case 3: the first parameter is different
        let mut first_parameters = overloads
            .iter()
            .map(|o| o.parameters.first().cloned().unwrap_or_default())
            // by default, the names are kind of bad, so we replace unstable patterns
            .map(|name| {
                if name.ends_with('?') {
                    format!("Opt{}", name.replace('?', ""))
                } else {
                    name
                }
            })
            .map(|name| {
                if name.ends_with("_1") {
                    name.replace("_1", "")
                } else {
                    name
                }
            })
            .map(to_upper_camel_case)
            .collect::<Vec<_>>();
        first_parameters.sort_unstable();
        let all_first_parameters = first_parameters.len();
        first_parameters.dedup();
        if all_first_parameters == first_parameters.len() {
            return OverloadVariantNamingStrategy::ConstantMapping(first_parameters);
        }

        panic!("could not determine a good naming strategy for {overloads:?}");
    }
    fn var_name(&self, overload: &Overload, i: usize) -> String {
        match self {
            OverloadVariantNamingStrategy::OutputType => overload.output_type.to_upper_camel_case(),
            OverloadVariantNamingStrategy::NumberOptions(ns) => {
                format!("{}Params", to_upper_camel_case(&ns[i].to_string()))
            }
            OverloadVariantNamingStrategy::ConstantMapping(ms) => ms[i].clone(),
        }
    }
}

fn generate_parameter_type(
    scope: &mut Scope,
    (name, var_name, param): (&str, &str, &str),
    parameters: &Vec<Parameter>,
) -> String {
    if let Some(param) = param.strip_suffix('?') {
        let param = parameters.iter()
            .find(|p| p.matches_overload_parameter_name(param))
            .unwrap_or_else(|| panic!("parameter {param} from the syntax overload of {name}::{var_name} does not have a syntax parameter"));
        let param_name = generate_parameter_variant(scope, &param.r#type);
        format!("Option<{param_name}>")
    } else {
        let param = parameters.iter()
            .find(|p| p.matches_overload_parameter_name(param))
            .unwrap_or_else(|| panic!("parameter {param} from the syntax overload of {name}::{var_name} does not have a syntax parameter"));
        generate_parameter_variant(scope, &param.r#type)
    }
}

fn generate_parameter_variant(scope: &mut Scope, param: &ParameterType) -> String {
    match &param {
        ParameterType::Literal(l) => l.to_upper_camel_case().to_string(),
        ParameterType::LiteralAnyOf(ls) => generate_any_of(scope, ls),
        ParameterType::Expression(e) => e.to_upper_camel_case().to_string(),
        ParameterType::ExpressionAnyOf(_) => "serde_json::Value".to_string(),
        ParameterType::Object(_) => "serde_json::Map".to_string(),
        ParameterType::Reference(r) => to_upper_camel_case(&r),
    }
}
fn generate_any_of(scope: &mut Scope, any_of: &[Literal]) -> String {
    let ts = any_of
        .iter()
        .map(|l| l.to_upper_camel_case())
        .collect::<Vec<_>>();
    let any_of_type = ts.join("Or");
    if scope.get_enum_mut(&any_of_type).is_none() {
        let enu = scope
            .new_enum(&any_of_type)
            .doc("Either of the below variants")
            .vis("pub")
            .derive("serde::Deserialize, PartialEq, Debug, Clone");
        for t in ts {
            enu.new_variant(&t).tuple(&t);
        }
    }
    any_of_type
}

fn generate_syntax_enum_deserializer(
    scope: &mut Scope,
    name: &str,
    values: &BTreeMap<String, SyntaxEnum>,
    example: &serde_json::Value,
) {
    let vis = generate_visitor(scope, name, example);

    let visit_seq = vis
        .new_fn("visit_seq")
        .generic("A: serde::de::SeqAccess<'de>")
        .arg_self()
        .arg("mut seq", "A")
        .ret("Result<Self::Value, A::Error>");
    visit_seq.line("use serde::Deserialize;");
    generate_visit_seq_field(visit_seq);
    // operator decoding
    visit_seq.line("// First element: operator string");
    visit_seq.line("let op: String = seq.next_element()?.ok_or_else(|| serde::de::Error::custom(\"missing operator\"))?;");
    visit_seq.line("match op.as_str() {");
    for (key, syntax_docs) in values {
        let syntax = &syntax_docs.syntax;
        let variant_name = to_upper_camel_case(key);

        visit_seq.line(format!("\"{key}\" => {{"));
        if syntax.overloads.len() == 1
            && let Some(overload) = syntax.overloads.first()
        {
            if syntax.has_variadic_overload() {
                generate_syntax_enum_deserializer_regular_variadic_variant(
                    visit_seq,
                    (&name, &variant_name),
                    overload,
                )
            } else {
                generate_syntax_enum_deserializer_regular_variant(
                    visit_seq,
                    (&name, &variant_name),
                    overload,
                );
            }
        } else {
            if syntax.has_variadic_overload() {
                unreachable!(
                    "{name}::{variant_name} needs multiple variadic overloads, i.e. {variant_name}Options implemented"
                );
            } else {
                generate_syntax_enum_deserializer_multi_overload_variant(
                    visit_seq,
                    (&name, &variant_name),
                );
            }
        }
        visit_seq.line("},");
    }

    let variants = values.keys().cloned().collect::<Vec<_>>();
    visit_seq.line(format!(
        "_ => Err(serde::de::Error::unknown_variant(&op, &[\"{}\"]))",
        variants.join("\", \"")
    ));
    visit_seq.line("}");
}

fn generate_visitor<'a>(scope: &'a mut Scope, name: &str, example: &Value) -> &'a mut Impl {
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
        .line(format!("deserializer.deserialize_seq({visitor_name})"));

    scope.new_struct(&visitor_name).doc(format!(
        "Visitor for deserializing the syntax enum [`{name}`]"
    ));

    let vis = scope
        .new_impl(&visitor_name)
        .generic("'de")
        .impl_trait("serde::de::Visitor<'de>")
        .associate_type("Value", name);
    vis.new_fn("expecting")
        .arg_ref_self()
        .arg("formatter", "&mut std::fmt::Formatter")
        .ret("std::fmt::Result")
        .line(format!(
            "formatter.write_str(r#\"an {name} like {example}\"#)"
        ));
    vis
}

/// generates a helper function for visiting a field
fn generate_visit_seq_field(visit_seq: &mut Function) {
    visit_seq
        .line("/// Reads the next element from the sequence or reports a missing field error.");
    visit_seq.line(
        "fn visit_seq_field<'de, A, T>(seq: &mut A, name: &'static str) -> Result<T, A::Error>",
    );
    visit_seq.line("where A: serde::de::SeqAccess<'de>, T: serde::Deserialize<'de> {");
    visit_seq.line("seq.next_element()?.ok_or_else(|| serde::de::Error::missing_field(name))");
    visit_seq.line("}");
    visit_seq.line("");
}

fn generate_syntax_enum_deserializer_multi_overload_variant(
    visit_seq: &mut Function,
    (name, variant_name): (&str, &str),
) {
    let options_name = format!("{variant_name}Options");

    visit_seq.line(format!(
        "// Delegate the remainder of the sequence to {options_name} deserialization"
    ));
    visit_seq
        .line("let remainder_of_sequence = serde::de::value::SeqAccessDeserializer::new(seq);");
    visit_seq.line(format!(
        "let options = {options_name}::deserialize(remainder_of_sequence)?;"
    ));
    visit_seq.line(format!("Ok({name}::{variant_name}(options))"));
}

fn generate_syntax_enum_deserializer_regular_variadic_variant(
    visit_seq: &mut Function,
    (name, variant_name): (&str, &str),
    overload: &Overload,
) {
    let position_of_variadic_separator = overload.position_of_variadic_separator();
    assert_ne!(position_of_variadic_separator, 0);
    visit_seq.line("let mut inputs = Vec::new();");
    if position_of_variadic_separator == 1 {
        visit_seq.line("while let Some(element) = seq.next_element()? {");
    } else {
        let base_name = to_snake_case(&overload.parameters[0]).replace("_1", "_i");
        visit_seq.line(format!(
            "while let Some({base_name}) = seq.next_element()? {{"
        ));
        let non_base_parameters = &overload.parameters[1..position_of_variadic_separator]
            .iter()
            .map(|p| (to_snake_case(p).replace("_1", "_i"), p.ends_with('?')))
            .collect::<Vec<_>>();
        for (param_name, is_optional) in non_base_parameters {
            if *is_optional {
                visit_seq.line(format!(
                    "let {param_name} = seq.next_element()?; // optional param"
                ));
            } else {
                visit_seq.line(format!("let {param_name} = seq.next_element()?.ok_or_else(|| serde::de::Error::custom(\"expected {param_name} in {name}::{variant_name}\"))?;"));
            }
        }
        visit_seq.line(format!(
            "let element = ({base_name},{});",
            non_base_parameters
                .iter()
                .map(|(p, _)| p.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    visit_seq.line("inputs.push(element);");
    visit_seq.line("}");
    visit_seq.line(format!("if inputs.is_empty() {{"));
    visit_seq.line(format!("return Err(serde::de::Error::custom(\"{name}::{variant_name} requires at least one argument\"));"));
    visit_seq.line("}");
    visit_seq.line(format!("Ok({name}::{variant_name}(inputs))"));
}
fn generate_syntax_enum_deserializer_regular_variant(
    visit_seq: &mut Function,
    (name, variant_name): (&str, &str),
    overload: &Overload,
) {
    for param in &overload.parameters {
        if let Some(param) = param.strip_suffix('?') {
            visit_seq.line(format!("let {param} = seq.next_element()?;"));
        } else {
            visit_seq.line(format!(
                "let {param} = visit_seq_field(&mut seq, \"{param}\")?;"
            ));
        };
    }
    if overload.parameters.is_empty() {
        visit_seq.line(format!("Ok({name}::{variant_name})"));
    } else {
        let parameters = overload
            .parameters
            .iter()
            .map(|p| p.strip_suffix('?').unwrap_or(p))
            .collect::<Vec<_>>();
        visit_seq.line(format!(
            "Ok({name}::{variant_name}({params}))",
            params = parameters.join(", ")
        ));
    }
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
                    "parameters": ["var_name_1", "var_value_1", "...", "var_name_n", "var_value_n", "expression"],
                    "output-type": "any"
                  }
                ],
                "parameters": [
                  {
                    "name": "var_name_i",
                    "type": "string literal",
                    "doc": "The name of the i-th variable."
                  },
                  {
                    "name": "var_value_i",
                    "type": "any",
                    "doc": "The value of the i-th variable."
                  },
                  {
                    "name": "expression",
                    "type": "any",
                    "doc": "The expression within which the named variables can be referenced."
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
            Let(Vec<(StringLiteral,Expression)>),
        }

        impl<'de> serde::Deserialize<'de> for Expression {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_seq(ExpressionVisitor)
            }
        }

        /// Visitor for deserializing the syntax enum [`Expression`]
        struct ExpressionVisitor;

        impl<'de> serde::de::Visitor<'de> for ExpressionVisitor {
            type Value = Expression;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(r#"an Expression like ["let","someNumber",500,["interpolate",["linear"],["var","someNumber"],274,"#edf8e9",1551,"#006d2c"]]"#)
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                use serde::Deserialize;
                /// Reads the next element from the sequence or reports a missing field error.
                fn visit_seq_field<'de, A, T>(seq: &mut A, name: &'static str) -> Result<T, A::Error>
                where A: serde::de::SeqAccess<'de>, T: serde::Deserialize<'de> {
                seq.next_element()?.ok_or_else(|| serde::de::Error::missing_field(name))
                }

                // First element: operator string
                let op: String = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("missing operator"))?;
                match op.as_str() {
                "let" => {
                let mut inputs = Vec::new();
                while let Some(var_name_i) = seq.next_element()? {
                let var_value_i = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("expected var_value_i in Expression::Let"))?;
                let element = (var_name_i,var_value_i);
                inputs.push(element);
                }
                if inputs.is_empty() {
                return Err(serde::de::Error::custom("Expression::Let requires at least one argument"));
                }
                Ok(Expression::Let(inputs))
                },
                _ => Err(serde::de::Error::unknown_variant(&op, &["let"]))
                }
            }
        }

        #[cfg(test)]
        mod test {
            use super::*;

            #[rstest::rstest]
            #[case::t_let(serde_json::json!(["let","someNumber",500,["interpolate",["linear"],["var","someNumber"],274,"#edf8e9",1551,"#006d2c"]]))]
            fn test_example_expression_decodes(#[case] example: serde_json::Value) {
                let _ = serde_json::from_value::<Expression>(example).expect("example should decode");
            }
        }
        "##);
    }
    #[test]
    fn test_generate_spec_interpolation() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "interpolation_name": {
          "doc": "First element in an interpolation array. May be followed by a number of arguments.",
          "type": "enum",
          "values": {
            "linear": {
              "doc": "Interpolates linearly between the pair of stops just less than and just greater than the input",
              "syntax": {
                "overloads": [
                  {
                    "parameters": [],
                    "output-type": "interpolation"
                  }
                ],
                "parameters": []
              },
              "example": ["linear"],
              "sdk-support": {},
              }
            },
          }
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r##"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        /// First element in an interpolation array. May be followed by a number of arguments.
        #[derive(PartialEq, Eq, Debug, Clone)]
        pub enum InterpolationName {
            /// Interpolates linearly between the pair of stops just less than and just greater than the input
            Linear,
        }

        impl<'de> serde::Deserialize<'de> for InterpolationName {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_seq(InterpolationNameVisitor)
            }
        }

        /// Visitor for deserializing the syntax enum [`InterpolationName`]
        struct InterpolationNameVisitor;

        impl<'de> serde::de::Visitor<'de> for InterpolationNameVisitor {
            type Value = InterpolationName;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(r#"an InterpolationName like ["linear"]"#)
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                use serde::Deserialize;
                /// Reads the next element from the sequence or reports a missing field error.
                fn visit_seq_field<'de, A, T>(seq: &mut A, name: &'static str) -> Result<T, A::Error>
                where A: serde::de::SeqAccess<'de>, T: serde::Deserialize<'de> {
                seq.next_element()?.ok_or_else(|| serde::de::Error::missing_field(name))
                }

                // First element: operator string
                let op: String = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("missing operator"))?;
                match op.as_str() {
                "linear" => {
                Ok(InterpolationName::Linear)
                },
                _ => Err(serde::de::Error::unknown_variant(&op, &["linear"]))
                }
            }
        }

        #[cfg(test)]
        mod test {
            use super::*;

            #[rstest::rstest]
            #[case::t_linear(serde_json::json!(["linear"]))]
            fn test_example_interpolation_name_decodes(#[case] example: serde_json::Value) {
                let _ = serde_json::from_value::<InterpolationName>(example).expect("example should decode");
            }
        }
        "##);
    }
    #[test]
    fn test_generate_spec_fmt() {
        let reference = json!({
          "$version": 8,
          "$root": {},
          "expression_name": {
            "doc": "First element in an expression array. May be followed by a number of arguments.",
            "type": "enum",
            "values": {
              "format": {
                "doc": "Returns a `formatted` string for displaying mixed-format text in the `text-field` property.",
                "syntax": {
                  "overloads": [
                    {
                      "parameters": ["input_1", "style_overrides_1?", "...", "input_n", "style_overrides_n?"],
                      "output-type": "formatted"
                    }
                  ],
                  "parameters": [
                    {
                      "name": "input_i",
                      "type": ["string", "image"]
                    },
                    {
                      "name": "style_overrides_i",
                      "type": {
                        "text-font": {
                          "type": "string",
                          "doc": "Overrides the font stack specified by the root layout property.",
                          "example": "Arial Unicode MS Regular"
                        }
                      }
                    }
                  ]
                },
                "example": ["format", ["upcase", ["get", "FacilityName"]], {"font-scale": 0.8}, "\n\n", {}, ["downcase", ["get", "Comments"]], {"font-scale": 0.6, "vertical-align": "center"}],
                "sdk-support": {}
              }
            }
          }
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r##"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        /// First element in an expression array. May be followed by a number of arguments.
        #[derive(PartialEq, Eq, Debug, Clone)]
        pub enum ExpressionName {
            /// Returns a `formatted` string for displaying mixed-format text in the `text-field` property.
            Format(Vec<(StringExpressionOrStringExpression,Object)>),
        }

        impl<'de> serde::Deserialize<'de> for ExpressionName {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_seq(ExpressionNameVisitor)
            }
        }

        /// Visitor for deserializing the syntax enum [`ExpressionName`]
        struct ExpressionNameVisitor;

        impl<'de> serde::de::Visitor<'de> for ExpressionNameVisitor {
            type Value = ExpressionName;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(r#"an ExpressionName like ["format",["upcase",["get","FacilityName"]],{"font-scale":0.8},"\n\n",{},["downcase",["get","Comments"]],{"font-scale":0.6,"vertical-align":"center"}]"#)
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                use serde::Deserialize;
                /// Reads the next element from the sequence or reports a missing field error.
                fn visit_seq_field<'de, A, T>(seq: &mut A, name: &'static str) -> Result<T, A::Error>
                where A: serde::de::SeqAccess<'de>, T: serde::Deserialize<'de> {
                seq.next_element()?.ok_or_else(|| serde::de::Error::missing_field(name))
                }

                // First element: operator string
                let op: String = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("missing operator"))?;
                match op.as_str() {
                "format" => {
                let mut inputs = Vec::new();
                while let Some(input_i) = seq.next_element()? {
                let style_overrides_i = seq.next_element()?; // optional param
                let element = (input_i,style_overrides_i);
                inputs.push(element);
                }
                if inputs.is_empty() {
                return Err(serde::de::Error::custom("ExpressionName::Format requires at least one argument"));
                }
                Ok(ExpressionName::Format(inputs))
                },
                _ => Err(serde::de::Error::unknown_variant(&op, &["format"]))
                }
            }
        }

        #[cfg(test)]
        mod test {
            use super::*;

            #[rstest::rstest]
            #[case::t_format(serde_json::json!(["format",["upcase",["get","FacilityName"]],{"font-scale":0.8},"\n\n",{},["downcase",["get","Comments"]],{"font-scale":0.6,"vertical-align":"center"}]))]
            fn test_example_expression_name_decodes(#[case] example: serde_json::Value) {
                let _ = serde_json::from_value::<ExpressionName>(example).expect("example should decode");
            }
        }
        "##);
    }
}

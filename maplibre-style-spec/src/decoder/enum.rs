use std::collections::BTreeMap;

use serde::de::Error;
use serde::{Deserialize, Deserializer};
use serde_json::{Number, Value};

use crate::decoder::ParsedItem;

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(untagged)]
pub enum EnumValues {
    Version(Vec<Number>),
    Enum(BTreeMap<String, EnumDocs>),
    SyntaxEnum(BTreeMap<String, SyntaxEnum>),
}
impl EnumValues {
    /// number of variants this enum contains
    pub fn len(&self) -> usize {
        match self {
            EnumValues::Version(numbers) => numbers.len(),
            EnumValues::Enum(btree_map) => btree_map.len(),
            EnumValues::SyntaxEnum(btree_map) => btree_map.len(),
        }
    }

    /// returns true if the enum contains no variants
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnumDocs {
    pub doc: String,
    #[serde(rename = "sdk-support")]
    pub sdk_support: Option<Value>,
}
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyntaxEnum {
    pub doc: String,
    #[serde(rename = "sdk-support")]
    pub sdk_support: Option<Value>,

    pub syntax: Syntax,
    pub example: Option<Value>,
    pub group: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Syntax {
    pub overloads: Vec<Overload>,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Overload {
    pub parameters: Vec<String>,
    #[serde(rename = "output-type")]
    pub output_type: ParameterType,
}
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parameter {
    name: String,
    r#type: ParameterType,
    doc: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(untagged)]
pub enum ParameterType {
    Literal(Literal),
    LiteralAnyOf(Vec<Literal>),
    Expression(Box<Expression>),
    ExpressionAnyOf(Vec<ParameterType>),
    Object(BTreeMap<String, ParsedItem>),
    Reference(String),
}
#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
pub enum Literal {
    #[serde(rename = "number literal")]
    Number,
    #[serde(rename = "string literal")]
    String,
    #[serde(rename = "GeoJSON object")]
    GeoJSONObject,
    #[serde(rename = "JSON object")]
    JSONObject,
    #[serde(rename = "JSON array")]
    JSONArray,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Any,
    Boolean,
    Number,
    String,
    Collator,
    Formatted,
    Image,
    Object,
    Color,
    Array {
        r#type: Option<ParameterType>,
        length: Option<usize>,
    },
}

impl<'de> Deserialize<'de> for Expression {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        // First: handle Array special cases
        if s.starts_with("array") {
            return deserialize_array_from_string(&s)
                .map_err(|e| Error::custom(format!("Failed to parse array from string: {e}")));
        }

        // Otherwise: treat the string as a simple variant
        match s.as_str() {
            "any" => Ok(Expression::Any),
            "boolean" => Ok(Expression::Boolean),
            "number" => Ok(Expression::Number),
            "string" => Ok(Expression::String),
            "collator" => Ok(Expression::Collator),
            "formatted" => Ok(Expression::Formatted),
            "image" => Ok(Expression::Image),
            "object" => Ok(Expression::Object),
            "color" => Ok(Expression::Color),

            other => Err(D::Error::unknown_variant(
                other,
                &[
                    "any",
                    "boolean",
                    "number",
                    "string",
                    "collator",
                    "formatted",
                    "image",
                    "object",
                    "color",
                    "array",
                ],
            )),
        }
    }
}

fn deserialize_array_from_string(s: &str) -> Result<Expression, String> {
    if s == "array" {
        return Ok(Expression::Array {
            r#type: None,
            length: None,
        });
    }

    let inner = s
        .strip_prefix("array<")
        .and_then(|x| x.strip_suffix('>'))
        .ok_or_else(|| "expected array<...>".to_string())?;

    let mut parts = inner.split(',').map(str::trim);

    let type_part = parts.next().unwrap();
    let len_part = parts.next();

    let r#type = match type_part {
        "any" => None,
        other => Some(
            ParameterType::deserialize(serde_json::Value::String(other.to_string()))
                .map_err(|e| e.to_string())?,
        ),
    };

    let length = if let Some(x) = len_part {
        Some(x.parse::<usize>().map_err(|e| e.to_string())?)
    } else {
        None
    };

    Ok(Expression::Array { r#type, length })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rstest::rstest;
    use serde_json::json;

    use super::*;

    #[rstest]
    #[case::array(r#""array""#, Expression::Array { r#type: None, length: None })]
    #[case::any_array(r#""array<any>""#, Expression::Array { r#type: None, length: None })]
    #[case::color_array(
            r#""array<color>""#,
            Expression::Array {
            r#type: Some(ParameterType::Expression(Box::new(Expression::Color))),
            length: None
                }
        )]
    #[case::color_array_with_length(
        r#""array<image, 2>""#,
        Expression::Array {
            r#type: Some(ParameterType::Expression(Box::new(Expression::Image))),
            length: Some(2)
            }
    )]
    #[case::any(r#""any""#, Expression::Any)]
    #[case::boolean(r#""boolean""#, Expression::Boolean)]
    #[case::number(r#""number""#, Expression::Number)]
    #[case::string(r##""string""##, Expression::String)]
    #[case::collator(r#""collator""#, Expression::Collator)]
    #[case::formatted(r#""formatted""#, Expression::Formatted)]
    #[case::image(r#""image""#, Expression::Image)]
    #[case::object(r#""object""#, Expression::Object)]
    #[case::color(r#""color""#, Expression::Color)]
    fn test_expression(#[case] input: &str, #[case] expected: Expression) {
        let expr: Expression = serde_json::from_str(input).unwrap();
        assert_eq!(expr, expected);
    }

    // expressions-parameter-types are a bit weird, so having a duplicate testcase for them is better debugging
    #[test]
    fn decode_within_expression() {
        let content = include_str!("../../../upstream/src/reference/v8.json");
        let top: HashMap<String, Value> = serde_json::from_str(content).unwrap();
        let expression = top
            .get("expression_name")
            .expect("expression_name is not in the object")
            .as_object()
            .expect("expression_name is not an object");
        let values = expression
            .get("values")
            .expect("values is not in the object")
            .as_object()
            .expect("values is not an object");
        for (k, v) in values {
            // the below two checks are just for better debugging experience
            let syntax = v.as_object().unwrap().get("syntax").expect("syntax exists");
            let params = syntax.as_object().unwrap().get("parameters");
            if let Some(params) = params {
                for (i, param) in params.as_array().unwrap().iter().enumerate() {
                    let param_type = param.as_object().unwrap().get("type").unwrap();
                    let _: ParameterType = serde_json::from_value(param_type.clone())
                        .unwrap_or_else(|e| panic!("Failed to decode ParameterType from the parameters of {k}.syntax.parameters[{i}] because {e:?}\nSerialised form was {param}"));
                }
            }
            let overloads = syntax
                .as_object()
                .unwrap()
                .get("overloads")
                .expect("parameters exists");
            for (i, overload) in overloads.as_array().unwrap().iter().enumerate() {
                let output_type = overload
                    .as_object()
                    .unwrap()
                    .get("output-type")
                    .unwrap_or_else(|| panic!("\"{k}\" does not have an output-type"));
                let _: ParameterType = serde_json::from_value(output_type.clone())
                    .unwrap_or_else(|e| panic!("Failed to decode ParameterType from the output_type of {k}.syntax.overloads[{i}] because {e:?}\nSerialised form was {output_type}"));
            }

            let _: SyntaxEnum = serde_json::from_value(v.clone())
                .unwrap_or_else(|e| panic!("Failed to decode SyntaxEnum from \"{k}\" because {e:?}\nSerialised form was {v}"));
        }
    }
    #[test]
    fn can_decode_interpolation() {
        let reference = json!({
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
                "sdk-support": {},
            }
        });
        let _: BTreeMap<String, SyntaxEnum> = serde_json::from_value(reference).unwrap();
    }
}
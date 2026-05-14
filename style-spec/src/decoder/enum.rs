use std::collections::BTreeMap;

use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Number, Value};

use crate::decoder::DecodedParsedItem;
use crate::generator::formatter::to_upper_camel_case;

#[derive(Debug, PartialEq, Clone)]
pub enum DecodedEnumValues {
    Version(Vec<Number>),
    Enum(BTreeMap<String, DecodedEnumDocs>),
    SyntaxEnum(BTreeMap<String, DecodedSyntaxEnum>),
}

impl Serialize for DecodedEnumValues {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            DecodedEnumValues::Version(v) => v.serialize(serializer),
            DecodedEnumValues::Enum(m) => m.serialize(serializer),
            DecodedEnumValues::SyntaxEnum(m) => m.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for DecodedEnumValues {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        if value.is_array() {
            return Vec::<Number>::deserialize(value)
                .map(DecodedEnumValues::Version)
                .map_err(|e| serde::de::Error::custom(format!("DecodedEnumValues::Version: {e}")));
        }
        if value.is_object() {
            // try DecodedSyntaxEnum first (more specific, has "syntax" field in values), fall back to Enum
            if let Ok(m) = BTreeMap::<String, DecodedSyntaxEnum>::deserialize(&value) {
                return Ok(DecodedEnumValues::SyntaxEnum(m));
            }
            return BTreeMap::<String, DecodedEnumDocs>::deserialize(value)
                .map(DecodedEnumValues::Enum)
                .map_err(|e| {
                    serde::de::Error::custom(format!(
                        "DecodedEnumValues: expected a map of DecodedEnumDocs or DecodedSyntaxEnum, got: {e}"
                    ))
                });
        }
        Err(serde::de::Error::custom(
            "expected an array of numbers (Version) or an object (Enum or DecodedSyntaxEnum)",
        ))
    }
}
impl DecodedEnumValues {
    /// number of variants this enum contains
    pub fn len(&self) -> usize {
        match self {
            DecodedEnumValues::Version(numbers) => numbers.len(),
            DecodedEnumValues::Enum(btree_map) => btree_map.len(),
            DecodedEnumValues::SyntaxEnum(btree_map) => btree_map.len(),
        }
    }

    /// returns true if the enum contains no variants
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_enum(&self) -> &BTreeMap<String, DecodedEnumDocs> {
        match self {
            DecodedEnumValues::SyntaxEnum(_) => panic!("Enum enum cannot be a DecodedSyntaxEnum"),
            DecodedEnumValues::Version(_) => panic!("Version enum cannot be an Enum"),
            DecodedEnumValues::Enum(btree_map) => btree_map,
        }
    }
    pub fn as_enum_mut(&mut self) -> &mut BTreeMap<String, DecodedEnumDocs> {
        match self {
            DecodedEnumValues::SyntaxEnum(_) => panic!("Enum enum cannot be a DecodedSyntaxEnum"),
            DecodedEnumValues::Version(_) => panic!("Version enum cannot be an Enum"),
            DecodedEnumValues::Enum(btree_map) => btree_map,
        }
    }
    pub fn as_syntax_enum(&self) -> &BTreeMap<String, DecodedSyntaxEnum> {
        match self {
            DecodedEnumValues::SyntaxEnum(btree_map) => btree_map,
            DecodedEnumValues::Version(_) => panic!("Version enum cannot be a DecodedSyntaxEnum"),
            DecodedEnumValues::Enum(_) => panic!("Enum enum cannot be a DecodedSyntaxEnum"),
        }
    }
    pub fn as_syntax_enum_mut(&mut self) -> &mut BTreeMap<String, DecodedSyntaxEnum> {
        match self {
            DecodedEnumValues::SyntaxEnum(btree_map) => btree_map,
            DecodedEnumValues::Version(_) => panic!("Version enum cannot be a DecodedSyntaxEnum"),
            DecodedEnumValues::Enum(_) => panic!("Enum enum cannot be a DecodedSyntaxEnum"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde_with::skip_serializing_none]
pub struct DecodedEnumDocs {
    pub doc: String,
    #[serde(rename = "sdk-support")]
    pub sdk_support: Option<Value>,
}
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde_with::skip_serializing_none]
pub struct DecodedSyntaxEnum {
    pub doc: String,
    #[serde(rename = "sdk-support")]
    pub sdk_support: Option<Value>,

    pub syntax: Syntax,
    pub example: Option<Value>,
    pub group: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde_with::skip_serializing_none]
pub struct Syntax {
    pub overloads: Vec<DecodedOverload>,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
}

impl Syntax {
    pub fn has_variadic_overload(&self) -> bool {
        self.overloads
            .iter()
            .any(|overload| overload.is_variadic(&self.parameters))
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde_with::skip_serializing_none]
pub struct DecodedOverload {
    pub parameters: Vec<String>,
    #[serde(rename = "output-type")]
    pub output_type: ParameterType,
}

impl DecodedOverload {
    pub fn position_of_variadic_separator(&self) -> usize {
        self.parameters
            .iter()
            .position(|p| p == "...")
            .expect("... parameter must be in a variadic list")
    }

    pub fn is_variadic(&self, params: &[Parameter]) -> bool {
        self.parameters.iter().any(|p| p == "...")
            || !self.parameters.iter().all(|overloaded_param| {
                params.iter().any(|actual_param| {
                    actual_param.matches_overload_parameter_name(overloaded_param)
                })
            })
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde_with::skip_serializing_none]
pub struct Parameter {
    pub name: String,
    pub r#type: ParameterType,
    pub doc: Option<String>,
}

impl Parameter {
    pub fn matches_overload_parameter_name(&self, overloaded_name: &str) -> bool {
        matches_template_name(&self.name, overloaded_name)
    }
}

/// Check whether a template parameter name (e.g. `val_i`, `stop_i_input`)
/// matches an overload parameter name (e.g. `val_1`, `stop_1_input`).
///
/// Shared implementation used by both [`Parameter`] and
/// [`crate::mir::types::MirParameter`].
pub fn matches_template_name(template: &str, overloaded_name: &str) -> bool {
    // Suffix-position template: `val_i` matches `val_1`, `val_2`, `val_n`, and optional variants.
    if let Some(maybe_template) = template.strip_suffix("_i") {
        for suffix in &["_1", "_2", "_n", "_1?", "_2?", "_n?"] {
            if let Some(param) = overloaded_name.strip_suffix(suffix) {
                return maybe_template == param;
            }
        }
        template == overloaded_name
    // Mid-name template: `stop_i_input` matches `stop_1_input`, `stop_n_input`.
    } else if let Some(pos) = template.find("_i_") {
        let prefix = &template[..pos];
        let suffix = &template[pos + 3..];
        let base = overloaded_name.strip_suffix('?').unwrap_or(overloaded_name);
        for marker in ["_1_", "_2_", "_n_"] {
            if let Some(p) = base.find(marker)
                && &base[..p] == prefix
                && &base[p + marker.len()..] == suffix
            {
                return true;
            }
        }
        template == overloaded_name
    } else if let Some(opt) = overloaded_name.strip_suffix('?') {
        template == opt
    } else {
        template == overloaded_name
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParameterType {
    Literal(Literal),
    LiteralAnyOf(Vec<Literal>),
    Expression(Box<Expression>),
    ExpressionAnyOf(Vec<ParameterType>),
    Object(BTreeMap<String, DecodedParsedItem>),
    Reference(String),
}

impl Serialize for ParameterType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ParameterType::Literal(l) => l.serialize(serializer),
            ParameterType::LiteralAnyOf(v) => v.serialize(serializer),
            ParameterType::Expression(e) => e.serialize(serializer),
            ParameterType::ExpressionAnyOf(v) => v.serialize(serializer),
            ParameterType::Object(m) => m.serialize(serializer),
            ParameterType::Reference(s) => s.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for ParameterType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match &value {
            Value::String(s) => {
                // try Literal first, then Expression, then Reference (catch-all)
                if let Ok(lit) = Literal::deserialize(&value) {
                    return Ok(ParameterType::Literal(lit));
                }
                if let Ok(expr) = Expression::deserialize(&value) {
                    return Ok(ParameterType::Expression(Box::new(expr)));
                }
                Ok(ParameterType::Reference(s.clone()))
            }
            Value::Array(_) => {
                // try LiteralAnyOf first, then ExpressionAnyOf
                if let Ok(v) = Vec::<Literal>::deserialize(&value) {
                    return Ok(ParameterType::LiteralAnyOf(v));
                }
                Vec::<ParameterType>::deserialize(value)
                    .map(ParameterType::ExpressionAnyOf)
                    .map_err(|e| {
                        serde::de::Error::custom(format!(
                            "ParameterType: array is not a valid LiteralAnyOf or ExpressionAnyOf: {e}"
                        ))
                    })
            }
            Value::Object(_) => BTreeMap::<String, DecodedParsedItem>::deserialize(value)
                .map(ParameterType::Object)
                .map_err(|e| serde::de::Error::custom(format!("ParameterType::Object: {e}"))),
            other => Err(serde::de::Error::custom(format!(
                "ParameterType: expected a string, array, or object, got {other}"
            ))),
        }
    }
}

impl ParameterType {
    pub fn to_upper_camel_case(&self) -> String {
        match self {
            ParameterType::Literal(l) => l.to_upper_camel_case().to_string(),
            ParameterType::LiteralAnyOf(ls) => ls
                .iter()
                .map(|l| l.to_upper_camel_case())
                .collect::<Vec<_>>()
                .join("Or"),
            ParameterType::Expression(e) => e.to_upper_camel_case().to_string(),
            ParameterType::ExpressionAnyOf(es) => es
                .iter()
                .map(|e| e.to_upper_camel_case())
                .collect::<Vec<_>>()
                .join("Or"),
            ParameterType::Object(_) => "Object".to_string(),
            ParameterType::Reference(r) => to_upper_camel_case(r),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
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
impl Literal {
    pub fn to_literal_type(&self) -> &'static str {
        match self {
            Literal::Number => "serde_json::Number",
            Literal::String => "String",
            Literal::GeoJSONObject => "geojson::GeoJson",
            Literal::JSONObject => "serde_json::Value",
            Literal::JSONArray => "Vec<serde_json::Value>",
        }
    }
    pub fn to_upper_camel_case(&self) -> &'static str {
        match self {
            Literal::Number => "NumberLiteral",
            Literal::String => "StringLiteral",
            Literal::GeoJSONObject => "GeoJSONObjectLiteral",
            Literal::JSONObject => "JSONObjectLiteral",
            Literal::JSONArray => "JSONArrayLiteral",
        }
    }
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

impl serde::ser::Serialize for Expression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let s = match self {
            Expression::Any => "any".to_string(),
            Expression::Boolean => "boolean".to_string(),
            Expression::Number => "number".to_string(),
            Expression::String => "string".to_string(),
            Expression::Collator => "collator".to_string(),
            Expression::Formatted => "formatted".to_string(),
            Expression::Image => "image".to_string(),
            Expression::Object => "object".to_string(),
            Expression::Color => "color".to_string(),
            Expression::Array { r#type, length } => {
                if r#type.is_none() && length.is_none() {
                    "array".to_string()
                } else {
                    // Serialize ParameterType to a string compatible with deserializer
                    let type_str = if let Some(t) = r#type {
                        // Serialize using JSON, then strip quotes if it's a string
                        let raw = serde_json::to_string(t).map_err(serde::ser::Error::custom)?;
                        // Remove quotes for string types, keep other JSON as-is
                        if raw.starts_with('"') && raw.ends_with('"') {
                            raw[1..raw.len() - 1].to_string()
                        } else {
                            raw
                        }
                    } else {
                        "any".to_string()
                    };

                    if let Some(len) = length {
                        format!("array<{type_str},{len}>")
                    } else {
                        format!("array<{type_str}>")
                    }
                }
            }
        };

        serializer.serialize_str(&s)
    }
}

impl Expression {
    pub fn to_expression_type(&self) -> &'static str {
        match self {
            Expression::Any => "Box<Expression>",
            Expression::Boolean => "Box<Boolean>",
            Expression::Number => "Box<NumberExpression>",
            Expression::String => "Box<String>",
            Expression::Collator => "Box<CollatorExpression>",
            Expression::Formatted => "Box<String>",
            Expression::Image => "Box<String>",
            Expression::Object => "Box<ObjectExpression>",
            Expression::Color => "Box<String>",
            Expression::Array { .. } => "Box<ArrayExpression>",
        }
    }
    pub fn to_upper_camel_case(&self) -> &'static str {
        match self {
            Expression::Any => "Expression",
            Expression::Boolean => "Boolean",
            Expression::Number => "NumberExpression",
            Expression::String => "String",
            Expression::Collator => "CollatorExpression",
            Expression::Formatted => "String",
            Expression::Image => "String",
            Expression::Object => "ObjectExpression",
            Expression::Color => "String",
            Expression::Array { .. } => "ArrayExpression",
        }
    }
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

            let _: DecodedSyntaxEnum = serde_json::from_value(v.clone())
                .unwrap_or_else(|e| panic!("Failed to decode DecodedSyntaxEnum from \"{k}\" because {e:?}\nSerialised form was {v}"));
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
        let _: BTreeMap<String, DecodedSyntaxEnum> = serde_json::from_value(reference).unwrap();
    }

    #[rstest]
    #[case::basic_exact_match("foo", "foo", true)]
    #[case::basic_mismatch("foo", "bar", false)]
    #[case::no_template_substition_1("foo_1", "foo_1", true)]
    #[case::no_template_substition_2("foo_2", "foo_2", true)]
    #[case::optional_suffix_match("foo", "foo?", true)]
    #[case::optional_suffix_mismatch("bar", "foo?", false)]
    #[case::template_mismatch_0("val_i", "val_0", false)]
    #[case::template_match_1("val_i", "val_1", true)]
    #[case::template_match_2("val_i", "val_2", true)]
    #[case::template_match_1_optional("val_i", "val_1?", true)]
    #[case::template_match_2_optional("val_i", "val_2?", true)]
    #[case::template_exact_fallback("val_i", "val_i", true)]
    #[case::template_invalid_numeric_suffix("val_i", "val_3", false)]
    #[case::template_base_name_mismatch("val_i", "other_1", false)]
    #[case::template_match_n("val_i", "val_n", true)]
    #[case::template_match_n_optional("val_i", "val_n?", true)]
    #[case::template_missing_suffix("val_i", "val", false)]
    #[case::mid_template_match_1("stop_i_input", "stop_1_input", true)]
    #[case::mid_template_match_n("stop_i_output", "stop_n_output", true)]
    #[case::mid_template_match_2("stop_i_input", "stop_2_input", true)]
    #[case::mid_template_suffix_mismatch("stop_i_input", "stop_1_output", false)]
    #[case::mid_template_prefix_mismatch("stop_i_input", "other_1_input", false)]
    #[case::mid_template_exact_fallback("stop_i_input", "stop_i_input", true)]
    fn test_parameter_matching(
        #[case] param_name: &str,
        #[case] overload: &str,
        #[case] expected: bool,
    ) {
        let param = Parameter {
            name: param_name.to_string(),
            r#type: ParameterType::Literal(Literal::Number),
            doc: None,
        };
        assert_eq!(param.matches_overload_parameter_name(overload), expected);
    }

    #[test]
    fn test_variadic_separator_is_found() {
        let overload = DecodedOverload {
            parameters: vec![
                "param1".to_string(),
                "param2".to_string(),
                "...".to_string(),
                "param4".to_string(),
            ],
            output_type: ParameterType::Literal(Literal::Number),
        };

        let position = overload.position_of_variadic_separator();
        assert_eq!(
            position, 2,
            "Expected the variadic separator '...' to be at index 2."
        );
    }

    #[test]
    #[should_panic(expected = "... parameter must be in a variadic list")]
    fn test_variadic_separator_is_missing_panics() {
        let overload = DecodedOverload {
            parameters: vec![],
            output_type: ParameterType::Literal(Literal::Number),
        };

        overload.position_of_variadic_separator();
    }
}

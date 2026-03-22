use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::decoder::ParsedItem;

#[derive(Debug, PartialEq, Clone)]
pub enum ArrayValue {
    Simple(SimpleArrayValue),
    Either(Vec<ArrayValue>),
    Complex(Box<ParsedItem>),
}

impl Serialize for ArrayValue {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ArrayValue::Simple(s) => s.serialize(serializer),
            ArrayValue::Either(v) => v.serialize(serializer),
            ArrayValue::Complex(p) => p.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for ArrayValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        if let Some(s) = value.as_str() {
            return SimpleArrayValue::deserialize(Value::String(s.to_owned()))
                .map(ArrayValue::Simple)
                .map_err(|e| serde::de::Error::custom(format!("ArrayValue::Simple: {e}")));
        }
        if value.is_array() {
            return Vec::<ArrayValue>::deserialize(value)
                .map(ArrayValue::Either)
                .map_err(|e| serde::de::Error::custom(format!("ArrayValue::Either: {e}")));
        }
        if value.is_object() {
            return ParsedItem::deserialize(value)
                .map(|p| ArrayValue::Complex(Box::new(p)))
                .map_err(|e| serde::de::Error::custom(format!("ArrayValue::Complex: {e}")));
        }
        Err(serde::de::Error::custom(
            "expected a string (Simple), array (Either), or object (Complex)",
        ))
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SimpleArrayValue {
    String,
    Number,
    #[serde(rename = "*")]
    Star,
    #[serde(rename = "function_stop")]
    FunctionStop,
    Layer,
    Enum,
    Color,
    #[serde(rename = "fontFaces")]
    FontFaces,
    #[serde(rename = "expression_name")]
    ExpressionName,
    #[serde(rename = "interpolation_name")]
    InterpolationName,
}

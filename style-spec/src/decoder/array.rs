use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::decoder::DecodedParsedItem;

#[derive(Debug, PartialEq, Clone)]
pub enum DecodedArrayValue {
    Simple(DecodedSimpleArrayValue),
    Either(Vec<DecodedArrayValue>),
    Complex(Box<DecodedParsedItem>),
}

impl Serialize for DecodedArrayValue {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            DecodedArrayValue::Simple(s) => s.serialize(serializer),
            DecodedArrayValue::Either(v) => v.serialize(serializer),
            DecodedArrayValue::Complex(p) => p.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for DecodedArrayValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        if let Some(s) = value.as_str() {
            return DecodedSimpleArrayValue::deserialize(Value::String(s.to_owned()))
                .map(DecodedArrayValue::Simple)
                .map_err(|e| serde::de::Error::custom(format!("DecodedArrayValue::Simple: {e}")));
        }
        if value.is_array() {
            return Vec::<DecodedArrayValue>::deserialize(value)
                .map(DecodedArrayValue::Either)
                .map_err(|e| serde::de::Error::custom(format!("DecodedArrayValue::Either: {e}")));
        }
        if value.is_object() {
            return DecodedParsedItem::deserialize(value)
                .map(|p| DecodedArrayValue::Complex(Box::new(p)))
                .map_err(|e| serde::de::Error::custom(format!("DecodedArrayValue::Complex: {e}")));
        }
        Err(serde::de::Error::custom(
            "expected a string (Simple), array (Either), or object (Complex)",
        ))
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DecodedSimpleArrayValue {
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

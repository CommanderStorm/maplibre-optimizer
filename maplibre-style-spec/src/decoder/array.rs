
use serde::Deserialize;

use crate::decoder::ParsedItem;

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(untagged)]
pub enum ArrayValue {
    Simple(SimpleArrayValue),
    Either(Vec<ArrayValue>),
    Complex(Box<ParsedItem>),
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
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

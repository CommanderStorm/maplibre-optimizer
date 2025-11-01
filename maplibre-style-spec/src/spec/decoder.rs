use std::collections::HashMap;

use serde::Deserialize;
use serde_json::{Number, Value};

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct StyleReference {
    #[serde(rename = "$version")]
    pub version: u8,
    #[serde(flatten)]
    pub fields: HashMap<String, TopLevelItem>,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(untagged)]
pub enum TopLevelItem {
    Item(ParsedItem),
    Group(HashMap<String, ParsedItem>),
    OneOf(Vec<String>),
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum ParsedItem {
    Number {
        #[serde(flatten)]
        common: CommonFields,
        default: Option<Number>,
        maximum: Option<Number>,
        minimum: Option<Number>,
        units: Option<String>,
        example: Option<Number>,
        period: Option<Number>,
    },
    Enum {
        #[serde(flatten)]
        common: CommonFields,
        default: Option<Value>,
        values: EnumValues,
        example: Option<Value>,
        required: Option<bool>,
    },
    Array {
        #[serde(flatten)]
        common: CommonFields,
        default: Option<Vec<Value>>,
        example: Option<Value>,
        value: ArrayValue,
        // if value is an enum
        values: Option<EnumValues>,
        // if value is a number
        minimum: Option<Number>,
        maximum: Option<Number>,

        required: Option<bool>,
        units: Option<String>,
        length: Option<usize>,
    },
    Color(Value),
    String(Value),
    Boolean(Value),
    #[serde(rename = "*")]
    Star(Value),
    #[serde(rename = "property-type")]
    PropertyType {
        doc: String,
        required: Option<bool>,
        example: Option<Value>,
    },
    ResolvedImage {
        #[serde(flatten)]
        common: CommonFields,
        tokens: Option<bool>,
    },
    PromoteId {
        doc: String,
    },
    NumberArray(Value),
    ColorArray(Value),
    VariableAnchorOffsetCollection(Value),
    Transition(Value),
    Terrain(Value),
    State(Value),
    Sprite(Value),
    Sources(Value),
    Source(Value),
    Sky(Value),
    ProjectionDefinition(Value),
    Projection(Value),
    Paint(Value),
    Padding(Value),
    Light(Value),
    Layout(Value),
    Formatted(Value),
    Filter(Value),
    Expression(Value),
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct CommonFields {
    doc: String,
    expression: Option<Expression>,
    #[serde(rename = "property-type")]
    property_type: Option<String>,
    #[serde(rename = "sdk-support")]
    sdk_support: Option<Value>,
    transition: Option<bool>,
    requires: Option<Vec<Requirement>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expression {
    interpolated: bool,
    parameters: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(untagged)]
pub enum Requirement {
    Exists(String),
    Equals(HashMap<String, Value>),
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
pub struct EnumValue {
    doc: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(untagged)]
pub enum EnumValues {
    Simple(Vec<Value>),
    Complex(HashMap<String, EnumValue>),
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(untagged)]
pub enum ArrayValue {
    Simple(SimpleArrayValue),
    Either(Vec<Box<ArrayValue>>),
    Complex(Box<ParsedItem>),
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SimpleArrayValue {
    String,
    Number,
    #[serde(rename = "*")]
    Star,
    FontFaces,
    #[serde(rename = "function_stop")]
    FunctionStop,
    Layer,
    Enum,
    Color,
}

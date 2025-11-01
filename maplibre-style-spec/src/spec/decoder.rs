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
        doc: String,
        default: Option<Number>,
        expression: Option<Expression>,
        #[serde(rename = "property-type")]
        property_type: Option<String>,
        #[serde(rename = "sdk-support")]
        sdk_support: Option<Value>,
        maximum: Option<Number>,
        minimum: Option<Number>,
        transition: Option<bool>,
        units: Option<String>,
        example: Option<Number>,
        period: Option<Number>,
        requires: Option<Vec<Requirement>>,
    },
    Enum {
        doc: String,
        default: Option<Value>,
        expression: Option<Expression>,
        #[serde(rename = "property-type")]
        property_type: Option<String>,
        #[serde(rename = "sdk-support")]
        sdk_support: Option<Value>,
        values: EnumValues,
        transition: Option<bool>,
        example: Option<Value>,
        required: Option<bool>,
        requires: Option<Vec<Requirement>>,
    },
    Array(Value),
    Color(Value),
    String(Value),
    Boolean(Value),
    #[serde(rename = "*")]
    Star(Value),
    #[serde(rename = "property-type")]
    PropertyType(Value),
    ResolvedImage(Value),
    PromoteId(Value),
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

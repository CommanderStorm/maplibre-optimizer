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

        units: Option<String>,
        length: Option<usize>,
    },
    Color {
        #[serde(flatten)]
        common: CommonFields,
        default: Option<Value>,
        overridable: Option<bool>,
        example: Option<Value>,
    },
    String {
        #[serde(flatten)]
        common: CommonFields,
        example: Option<String>,
        overridable: Option<bool>,
        default: Option<String>,
    },
    Boolean {
        #[serde(flatten)]
        common: CommonFields,
        default: Option<bool>,
    },
    #[serde(rename = "*")]
    Star {
        doc: String,
        example: Option<Value>,
        required: Option<bool>,
    },
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
    NumberArray {
        #[serde(flatten)]
        common: CommonFields,

        default: Option<Number>,
        minimum: Option<Number>,
        maximum: Option<Number>,
    },
    ColorArray {
        #[serde(flatten)]
        common: CommonFields,

        default: Option<String>,
    },
    VariableAnchorOffsetCollection {
        #[serde(flatten)]
        common: CommonFields,
    },
    Transition {
        doc: String,
        example: Value,
    },
    Terrain {
        doc: String,
        example: Value,
    },
    State {
        #[serde(flatten)]
        common: CommonFields,
        default: Value,
        example: Value,
    },
    Sprite {
        doc: String,
        example: Value,
    },
    Sources {
        doc: String,
        example: Value,
        required: bool,
    },
    Source {
        doc: String,
    },
    Sky {
        doc: String,
        example: Value,
    },
    ProjectionDefinition {
        #[serde(flatten)]
        common: CommonFields,
        default: String,
    },
    Projection {
        doc: String,
        example: Value,
    },
    Paint {
        doc: String,
    },
    Padding {
        #[serde(flatten)]
        common: CommonFields,
        units: String,
        default: Vec<Number>,
    },
    Light {
        doc: String,
        example: Value,
    },
    Layout {
        doc: String,
    },
    Formatted {
        #[serde(flatten)]
        common: CommonFields,
        tokens: bool,
        default: String,
    },
    Filter {
        doc: String,
    },
    Expression {
        doc: String,
    },
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
    required: Option<bool>,
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

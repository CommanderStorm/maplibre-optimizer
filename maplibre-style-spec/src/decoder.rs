use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::{Number, Value};

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct StyleReference {
    /// version of the REFERENCE
    ///
    /// version of the style spec style is defined in $root
    #[serde(rename = "$version")]
    pub version: u8,

    /// defines the layout of the style spec
    #[serde(rename = "$root")]
    pub root: BTreeMap<String, ParsedItem>,

    /// definitions of the items referenced in the root
    #[serde(flatten)]
    pub fields: BTreeMap<String, TopLevelItem>,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(untagged)]
pub enum TopLevelItem {
    Item(Box<ParsedItem>),
    Group(BTreeMap<String, ParsedItem>),
    OneOf(Vec<String>),
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum ParsedItem {
    Number {
        #[serde(flatten)]
        common: Fields,
        default: Option<Number>,
        maximum: Option<Number>,
        minimum: Option<Number>,
        period: Option<Number>,
    },
    Enum {
        #[serde(flatten)]
        common: Fields,

        default: Option<Value>,
        values: EnumValues,
    },
    Array {
        #[serde(flatten)]
        common: Fields,
        default: Option<Vec<Value>>,
        value: ArrayValue,
        // if value is an enum
        values: Option<EnumValues>,
        // if value is a number
        minimum: Option<Number>,
        maximum: Option<Number>,

        length: Option<usize>,
    },
    Color {
        #[serde(flatten)]
        common: Fields,
        default: Option<Value>,
    },
    String {
        #[serde(flatten)]
        common: Fields,
        default: Option<String>,
    },
    Boolean {
        #[serde(flatten)]
        common: Fields,
        default: Option<bool>,
    },
    #[serde(rename = "*")]
    Star(Fields),
    #[serde(rename = "property-type")]
    PropertyType(Fields),
    ResolvedImage {
        #[serde(flatten)]
        common: Fields,

        /// can autocomplete fields from layers
        tokens: Option<bool>,
    },
    PromoteId(Fields),
    NumberArray {
        #[serde(flatten)]
        common: Fields,

        default: Option<Number>,
        minimum: Option<Number>,
        maximum: Option<Number>,
    },
    ColorArray {
        #[serde(flatten)]
        common: Fields,

        default: Option<String>,
    },
    VariableAnchorOffsetCollection(Fields),
    Transition(Fields),
    Terrain(Fields),
    State {
        #[serde(flatten)]
        common: Fields,
        default: Value,
    },
    Sprite(Fields),
    Sources(Fields),
    Source(Fields),
    Sky(Fields),
    ProjectionDefinition {
        #[serde(flatten)]
        common: Fields,
        default: String,
    },
    Projection(Fields),
    Paint(Fields),
    Padding {
        #[serde(flatten)]
        common: Fields,
        default: Vec<Number>,
    },
    Light(Fields),
    Layout(Fields),
    Formatted {
        #[serde(flatten)]
        common: Fields,
        /// can autocomplete fields from layers
        tokens: bool,
        default: String,
    },
    Filter(Fields),
    Expression(Fields),
}

impl ParsedItem {
    pub fn doc(&self) -> &str {
        match self {
            ParsedItem::Number {
                common,
                default: _default,
                maximum: _maximum,
                minimum: _minimum,
                period: _period,
            } => &common.doc,
            ParsedItem::Enum {
                common,
                default: _default,
                values: _values,
            } => &common.doc,
            ParsedItem::Array {
                common,
                default: _default,
                value: _value,
                values: _values,
                minimum: _minimum,
                maximum: _maximum,
                length: _length,
            } => &common.doc,
            ParsedItem::Color {
                common,
                default: _default,
            } => &common.doc,
            ParsedItem::String {
                common,
                default: _default,
            } => &common.doc,
            ParsedItem::Boolean {
                common,
                default: _default,
            } => &common.doc,
            ParsedItem::Star(common) => &common.doc,
            ParsedItem::PropertyType(common) => &common.doc,
            ParsedItem::ResolvedImage {
                common,
                tokens: _tokens,
            } => &common.doc,
            ParsedItem::PromoteId(common) => &common.doc,
            ParsedItem::NumberArray {
                common,
                default: _default,
                minimum: _minimum,
                maximum: _maximum,
            } => &common.doc,
            ParsedItem::ColorArray {
                common,
                default: _default,
            } => &common.doc,
            ParsedItem::VariableAnchorOffsetCollection(common) => &common.doc,
            ParsedItem::Transition(common) => &common.doc,
            ParsedItem::Terrain(common) => &common.doc,
            ParsedItem::State {
                common,
                default: _default,
            } => &common.doc,
            ParsedItem::Sprite(common) => &common.doc,
            ParsedItem::Sources(common) => &common.doc,
            ParsedItem::Source(common) => &common.doc,
            ParsedItem::Sky(common) => &common.doc,
            ParsedItem::ProjectionDefinition {
                common,
                default: _default,
            } => &common.doc,
            ParsedItem::Projection(common) => &common.doc,
            ParsedItem::Paint(common) => &common.doc,
            ParsedItem::Padding {
                common,
                default: _default,
            } => &common.doc,
            ParsedItem::Light(common) => &common.doc,
            ParsedItem::Layout(common) => &common.doc,
            ParsedItem::Formatted {
                common,
                tokens: _tokens,
                default: _default,
            } => &common.doc,
            ParsedItem::Filter(common) => &common.doc,
            ParsedItem::Expression(common) => &common.doc,
        }
    }
}

#[derive(Default, Debug, PartialEq, Clone, Deserialize)]
pub struct Fields {
    // metadata fields
    pub doc: String,
    pub example: Option<Value>,
    pub units: Option<String>,

    // data fields
    pub expression: Option<Expression>,
    #[serde(rename = "property-type")]
    pub property_type: Option<String>,
    #[serde(rename = "sdk-support")]
    pub sdk_support: Option<Value>,

    // behaviour fields
    pub transition: Option<bool>,
    pub required: Option<bool>,
    pub overridable: Option<bool>,

    pub requires: Option<Vec<Requirement>>,
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expression {
    pub interpolated: bool,
    pub parameters: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(untagged)]
pub enum Requirement {
    Exists(String),
    Equals(BTreeMap<String, Value>),
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
pub struct EnumValue {
    pub doc: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(untagged)]
pub enum EnumValues {
    Simple(Vec<String>),
    Numeric(Vec<Number>),
    Complex(BTreeMap<String, EnumValue>),
}

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
    FontFaces,
    #[serde(rename = "function_stop")]
    FunctionStop,
    Layer,
    Enum,
    Color,
}

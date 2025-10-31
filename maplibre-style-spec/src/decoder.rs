use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

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
    Number(Value),
    Enum(Value),
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

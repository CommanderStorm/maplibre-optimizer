use serde::Deserialize;
use serde_json::{Number, Value};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
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
#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyntaxEnum {
    pub doc: String,
    #[serde(rename = "sdk-support")]
    pub sdk_support: Option<Value>,

    pub syntax: Syntax,
    pub example: Option<Value>,
    pub group: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Syntax {
    pub overloads: Vec<Overload>,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Overload {
    pub parameters: Vec<String>,
    #[serde(rename = "output-type")]
    pub output_type: ParameterType,
}
#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parameter {
    name: String,
    r#type: ParameterType,
    doc: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
pub enum ParameterType {
    #[serde(rename = "number literal")]
    NumberLiteral,
    #[serde(rename = "string literal")]
    StringLiteral,
    #[serde(rename = "any")]
    Any,
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "GeoJSON object")]
    GeoJSONObject,
    #[serde(rename = "JSON object")]
    JSONObject,
    #[serde(rename = "JSON array")]
    JSONArray,
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "string")]
    String,
    #[serde(rename = "collator")]
    Collator,
    #[serde(rename = "array")]
    Array,
    #[serde(rename = "array<type>", alias = "array<T>")]
    ArrayType,
    #[serde(rename = "T")]
    Type,
    #[serde(rename = "array<type, length>")]
    ArrayTypeLength,
    #[serde(rename = "formatted")]
    Formatted,
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "object")]
    Object,
    #[serde(rename = "color")]
    Color,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    // expressions are a bit weird, so having a duplicate testcase for them is better debugging
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
            let _: SyntaxEnum = serde_json::from_value(v.clone())
                .unwrap_or_else(|e| panic!("Failed to decode SyntaxEnum from {k}: {e:?}"));
        }
    }
}

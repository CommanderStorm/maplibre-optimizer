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
    PropertyType(Value),
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
    FontFaces(Fields),
}

impl ParsedItem {
    pub fn doc(&self) -> &str {
        self.common().doc.as_str()
    }
    pub fn optional(&self) -> bool {
        !self.common().required.unwrap_or(false)
    }
    fn common(&self) -> &Fields {
        match self {
            ParsedItem::Number {
                common,
                default: _default,
                maximum: _maximum,
                minimum: _minimum,
                period: _period,
            } => common,
            ParsedItem::Enum {
                common,
                default: _default,
                values: _values,
            } => common,
            ParsedItem::Array {
                common,
                default: _default,
                value: _value,
                values: _values,
                minimum: _minimum,
                maximum: _maximum,
                length: _length,
            } => common,
            ParsedItem::Color {
                common,
                default: _default,
            } => common,
            ParsedItem::String {
                common,
                default: _default,
            } => common,
            ParsedItem::Boolean {
                common,
                default: _default,
            } => common,
            ParsedItem::Star(common) => common,
            ParsedItem::PropertyType(_) => unreachable!(),
            ParsedItem::ResolvedImage {
                common,
                tokens: _tokens,
            } => common,
            ParsedItem::PromoteId(common) => common,
            ParsedItem::NumberArray {
                common,
                default: _default,
                minimum: _minimum,
                maximum: _maximum,
            } => common,
            ParsedItem::ColorArray {
                common,
                default: _default,
            } => common,
            ParsedItem::VariableAnchorOffsetCollection(common) => common,
            ParsedItem::Transition(common) => common,
            ParsedItem::Terrain(common) => common,
            ParsedItem::State {
                common,
                default: _default,
            } => common,
            ParsedItem::Sprite(common) => common,
            ParsedItem::Sources(common) => common,
            ParsedItem::Source(common) => common,
            ParsedItem::Sky(common) => common,
            ParsedItem::ProjectionDefinition {
                common,
                default: _default,
            } => common,
            ParsedItem::Projection(common) => common,
            ParsedItem::Paint(common) => common,
            ParsedItem::Padding {
                common,
                default: _default,
            } => common,
            ParsedItem::Light(common) => common,
            ParsedItem::Layout(common) => common,
            ParsedItem::Formatted {
                common,
                tokens: _tokens,
                default: _default,
            } => common,
            ParsedItem::Filter(common) => common,
            ParsedItem::Expression(common) => common,
            ParsedItem::FontFaces(common) => common,
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
    pub property_type: Option<PropertyType>,
    #[serde(rename = "sdk-support")]
    pub sdk_support: Option<Value>,

    // behaviour fields
    pub transition: Option<bool>,
    pub required: Option<bool>,
    pub overridable: Option<bool>,

    pub requires: Option<Vec<Requirement>>,
}

impl Fields {
    pub fn doc_with_range(
        &self,
        max: Option<&Number>,
        min: Option<&Number>,
        period: Option<&Number>,
    ) -> String {
        let mut doc = self.doc.clone();
        if max.is_some() || min.is_some() || period.is_some() {
            if !doc.is_empty() {
                doc.push_str("\n\n");
            }
            doc.push_str("Range: ");
            if min.is_some() || max.is_some() {
                if let Some(min) = min {
                    doc.push_str(&min.to_string());
                }
                doc.push_str("..");
                if let Some(max) = max {
                    doc.push('=');
                    doc.push_str(&max.to_string());
                }
                if period.is_some() {
                    doc.push(' ');
                }
            }
            if let Some(period) = period {
                doc.push_str(&format!("every {period}\n"))
            }
        }
        doc
    }
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
#[serde(deny_unknown_fields)]
pub struct EnumDocs {
    pub doc: String,
    #[serde(rename = "sdk-support")]
    pub sdk_support: Option<Value>,
    // for expression only
    pub syntax: Option<Syntax>,
    pub example: Option<Value>,
    pub group: Option<String>,
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
    description: Option<String>,
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
    #[serde(rename = "array | string")]
    ArrayOrString,
    #[serde(rename = "string | number")]
    StringOrNumber,
    #[serde(rename = "formatted")]
    Formatted,
    #[serde(rename = "string | image")]
    StringOrImage,
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "object")]
    Object,
    #[serde(
        rename = "number | array<number> | color | array<color> | projection | array<projection>"
    )]
    NumberOrArrayNumberOrColorOrArrayColorOrProjectionOrArrayProjection,
    #[serde(rename = "number | array<number> | color | array<color> | projection")]
    NumberOrArrayNumberOrColorOrArrayColorOrProjectionOrProjection,
    #[serde(rename = "color | array<color>")]
    ColorOrColorArray,
    #[serde(rename = "color")]
    Color,
    #[serde(
        rename = "string literal | number literal | array<string literal> | array<number literal>"
    )]
    StringLiteralOrNumberLiteralOrArrayStringLiteralOrArrayNumberLiteral,
    // below are variants which are ts defintions-ish
    #[serde(rename = "\"string\" | \"number\" | \"boolean\"")]
    StringOrNumberOrBoolean,
    #[serde(
        rename = "{ \"case-sensitive\"?: boolean, \"diacritic-sensitive\"?: boolean, \"locale\"?: string }"
    )]
    CollatorOptions,
    #[serde(
        rename = "{ \"text-font\"?: string, \"text-color\"?: color, \"font-scale\"?: number, \"vertical-align\"?: \"bottom\" | \"center\" | \"top\" }"
    )]
    FormattingOptions,
    #[serde(rename = "[\"linear\"] | [\"exponential\", base] | [\"cubic-bezier\", x1, y1, x2, y2]")]
    InterpolationType,
    #[serde(
        rename = "{ \"locale\"?: string, \"currency\"?: string, \"min-fraction-digits\"?: number, \"max-fraction-digits\"?: number }"
    )]
    LocaleOptions,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(untagged)]
pub enum EnumValues {
    Version(Vec<Number>),
    Enum(BTreeMap<String, EnumDocs>),
}
impl EnumValues {
    /// number of variants this enum contains
    pub fn len(&self) -> usize {
        match self {
            EnumValues::Version(numbers) => numbers.len(),
            EnumValues::Enum(btree_map) => btree_map.len(),
        }
    }
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
    #[serde(rename = "function_stop")]
    FunctionStop,
    Layer,
    Enum,
    Color,
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum PropertyType {
    /// Property should be specified using a color ramp from which the output color can be sampled based on a property calculation.
    ColorRamp,
    /// Property is constant across all zoom levels and property values.
    Constant,
    /// Property is non-interpolable; rather, its values will be cross-faded to smoothly transition between integer zooms.
    CrossFaded,
    /// Property is non-interpolable; rather, its values will be cross-faded to smoothly transition between integer zooms. It can be represented using a property expression.
    CrossFadedDataDriven,
    /// Property is interpolable but cannot be represented using a property expression.
    DataConstant,
    /// Property is interpolable and can be represented using a property expression.
    DataDriven,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    // expressions are a bit weird, so having a duplicate testcase for them is better debugging
    #[test]
    fn decode_within_expression() {
        let content = include_str!("../tests/upstream/src/reference/v8.json");
        let top: HashMap<String, Value> = serde_json::from_str(content).unwrap();
        let expression_name = top.get("expression_name").unwrap().as_object().unwrap();
        let values = expression_name.get("values").unwrap().as_object().unwrap();
        for (k, v) in values {
            let _: EnumDocs = serde_json::from_value(v.clone())
                .expect(&format!("Failed to decode EnumDocs of {k}"));
        }
    }

    // expressions are a bit weird, so having a duplicate testcase for them is better debugging
    #[test]
    fn decode_property_types() {
        let content = include_str!("../tests/upstream/src/reference/v8.json");
        let top: HashMap<String, Value> = serde_json::from_str(content).unwrap();
        let property_type = top.get("property-type").unwrap().as_object().unwrap();
        for k in property_type.keys() {
            let _: PropertyType = serde_json::from_str(&format!("\"{k}\"")).unwrap();
        }
    }
}

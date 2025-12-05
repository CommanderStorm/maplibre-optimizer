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
pub enum PrimitiveType {
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
    ResolvedImage {
        #[serde(flatten)]
        common: Fields,

        /// can autocomplete fields from layers
        tokens: Option<bool>,
    },
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
    State {
        #[serde(flatten)]
        common: Fields,
        default: Value,
    },
    Padding {
        #[serde(flatten)]
        common: Fields,
        default: Vec<Number>,
    },
    Formatted {
        #[serde(flatten)]
        common: Fields,
        /// can autocomplete fields from layers
        tokens: bool,
        default: String,
    },

    // meta types
    #[serde(rename = "*")]
    Star(Fields),
    #[serde(rename = "property-type")]
    PropertyType(Value),

    // below are types which are only primitives due to bad spec work upstream
    ProjectionDefinition {
        #[serde(flatten)]
        common: Fields,
        default: String,
    },
    VariableAnchorOffsetCollection(Fields),
    Sprite(Fields),
    PromoteId(Fields),
}

impl PrimitiveType {
    fn common(&self) -> &Fields {
        match self {
            Self::Number { common, .. } => common,
            Self::Enum { common, .. } => common,
            Self::Array { common, .. } => common,
            Self::Color { common, .. } => common,
            Self::String { common, .. } => common,
            Self::Boolean { common, .. } => common,
            Self::ResolvedImage { common, .. } => common,
            Self::NumberArray { common, .. } => common,
            Self::ColorArray { common, .. } => common,
            Self::Padding { common, .. } => common,
            Self::Formatted { common, .. } => common,
            // meta-types, not something proper but still useful to handle explicitly
            Self::Star(common) => common,
            Self::State { common, .. } => common,
            Self::PropertyType(_) => unreachable!(),
            // below are types which are only primitives due to bad spec work upstream
            Self::ProjectionDefinition { common, .. } => common,
            Self::VariableAnchorOffsetCollection(common) => common,
            Self::Sprite(common) => common,
            Self::PromoteId(common) => common,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(untagged)]
pub enum ParsedItem {
    Primitive(PrimitiveType),
    Reference {
        #[serde(rename = "type")]
        references: String,
        #[serde(flatten)]
        common: Fields,
    },
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
            ParsedItem::Primitive(p) => p.common(),
            ParsedItem::Reference { common, .. } => common,
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
#[serde(untagged)]
pub enum EnumValues {
    Version(Vec<Number>),
    Enum(BTreeMap<String, enum_decoder::EnumDocs>),
    SyntaxEnum(BTreeMap<String, enum_decoder::SyntaxEnum>),
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
}
pub mod enum_decoder {
    use super::*;

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
        #[serde(rename = "formatted")]
        Formatted,
        #[serde(rename = "image")]
        Image,
        #[serde(rename = "object")]
        Object,
        #[serde(rename = "color")]
        Color,
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
    #[serde(rename = "fontFaces")]
    FontFaces,
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
        let content = include_str!("../../upstream/src/reference/v8.json");
        let top: HashMap<String, Value> = serde_json::from_str(content).unwrap();
        let expression = top.get("expression").unwrap().as_object().unwrap();
        let values = expression.get("values").unwrap().as_object().unwrap();
        for (k, v) in values {
            if v.as_object().unwrap().contains_key("syntax") {
                let _: enum_decoder::SyntaxEnum = serde_json::from_value(v.clone())
                    .expect(&format!("Failed to decode EnumDocs of {k}"));
            } else {
                let _: enum_decoder::EnumDocs = serde_json::from_value(v.clone())
                    .expect(&format!("Failed to decode EnumDocs of {k}"));
            }
        }
    }

    // expressions are a bit weird, so having a duplicate testcase for them is better debugging
    #[test]
    fn decode_property_types() {
        let content = include_str!("../../upstream/src/reference/v8.json");
        let top: HashMap<String, Value> = serde_json::from_str(content).unwrap();
        let property_type = top.get("property-type").unwrap().as_object().unwrap();
        for k in property_type.keys() {
            let _: PropertyType = serde_json::from_str(&format!("\"{k}\"")).unwrap();
        }
    }
}

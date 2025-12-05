pub mod array;
pub mod r#enum;
pub mod property_type;

use std::collections::BTreeMap;

use crate::decoder::array::ArrayValue;
use crate::decoder::property_type::PropertyType;
use crate::decoder::r#enum::EnumValues;
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

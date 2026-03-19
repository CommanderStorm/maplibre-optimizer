pub mod array;
pub mod r#enum;
pub mod property_type;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};

use crate::decoder::array::ArrayValue;
use crate::decoder::r#enum::EnumValues;
use crate::decoder::property_type::PropertyType;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TopLevelItem {
    Item(Box<ParsedItem>),
    Group(BTreeMap<String, ParsedItem>),
    OneOf(Vec<String>),
}

impl TopLevelItem {
    pub fn as_item(&self) -> &ParsedItem {
        match self {
            TopLevelItem::Item(item) => &item,
            TopLevelItem::Group(_) => panic!("cannot get item from group"),
            TopLevelItem::OneOf(_) => panic!("cannot get item from oneof"),
        }
    }
    pub fn as_item_mut(&mut self) -> &mut ParsedItem {
        match self {
            TopLevelItem::Item(item) => item,
            TopLevelItem::Group(_) => panic!("cannot get item from group"),
            TopLevelItem::OneOf(_) => panic!("cannot get item from oneof"),
        }
    }
    pub fn as_group(&self) -> &BTreeMap<String, ParsedItem> {
        match self {
            TopLevelItem::Item(_) => panic!("cannot get group from item"),
            TopLevelItem::Group(group) => group,
            TopLevelItem::OneOf(_) => panic!("cannot get group from oneof"),
        }
    }
    pub fn as_group_mut(&mut self) -> &mut BTreeMap<String, ParsedItem> {
        match self {
            TopLevelItem::Item(_) => panic!("cannot get group from item"),
            TopLevelItem::Group(group) => group,
            TopLevelItem::OneOf(_) => panic!("cannot get group from oneof"),
        }
    }
    pub fn as_one_of(&self) -> &[String] {
        match self {
            TopLevelItem::Item(_) => panic!("cannot get oneof from item"),
            TopLevelItem::Group(_) => panic!("cannot get oneof from group"),
            TopLevelItem::OneOf(one_of) => one_of,
        }
    }
    pub fn as_one_of_mut(&mut self) -> &mut [String] {
        match self {
            TopLevelItem::Item(_) => panic!("cannot get oneof from item"),
            TopLevelItem::Group(_) => panic!("cannot get oneof from group"),
            TopLevelItem::OneOf(one_of) => one_of,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde_with::skip_serializing_none]
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
    pub fn as_enum(&self) -> (&EnumValues, &Fields, Option<&Value>) {
        match self {
            Self::Enum {
                values,
                common,
                default,
            } => (values, common, default.as_ref()),
            _ => panic!("cannot downcast as enum"),
        }
    }
    pub fn enum_values_mut(&mut self) -> &mut EnumValues {
        match self {
            Self::Enum { values, .. } => values,
            _ => panic!("cannot downcast as enum"),
        }
    }
    pub fn as_array(&self) -> (&ArrayValue, Option<&EnumValues>) {
        match self {
            Self::Array { value, values, .. } => (value, values.as_ref()),
            _ => panic!("cannot downcast as array"),
        }
    }

    pub fn as_number(&self) -> (&Fields, Option<f64>, Option<f64>) {
        match self {
            Self::Number { common, minimum, maximum, .. } => (
                common,
                minimum.as_ref().and_then(|n| n.as_f64()),
                maximum.as_ref().and_then(|n| n.as_f64()),
            ),
            _ => panic!("cannot downcast as number"),
        }
    }

    pub fn as_color(&self) -> (&Fields, Option<&Value>) {
        match self {
            Self::Color { common, default } => (common, default.as_ref()),
            _ => panic!("cannot downcast as color"),
        }
    }

    pub fn as_string(&self) -> (&Fields, Option<&str>) {
        match self {
            Self::String { common, default } => (common, default.as_deref()),
            _ => panic!("cannot downcast as string"),
        }
    }

    pub fn as_boolean(&self) -> (&Fields, Option<bool>) {
        match self {
            Self::Boolean { common, default } => (common, *default),
            _ => panic!("cannot downcast as boolean"),
        }
    }

    pub fn as_resolved_image(&self) -> (&Fields, Option<bool>) {
        match self {
            Self::ResolvedImage { common, tokens } => (common, *tokens),
            _ => panic!("cannot downcast as resolved_image"),
        }
    }

    pub fn as_number_array(&self) -> (&Fields, Option<f64>, Option<f64>) {
        match self {
            Self::NumberArray { common, minimum, maximum, .. } => (
                common,
                minimum.as_ref().and_then(|n| n.as_f64()),
                maximum.as_ref().and_then(|n| n.as_f64()),
            ),
            _ => panic!("cannot downcast as number_array"),
        }
    }

    pub fn as_color_array(&self) -> (&Fields, Option<&str>) {
        match self {
            Self::ColorArray { common, default } => (common, default.as_deref()),
            _ => panic!("cannot downcast as color_array"),
        }
    }

    pub fn as_padding(&self) -> (&Fields, &[Number]) {
        match self {
            Self::Padding { common, default } => (common, default.as_slice()),
            _ => panic!("cannot downcast as padding"),
        }
    }

    pub fn as_formatted(&self) -> (&Fields, bool) {
        match self {
            Self::Formatted { common, tokens, .. } => (common, *tokens),
            _ => panic!("cannot downcast as formatted"),
        }
    }

    pub fn as_state(&self) -> (&Fields, &Value) {
        match self {
            Self::State { common, default } => (common, default),
            _ => panic!("cannot downcast as state"),
        }
    }

    pub fn as_projection_definition(&self) -> (&Fields, &str) {
        match self {
            Self::ProjectionDefinition { common, default } => (common, default.as_str()),
            _ => panic!("cannot downcast as projection_definition"),
        }
    }

    pub fn as_variable_anchor_offset_collection(&self) -> &Fields {
        match self {
            Self::VariableAnchorOffsetCollection(common) => common,
            _ => panic!("cannot downcast as variable_anchor_offset_collection"),
        }
    }

    pub fn as_sprite(&self) -> &Fields {
        match self {
            Self::Sprite(common) => common,
            _ => panic!("cannot downcast as sprite"),
        }
    }

    pub fn as_promote_id(&self) -> &Fields {
        match self {
            Self::PromoteId(common) => common,
            _ => panic!("cannot downcast as promote_id"),
        }
    }

    pub fn as_star(&self) -> &Fields {
        match self {
            Self::Star(common) => common,
            _ => panic!("cannot downcast as star"),
        }
    }

    pub fn is_number(&self) -> bool { matches!(self, Self::Number { .. }) }
    pub fn is_string(&self) -> bool { matches!(self, Self::String { .. }) }
    pub fn is_boolean(&self) -> bool { matches!(self, Self::Boolean { .. }) }
    pub fn is_color(&self) -> bool { matches!(self, Self::Color { .. }) }
    pub fn is_enum(&self) -> bool { matches!(self, Self::Enum { .. }) }
    pub fn is_array(&self) -> bool { matches!(self, Self::Array { .. }) }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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

    pub fn as_primitive(&self) -> &PrimitiveType {
        match self {
            ParsedItem::Primitive(p) => p,
            ParsedItem::Reference { .. } => panic!("cannot get primitive from reference"),
        }
    }
    pub fn as_primitive_mut(&mut self) -> &mut PrimitiveType {
        match self {
            ParsedItem::Primitive(p) => p,
            ParsedItem::Reference { .. } => panic!("cannot get primitive from reference"),
        }
    }
    pub fn as_reference(&self) -> &str {
        match self {
            ParsedItem::Primitive(_) => panic!("cannot get reference from primitive"),
            ParsedItem::Reference { references, .. } => references,
        }
    }
    pub fn as_reference_mut(&mut self) -> &mut String {
        match self {
            ParsedItem::Primitive(_) => panic!("cannot get reference from primitive"),
            ParsedItem::Reference { references, .. } => references,
        }
    }
}

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
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

#[derive(Default, Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde_with::skip_serializing_none]
pub struct Expression {
    pub interpolated: bool,
    pub parameters: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Requirement {
    Exists(String),
    Equals(BTreeMap<String, Value>),
}

pub mod array;
pub mod r#enum;
pub mod property_type;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};

use crate::decoder::array::DecodedArrayValue;
use crate::decoder::r#enum::DecodedEnumValues;
use crate::decoder::property_type::DecodedPropertyType;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct StyleReference {
    /// version of the REFERENCE
    ///
    /// version of the style spec style is defined in $root
    #[serde(rename = "$version")]
    pub version: u8,

    /// defines the layout of the style spec
    #[serde(rename = "$root")]
    pub root: BTreeMap<String, DecodedParsedItem>,

    /// definitions of the items referenced in the root
    #[serde(flatten)]
    pub fields: BTreeMap<String, DecodedTopLevelItem>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum DecodedTopLevelItem {
    Item(Box<DecodedParsedItem>),
    Group(BTreeMap<String, DecodedParsedItem>),
    OneOf(Vec<String>),
}

impl Serialize for DecodedTopLevelItem {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            DecodedTopLevelItem::Item(item) => item.serialize(serializer),
            DecodedTopLevelItem::Group(group) => group.serialize(serializer),
            DecodedTopLevelItem::OneOf(one_of) => one_of.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for DecodedTopLevelItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        if value.is_array() {
            return Vec::<String>::deserialize(value)
                .map(DecodedTopLevelItem::OneOf)
                .map_err(|e| serde::de::Error::custom(format!("DecodedTopLevelItem::OneOf: {e}")));
        }
        if value.is_object() {
            // try Item first (has a "type" field), fall back to Group
            if let Ok(item) = DecodedParsedItem::deserialize(&value) {
                return Ok(DecodedTopLevelItem::Item(Box::new(item)));
            }
            return BTreeMap::<String, DecodedParsedItem>::deserialize(value)
                .map(DecodedTopLevelItem::Group)
                .map_err(|e| serde::de::Error::custom(format!("DecodedTopLevelItem::Group: {e}")));
        }
        Err(serde::de::Error::custom(
            "expected an object (Item or Group) or an array of strings (OneOf)",
        ))
    }
}

impl DecodedTopLevelItem {
    pub fn as_item(&self) -> &DecodedParsedItem {
        match self {
            DecodedTopLevelItem::Item(item) => item,
            DecodedTopLevelItem::Group(_) => panic!("cannot get item from group"),
            DecodedTopLevelItem::OneOf(_) => panic!("cannot get item from oneof"),
        }
    }
    pub fn as_item_mut(&mut self) -> &mut DecodedParsedItem {
        match self {
            DecodedTopLevelItem::Item(item) => item,
            DecodedTopLevelItem::Group(_) => panic!("cannot get item from group"),
            DecodedTopLevelItem::OneOf(_) => panic!("cannot get item from oneof"),
        }
    }
    pub fn as_group(&self) -> &BTreeMap<String, DecodedParsedItem> {
        match self {
            DecodedTopLevelItem::Item(_) => panic!("cannot get group from item"),
            DecodedTopLevelItem::Group(group) => group,
            DecodedTopLevelItem::OneOf(_) => panic!("cannot get group from oneof"),
        }
    }
    pub fn as_group_mut(&mut self) -> &mut BTreeMap<String, DecodedParsedItem> {
        match self {
            DecodedTopLevelItem::Item(_) => panic!("cannot get group from item"),
            DecodedTopLevelItem::Group(group) => group,
            DecodedTopLevelItem::OneOf(_) => panic!("cannot get group from oneof"),
        }
    }
    pub fn as_one_of(&self) -> &[String] {
        match self {
            DecodedTopLevelItem::Item(_) => panic!("cannot get oneof from item"),
            DecodedTopLevelItem::Group(_) => panic!("cannot get oneof from group"),
            DecodedTopLevelItem::OneOf(one_of) => one_of,
        }
    }
    pub fn as_one_of_mut(&mut self) -> &mut [String] {
        match self {
            DecodedTopLevelItem::Item(_) => panic!("cannot get oneof from item"),
            DecodedTopLevelItem::Group(_) => panic!("cannot get oneof from group"),
            DecodedTopLevelItem::OneOf(one_of) => one_of,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde_with::skip_serializing_none]
pub enum DecodedPrimitiveType {
    Number {
        #[serde(flatten)]
        common: DecodedFields,
        default: Option<Number>,
        maximum: Option<Number>,
        minimum: Option<Number>,
        period: Option<Number>,
    },
    Enum {
        #[serde(flatten)]
        common: DecodedFields,

        default: Option<Value>,
        values: DecodedEnumValues,
    },
    Array {
        #[serde(flatten)]
        common: DecodedFields,
        default: Option<Vec<Value>>,
        value: DecodedArrayValue,
        // if value is an enum
        values: Option<DecodedEnumValues>,
        // if value is a number
        minimum: Option<Number>,
        maximum: Option<Number>,

        length: Option<usize>,
    },
    Color {
        #[serde(flatten)]
        common: DecodedFields,
        default: Option<Value>,
    },
    String {
        #[serde(flatten)]
        common: DecodedFields,
        default: Option<String>,
    },
    Boolean {
        #[serde(flatten)]
        common: DecodedFields,
        default: Option<bool>,
    },
    ResolvedImage {
        #[serde(flatten)]
        common: DecodedFields,

        /// can autocomplete fields from layers
        tokens: Option<bool>,
    },
    NumberArray {
        #[serde(flatten)]
        common: DecodedFields,

        default: Option<Number>,
        minimum: Option<Number>,
        maximum: Option<Number>,
    },
    ColorArray {
        #[serde(flatten)]
        common: DecodedFields,

        default: Option<String>,
    },
    State {
        #[serde(flatten)]
        common: DecodedFields,
        default: Value,
    },
    Padding {
        #[serde(flatten)]
        common: DecodedFields,
        default: Vec<Number>,
    },
    Formatted {
        #[serde(flatten)]
        common: DecodedFields,
        /// can autocomplete fields from layers
        tokens: bool,
        default: String,
    },

    // meta types
    #[serde(rename = "*")]
    Star(DecodedFields),
    #[serde(rename = "property-type")]
    PropertyType(Value),

    // below are types which are only primitives due to bad spec work upstream
    ProjectionDefinition {
        #[serde(flatten)]
        common: DecodedFields,
        default: String,
    },
    VariableAnchorOffsetCollection(DecodedFields),
    Sprite(DecodedFields),
    PromoteId(DecodedFields),
}

impl DecodedPrimitiveType {
    fn common(&self) -> &DecodedFields {
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
    pub fn as_enum(&self) -> (&DecodedEnumValues, &DecodedFields, Option<&Value>) {
        match self {
            Self::Enum {
                values,
                common,
                default,
            } => (values, common, default.as_ref()),
            _ => panic!("cannot downcast as enum"),
        }
    }
    pub fn enum_values_mut(&mut self) -> &mut DecodedEnumValues {
        match self {
            Self::Enum { values, .. } => values,
            _ => panic!("cannot downcast as enum"),
        }
    }
    pub fn as_array(&self) -> (&DecodedArrayValue, Option<&DecodedEnumValues>) {
        match self {
            Self::Array { value, values, .. } => (value, values.as_ref()),
            _ => panic!("cannot downcast as array"),
        }
    }

    pub fn as_number(&self) -> (&DecodedFields, Option<f64>, Option<f64>) {
        match self {
            Self::Number {
                common,
                minimum,
                maximum,
                ..
            } => (
                common,
                minimum.as_ref().and_then(|n| n.as_f64()),
                maximum.as_ref().and_then(|n| n.as_f64()),
            ),
            _ => panic!("cannot downcast as number"),
        }
    }

    pub fn as_color(&self) -> (&DecodedFields, Option<&Value>) {
        match self {
            Self::Color { common, default } => (common, default.as_ref()),
            _ => panic!("cannot downcast as color"),
        }
    }

    pub fn as_string(&self) -> (&DecodedFields, Option<&str>) {
        match self {
            Self::String { common, default } => (common, default.as_deref()),
            _ => panic!("cannot downcast as string"),
        }
    }

    pub fn as_boolean(&self) -> (&DecodedFields, Option<bool>) {
        match self {
            Self::Boolean { common, default } => (common, *default),
            _ => panic!("cannot downcast as boolean"),
        }
    }

    pub fn as_resolved_image(&self) -> (&DecodedFields, Option<bool>) {
        match self {
            Self::ResolvedImage { common, tokens } => (common, *tokens),
            _ => panic!("cannot downcast as resolved_image"),
        }
    }

    pub fn as_number_array(&self) -> (&DecodedFields, Option<f64>, Option<f64>) {
        match self {
            Self::NumberArray {
                common,
                minimum,
                maximum,
                ..
            } => (
                common,
                minimum.as_ref().and_then(|n| n.as_f64()),
                maximum.as_ref().and_then(|n| n.as_f64()),
            ),
            _ => panic!("cannot downcast as number_array"),
        }
    }

    pub fn as_color_array(&self) -> (&DecodedFields, Option<&str>) {
        match self {
            Self::ColorArray { common, default } => (common, default.as_deref()),
            _ => panic!("cannot downcast as color_array"),
        }
    }

    pub fn as_padding(&self) -> (&DecodedFields, &[Number]) {
        match self {
            Self::Padding { common, default } => (common, default.as_slice()),
            _ => panic!("cannot downcast as padding"),
        }
    }

    pub fn as_formatted(&self) -> (&DecodedFields, bool) {
        match self {
            Self::Formatted { common, tokens, .. } => (common, *tokens),
            _ => panic!("cannot downcast as formatted"),
        }
    }

    pub fn as_state(&self) -> (&DecodedFields, &Value) {
        match self {
            Self::State { common, default } => (common, default),
            _ => panic!("cannot downcast as state"),
        }
    }

    pub fn as_projection_definition(&self) -> (&DecodedFields, &str) {
        match self {
            Self::ProjectionDefinition { common, default } => (common, default.as_str()),
            _ => panic!("cannot downcast as projection_definition"),
        }
    }

    pub fn as_variable_anchor_offset_collection(&self) -> &DecodedFields {
        match self {
            Self::VariableAnchorOffsetCollection(common) => common,
            _ => panic!("cannot downcast as variable_anchor_offset_collection"),
        }
    }

    pub fn as_sprite(&self) -> &DecodedFields {
        match self {
            Self::Sprite(common) => common,
            _ => panic!("cannot downcast as sprite"),
        }
    }

    pub fn as_promote_id(&self) -> &DecodedFields {
        match self {
            Self::PromoteId(common) => common,
            _ => panic!("cannot downcast as promote_id"),
        }
    }

    pub fn as_star(&self) -> &DecodedFields {
        match self {
            Self::Star(common) => common,
            _ => panic!("cannot downcast as star"),
        }
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number { .. })
    }
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String { .. })
    }
    pub fn is_boolean(&self) -> bool {
        matches!(self, Self::Boolean { .. })
    }
    pub fn is_color(&self) -> bool {
        matches!(self, Self::Color { .. })
    }
    pub fn is_enum(&self) -> bool {
        matches!(self, Self::Enum { .. })
    }
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array { .. })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum DecodedParsedItem {
    Primitive(DecodedPrimitiveType),
    Reference {
        references: String,
        common: DecodedFields,
    },
}

impl Serialize for DecodedParsedItem {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            DecodedParsedItem::Primitive(p) => p.serialize(serializer),
            DecodedParsedItem::Reference { references, common } => {
                use serde::ser::SerializeMap;
                // flatten common fields and add "type"
                let value = serde_json::to_value(common).map_err(serde::ser::Error::custom)?;
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("type", references)?;
                if let Value::Object(obj) = value {
                    for (k, v) in &obj {
                        map.serialize_entry(k, v)?;
                    }
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for DecodedParsedItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        // try Primitive first (internally tagged by "type" with known variant names)
        if let Ok(p) = DecodedPrimitiveType::deserialize(&value) {
            return Ok(DecodedParsedItem::Primitive(p));
        }
        // fall back to Reference: object with a "type" string that's a reference name
        let obj = value
            .as_object()
            .ok_or_else(|| serde::de::Error::custom("DecodedParsedItem: expected an object"))?;
        let references = obj
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                serde::de::Error::custom(
                    "DecodedParsedItem: expected a known primitive type or a \"type\" field with a reference string",
                )
            })?
            .to_owned();
        // deserialize remaining fields as DecodedFields (ignoring "type")
        let common = DecodedFields::deserialize(&value).map_err(|e| {
            serde::de::Error::custom(format!("DecodedParsedItem::Reference fields: {e}"))
        })?;
        Ok(DecodedParsedItem::Reference { references, common })
    }
}

impl DecodedParsedItem {
    pub fn doc(&self) -> &str {
        self.common().doc.as_str()
    }
    pub fn optional(&self) -> bool {
        !self.common().required.unwrap_or(false)
    }
    fn common(&self) -> &DecodedFields {
        match self {
            DecodedParsedItem::Primitive(p) => p.common(),
            DecodedParsedItem::Reference { common, .. } => common,
        }
    }

    pub fn as_primitive(&self) -> &DecodedPrimitiveType {
        match self {
            DecodedParsedItem::Primitive(p) => p,
            DecodedParsedItem::Reference { .. } => panic!("cannot get primitive from reference"),
        }
    }
    pub fn as_primitive_mut(&mut self) -> &mut DecodedPrimitiveType {
        match self {
            DecodedParsedItem::Primitive(p) => p,
            DecodedParsedItem::Reference { .. } => panic!("cannot get primitive from reference"),
        }
    }
    pub fn as_reference(&self) -> &str {
        match self {
            DecodedParsedItem::Primitive(_) => panic!("cannot get reference from primitive"),
            DecodedParsedItem::Reference { references, .. } => references,
        }
    }
    pub fn as_reference_mut(&mut self) -> &mut String {
        match self {
            DecodedParsedItem::Primitive(_) => panic!("cannot get reference from primitive"),
            DecodedParsedItem::Reference { references, .. } => references,
        }
    }
}

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct DecodedFields {
    // metadata fields
    pub doc: String,
    pub example: Option<Value>,
    pub units: Option<String>,

    // data fields
    pub expression: Option<Expression>,
    #[serde(rename = "property-type")]
    pub property_type: Option<DecodedPropertyType>,
    #[serde(rename = "sdk-support")]
    pub sdk_support: Option<Value>,

    // behaviour fields
    pub transition: Option<bool>,
    pub required: Option<bool>,
    pub overridable: Option<bool>,

    pub requires: Option<Vec<DecodedRequirement>>,
}

impl DecodedFields {
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DecodedRequirement {
    Exists(String),
    Equals(BTreeMap<String, Value>),
}

impl Serialize for DecodedRequirement {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            DecodedRequirement::Exists(s) => s.serialize(serializer),
            DecodedRequirement::Equals(m) => m.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for DecodedRequirement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct RequirementVisitor;
        impl<'de> serde::de::Visitor<'de> for RequirementVisitor {
            type Value = DecodedRequirement;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a string (Exists) or a map (Equals)")
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(DecodedRequirement::Exists(v.to_owned()))
            }
            fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(DecodedRequirement::Exists(v))
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                map: A,
            ) -> Result<Self::Value, A::Error> {
                let m = BTreeMap::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
                Ok(DecodedRequirement::Equals(m))
            }
        }
        deserializer.deserialize_any(RequirementVisitor)
    }
}

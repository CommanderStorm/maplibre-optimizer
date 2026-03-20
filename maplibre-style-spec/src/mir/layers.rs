use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::mir::types::{ExpressionCapabilities, IntermediateType};

/// Which sub-object of a layer a property belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropertySection {
    Paint,
    Layout,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntermediateLayerField {
    pub r#type: IntermediateType,
    pub default: Option<serde_json::Value>,
    pub doc: String,
    pub required: bool,
    pub expression: Option<ExpressionCapabilities>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntermediateLayers {
    /// Fields common to every layer type.
    pub common_fields: BTreeMap<String, IntermediateLayerField>,
    pub layer_types: BTreeMap<String, IntermediateLayerType>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntermediateLayerType {
    pub layout: BTreeMap<String, IntermediateLayerField>,
    pub paint: BTreeMap<String, IntermediateLayerField>,
}

impl IntermediateLayers {
    /// Look up a field definition by layer type, section, and property name.
    pub fn field_for(
        &self,
        layer_type: &str,
        section: PropertySection,
        property: &str,
    ) -> Option<&IntermediateLayerField> {
        let lt = self.layer_types.get(layer_type)?;
        match section {
            PropertySection::Paint => lt.paint.get(property),
            PropertySection::Layout => lt.layout.get(property),
        }
    }

    /// Return the spec default for a paint/layout property, if one is defined.
    pub fn field_default(
        &self,
        layer_type: &str,
        section: PropertySection,
        property: &str,
    ) -> Option<&serde_json::Value> {
        self.field_for(layer_type, section, property)?
            .default
            .as_ref()
    }
}

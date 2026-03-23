use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::mir::types::{MirExpressionCapabilities, MirType};

/// Which sub-object of a layer a property belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MirPropertySection {
    Paint,
    Layout,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirLayerField {
    pub r#type: MirType,
    pub default: Option<serde_json::Value>,
    pub doc: String,
    pub required: bool,
    pub expression: Option<MirExpressionCapabilities>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirLayers {
    /// DecodedFields common to every layer type.
    pub common_fields: BTreeMap<String, MirLayerField>,
    pub layer_types: BTreeMap<String, MirLayerType>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirLayerType {
    pub layout: BTreeMap<String, MirLayerField>,
    pub paint: BTreeMap<String, MirLayerField>,
}

impl MirLayers {
    /// Look up a field definition by layer type, section, and property name.
    pub fn field_for(
        &self,
        layer_type: &str,
        section: MirPropertySection,
        property: &str,
    ) -> Option<&MirLayerField> {
        let lt = self.layer_types.get(layer_type)?;
        match section {
            MirPropertySection::Paint => lt.paint.get(property),
            MirPropertySection::Layout => lt.layout.get(property),
        }
    }

    /// Return the spec default for a paint/layout property, if one is defined.
    pub fn field_default(
        &self,
        layer_type: &str,
        section: MirPropertySection,
        property: &str,
    ) -> Option<&serde_json::Value> {
        self.field_for(layer_type, section, property)?
            .default
            .as_ref()
    }
}

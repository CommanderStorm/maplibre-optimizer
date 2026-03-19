use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::mir::types::{ExpressionCapabilities, IntermediateType};

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

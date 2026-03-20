use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::mir::types::MirField;

/// All data source type definitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntermediateSources {
    /// Keyed by source type name (e.g. `"vector"`, `"geojson"`).
    pub source_types: BTreeMap<String, SourceTypeDef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceTypeDef {
    pub fields: Vec<MirField>,
}

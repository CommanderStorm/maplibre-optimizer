use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::mir::types::MirField;

/// All data source type definitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirSources {
    /// Keyed by source type name (e.g. `"vector"`, `"geojson"`).
    pub source_types: BTreeMap<String, MirSourceTypeDef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirSourceTypeDef {
    pub fields: Vec<MirField>,
}

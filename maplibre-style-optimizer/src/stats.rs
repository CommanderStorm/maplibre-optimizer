//! Data-dependent tile statistics for enriching optimization passes.
//!
//! [`TileStatistics`] captures per-source-layer statistics gathered from actual vector tiles.
//! It is consumed optionally by the optimizer to enable data-driven optimizations such as
//! selectivity reordering, geometry-type dead elimination, and zoom coverage tightening.

use std::collections::BTreeMap;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Cardinality threshold: promote from full `value_counts` to cardinality-only
/// once distinct values exceed this limit.
pub const CARDINALITY_THRESHOLD: u64 = 200;

/// Statistics gathered from a set of vector tiles, keyed by source name then source-layer name.
/// "Source name" matches the key in the style's root `"sources"` map.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TileStatistics {
    pub sources: BTreeMap<String, SourceStats>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SourceStats {
    /// Keyed by source-layer name (the MVT layer name, matches `source-layer` in style layers).
    pub layers: BTreeMap<String, LayerStats>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LayerStats {
    /// Total number of feature occurrences seen across all sampled tiles.
    ///
    /// **Important:** this is not a count of unique geographic features. In MVT, a feature that
    /// spans multiple tiles is encoded once per tile it touches. At higher zoom levels a single
    /// road or polygon will appear in many tiles and be counted multiple times. This is a known
    /// limitation: absolute counts are inflated, but *ratios* (selectivity estimates) remain
    /// consistent provided statistics are gathered uniformly across a zoom level.
    pub total_features: u64,
    /// Feature occurrence count broken down by zoom level.
    /// Use `features_by_zoom.keys()` to derive which zoom levels have coverage.
    pub features_by_zoom: BTreeMap<u8, u64>,
    /// Geometry type breakdown.
    pub geometry_types: GeometryTypeStats,
    /// Whether any feature in this layer had an MVT feature ID set.
    pub has_feature_ids: bool,
    /// Per-property statistics, keyed by property name.
    pub properties: BTreeMap<String, PropertyStats>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GeometryTypeStats {
    /// MVT `GeomType::UNKNOWN` (0). Features with this type are not renderable by any layer type.
    pub unknown: u64,
    /// MVT `GeomType::POINT` (1).
    pub point: u64,
    /// MVT `GeomType::LINESTRING` (2).
    pub linestring: u64,
    /// MVT `GeomType::POLYGON` (3).
    pub polygon: u64,
}

/// Statistics for a single property across all features in a source-layer.
///
/// Variant selection by MVT wire type:
/// - `bool_value`                          → `Bool`
/// - `int_value` / `sint_value`            → `Integer`  (sint64 is zigzag-encoded i64)
/// - `uint_value`                          → `UnsignedInteger`
/// - `float_value` / `double_value`        → `Double`   (f32 widens to f64 losslessly)
/// - `string_value`                        → `String`
/// - multiple wire types for the same key  → `Mixed`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PropertyStats {
    Bool {
        present_count: u64,
        true_count: u64,
    },
    Integer {
        present_count: u64,
        min: i64,
        max: i64,
        cardinality: u64,
        /// Full value → frequency map. `None` if cardinality exceeded threshold.
        /// `BTreeMap` so range-predicate prefix sums are efficient.
        value_counts: Option<BTreeMap<i64, u64>>,
    },
    UnsignedInteger {
        present_count: u64,
        min: u64,
        max: u64,
        cardinality: u64,
        /// Full value → frequency map. `None` if cardinality exceeded threshold.
        value_counts: Option<BTreeMap<u64, u64>>,
    },
    Double {
        present_count: u64,
        min: f64,
        max: f64,
        cardinality: u64,
    },
    String {
        present_count: u64,
        cardinality: u64,
        /// Full value → frequency map. `None` if cardinality exceeded threshold.
        /// `IndexMap` preserves insertion order; insert in descending frequency for match-arm reordering.
        value_counts: Option<IndexMap<String, u64>>,
    },
    /// Property has mixed MVT wire types across features.
    Mixed {
        present_count: u64,
        cardinality: u64,
    },
}

impl PropertyStats {
    /// Number of features that have this property set.
    #[must_use]
    pub fn present_count(&self) -> u64 {
        match self {
            Self::Bool { present_count, .. }
            | Self::Integer { present_count, .. }
            | Self::UnsignedInteger { present_count, .. }
            | Self::Double { present_count, .. }
            | Self::String { present_count, .. }
            | Self::Mixed { present_count, .. } => *present_count,
        }
    }
}

impl TileStatistics {
    /// Look up stats for a given source and source-layer.
    #[must_use]
    pub fn layer_stats(&self, source: &str, source_layer: &str) -> Option<&LayerStats> {
        self.sources.get(source)?.layers.get(source_layer)
    }
}

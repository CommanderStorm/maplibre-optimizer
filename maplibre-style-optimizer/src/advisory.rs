//! Data rewrite advisory: describes tile data transformations that pair with a rewritten style.
//!
//! The advisory tells a tile-processing tool which columns to drop, which strings to encode as
//! integers, and which rows can be filtered out. The optimizer also rewrites the style's
//! expressions so that `render(rewritten_style, rewritten_data) ≡ render(original_style,
//! original_data)`.

use std::collections::BTreeMap;
use std::fmt;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::optimize::field_analysis::{FieldAnalysis, FieldUsage};
use crate::stats::{PropertyStats, TileStatistics};

/// A key identifying a source-layer within a tile source.
///
/// Serializes as `"source/source_layer"` so it can be used as a JSON map key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceLayerKey {
    pub source: String,
    pub source_layer: String,
}

impl fmt::Display for SourceLayerKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.source, self.source_layer)
    }
}

impl Serialize for SourceLayerKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SourceLayerKey {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let (source, source_layer) = s
            .split_once('/')
            .ok_or_else(|| serde::de::Error::custom("expected 'source/source_layer'"))?;
        Ok(Self {
            source: source.to_string(),
            source_layer: source_layer.to_string(),
        })
    }
}

/// The complete advisory: what data transformations to apply, keyed by source-layer.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DataRewriteAdvisory {
    pub rewrites: BTreeMap<SourceLayerKey, SourceLayerAdvisory>,
}

/// Advisories for a single source-layer.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceLayerAdvisory {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub drop_columns: Vec<DropColumnAdvisory>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub encode_strings: Vec<EncodeStringAdvisory>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filter_rows: Vec<FilterRowAdvisory>,
}

/// Drop an unreferenced column from the tile data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropColumnAdvisory {
    pub property: String,
}

/// Encode a string property as integers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodeStringAdvisory {
    pub property: String,
    /// Original string value → integer encoding, ordered by descending frequency.
    pub mapping: IndexMap<String, i64>,
    /// Integer assigned to string values NOT in the mapping.
    /// Needed so match fallbacks still trigger for unmapped data rows.
    /// `None` if every data value is in the mapping.
    pub unmapped_sentinel: Option<i64>,
}

/// Filter rows: only retain features with these values for the given property.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRowAdvisory {
    pub property: String,
    /// Values that ARE referenced — rows with other values can be dropped.
    pub retained_values: Vec<Value>,
}

impl SourceLayerAdvisory {
    fn is_empty(&self) -> bool {
        self.drop_columns.is_empty()
            && self.encode_strings.is_empty()
            && self.filter_rows.is_empty()
    }
}

/// Compute the data rewrite advisory from field analysis and tile statistics.
#[must_use]
pub fn compute_advisory(
    field_analysis: &FieldAnalysis,
    stats: &TileStatistics,
) -> DataRewriteAdvisory {
    let mut advisory = DataRewriteAdvisory::default();

    for (key, field_usages) in field_analysis {
        let Some(layer_stats) = stats.layer_stats(&key.source, &key.source_layer) else {
            continue;
        };

        // Check if ["properties"] was used — if so, skip all advisories for this source-layer
        if field_usages.contains_key("__all_properties__") {
            continue;
        }

        let mut sl_advisory = SourceLayerAdvisory::default();

        // Drop columns: properties in stats but not referenced by any layer
        for prop_name in layer_stats.properties.keys() {
            if !field_usages.contains_key(prop_name) {
                sl_advisory.drop_columns.push(DropColumnAdvisory {
                    property: prop_name.clone(),
                });
            }
        }

        // Encode strings and filter rows
        for (prop_name, usage) in field_usages {
            if prop_name == "__all_properties__" {
                continue;
            }
            let Some(prop_stats) = layer_stats.properties.get(prop_name) else {
                continue;
            };

            try_encode_string(prop_name, usage, prop_stats, &mut sl_advisory);
            try_filter_rows(prop_name, usage, &mut sl_advisory);
        }

        if !sl_advisory.is_empty() {
            advisory.rewrites.insert(key.clone(), sl_advisory);
        }
    }

    advisory
}

/// Try to generate a string encoding advisory for a property.
#[expect(clippy::cast_possible_wrap)]
fn try_encode_string(
    prop_name: &str,
    usage: &FieldUsage,
    prop_stats: &PropertyStats,
    advisory: &mut SourceLayerAdvisory,
) {
    // Must be a string property with value_counts
    let PropertyStats::String {
        value_counts: Some(value_counts),
        ..
    } = prop_stats
    else {
        return;
    };

    // Must not be used continuously
    if usage.used_continuously {
        return;
    }

    // Must have discrete comparisons only
    let Some(compared_values) = &usage.compared_values else {
        return;
    };

    // Build mapping: assign integers 0..N-1 ordered by descending frequency
    // (IndexMap from stats already preserves frequency order)
    let mut mapping = IndexMap::new();
    for (i, (string_val, _count)) in value_counts.iter().enumerate() {
        mapping.insert(string_val.clone(), i as i64);
    }

    // Determine sentinel: if the style's compared_values is a strict subset of data values,
    // unmapped data values need a sentinel
    let data_values: Vec<Value> = value_counts
        .keys()
        .map(|s| Value::String(s.clone()))
        .collect();
    let all_compared_in_data = compared_values.iter().all(|cv| data_values.contains(cv));
    let unmapped_sentinel = if all_compared_in_data && compared_values.len() < data_values.len() {
        Some(mapping.len() as i64)
    } else if !all_compared_in_data {
        // Style references values not in data — sentinel not needed, data values are all mapped
        None
    } else {
        // compared_values == data_values — perfect coverage
        None
    };

    advisory.encode_strings.push(EncodeStringAdvisory {
        property: prop_name.to_string(),
        mapping,
        unmapped_sentinel,
    });
}

/// Try to generate a row filtering advisory for a property.
fn try_filter_rows(prop_name: &str, usage: &FieldUsage, advisory: &mut SourceLayerAdvisory) {
    // Must have discrete comparisons
    let Some(compared_values) = &usage.compared_values else {
        return;
    };

    // Must not be used continuously
    if usage.used_continuously {
        return;
    }

    // Must be used ONLY in filter context — paint/layout usage means the fallback
    // renders for non-matching rows
    if !usage.in_filter || usage.in_paint_layout {
        return;
    }

    // Must have at least one compared value to make filtering useful
    if compared_values.is_empty() {
        return;
    }

    advisory.filter_rows.push(FilterRowAdvisory {
        property: prop_name.to_string(),
        retained_values: compared_values.clone(),
    });
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use indexmap::IndexMap;
    use serde_json::json;

    use super::*;
    use crate::stats::{LayerStats, PropertyStats, SourceStats};

    fn make_key(source: &str, source_layer: &str) -> SourceLayerKey {
        SourceLayerKey {
            source: source.into(),
            source_layer: source_layer.into(),
        }
    }

    fn make_string_stats(values: &[(&str, u64)]) -> PropertyStats {
        let mut vc = IndexMap::new();
        for (v, c) in values {
            vc.insert((*v).to_string(), *c);
        }
        PropertyStats::String {
            present_count: values.iter().map(|(_, c)| c).sum(),
            cardinality: values.len() as u64,
            value_counts: Some(vc),
        }
    }

    fn make_stats_with_props(props: BTreeMap<String, PropertyStats>) -> TileStatistics {
        let mut layers = BTreeMap::new();
        layers.insert(
            "sl".to_string(),
            LayerStats {
                total_features: 1000,
                properties: props,
                ..Default::default()
            },
        );
        TileStatistics {
            sources: BTreeMap::from([("src".to_string(), SourceStats { layers })]),
        }
    }

    #[test]
    fn drop_unreferenced_column() {
        let key = make_key("src", "sl");
        let mut props = BTreeMap::new();
        props.insert("class".to_string(), make_string_stats(&[("water", 500)]));
        props.insert("unused".to_string(), make_string_stats(&[("x", 100)]));

        let stats = make_stats_with_props(props);

        let mut analysis: FieldAnalysis = BTreeMap::new();
        let mut fields = BTreeMap::new();
        fields.insert(
            "class".to_string(),
            FieldUsage {
                layer_indices: BTreeSet::from([0]),
                compared_values: Some(vec![json!("water")]),
                in_filter: true,
                ..Default::default()
            },
        );
        analysis.insert(key.clone(), fields);

        let advisory = compute_advisory(&analysis, &stats);
        let sl = &advisory.rewrites[&key];
        assert_eq!(sl.drop_columns.len(), 1);
        assert_eq!(sl.drop_columns[0].property, "unused");
    }

    #[test]
    fn encode_string_basic() {
        let key = make_key("src", "sl");
        let mut props = BTreeMap::new();
        props.insert(
            "class".to_string(),
            make_string_stats(&[("water", 500), ("forest", 300), ("grass", 200)]),
        );
        let stats = make_stats_with_props(props);

        let mut analysis: FieldAnalysis = BTreeMap::new();
        let mut fields = BTreeMap::new();
        fields.insert(
            "class".to_string(),
            FieldUsage {
                layer_indices: BTreeSet::from([0]),
                compared_values: Some(vec![json!("water"), json!("forest")]),
                in_filter: true,
                ..Default::default()
            },
        );
        analysis.insert(key.clone(), fields);

        let advisory = compute_advisory(&analysis, &stats);
        let sl = &advisory.rewrites[&key];
        assert_eq!(sl.encode_strings.len(), 1);
        let enc = &sl.encode_strings[0];
        assert_eq!(enc.property, "class");
        assert_eq!(enc.mapping["water"], 0);
        assert_eq!(enc.mapping["forest"], 1);
        assert_eq!(enc.mapping["grass"], 2);
        // Style only uses water+forest but data has grass too — sentinel needed
        assert_eq!(enc.unmapped_sentinel, Some(3));
    }

    #[test]
    fn no_encode_when_continuous() {
        let key = make_key("src", "sl");
        let mut props = BTreeMap::new();
        props.insert("name".to_string(), make_string_stats(&[("foo", 100)]));
        let stats = make_stats_with_props(props);

        let mut analysis: FieldAnalysis = BTreeMap::new();
        let mut fields = BTreeMap::new();
        fields.insert(
            "name".to_string(),
            FieldUsage {
                layer_indices: BTreeSet::from([0]),
                compared_values: None, // used continuously
                in_filter: false,
                in_paint_layout: true,
                used_continuously: true,
                ..Default::default()
            },
        );
        analysis.insert(key.clone(), fields);

        let advisory = compute_advisory(&analysis, &stats);
        assert!(
            advisory.rewrites.get(&key).is_none()
                || advisory.rewrites[&key].encode_strings.is_empty()
        );
    }

    #[test]
    fn filter_rows_when_filter_only() {
        let key = make_key("src", "sl");
        let mut props = BTreeMap::new();
        props.insert(
            "class".to_string(),
            make_string_stats(&[("water", 500), ("forest", 300), ("grass", 200)]),
        );
        let stats = make_stats_with_props(props);

        let mut analysis: FieldAnalysis = BTreeMap::new();
        let mut fields = BTreeMap::new();
        fields.insert(
            "class".to_string(),
            FieldUsage {
                layer_indices: BTreeSet::from([0]),
                compared_values: Some(vec![json!("water")]),
                in_filter: true,
                in_paint_layout: false,
                ..Default::default()
            },
        );
        analysis.insert(key.clone(), fields);

        let advisory = compute_advisory(&analysis, &stats);
        let sl = &advisory.rewrites[&key];
        assert_eq!(sl.filter_rows.len(), 1);
        assert_eq!(sl.filter_rows[0].property, "class");
        assert!(sl.filter_rows[0].retained_values.contains(&json!("water")));
    }

    #[test]
    fn no_filter_rows_when_paint_layout() {
        let key = make_key("src", "sl");
        let mut props = BTreeMap::new();
        props.insert(
            "class".to_string(),
            make_string_stats(&[("water", 500), ("forest", 300)]),
        );
        let stats = make_stats_with_props(props);

        let mut analysis: FieldAnalysis = BTreeMap::new();
        let mut fields = BTreeMap::new();
        fields.insert(
            "class".to_string(),
            FieldUsage {
                layer_indices: BTreeSet::from([0]),
                compared_values: Some(vec![json!("water")]),
                in_filter: true,
                in_paint_layout: true, // also used in paint
                ..Default::default()
            },
        );
        analysis.insert(key.clone(), fields);

        let advisory = compute_advisory(&analysis, &stats);
        assert!(
            advisory.rewrites.get(&key).is_none() || advisory.rewrites[&key].filter_rows.is_empty()
        );
    }

    #[test]
    fn advisory_serializes_to_json() {
        let mut advisory = DataRewriteAdvisory::default();
        let mut sl = SourceLayerAdvisory::default();
        sl.drop_columns.push(DropColumnAdvisory {
            property: "unused".into(),
        });
        let mut mapping = IndexMap::new();
        mapping.insert("water".to_string(), 0i64);
        mapping.insert("forest".to_string(), 1);
        sl.encode_strings.push(EncodeStringAdvisory {
            property: "class".into(),
            mapping,
            unmapped_sentinel: Some(2),
        });
        advisory.rewrites.insert(make_key("src", "sl"), sl);

        let json = serde_json::to_string_pretty(&advisory).unwrap();
        let parsed: DataRewriteAdvisory = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.rewrites.len(), 1);
    }
}

//! Tile data pruning advisories.
//!
//! Analyzes a style + tile statistics to produce a structured report of data that can be
//! safely pruned from vector tiles without affecting rendering. This is the **inverse** of
//! style optimization: instead of modifying the style to match the data, it tells the tile
//! pipeline which rows (features) and columns (properties) the style will never select.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::optimize::selectivity::{extract_get_and_literal, get_prop_name};
use crate::optimize::source_util::precompute_vector_layer_info;
use crate::optimize::walk::collect_layer_types;
use crate::stats::TileStatistics;

/// Advisory report: what tile data can be pruned given this style.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilePruningAdvisory {
    /// Keyed by source name.
    pub sources: BTreeMap<String, SourceAdvisory>,
}

/// Advisory for a single source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceAdvisory {
    /// Per-source-layer advisories.
    pub layers: BTreeMap<String, SourceLayerAdvisory>,
    /// Source-layers present in stats but never referenced by any style layer.
    pub unused_source_layers: Vec<String>,
}

/// Advisory for a single source-layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLayerAdvisory {
    /// Properties never referenced by any expression in any layer targeting this source-layer.
    pub unused_properties: Vec<String>,
    /// Geometry types that no style layer can render from this source-layer.
    pub unused_geometry_types: Vec<GeometryType>,
    /// Zoom levels where no style layer is active for this source-layer.
    pub unused_zoom_levels: Vec<u8>,
    /// Per-property value advisories: values that no filter ever selects.
    /// Only populated when stats have full `value_counts` and all filters are analyzable.
    pub unused_property_values: BTreeMap<String, UnusedValues>,
}

/// MVT geometry type for advisory output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GeometryType {
    Point,
    LineString,
    Polygon,
}

/// Describes which values of a property are never matched by any filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnusedValues {
    /// These specific values are never matched by any filter.
    Specific(Vec<Value>),
}

/// Compute tile pruning advisories from a style JSON and tile statistics.
///
/// The style is read-only; this function produces a report, it does not modify anything.
#[must_use]
pub fn compute_advisory(style: &Value, stats: &TileStatistics) -> TilePruningAdvisory {
    let style_info = collect_style_info(style);

    let mut sources: BTreeMap<String, SourceAdvisory> = BTreeMap::new();

    for (source_name, source_stats) in &stats.sources {
        let referenced_layers: HashSet<&str> = style_info
            .layer_refs
            .iter()
            .filter(|r| r.source == *source_name)
            .map(|r| r.source_layer.as_str())
            .collect();

        let unused_source_layers: Vec<String> = source_stats
            .layers
            .keys()
            .filter(|sl| !referenced_layers.contains(sl.as_str()))
            .cloned()
            .collect();

        let mut layer_advisories = BTreeMap::new();

        for (sl_name, layer_stats) in &source_stats.layers {
            if !referenced_layers.contains(sl_name.as_str()) {
                continue; // already reported as unused
            }

            let targeting: Vec<&StyleLayerRef> = style_info
                .layer_refs
                .iter()
                .filter(|r| r.source == *source_name && r.source_layer == *sl_name)
                .collect();

            layer_advisories.insert(
                sl_name.clone(),
                SourceLayerAdvisory {
                    unused_properties: compute_unused_properties(&targeting, layer_stats),
                    unused_geometry_types: compute_unused_geometry_types(&targeting, layer_stats),
                    unused_zoom_levels: compute_unused_zoom_levels(&targeting, layer_stats),
                    unused_property_values: compute_unused_property_values(&targeting, layer_stats),
                },
            );
        }

        sources.insert(
            source_name.clone(),
            SourceAdvisory {
                layers: layer_advisories,
                unused_source_layers,
            },
        );
    }

    TilePruningAdvisory { sources }
}

/// Per-layer information extracted from the style, relevant to advisory computation.
struct StyleLayerRef {
    source: String,
    source_layer: String,
    layer_type: String,
    minzoom: Option<u8>,
    maxzoom: Option<u8>,
    /// Property names accessed by `["get", prop]` or `["has", prop]` in filter/paint/layout.
    referenced_properties: HashSet<String>,
    /// Filter expression (if any), for value analysis.
    filter: Option<Value>,
}

/// Aggregated style information.
struct StyleInfo {
    layer_refs: Vec<StyleLayerRef>,
}

/// Walk the style JSON once and collect all information needed for advisory passes.
///
/// Reuses [`precompute_vector_layer_info`] for source/source-layer extraction and
/// [`collect_layer_types`] for ref-chain resolution.
fn collect_style_info(style: &Value) -> StyleInfo {
    let vector_layer_info = precompute_vector_layer_info(style);
    let layer_types = collect_layer_types(style);

    let Some(layers) = style
        .as_object()
        .and_then(|r| r.get("layers"))
        .and_then(Value::as_array)
    else {
        return StyleInfo {
            layer_refs: Vec::new(),
        };
    };

    let mut layer_refs = Vec::new();

    for (i, layer) in layers.iter().enumerate() {
        let Some(info) = vector_layer_info.get(i).and_then(|v| v.as_ref()) else {
            continue;
        };
        let Some(obj) = layer.as_object() else {
            continue;
        };

        let layer_type = layer_types
            .get(i)
            .and_then(|t| t.as_deref())
            .unwrap_or("")
            .to_string();

        let minzoom = obj
            .get("minzoom")
            .and_then(Value::as_u64)
            .and_then(|v| u8::try_from(v).ok());
        let maxzoom = obj
            .get("maxzoom")
            .and_then(Value::as_u64)
            .and_then(|v| u8::try_from(v).ok());

        let mut referenced_properties = HashSet::new();
        if let Some(filter) = obj.get("filter") {
            collect_property_refs(filter, &mut referenced_properties);
        }
        if let Some(paint) = obj.get("paint").and_then(Value::as_object) {
            for v in paint.values() {
                collect_property_refs(v, &mut referenced_properties);
            }
        }
        if let Some(layout) = obj.get("layout").and_then(Value::as_object) {
            for v in layout.values() {
                collect_property_refs(v, &mut referenced_properties);
            }
        }

        layer_refs.push(StyleLayerRef {
            source: info.source.clone(),
            source_layer: info.source_layer.clone(),
            layer_type,
            minzoom,
            maxzoom,
            referenced_properties,
            filter: obj.get("filter").cloned(),
        });
    }

    StyleInfo { layer_refs }
}

/// Recursively collect property names from `["get", "prop"]` and `["has", "prop"]` expressions.
fn collect_property_refs(expr: &Value, out: &mut HashSet<String>) {
    match expr {
        Value::Array(arr) => {
            if arr.len() == 2 {
                let op = arr[0].as_str();
                if (op == Some("get") || op == Some("has"))
                    && let Some(prop) = arr[1].as_str()
                {
                    out.insert(prop.to_string());
                }
            }
            for child in arr {
                collect_property_refs(child, out);
            }
        }
        Value::Object(map) => {
            for v in map.values() {
                collect_property_refs(v, out);
            }
        }
        _ => {}
    }
}

// ── Pass 1: Unused properties ────────────────────────────────────────────────

fn compute_unused_properties(
    targeting: &[&StyleLayerRef],
    layer_stats: &crate::stats::LayerStats,
) -> Vec<String> {
    let referenced: HashSet<&str> = targeting
        .iter()
        .flat_map(|r| r.referenced_properties.iter().map(String::as_str))
        .collect();

    let mut unused: Vec<String> = layer_stats
        .properties
        .keys()
        .filter(|prop| !referenced.contains(prop.as_str()))
        .cloned()
        .collect();
    unused.sort();
    unused
}

// ── Pass 2: Unused geometry types ────────────────────────────────────────────

/// Determine which geometry types are needed based on the layer types targeting this source-layer.
fn compute_unused_geometry_types(
    targeting: &[&StyleLayerRef],
    layer_stats: &crate::stats::LayerStats,
) -> Vec<GeometryType> {
    let mut needed = HashSet::new();

    for r in targeting {
        match r.layer_type.as_str() {
            "fill" | "fill-extrusion" => {
                needed.insert(GeometryType::Polygon);
            }
            "circle" | "heatmap" => {
                needed.insert(GeometryType::Point);
            }
            "line" => {
                needed.insert(GeometryType::LineString);
                needed.insert(GeometryType::Polygon);
            }
            "symbol" => {
                needed.insert(GeometryType::Point);
                needed.insert(GeometryType::LineString);
            }
            // Unknown or other types — conservatively assume all are needed.
            _ => return vec![],
        }
    }

    let gt = &layer_stats.geometry_types;
    let mut unused = Vec::new();
    if gt.point > 0 && !needed.contains(&GeometryType::Point) {
        unused.push(GeometryType::Point);
    }
    if gt.linestring > 0 && !needed.contains(&GeometryType::LineString) {
        unused.push(GeometryType::LineString);
    }
    if gt.polygon > 0 && !needed.contains(&GeometryType::Polygon) {
        unused.push(GeometryType::Polygon);
    }
    unused
}

// ── Pass 3: Unused zoom levels ───────────────────────────────────────────────

fn compute_unused_zoom_levels(
    targeting: &[&StyleLayerRef],
    layer_stats: &crate::stats::LayerStats,
) -> Vec<u8> {
    if targeting.is_empty() {
        return layer_stats.features_by_zoom.keys().copied().collect();
    }

    let all_zooms: BTreeSet<u8> = layer_stats.features_by_zoom.keys().copied().collect();
    if all_zooms.is_empty() {
        return vec![];
    }

    let global_min = *all_zooms.first().unwrap_or(&0);
    let global_max = *all_zooms.last().unwrap_or(&22);

    let mut active_zooms: BTreeSet<u8> = BTreeSet::new();
    for r in targeting {
        let min = r.minzoom.unwrap_or(global_min);
        let max = r.maxzoom.unwrap_or(global_max);
        for z in min..=max {
            active_zooms.insert(z);
        }
    }

    all_zooms.difference(&active_zooms).copied().collect()
}

// ── Pass 4: Unused property values ───────────────────────────────────────────

fn compute_unused_property_values(
    targeting: &[&StyleLayerRef],
    layer_stats: &crate::stats::LayerStats,
) -> BTreeMap<String, UnusedValues> {
    let mut result = BTreeMap::new();

    for (prop_name, prop_stats) in &layer_stats.properties {
        let all_values: Vec<Value> = match prop_stats {
            crate::stats::PropertyStats::String {
                value_counts: Some(vc),
                ..
            } => vc.keys().map(|k| Value::String(k.clone())).collect(),
            crate::stats::PropertyStats::Integer {
                value_counts: Some(vc),
                ..
            } => vc.keys().map(|k| Value::Number((*k).into())).collect(),
            crate::stats::PropertyStats::UnsignedInteger {
                value_counts: Some(vc),
                ..
            } => vc.keys().map(|k| Value::Number((*k).into())).collect(),
            _ => continue,
        };

        let mut selected_values: HashSet<String> = HashSet::new();
        let mut all_analyzable = true;

        for layer_ref in targeting {
            match &layer_ref.filter {
                None => {
                    // No filter means all features pass — all values are needed.
                    all_analyzable = false;
                    break;
                }
                Some(filter) => {
                    if !extract_selected_values(filter, prop_name, &mut selected_values) {
                        all_analyzable = false;
                        break;
                    }
                }
            }
        }

        if !all_analyzable || selected_values.is_empty() {
            continue;
        }

        let unused: Vec<Value> = all_values
            .into_iter()
            .filter(|v| !selected_values.contains(&value_key(v)))
            .collect();

        if !unused.is_empty() {
            result.insert(prop_name.clone(), UnusedValues::Specific(unused));
        }
    }

    result
}

/// Extract values selected for `prop_name` from a filter expression.
/// Returns `true` if the filter is analyzable (closed-world: we can enumerate all selected values).
/// Adds selected values to `out`.
fn extract_selected_values(filter: &Value, prop_name: &str, out: &mut HashSet<String>) -> bool {
    let Value::Array(arr) = filter else {
        return false;
    };
    if arr.is_empty() {
        return false;
    }
    let Some(op) = arr[0].as_str() else {
        return false;
    };

    match op {
        "==" if arr.len() == 3 => {
            if let Some((prop, val)) = extract_get_and_literal(&arr[1], &arr[2])
                && prop == prop_name
            {
                out.insert(value_key(&val));
                return true;
            }
            false
        }
        "in" if arr.len() == 3 => {
            if let Some(prop) = get_prop_name(&arr[1])
                && prop == prop_name
                && let Some(values) = extract_literal_array(&arr[2])
            {
                for v in &values {
                    out.insert(value_key(v));
                }
                return true;
            }
            false
        }
        "match" if arr.len() >= 3 => {
            if let Some(prop) = get_prop_name(&arr[1])
                && prop == prop_name
            {
                // Labels are at even positions in rest; last element is default if count is odd.
                let rest = &arr[2..];
                let pairs_end = if rest.len() % 2 == 1 {
                    rest.len() - 1
                } else {
                    rest.len()
                };

                for i in (0..pairs_end).step_by(2) {
                    match &rest[i] {
                        Value::Array(label_arr) => {
                            if label_arr.len() == 2 && label_arr[0].as_str() == Some("literal") {
                                if let Value::Array(vals) = &label_arr[1] {
                                    for v in vals {
                                        out.insert(value_key(v));
                                    }
                                }
                            } else {
                                for v in label_arr {
                                    out.insert(value_key(v));
                                }
                            }
                        }
                        v => {
                            out.insert(value_key(v));
                        }
                    }
                }
                return true;
            }
            false
        }
        "all" => {
            // Conjunction: if ANY child constrains this property, we can use it.
            let mut any_constrains = false;
            for child in arr.iter().skip(1) {
                if extract_selected_values(child, prop_name, out) {
                    any_constrains = true;
                }
            }
            any_constrains
        }
        "any" => {
            // Disjunction: ALL children must constrain this property for closed-world analysis.
            let mut temp = HashSet::new();
            for child in arr.iter().skip(1) {
                if !extract_selected_values(child, prop_name, &mut temp) {
                    return false;
                }
            }
            if temp.is_empty() {
                return false;
            }
            out.extend(temp);
            true
        }
        _ => false,
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn extract_literal_array(v: &Value) -> Option<Vec<Value>> {
    let Value::Array(arr) = v else { return None };
    if arr.len() == 2
        && arr[0].as_str() == Some("literal")
        && let Value::Array(values) = &arr[1]
    {
        return Some(values.clone());
    }
    None
}

/// Produce a canonical string key for a JSON value, for use in `HashSet`.
fn value_key(v: &Value) -> String {
    match v {
        Value::String(s) => format!("s:{s}"),
        Value::Number(n) => format!("n:{n}"),
        Value::Bool(b) => format!("b:{b}"),
        Value::Null => "null".to_string(),
        other => format!("j:{other}"),
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use indexmap::IndexMap;
    use serde_json::json;

    use super::*;
    use crate::stats::{GeometryTypeStats, LayerStats, PropertyStats, SourceStats, TileStatistics};

    fn sample_style() -> Value {
        json!({
            "version": 8,
            "sources": {
                "openmaptiles": {
                    "type": "vector",
                    "url": "https://example.com/tiles.json"
                }
            },
            "layers": [
                {
                    "id": "road-primary",
                    "type": "line",
                    "source": "openmaptiles",
                    "source-layer": "transportation",
                    "minzoom": 5,
                    "maxzoom": 14,
                    "filter": ["==", ["get", "class"], "primary"],
                    "paint": {
                        "line-color": "#fff",
                        "line-width": ["interpolate", ["linear"], ["zoom"], 5, 1, 14, 8]
                    }
                },
                {
                    "id": "road-motorway",
                    "type": "line",
                    "source": "openmaptiles",
                    "source-layer": "transportation",
                    "filter": ["in", ["get", "class"], ["literal", ["motorway", "trunk"]]],
                    "paint": {
                        "line-color": "#f00"
                    }
                },
                {
                    "id": "water-fill",
                    "type": "fill",
                    "source": "openmaptiles",
                    "source-layer": "water",
                    "paint": {
                        "fill-color": "#00f"
                    }
                }
            ]
        })
    }

    fn sample_stats() -> TileStatistics {
        let mut transport_props = BTreeMap::new();
        let mut vc = IndexMap::new();
        vc.insert("motorway".to_string(), 5000u64);
        vc.insert("trunk".to_string(), 3000u64);
        vc.insert("primary".to_string(), 10_000u64);
        vc.insert("secondary".to_string(), 20_000u64);
        vc.insert("tertiary".to_string(), 30_000u64);
        vc.insert("service".to_string(), 32_000u64);
        transport_props.insert(
            "class".to_string(),
            PropertyStats::String {
                present_count: 100_000,
                cardinality: 6,
                value_counts: Some(vc),
            },
        );
        transport_props.insert(
            "subclass".to_string(),
            PropertyStats::String {
                present_count: 50_000,
                cardinality: 10,
                value_counts: None,
            },
        );
        transport_props.insert(
            "brunnel".to_string(),
            PropertyStats::String {
                present_count: 5000,
                cardinality: 3,
                value_counts: None,
            },
        );

        let mut transport_zoom = BTreeMap::new();
        transport_zoom.insert(4u8, 1000u64);
        transport_zoom.insert(5, 5000);
        transport_zoom.insert(6, 10_000);
        transport_zoom.insert(10, 30_000);
        transport_zoom.insert(14, 54_000);

        let mut water_props = BTreeMap::new();
        water_props.insert(
            "class".to_string(),
            PropertyStats::String {
                present_count: 10_000,
                cardinality: 3,
                value_counts: None,
            },
        );

        let mut layers = BTreeMap::new();
        layers.insert(
            "transportation".to_string(),
            LayerStats {
                total_features: 100_000,
                features_by_zoom: transport_zoom,
                geometry_types: GeometryTypeStats {
                    unknown: 0,
                    point: 500,
                    linestring: 90_000,
                    polygon: 9500,
                },
                has_feature_ids: false,
                properties: transport_props,
            },
        );
        layers.insert(
            "water".to_string(),
            LayerStats {
                total_features: 10_000,
                features_by_zoom: BTreeMap::from([(5, 2000), (10, 8000)]),
                geometry_types: GeometryTypeStats {
                    unknown: 0,
                    point: 0,
                    linestring: 0,
                    polygon: 10_000,
                },
                has_feature_ids: false,
                properties: water_props,
            },
        );
        layers.insert(
            "poi".to_string(),
            LayerStats {
                total_features: 5000,
                features_by_zoom: BTreeMap::from([(14, 5000)]),
                geometry_types: GeometryTypeStats {
                    unknown: 0,
                    point: 5000,
                    linestring: 0,
                    polygon: 0,
                },
                has_feature_ids: false,
                properties: BTreeMap::new(),
            },
        );

        let mut sources = BTreeMap::new();
        sources.insert("openmaptiles".to_string(), SourceStats { layers });

        TileStatistics { sources }
    }

    #[test]
    fn unused_source_layers() {
        let style = sample_style();
        let stats = sample_stats();
        let advisory = compute_advisory(&style, &stats);

        let source = &advisory.sources["openmaptiles"];
        assert!(source.unused_source_layers.contains(&"poi".to_string()));
        assert!(
            !source
                .unused_source_layers
                .contains(&"transportation".to_string())
        );
        assert!(!source.unused_source_layers.contains(&"water".to_string()));
    }

    #[test]
    fn unused_properties() {
        let style = sample_style();
        let stats = sample_stats();
        let advisory = compute_advisory(&style, &stats);

        let transport = &advisory.sources["openmaptiles"].layers["transportation"];
        assert!(!transport.unused_properties.contains(&"class".to_string()));
        assert!(
            transport
                .unused_properties
                .contains(&"subclass".to_string())
        );
        assert!(transport.unused_properties.contains(&"brunnel".to_string()));
    }

    #[test]
    fn unused_geometry_types() {
        let style = sample_style();
        let stats = sample_stats();
        let advisory = compute_advisory(&style, &stats);

        let transport = &advisory.sources["openmaptiles"].layers["transportation"];
        assert!(
            transport
                .unused_geometry_types
                .contains(&GeometryType::Point)
        );
        assert!(
            !transport
                .unused_geometry_types
                .contains(&GeometryType::LineString)
        );
        assert!(
            !transport
                .unused_geometry_types
                .contains(&GeometryType::Polygon)
        );

        let water = &advisory.sources["openmaptiles"].layers["water"];
        assert!(water.unused_geometry_types.is_empty());
    }

    #[test]
    fn unused_zoom_levels() {
        let style = sample_style();
        let stats = sample_stats();
        let advisory = compute_advisory(&style, &stats);

        let transport = &advisory.sources["openmaptiles"].layers["transportation"];
        // road-motorway has no zoom constraints → defaults to data range 4..=14, covering all.
        assert!(transport.unused_zoom_levels.is_empty());
    }

    #[test]
    fn unused_zoom_levels_with_restricted_range() {
        let style = json!({
            "version": 8,
            "sources": { "src": { "type": "vector" } },
            "layers": [{
                "id": "l1",
                "type": "line",
                "source": "src",
                "source-layer": "roads",
                "minzoom": 6,
                "maxzoom": 10
            }]
        });

        let stats = TileStatistics {
            sources: BTreeMap::from([(
                "src".to_string(),
                SourceStats {
                    layers: BTreeMap::from([(
                        "roads".to_string(),
                        LayerStats {
                            total_features: 100,
                            features_by_zoom: BTreeMap::from([
                                (4, 10),
                                (6, 20),
                                (10, 30),
                                (14, 40),
                            ]),
                            geometry_types: GeometryTypeStats::default(),
                            has_feature_ids: false,
                            properties: BTreeMap::new(),
                        },
                    )]),
                },
            )]),
        };

        let advisory = compute_advisory(&style, &stats);
        let roads = &advisory.sources["src"].layers["roads"];
        assert!(roads.unused_zoom_levels.contains(&4));
        assert!(roads.unused_zoom_levels.contains(&14));
        assert!(!roads.unused_zoom_levels.contains(&6));
        assert!(!roads.unused_zoom_levels.contains(&10));
    }

    #[test]
    fn unused_property_values() {
        let style = sample_style();
        let stats = sample_stats();
        let advisory = compute_advisory(&style, &stats);

        let transport = &advisory.sources["openmaptiles"].layers["transportation"];
        if let Some(UnusedValues::Specific(unused)) = transport.unused_property_values.get("class")
        {
            let strs: Vec<&str> = unused.iter().filter_map(Value::as_str).collect();
            assert!(strs.contains(&"secondary"));
            assert!(strs.contains(&"tertiary"));
            assert!(strs.contains(&"service"));
            assert!(!strs.contains(&"primary"));
            assert!(!strs.contains(&"motorway"));
            assert!(!strs.contains(&"trunk"));
        } else {
            panic!("Expected unused property values for 'class'");
        }
    }

    #[test]
    fn no_unused_values_without_filter() {
        let style = json!({
            "version": 8,
            "sources": { "s": { "type": "vector" } },
            "layers": [{
                "id": "l",
                "type": "fill",
                "source": "s",
                "source-layer": "land"
            }]
        });

        let mut props = BTreeMap::new();
        let mut vc = IndexMap::new();
        vc.insert("forest".to_string(), 100u64);
        vc.insert("urban".to_string(), 200u64);
        props.insert(
            "class".to_string(),
            PropertyStats::String {
                present_count: 300,
                cardinality: 2,
                value_counts: Some(vc),
            },
        );

        let stats = TileStatistics {
            sources: BTreeMap::from([(
                "s".to_string(),
                SourceStats {
                    layers: BTreeMap::from([(
                        "land".to_string(),
                        LayerStats {
                            total_features: 300,
                            features_by_zoom: BTreeMap::new(),
                            geometry_types: GeometryTypeStats {
                                polygon: 300,
                                ..GeometryTypeStats::default()
                            },
                            has_feature_ids: false,
                            properties: props,
                        },
                    )]),
                },
            )]),
        };

        let advisory = compute_advisory(&style, &stats);
        let land = &advisory.sources["s"].layers["land"];
        assert!(land.unused_property_values.is_empty());
    }

    #[test]
    fn collect_property_refs_recursive() {
        let expr = json!([
            "interpolate",
            ["linear"],
            ["zoom"],
            5,
            ["get", "width"],
            10,
            ["*", ["get", "lanes"], 2]
        ]);
        let mut out = HashSet::new();
        collect_property_refs(&expr, &mut out);
        assert!(out.contains("width"));
        assert!(out.contains("lanes"));
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn empty_style_produces_empty_advisory() {
        let style = json!({ "version": 8 });
        let stats = TileStatistics::default();
        let advisory = compute_advisory(&style, &stats);
        assert!(advisory.sources.is_empty());
    }

    #[test]
    fn serialization_roundtrip() {
        let style = sample_style();
        let stats = sample_stats();
        let advisory = compute_advisory(&style, &stats);

        let json = serde_json::to_string_pretty(&advisory).unwrap();
        let parsed: TilePruningAdvisory = serde_json::from_str(&json).unwrap();
        assert_eq!(advisory.sources.len(), parsed.sources.len());
    }
}

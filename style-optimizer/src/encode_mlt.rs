//! MVT → MLT encoding bridge.
//!
//! Converts raw MVT protobuf bytes into MLT (`MapLibre` Tiles) binary format
//! using `mlt-core`.

use std::collections::BTreeMap;

use anyhow::Context;
use mlt_core::OwnedLayer;
use mlt_core::geojson::Feature;
use mlt_core::optimizer::AutomaticOptimisation;
use mlt_core::v01::{
    DecodedGeometry, DecodedId, DecodedProperty, OwnedGeometry, OwnedId, OwnedLayer01,
    OwnedProperty, PropValue,
};
use serde_json::Value;

/// Convert raw MVT protobuf bytes into MLT binary format.
///
/// Pipeline:
/// 1. Parse MVT via `mlt_core::mvt::mvt_to_feature_collection`
/// 2. Group features by `_layer` property
/// 3. For each layer, build decoded `OwnedLayer01` (geometry, IDs, properties)
/// 4. Run automatic encoding optimisation
/// 5. Serialize to MLT binary
pub fn mvt_to_mlt(mvt_bytes: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    let fc = mlt_core::mvt::mvt_to_feature_collection(mvt_bytes)
        .context("parse MVT for MLT conversion")?;

    // Group features by _layer.
    let mut by_layer: BTreeMap<String, (u32, Vec<&Feature>)> = BTreeMap::new();
    for feature in &fc.features {
        let layer_name = feature
            .properties
            .get("_layer")
            .and_then(Value::as_str)
            .unwrap_or("default");
        #[expect(clippy::cast_possible_truncation)]
        let extent = feature
            .properties
            .get("_extent")
            .and_then(Value::as_u64)
            .unwrap_or(4096) as u32;
        by_layer
            .entry(layer_name.to_string())
            .or_insert_with(|| (extent, Vec::new()))
            .1
            .push(feature);
    }

    let mut output = Vec::new();

    for (layer_name, (extent, features)) in &by_layer {
        let owned_layer = build_owned_layer(layer_name, *extent, features);
        let mut layer = OwnedLayer::Tag01(owned_layer);
        layer
            .automatic_encoding_optimisation()
            .context("MLT encoding optimisation")?;
        layer
            .write_to(&mut output)
            .context("write MLT layer bytes")?;
    }

    Ok(output)
}

/// Build an `OwnedLayer01` from grouped `FeatureCollection` features.
fn build_owned_layer(name: &str, extent: u32, features: &[&Feature]) -> OwnedLayer01 {
    // Geometry
    let mut geom = DecodedGeometry::default();
    for f in features {
        geom.push_geom(&f.geometry);
    }

    // IDs
    let ids: Vec<Option<u64>> = features.iter().map(|f| f.id).collect();
    let has_any_id = ids.iter().any(Option::is_some);
    let decoded_id = if has_any_id {
        Some(DecodedId(ids))
    } else {
        None
    };

    // Properties: collect all unique property names (excluding _layer, _extent).
    let mut prop_names: Vec<String> = Vec::new();
    {
        let mut seen = std::collections::HashSet::new();
        for f in features {
            for key in f.properties.keys() {
                if key == "_layer" || key == "_extent" {
                    continue;
                }
                if seen.insert(key.clone()) {
                    prop_names.push(key.clone());
                }
            }
        }
    }

    let properties: Vec<OwnedProperty> = prop_names
        .iter()
        .map(|prop_name| {
            let prop_value = build_prop_value(prop_name, features);
            let decoded = DecodedProperty::from_parts(prop_name.clone(), prop_value);
            OwnedProperty::Decoded(decoded)
        })
        .collect();

    OwnedLayer01 {
        name: name.to_string(),
        extent,
        id: OwnedId::Decoded(decoded_id),
        geometry: OwnedGeometry::Decoded(geom),
        properties,
    }
}

/// Determine the best `PropValue` variant for a property across all features.
///
/// Inspects JSON values to decide if the property is string, integer, float, or bool,
/// then builds the corresponding typed vector.
fn build_prop_value(prop_name: &str, features: &[&Feature]) -> PropValue {
    // Detect the dominant type.
    let mut has_string = false;
    let mut has_int = false;
    let mut has_float = false;
    let mut has_bool = false;

    for f in features {
        match f.properties.get(prop_name) {
            Some(Value::String(_)) => has_string = true,
            Some(Value::Number(n)) => {
                if n.is_f64() && !n.is_i64() && !n.is_u64() {
                    has_float = true;
                } else {
                    has_int = true;
                }
            }
            Some(Value::Bool(_)) => has_bool = true,
            _ => {} // null or absent → None
        }
    }

    // If mixed types or strings, fall back to string encoding.
    let type_count =
        u32::from(has_string) + u32::from(has_int) + u32::from(has_float) + u32::from(has_bool);

    if has_string || type_count > 1 {
        // Encode everything as strings.
        let values: Vec<Option<String>> = features
            .iter()
            .map(|f| match f.properties.get(prop_name) {
                Some(Value::String(s)) => Some(s.clone()),
                Some(Value::Number(n)) => Some(n.to_string()),
                Some(Value::Bool(b)) => Some(b.to_string()),
                _ => None,
            })
            .collect();
        return PropValue::Str(values.into());
    }

    if has_bool {
        let values: Vec<Option<bool>> = features
            .iter()
            .map(|f| match f.properties.get(prop_name) {
                Some(Value::Bool(b)) => Some(*b),
                _ => None,
            })
            .collect();
        return PropValue::Bool(values);
    }

    if has_float {
        let values: Vec<Option<f64>> = features
            .iter()
            .map(|f| match f.properties.get(prop_name) {
                Some(Value::Number(n)) => n.as_f64(),
                _ => None,
            })
            .collect();
        return PropValue::F64(values);
    }

    if has_int {
        let values: Vec<Option<i64>> = features
            .iter()
            .map(|f| match f.properties.get(prop_name) {
                Some(Value::Number(n)) => n
                    .as_i64()
                    .or_else(|| n.as_u64().and_then(|u| i64::try_from(u).ok())),
                _ => None,
            })
            .collect();
        return PropValue::I64(values);
    }

    // All null/absent — encode as empty bool column.
    let values: Vec<Option<bool>> = vec![None; features.len()];
    PropValue::Bool(values)
}

#[cfg(test)]
mod tests {
    use prost::Message;

    use super::*;
    use crate::mvt;

    /// Build a minimal MVT tile, encode to bytes, convert to MLT, and verify
    /// the result can be parsed back by `mlt-core`.
    #[test]
    fn roundtrip_simple_tile() {
        let tile = mvt::Tile {
            layers: vec![mvt::tile::Layer {
                version: 2,
                name: "test".to_string(),
                features: vec![mvt::tile::Feature {
                    id: Some(1),
                    tags: vec![0, 0],
                    r#type: Some(mvt::tile::GeomType::Point.into()),
                    // MoveTo(1) at (10, 20) in MVT command encoding.
                    geometry: vec![
                        (1 << 3) | 1, // command: MoveTo, count=1
                        20,           // zigzag(10)
                        40,           // zigzag(20)
                    ],
                }],
                keys: vec!["name".to_string()],
                values: vec![mvt::tile::Value {
                    string_value: Some("hello".to_string()),
                    ..Default::default()
                }],
                extent: Some(4096),
            }],
        };

        let encoded = tile.encode_to_vec();
        let mlt_result = mvt_to_mlt(encoded).unwrap();

        // Verify we can parse the MLT output.
        assert!(!mlt_result.is_empty());
        let layers = mlt_core::parse_layers(&mlt_result).unwrap();
        assert_eq!(layers.len(), 1);
    }
}

//! Advisory-based MVT tile pruning.
//!
//! Applies a [`SourceAdvisory`] to a decoded MVT [`Tile`](crate::mvt::Tile),
//! removing unused layers, features, properties, and geometry types.

use std::collections::HashSet;

use serde_json::Value;

use crate::advisory::{GeometryType, SourceAdvisory, SourceLayerAdvisory, UnusedValues};
use crate::mvt;

/// Prune a decoded MVT tile according to the advisory for the given zoom level.
///
/// Mutates `tile` in place: removes unused layers, filters features by geometry type
/// and property values, and strips unused property keys from remaining features.
pub fn prune_tile(tile: &mut mvt::Tile, advisory: &SourceAdvisory, zoom: u8) {
    // Remove entirely unused source-layers.
    let unused_set: HashSet<&str> = advisory
        .unused_source_layers
        .iter()
        .map(String::as_str)
        .collect();
    tile.layers
        .retain(|l| !unused_set.contains(l.name.as_str()));

    // Apply per-layer advisories.
    for layer in &mut tile.layers {
        if let Some(layer_advisory) = advisory.layers.get(&layer.name) {
            prune_layer(layer, layer_advisory, zoom);
        }
    }

    // Remove layers that became empty after pruning.
    tile.layers.retain(|l| !l.features.is_empty());
}

fn prune_layer(layer: &mut mvt::tile::Layer, advisory: &SourceLayerAdvisory, zoom: u8) {
    // If this zoom is entirely unused, clear all features.
    if advisory.unused_zoom_levels.contains(&zoom) {
        layer.features.clear();
        return;
    }

    // Filter by geometry type.
    if !advisory.unused_geometry_types.is_empty() {
        let unused_geom: HashSet<i32> = advisory
            .unused_geometry_types
            .iter()
            .map(|gt| match gt {
                GeometryType::Point => mvt::tile::GeomType::Point.into(),
                GeometryType::LineString => mvt::tile::GeomType::Linestring.into(),
                GeometryType::Polygon => mvt::tile::GeomType::Polygon.into(),
            })
            .collect();
        layer
            .features
            .retain(|f| !unused_geom.contains(&f.r#type.unwrap_or(0)));
    }

    // Filter by unused property values.
    filter_by_property_values(layer, &advisory.unused_property_values);

    // Strip unused properties from remaining features.
    if !advisory.unused_properties.is_empty() {
        strip_unused_properties(layer, &advisory.unused_properties);
    }
}

/// Remove features that match specific unused property values.
fn filter_by_property_values(
    layer: &mut mvt::tile::Layer,
    unused_property_values: &std::collections::BTreeMap<String, UnusedValues>,
) {
    if unused_property_values.is_empty() {
        return;
    }

    // For each (prop_name, unused_values), build a set of (key_idx, value_idx) pairs to reject.
    let mut reject_checks: Vec<(u32, HashSet<u32>)> = Vec::new();

    for (prop_name, unused) in unused_property_values {
        let UnusedValues::Specific(unused_vals) = unused;

        let Some(key_idx) = layer.keys.iter().position(|k| k == prop_name) else {
            continue;
        };
        #[expect(clippy::cast_possible_truncation)]
        let key_idx = key_idx as u32;

        let mut bad_val_indices = HashSet::new();
        for val in unused_vals {
            for (vi, layer_val) in layer.values.iter().enumerate() {
                if mvt_value_matches_json(layer_val, val) {
                    #[expect(clippy::cast_possible_truncation)]
                    bad_val_indices.insert(vi as u32);
                }
            }
        }

        if !bad_val_indices.is_empty() {
            reject_checks.push((key_idx, bad_val_indices));
        }
    }

    if reject_checks.is_empty() {
        return;
    }

    layer.features.retain(|feature| {
        let tags = &feature.tags;
        let mut i = 0;
        while i + 1 < tags.len() {
            let ki = tags[i];
            let vi = tags[i + 1];
            for (reject_key, reject_vals) in &reject_checks {
                if ki == *reject_key && reject_vals.contains(&vi) {
                    return false;
                }
            }
            i += 2;
        }
        true
    });
}

/// Build a remapping table: given an array and a set of used indices,
/// produce a remap vector and the compacted array.
///
/// MVT tag indices are `u32`; layer key/value arrays are typically small,
/// so truncation from `usize` is safe in practice.
#[expect(clippy::cast_possible_truncation)]
fn build_remap<T: Clone>(items: &[T], used: &HashSet<u32>) -> (Vec<Option<u32>>, Vec<T>) {
    let mut remap = vec![None; items.len()];
    let mut new_items = Vec::new();
    for (old_idx, item) in items.iter().enumerate() {
        if used.contains(&(old_idx as u32)) {
            remap[old_idx] = Some(new_items.len() as u32);
            new_items.push(item.clone());
        }
    }
    (remap, new_items)
}

/// Strip unused property keys from all features, rebuilding `keys`, `values`, and `tags`.
fn strip_unused_properties(layer: &mut mvt::tile::Layer, unused_props: &[String]) {
    let unused_key_indices: HashSet<usize> = layer
        .keys
        .iter()
        .enumerate()
        .filter(|(_, k)| unused_props.contains(k))
        .map(|(i, _)| i)
        .collect();

    if unused_key_indices.is_empty() {
        return;
    }

    // Collect which key/value indices are still referenced after stripping.
    let mut used_keys: HashSet<u32> = HashSet::new();
    let mut used_values: HashSet<u32> = HashSet::new();

    for feature in &layer.features {
        let mut i = 0;
        while i + 1 < feature.tags.len() {
            let ki = feature.tags[i] as usize;
            if !unused_key_indices.contains(&ki) {
                used_keys.insert(feature.tags[i]);
                used_values.insert(feature.tags[i + 1]);
            }
            i += 2;
        }
    }

    let (key_remap, new_keys) = build_remap(&layer.keys, &used_keys);
    let (val_remap, new_values) = build_remap(&layer.values, &used_values);

    // Rewrite tags in all features.
    for feature in &mut layer.features {
        let mut new_tags = Vec::new();
        let mut i = 0;
        while i + 1 < feature.tags.len() {
            let ki = feature.tags[i] as usize;
            let vi = feature.tags[i + 1] as usize;
            if let (Some(new_ki), Some(new_vi)) = (
                key_remap.get(ki).copied().flatten(),
                val_remap.get(vi).copied().flatten(),
            ) {
                new_tags.push(new_ki);
                new_tags.push(new_vi);
            }
            i += 2;
        }
        feature.tags = new_tags;
    }

    layer.keys = new_keys;
    layer.values = new_values;
}

/// Check whether an MVT `Value` matches a JSON `Value`.
fn mvt_value_matches_json(mvt_val: &mvt::tile::Value, json_val: &Value) -> bool {
    match json_val {
        Value::String(s) => mvt_val.string_value.as_deref() == Some(s.as_str()),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                mvt_val.int_value == Some(i)
                    || mvt_val.sint_value == Some(i)
                    || (i >= 0 && mvt_val.uint_value == Some(i.cast_unsigned()))
            } else if let Some(u) = n.as_u64() {
                mvt_val.uint_value == Some(u)
            } else if let Some(f) = n.as_f64() {
                mvt_val.double_value == Some(f)
                    || mvt_val
                        .float_value
                        .is_some_and(|fv| (f64::from(fv) - f).abs() < f64::EPSILON)
            } else {
                false
            }
        }
        Value::Bool(b) => mvt_val.bool_value == Some(*b),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn make_string_value(s: &str) -> mvt::tile::Value {
        mvt::tile::Value {
            string_value: Some(s.to_string()),
            ..Default::default()
        }
    }

    fn make_test_tile() -> mvt::Tile {
        mvt::Tile {
            layers: vec![
                mvt::tile::Layer {
                    version: 2,
                    name: "roads".to_string(),
                    features: vec![
                        mvt::tile::Feature {
                            id: Some(1),
                            tags: vec![0, 0], // class=primary
                            r#type: Some(mvt::tile::GeomType::Linestring.into()),
                            geometry: vec![],
                        },
                        mvt::tile::Feature {
                            id: Some(2),
                            tags: vec![0, 1], // class=secondary
                            r#type: Some(mvt::tile::GeomType::Linestring.into()),
                            geometry: vec![],
                        },
                        mvt::tile::Feature {
                            id: Some(3),
                            tags: vec![0, 0, 1, 2], // class=primary, surface=paved
                            r#type: Some(mvt::tile::GeomType::Point.into()),
                            geometry: vec![],
                        },
                    ],
                    keys: vec!["class".to_string(), "surface".to_string()],
                    values: vec![
                        make_string_value("primary"),
                        make_string_value("secondary"),
                        make_string_value("paved"),
                    ],
                    extent: Some(4096),
                },
                mvt::tile::Layer {
                    version: 2,
                    name: "water".to_string(),
                    features: vec![mvt::tile::Feature {
                        id: Some(10),
                        tags: vec![],
                        r#type: Some(mvt::tile::GeomType::Polygon.into()),
                        geometry: vec![],
                    }],
                    keys: vec![],
                    values: vec![],
                    extent: Some(4096),
                },
                mvt::tile::Layer {
                    version: 2,
                    name: "poi".to_string(),
                    features: vec![mvt::tile::Feature {
                        id: Some(20),
                        tags: vec![],
                        r#type: Some(mvt::tile::GeomType::Point.into()),
                        geometry: vec![],
                    }],
                    keys: vec![],
                    values: vec![],
                    extent: Some(4096),
                },
            ],
        }
    }

    fn make_advisory() -> SourceAdvisory {
        SourceAdvisory {
            unused_source_layers: vec!["poi".to_string()],
            layers: BTreeMap::from([
                (
                    "roads".to_string(),
                    SourceLayerAdvisory {
                        unused_properties: vec!["surface".to_string()],
                        unused_geometry_types: vec![GeometryType::Point],
                        unused_zoom_levels: vec![0, 1, 2, 3],
                        unused_property_values: BTreeMap::from([(
                            "class".to_string(),
                            UnusedValues::Specific(vec![Value::String("secondary".to_string())]),
                        )]),
                    },
                ),
                (
                    "water".to_string(),
                    SourceLayerAdvisory {
                        unused_properties: vec![],
                        unused_geometry_types: vec![],
                        unused_zoom_levels: vec![],
                        unused_property_values: BTreeMap::new(),
                    },
                ),
            ]),
        }
    }

    #[test]
    fn removes_unused_source_layers() {
        let mut tile = make_test_tile();
        let advisory = make_advisory();
        prune_tile(&mut tile, &advisory, 10);

        let names: Vec<&str> = tile.layers.iter().map(|l| l.name.as_str()).collect();
        assert!(!names.contains(&"poi"));
        assert!(names.contains(&"roads"));
        assert!(names.contains(&"water"));
    }

    #[test]
    fn filters_geometry_types() {
        let mut tile = make_test_tile();
        let advisory = make_advisory();
        prune_tile(&mut tile, &advisory, 10);

        let roads = tile.layers.iter().find(|l| l.name == "roads").unwrap();
        // Point features should be removed, leaving only linestrings.
        // But feature id=2 (secondary) is also removed by value filter.
        // So only feature id=1 (primary, linestring) should remain.
        assert_eq!(roads.features.len(), 1);
        assert_eq!(roads.features[0].id, Some(1));
    }

    #[test]
    fn strips_unused_properties() {
        let mut tile = make_test_tile();
        let advisory = make_advisory();
        prune_tile(&mut tile, &advisory, 10);

        let roads = tile.layers.iter().find(|l| l.name == "roads").unwrap();
        assert!(!roads.keys.contains(&"surface".to_string()));
        // "class" should still be there.
        assert!(roads.keys.contains(&"class".to_string()));
    }

    #[test]
    fn clears_features_at_unused_zoom() {
        let mut tile = make_test_tile();
        let advisory = make_advisory();
        prune_tile(&mut tile, &advisory, 2);

        // Roads layer should have no features at zoom 2 (unused).
        // After pruning, empty layers are removed.
        let roads = tile.layers.iter().find(|l| l.name == "roads");
        assert!(
            roads.is_none(),
            "roads layer should be removed (empty at zoom 2)"
        );
    }

    #[test]
    fn filters_unused_property_values() {
        let mut tile = make_test_tile();
        // Use a simpler advisory that only filters property values, not geometry types.
        let advisory = SourceAdvisory {
            unused_source_layers: vec![],
            layers: BTreeMap::from([(
                "roads".to_string(),
                SourceLayerAdvisory {
                    unused_properties: vec![],
                    unused_geometry_types: vec![],
                    unused_zoom_levels: vec![],
                    unused_property_values: BTreeMap::from([(
                        "class".to_string(),
                        UnusedValues::Specific(vec![Value::String("secondary".to_string())]),
                    )]),
                },
            )]),
        };
        prune_tile(&mut tile, &advisory, 10);

        let roads = tile.layers.iter().find(|l| l.name == "roads").unwrap();
        // Feature id=2 (class=secondary) should be removed.
        let ids: Vec<u64> = roads.features.iter().filter_map(|f| f.id).collect();
        assert!(!ids.contains(&2));
        assert!(ids.contains(&1));
        assert!(ids.contains(&3));
    }
}

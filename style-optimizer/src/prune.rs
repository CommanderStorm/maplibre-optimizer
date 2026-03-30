//! Advisory-based MVT tile pruning.
//!
//! Applies a [`SourceAdvisory`] to a decoded MVT [`Tile`](crate::mvt::Tile),
//! removing unused layers, features, properties, and geometry types.

use std::collections::HashSet;

use maplibre_style_spec::spec::expressions::{
    Any, Boolean, ExprOrLiteral, String as StringExpr,
};
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

    // Filter by geometry type: keep only types that are used at this zoom.
    let allowed_geom: HashSet<i32> = advisory
        .used_geometry_types
        .iter()
        .filter(|(_, zr)| zr.contains(zoom))
        .map(|(gt, _)| geom_type_to_mvt(*gt))
        .collect();
    // Only filter if we have any geometry type info (empty map = no info, keep all).
    if !advisory.used_geometry_types.is_empty() {
        layer
            .features
            .retain(|f| allowed_geom.contains(&f.r#type.unwrap_or(0)));
    }

    // Filter by unused property values.
    filter_by_property_values(layer, &advisory.unused_property_values);

    // Reorder features by priority: features matching more style-layer filters come first.
    reorder_features_by_priority(layer, &advisory.layer_filters);

    // Strip feature IDs when no targeting layer uses them.
    if !advisory.feature_ids_needed {
        for feature in &mut layer.features {
            feature.id = None;
        }
    }

    // Strip properties that are either not in used_properties or outside their zoom range.
    let props_to_strip: Vec<String> = layer
        .keys
        .iter()
        .filter(|key| {
            !advisory
                .used_properties
                .get(key.as_str())
                .is_some_and(|zr| zr.contains(zoom))
        })
        .cloned()
        .collect();
    if !props_to_strip.is_empty() {
        strip_unused_properties(layer, &props_to_strip);
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

/// Replace string property values with frequency-ordered unsigned integers.
///
/// For each `(prop_name, value_table)` in `interned`, finds the property key in the layer,
/// maps each string value to its index in `value_table`, and replaces the MVT value entry
/// with a uint value. After all properties are processed, unused values are compacted.
pub fn intern_string_properties(
    layer: &mut mvt::tile::Layer,
    interned: &std::collections::BTreeMap<String, Vec<String>>,
) {
    if interned.is_empty() {
        return;
    }

    for (prop_name, value_table) in interned {
        let Some(key_idx) = layer.keys.iter().position(|k| k == prop_name) else {
            continue;
        };
        #[expect(clippy::cast_possible_truncation)]
        let key_idx = key_idx as u32;

        // Build string → uint mapping from the value table.
        let str_to_uint: std::collections::HashMap<&str, u64> = value_table
            .iter()
            .enumerate()
            .map(|(i, s)| (s.as_str(), i as u64))
            .collect();

        // Walk all features' tags and replace matching string values with uint values.
        for feature in &mut layer.features {
            let mut i = 0;
            while i + 1 < feature.tags.len() {
                if feature.tags[i] == key_idx {
                    let val_idx = feature.tags[i + 1] as usize;
                    if let Some(val) = layer.values.get(val_idx)
                        && let Some(s) = val.string_value.as_deref()
                        && let Some(&uint_val) = str_to_uint.get(s)
                    {
                        let new_vi = find_or_create_uint_value(&mut layer.values, uint_val);
                        feature.tags[i + 1] = new_vi;
                    }
                }
                i += 2;
            }
        }
    }

    // Compact unused values.
    let mut used_values: HashSet<u32> = HashSet::new();
    for feature in &layer.features {
        let mut i = 0;
        while i + 1 < feature.tags.len() {
            used_values.insert(feature.tags[i + 1]);
            i += 2;
        }
    }

    let (val_remap, new_values) = build_remap(&layer.values, &used_values);
    for feature in &mut layer.features {
        let mut i = 0;
        while i + 1 < feature.tags.len() {
            let vi = feature.tags[i + 1] as usize;
            if let Some(new_vi) = val_remap.get(vi).copied().flatten() {
                feature.tags[i + 1] = new_vi;
            }
            i += 2;
        }
    }
    layer.values = new_values;
}

/// Find an existing uint value entry or append one. Returns the index.
#[expect(clippy::cast_possible_truncation)]
fn find_or_create_uint_value(values: &mut Vec<mvt::tile::Value>, uint_val: u64) -> u32 {
    for (i, v) in values.iter().enumerate() {
        if v.uint_value == Some(uint_val)
            && v.string_value.is_none()
            && v.int_value.is_none()
            && v.sint_value.is_none()
            && v.float_value.is_none()
            && v.double_value.is_none()
            && v.bool_value.is_none()
        {
            return i as u32;
        }
    }
    let idx = values.len() as u32;
    values.push(mvt::tile::Value {
        uint_value: Some(uint_val),
        ..Default::default()
    });
    idx
}

fn geom_type_to_mvt(gt: GeometryType) -> i32 {
    match gt {
        GeometryType::Point => mvt::tile::GeomType::Point.into(),
        GeometryType::LineString => mvt::tile::GeomType::Linestring.into(),
        GeometryType::Polygon => mvt::tile::GeomType::Polygon.into(),
    }
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

// ── Feature priority reordering ──────────────────────────────────────────────

/// A resolved value from evaluating an expression against a feature.
enum Resolved<'a> {
    Null,
    Bool(bool),
    Number(f64),
    Str(&'a str),
}

impl Resolved<'_> {
    fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }
}

impl PartialEq for Resolved<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::Str(a), Self::Str(b)) => a == b,
            _ => false,
        }
    }
}

/// Look up a property value in an MVT feature's tag pairs.
fn lookup_property<'a>(
    prop_name: &str,
    feature: &mvt::tile::Feature,
    keys: &[String],
    values: &'a [mvt::tile::Value],
) -> Option<&'a mvt::tile::Value> {
    let mut i = 0;
    while i + 1 < feature.tags.len() {
        let ki = feature.tags[i] as usize;
        let vi = feature.tags[i + 1] as usize;
        if keys.get(ki).is_some_and(|k| k == prop_name) {
            return values.get(vi);
        }
        i += 2;
    }
    None
}

/// Convert an MVT value to a `Resolved`.
fn mvt_value_to_resolved(v: &mvt::tile::Value) -> Resolved<'_> {
    if let Some(s) = v.string_value.as_deref() {
        Resolved::Str(s)
    } else if let Some(b) = v.bool_value {
        Resolved::Bool(b)
    } else if let Some(n) = v.double_value {
        Resolved::Number(n)
    } else if let Some(n) = v.float_value {
        Resolved::Number(f64::from(n))
    } else if let Some(n) = v.int_value {
        #[expect(clippy::cast_precision_loss)]
        Resolved::Number(n as f64)
    } else if let Some(n) = v.uint_value {
        #[expect(clippy::cast_precision_loss)]
        Resolved::Number(n as f64)
    } else if let Some(n) = v.sint_value {
        #[expect(clippy::cast_precision_loss)]
        Resolved::Number(n as f64)
    } else {
        Resolved::Null
    }
}

/// Resolve an `ExprOrLiteral` to a comparable value using feature properties.
fn resolve_expr<'a>(
    expr: &'a ExprOrLiteral,
    feature: &mvt::tile::Feature,
    keys: &[String],
    values: &'a [mvt::tile::Value],
) -> Resolved<'a> {
    match expr {
        ExprOrLiteral::Bool(b) => Resolved::Bool(*b),
        ExprOrLiteral::NumberLiteral(n) => {
            Resolved::Number(n.as_f64().unwrap_or(f64::NAN))
        }
        ExprOrLiteral::StringLiteral(s) => Resolved::Str(s.as_str()),
        ExprOrLiteral::AnyExpr(any) => match any.as_ref() {
            Any::Get(prop, None) => {
                // Resolve the property name from the String expression.
                let prop_name = resolve_string_expr(prop);
                match prop_name {
                    Some(name) => lookup_property(name, feature, keys, values)
                        .map_or(Resolved::Null, mvt_value_to_resolved),
                    None => Resolved::Null,
                }
            }
            Any::Id => feature
                .id
                .map_or(Resolved::Null, |id| {
                    #[expect(clippy::cast_precision_loss)]
                    Resolved::Number(id as f64)
                }),
            _ => Resolved::Null,
        },
        ExprOrLiteral::StringExpr(s) => match s.as_ref() {
            StringExpr::GeometryType => {
                let gt = feature.r#type.unwrap_or(0);
                let name = match mvt::tile::GeomType::try_from(gt) {
                    Ok(mvt::tile::GeomType::Point) => "Point",
                    Ok(mvt::tile::GeomType::Linestring) => "LineString",
                    Ok(mvt::tile::GeomType::Polygon) => "Polygon",
                    _ => "Unknown",
                };
                Resolved::Str(name)
            }
            StringExpr::Literal(s) => Resolved::Str(s.as_str()),
            _ => Resolved::Null,
        },
        _ => Resolved::Null,
    }
}

/// Extract a literal string from a `String` expression (the property name in `["get", prop]`).
fn resolve_string_expr(expr: &StringExpr) -> Option<&str> {
    match expr {
        StringExpr::Literal(s) => Some(s.as_str()),
        _ => None,
    }
}

/// Check if a feature has a property by name.
fn feature_has_property(
    prop_name: &str,
    feature: &mvt::tile::Feature,
    keys: &[String],
) -> bool {
    let mut i = 0;
    while i + 1 < feature.tags.len() {
        let ki = feature.tags[i] as usize;
        if keys.get(ki).is_some_and(|k| k == prop_name) {
            return true;
        }
        i += 2;
    }
    false
}

/// Evaluate a typed `Boolean` filter expression against an MVT feature.
/// Returns `true` if the feature matches the filter.
fn eval_filter(
    filter: &Boolean,
    feature: &mvt::tile::Feature,
    keys: &[String],
    values: &[mvt::tile::Value],
) -> bool {
    match filter {
        Boolean::Literal(b) => *b,
        Boolean::Not(inner) => !eval_filter(inner, feature, keys, values),
        Boolean::All(conditions) => conditions
            .iter()
            .all(|c| eval_filter(c, feature, keys, values)),
        Boolean::Any(conditions) => conditions
            .iter()
            .any(|c| eval_filter(c, feature, keys, values)),
        Boolean::EqualEqual(a, b, _) => {
            let ra = resolve_expr(a, feature, keys, values);
            let rb = resolve_expr(b, feature, keys, values);
            ra == rb
        }
        Boolean::NotEqual(a, b, _) => {
            let ra = resolve_expr(a, feature, keys, values);
            let rb = resolve_expr(b, feature, keys, values);
            ra != rb
        }
        Boolean::Less(a, b, _) => {
            let ra = resolve_expr(a, feature, keys, values);
            let rb = resolve_expr(b, feature, keys, values);
            matches!((ra.as_f64(), rb.as_f64()), (Some(x), Some(y)) if x < y)
        }
        Boolean::LessEqual(a, b, _) => {
            let ra = resolve_expr(a, feature, keys, values);
            let rb = resolve_expr(b, feature, keys, values);
            matches!((ra.as_f64(), rb.as_f64()), (Some(x), Some(y)) if x <= y)
        }
        Boolean::Greater(a, b, _) => {
            let ra = resolve_expr(a, feature, keys, values);
            let rb = resolve_expr(b, feature, keys, values);
            matches!((ra.as_f64(), rb.as_f64()), (Some(x), Some(y)) if x > y)
        }
        Boolean::GreaterEqual(a, b, _) => {
            let ra = resolve_expr(a, feature, keys, values);
            let rb = resolve_expr(b, feature, keys, values);
            matches!((ra.as_f64(), rb.as_f64()), (Some(x), Some(y)) if x >= y)
        }
        Boolean::Has(prop, None) => {
            resolve_string_expr(prop)
                .is_some_and(|name| feature_has_property(name, feature, keys))
        }
        Boolean::In(needle, haystack) => {
            let rn = resolve_expr(needle, feature, keys, values);
            // If haystack is a literal array, check membership.
            if let ExprOrLiteral::AnyExpr(any) = haystack
                && let Any::Get(prop, None) = any.as_ref()
            {
                // ["in", needle, ["get", prop]] — check if needle is substring
                if let (Some(name), Resolved::Str(needle_str)) =
                    (resolve_string_expr(prop), &rn)
                {
                    return lookup_property(name, feature, keys, values)
                        .and_then(|v| v.string_value.as_deref())
                        .is_some_and(|s| s.contains(needle_str));
                }
            }
            // For literal arrays (not common in filter context), conservative false.
            false
        }
        Boolean::AnyExpr(any) => match any.as_ref() {
            Any::Case(arms) => {
                let (cases, fallback) = arms;
                for (cond, result) in cases {
                    if eval_filter(cond, feature, keys, values) {
                        return resolve_expr(result, feature, keys, values)
                            != Resolved::Null
                            && resolve_expr(result, feature, keys, values)
                                != Resolved::Bool(false);
                    }
                }
                let r = resolve_expr(fallback, feature, keys, values);
                r != Resolved::Null && r != Resolved::Bool(false)
            }
            _ => false, // Conservative: unsupported expression → no match.
        },
        // Unsupported variants → conservative false.
        _ => false,
    }
}

/// Count how many of the given filters a feature matches.
fn count_matching_filters(
    feature: &mvt::tile::Feature,
    keys: &[String],
    values: &[mvt::tile::Value],
    filters: &[Boolean],
) -> usize {
    filters
        .iter()
        .filter(|f| eval_filter(f, feature, keys, values))
        .count()
}

/// Reorder features within a layer by the number of style-layer filters they match.
/// Features matching more filters are placed first (higher priority).
/// Uses stable sort to preserve original order for ties.
fn reorder_features_by_priority(
    layer: &mut mvt::tile::Layer,
    layer_filters: &[Boolean],
) {
    if layer_filters.is_empty() {
        return;
    }

    // Pre-compute scores to avoid O(n*m) comparisons during sort.
    let scores: Vec<usize> = layer
        .features
        .iter()
        .map(|f| count_matching_filters(f, &layer.keys, &layer.values, layer_filters))
        .collect();

    // Build index array and sort by score descending.
    let mut indices: Vec<usize> = (0..layer.features.len()).collect();
    indices.sort_by(|&a, &b| scores[b].cmp(&scores[a]));

    // Apply the permutation.
    let old_features = std::mem::take(&mut layer.features);
    layer.features = indices.into_iter().map(|i| old_features[i].clone()).collect();
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::advisory::ZoomRange;

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
                        used_properties: BTreeMap::from([("class".to_string(), ZoomRange::All)]),
                        used_geometry_types: BTreeMap::from([(
                            GeometryType::LineString,
                            ZoomRange::All,
                        )]),
                        unused_zoom_levels: vec![0, 1, 2, 3],
                        unused_property_values: BTreeMap::from([(
                            "class".to_string(),
                            UnusedValues::Specific(vec![Value::String("secondary".to_string())]),
                        )]),
                        interned_properties: BTreeMap::new(),
                        feature_ids_needed: false,
                        combined_filter: None,
                        layer_filters: vec![],
                    },
                ),
                (
                    "water".to_string(),
                    SourceLayerAdvisory {
                        used_properties: BTreeMap::new(),
                        used_geometry_types: BTreeMap::new(),
                        unused_zoom_levels: vec![],
                        unused_property_values: BTreeMap::new(),
                        interned_properties: BTreeMap::new(),
                        feature_ids_needed: false,
                        combined_filter: None,
                        layer_filters: vec![],
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
        // So only the primary linestring feature should remain.
        assert_eq!(roads.features.len(), 1);
        assert_eq!(
            roads.features[0].r#type,
            Some(mvt::tile::GeomType::Linestring.into())
        );
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
                    used_properties: BTreeMap::from([
                        ("class".to_string(), ZoomRange::All),
                        ("surface".to_string(), ZoomRange::All),
                    ]),
                    used_geometry_types: BTreeMap::new(),
                    unused_zoom_levels: vec![],
                    unused_property_values: BTreeMap::from([(
                        "class".to_string(),
                        UnusedValues::Specific(vec![Value::String("secondary".to_string())]),
                    )]),
                    interned_properties: BTreeMap::new(),
                    feature_ids_needed: false,
                    combined_filter: None,
                    layer_filters: vec![],
                },
            )]),
        };
        prune_tile(&mut tile, &advisory, 10);

        let roads = tile.layers.iter().find(|l| l.name == "roads").unwrap();
        // Feature id=2 (class=secondary) should be removed, leaving 2 features.
        assert_eq!(roads.features.len(), 2);
        // IDs are stripped (feature_ids_needed: false), so check geometry types instead.
        // Remaining: primary linestring (id was 1) and primary+paved point (id was 3).
        let geom_types: Vec<_> = roads.features.iter().map(|f| f.r#type).collect();
        assert!(geom_types.contains(&Some(mvt::tile::GeomType::Linestring.into())));
        assert!(geom_types.contains(&Some(mvt::tile::GeomType::Point.into())));
    }

    #[test]
    fn per_zoom_property_stripping() {
        // Property "surface" has zoom range 10-16. At z14 it should be kept, at z5 stripped.
        let mut tile = make_test_tile();
        let advisory = SourceAdvisory {
            unused_source_layers: vec![],
            layers: BTreeMap::from([(
                "roads".to_string(),
                SourceLayerAdvisory {
                    used_properties: BTreeMap::from([
                        ("class".to_string(), ZoomRange::All),
                        ("surface".to_string(), ZoomRange::Range(10, 16)),
                    ]),
                    used_geometry_types: BTreeMap::new(),
                    unused_zoom_levels: vec![],
                    unused_property_values: BTreeMap::new(),
                    interned_properties: BTreeMap::new(),
                    feature_ids_needed: false,
                    combined_filter: None,
                    layer_filters: vec![],
                },
            )]),
        };

        // At z14 (inside range), surface should be kept.
        let mut tile_z14 = tile.clone();
        prune_tile(&mut tile_z14, &advisory, 14);
        let roads = tile_z14.layers.iter().find(|l| l.name == "roads").unwrap();
        assert!(roads.keys.contains(&"surface".to_string()));

        // At z5 (outside range), surface should be stripped.
        prune_tile(&mut tile, &advisory, 5);
        let roads = tile.layers.iter().find(|l| l.name == "roads").unwrap();
        assert!(!roads.keys.contains(&"surface".to_string()));
        // "class" should still be there (ZoomRange::All).
        assert!(roads.keys.contains(&"class".to_string()));
    }

    #[test]
    fn per_zoom_geometry_filtering() {
        // Point geometry type has zoom range 10-16. At z12 kept, at z5 stripped.
        let mut tile = make_test_tile();
        let advisory = SourceAdvisory {
            unused_source_layers: vec![],
            layers: BTreeMap::from([(
                "roads".to_string(),
                SourceLayerAdvisory {
                    used_properties: BTreeMap::new(),
                    used_geometry_types: BTreeMap::from([
                        (GeometryType::Point, ZoomRange::Range(10, 16)),
                        (GeometryType::LineString, ZoomRange::All),
                    ]),
                    unused_zoom_levels: vec![],
                    unused_property_values: BTreeMap::new(),
                    interned_properties: BTreeMap::new(),
                    feature_ids_needed: false,
                    combined_filter: None,
                    layer_filters: vec![],
                },
            )]),
        };

        // At z12 (inside range), Point features should be kept.
        let mut tile_z12 = tile.clone();
        prune_tile(&mut tile_z12, &advisory, 12);
        let roads = tile_z12.layers.iter().find(|l| l.name == "roads").unwrap();
        let has_point = roads
            .features
            .iter()
            .any(|f| f.r#type == Some(mvt::tile::GeomType::Point.into()));
        assert!(has_point, "Point features should be kept at z12");

        // At z5 (outside range), Point features should be removed.
        prune_tile(&mut tile, &advisory, 5);
        let roads = tile.layers.iter().find(|l| l.name == "roads").unwrap();
        let has_point = roads
            .features
            .iter()
            .any(|f| f.r#type == Some(mvt::tile::GeomType::Point.into()));
        assert!(!has_point, "Point features should be stripped at z5");
    }

    #[test]
    fn strips_feature_ids_when_not_needed() {
        let mut tile = make_test_tile();
        let advisory = make_advisory();
        // make_advisory sets feature_ids_needed: false for both layers.
        prune_tile(&mut tile, &advisory, 10);

        for layer in &tile.layers {
            for feature in &layer.features {
                assert_eq!(
                    feature.id, None,
                    "feature IDs should be stripped when not needed"
                );
            }
        }
    }

    #[test]
    fn preserves_feature_ids_when_needed() {
        let mut tile = make_test_tile();
        let mut advisory = make_advisory();
        // Mark roads as needing feature IDs.
        advisory.layers.get_mut("roads").unwrap().feature_ids_needed = true;

        prune_tile(&mut tile, &advisory, 10);

        let roads = tile.layers.iter().find(|l| l.name == "roads").unwrap();
        assert!(
            roads.features.iter().all(|f| f.id.is_some()),
            "feature IDs should be preserved when needed"
        );
    }

    #[test]
    fn intern_string_properties_replaces_values() {
        let mut layer = mvt::tile::Layer {
            version: 2,
            name: "roads".to_string(),
            features: vec![
                mvt::tile::Feature {
                    id: None,
                    tags: vec![0, 0], // class=primary
                    r#type: Some(mvt::tile::GeomType::Linestring.into()),
                    geometry: vec![],
                },
                mvt::tile::Feature {
                    id: None,
                    tags: vec![0, 1], // class=secondary
                    r#type: Some(mvt::tile::GeomType::Linestring.into()),
                    geometry: vec![],
                },
                mvt::tile::Feature {
                    id: None,
                    tags: vec![0, 0, 1, 2], // class=primary, surface=paved
                    r#type: Some(mvt::tile::GeomType::Linestring.into()),
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
        };

        let interned = BTreeMap::from([(
            "class".to_string(),
            vec!["primary".to_string(), "secondary".to_string()],
        )]);

        intern_string_properties(&mut layer, &interned);

        // "class" values should now be uint: primary=0, secondary=1.
        // "surface" values should be unchanged.
        for feature in &layer.features {
            let mut i = 0;
            while i + 1 < feature.tags.len() {
                let key = &layer.keys[feature.tags[i] as usize];
                let val = &layer.values[feature.tags[i + 1] as usize];
                if key == "class" {
                    assert!(
                        val.uint_value.is_some(),
                        "class value should be uint, got {val:?}"
                    );
                    assert!(val.string_value.is_none());
                } else if key == "surface" {
                    assert_eq!(
                        val.string_value.as_deref(),
                        Some("paved"),
                        "surface should remain a string"
                    );
                }
                i += 2;
            }
        }

        // Check specific interned values.
        let f0_class_val = &layer.values[layer.features[0].tags[1] as usize];
        assert_eq!(f0_class_val.uint_value, Some(0)); // "primary" → 0

        let f1_class_val = &layer.values[layer.features[1].tags[1] as usize];
        assert_eq!(f1_class_val.uint_value, Some(1)); // "secondary" → 1

        // Unused original string values for "primary"/"secondary" should be compacted away.
        let string_values: Vec<&str> = layer
            .values
            .iter()
            .filter_map(|v| v.string_value.as_deref())
            .collect();
        assert!(
            !string_values.contains(&"primary"),
            "original string 'primary' should be compacted"
        );
        assert!(
            !string_values.contains(&"secondary"),
            "original string 'secondary' should be compacted"
        );
        // "paved" should still exist since surface wasn't interned.
        assert!(
            string_values.contains(&"paved"),
            "'paved' should remain in values"
        );
    }

    // ── Feature priority reordering tests ────────────────────────────────────

    /// Helper: parse a JSON filter expression into a typed `Boolean`.
    fn parse_filter(json: Value) -> Boolean {
        serde_json::from_value(json).expect("valid filter expression")
    }

    /// Helper: build a tile with a single source-layer containing features with string properties.
    fn make_reorder_tile() -> mvt::Tile {
        // 3 features: class=road, class=rail, class=path
        mvt::Tile {
            layers: vec![mvt::tile::Layer {
                version: 2,
                name: "transport".to_string(),
                features: vec![
                    mvt::tile::Feature {
                        id: Some(1),
                        tags: vec![0, 0], // class=road
                        r#type: Some(mvt::tile::GeomType::Linestring.into()),
                        geometry: vec![],
                    },
                    mvt::tile::Feature {
                        id: Some(2),
                        tags: vec![0, 1], // class=rail
                        r#type: Some(mvt::tile::GeomType::Linestring.into()),
                        geometry: vec![],
                    },
                    mvt::tile::Feature {
                        id: Some(3),
                        tags: vec![0, 2], // class=path
                        r#type: Some(mvt::tile::GeomType::Linestring.into()),
                        geometry: vec![],
                    },
                ],
                keys: vec!["class".to_string()],
                values: vec![
                    make_string_value("road"),
                    make_string_value("rail"),
                    make_string_value("path"),
                ],
                extent: Some(4096),
            }],
        }
    }

    #[test]
    fn reorders_features_by_filter_match_count() {
        let mut tile = make_reorder_tile();

        // Filter 1: class == "road"
        let f1 = parse_filter(serde_json::json!(["==", ["get", "class"], "road"]));
        // Filter 2: class == "rail"
        let f2 = parse_filter(serde_json::json!(["==", ["get", "class"], "rail"]));
        // Filter 3: class == "road" OR class == "rail"
        let f3 = parse_filter(serde_json::json!([
            "any",
            ["==", ["get", "class"], "road"],
            ["==", ["get", "class"], "rail"]
        ]));

        let advisory = SourceAdvisory {
            unused_source_layers: vec![],
            layers: BTreeMap::from([(
                "transport".to_string(),
                SourceLayerAdvisory {
                    used_properties: BTreeMap::from([("class".to_string(), ZoomRange::All)]),
                    used_geometry_types: BTreeMap::new(),
                    unused_zoom_levels: vec![],
                    unused_property_values: BTreeMap::new(),
                    interned_properties: BTreeMap::new(),
                    feature_ids_needed: true,
                    combined_filter: None,
                    layer_filters: vec![f1, f2, f3],
                },
            )]),
        };

        prune_tile(&mut tile, &advisory, 10);

        let transport = tile.layers.iter().find(|l| l.name == "transport").unwrap();
        let ids: Vec<u64> = transport
            .features
            .iter()
            .map(|f| f.id.unwrap())
            .collect();

        // road matches f1 + f3 = 2, rail matches f2 + f3 = 2, path matches 0.
        // road and rail tied at 2, so they keep original order. path last.
        assert_eq!(ids, vec![1, 2, 3]);

        // Verify path (id=3) is last — it matches 0 filters.
        assert_eq!(ids[2], 3);
        // Verify road and rail are before path.
        assert!(ids[0] == 1 || ids[0] == 2);
        assert!(ids[1] == 1 || ids[1] == 2);
    }

    #[test]
    fn preserves_order_for_equal_scores() {
        let mut tile = make_reorder_tile();

        // All features match the same number of filters (1 each).
        // Original order should be preserved.
        let f1 = parse_filter(serde_json::json!(["has", "class"]));

        let advisory = SourceAdvisory {
            unused_source_layers: vec![],
            layers: BTreeMap::from([(
                "transport".to_string(),
                SourceLayerAdvisory {
                    used_properties: BTreeMap::from([("class".to_string(), ZoomRange::All)]),
                    used_geometry_types: BTreeMap::new(),
                    unused_zoom_levels: vec![],
                    unused_property_values: BTreeMap::new(),
                    interned_properties: BTreeMap::new(),
                    feature_ids_needed: true,
                    combined_filter: None,
                    layer_filters: vec![f1],
                },
            )]),
        };

        prune_tile(&mut tile, &advisory, 10);

        let transport = tile.layers.iter().find(|l| l.name == "transport").unwrap();
        let ids: Vec<u64> = transport
            .features
            .iter()
            .map(|f| f.id.unwrap())
            .collect();
        // All match equally, so original order preserved.
        assert_eq!(ids, vec![1, 2, 3]);
    }

    #[test]
    fn no_reorder_when_no_filters() {
        let mut tile = make_reorder_tile();

        let advisory = SourceAdvisory {
            unused_source_layers: vec![],
            layers: BTreeMap::from([(
                "transport".to_string(),
                SourceLayerAdvisory {
                    used_properties: BTreeMap::from([("class".to_string(), ZoomRange::All)]),
                    used_geometry_types: BTreeMap::new(),
                    unused_zoom_levels: vec![],
                    unused_property_values: BTreeMap::new(),
                    interned_properties: BTreeMap::new(),
                    feature_ids_needed: true,
                    combined_filter: None,
                    layer_filters: vec![],
                },
            )]),
        };

        prune_tile(&mut tile, &advisory, 10);

        let transport = tile.layers.iter().find(|l| l.name == "transport").unwrap();
        let ids: Vec<u64> = transport
            .features
            .iter()
            .map(|f| f.id.unwrap())
            .collect();
        // No filters → no reordering.
        assert_eq!(ids, vec![1, 2, 3]);
    }

    #[test]
    fn geometry_type_filter_evaluation() {
        // Tile with mixed geometry types.
        let mut tile = mvt::Tile {
            layers: vec![mvt::tile::Layer {
                version: 2,
                name: "mixed".to_string(),
                features: vec![
                    mvt::tile::Feature {
                        id: Some(1),
                        tags: vec![],
                        r#type: Some(mvt::tile::GeomType::Linestring.into()),
                        geometry: vec![],
                    },
                    mvt::tile::Feature {
                        id: Some(2),
                        tags: vec![],
                        r#type: Some(mvt::tile::GeomType::Point.into()),
                        geometry: vec![],
                    },
                ],
                keys: vec![],
                values: vec![],
                extent: Some(4096),
            }],
        };

        // Filter: geometry-type == "Point"
        let f1 = parse_filter(serde_json::json!(["==", ["geometry-type"], "Point"]));

        let advisory = SourceAdvisory {
            unused_source_layers: vec![],
            layers: BTreeMap::from([(
                "mixed".to_string(),
                SourceLayerAdvisory {
                    used_properties: BTreeMap::new(),
                    used_geometry_types: BTreeMap::new(),
                    unused_zoom_levels: vec![],
                    unused_property_values: BTreeMap::new(),
                    interned_properties: BTreeMap::new(),
                    feature_ids_needed: true,
                    combined_filter: None,
                    layer_filters: vec![f1],
                },
            )]),
        };

        prune_tile(&mut tile, &advisory, 10);

        let mixed = tile.layers.iter().find(|l| l.name == "mixed").unwrap();
        let ids: Vec<u64> = mixed.features.iter().map(|f| f.id.unwrap()).collect();
        // Point (id=2) matches 1 filter, LineString (id=1) matches 0.
        // So Point should come first.
        assert_eq!(ids, vec![2, 1]);
    }

}

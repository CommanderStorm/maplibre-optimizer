//! Shared utilities for layer-index building and source collection.
//!
//! Used by both `dead` (dead-code elimination) and `strip` (cleanup pass).

use std::collections::{HashMap, HashSet};

use serde_json::Value;

/// Build a map from layer `id` → index in the layers array.
pub(super) fn build_layer_index(layers: &[Value]) -> HashMap<String, usize> {
    let mut m = HashMap::new();
    for (i, layer) in layers.iter().enumerate() {
        if let Some(id) = layer
            .as_object()
            .and_then(|o| o.get("id"))
            .and_then(Value::as_str)
        {
            m.insert(id.to_string(), i);
        }
    }
    m
}

/// Follow `ref` chains to find the source name for a layer. Returns `None` on cycles.
pub(super) fn resolve_source_for_layer(
    idx: usize,
    layers: &[Value],
    by_id: &HashMap<String, usize>,
    visited: &mut HashSet<usize>,
) -> Option<String> {
    if !visited.insert(idx) {
        return None;
    }
    let obj = layers.get(idx)?.as_object()?;
    if let Some(r) = obj.get("ref").and_then(Value::as_str) {
        return resolve_source_for_layer(*by_id.get(r)?, layers, by_id, visited);
    }
    obj.get("source")
        .and_then(Value::as_str)
        .map(str::to_string)
}

/// Follow `ref` chains to find the effective layer type. Returns `None` on cycles.
pub(super) fn layer_type_str(
    start: usize,
    layers: &[Value],
    by_id: &HashMap<String, usize>,
) -> Option<String> {
    let mut current = start;
    let mut visited = HashSet::new();
    loop {
        if !visited.insert(current) {
            return None; // cycle
        }
        let obj = layers.get(current)?.as_object()?;
        if let Some(t) = obj.get("type").and_then(Value::as_str) {
            return Some(t.to_string());
        }
        let r = obj.get("ref").and_then(Value::as_str)?;
        current = *by_id.get(r)?;
    }
}

/// Collect the set of source IDs referenced by surviving layers.
pub(super) fn collect_used_sources(
    layers: &[Value],
    by_id: &HashMap<String, usize>,
) -> HashSet<String> {
    let mut used = HashSet::new();
    for i in 0..layers.len() {
        let Some(ty) = layer_type_str(i, layers, by_id) else {
            continue;
        };
        if ty == "background" {
            continue;
        }
        let mut visited = HashSet::new();
        if let Some(src) = resolve_source_for_layer(i, layers, by_id, &mut visited) {
            used.insert(src);
        }
    }
    used
}

/// Check if a source is a `"vector"` type by looking it up in the style root.
pub(super) fn is_vector_source(style_root: &Value, source_name: &str) -> bool {
    style_root
        .as_object()
        .and_then(|r| r.get("sources"))
        .and_then(Value::as_object)
        .and_then(|s| s.get(source_name))
        .and_then(Value::as_object)
        .and_then(|s| s.get("type"))
        .and_then(Value::as_str)
        == Some("vector")
}

/// Extract `(source, source-layer)` from a layer object.
pub(super) fn layer_source_info(layer: &Value) -> Option<(&str, &str)> {
    let obj = layer.as_object()?;
    let source = obj.get("source")?.as_str()?;
    let source_layer = obj.get("source-layer")?.as_str()?;
    Some((source, source_layer))
}

/// Information about a layer's vector source, pre-computed for use by visitors.
#[derive(Clone, Debug)]
pub(super) struct VectorLayerInfo {
    pub source: String,
    pub source_layer: String,
}

/// Pre-compute vector layer info for all layers in the style.
/// Returns a vec indexed by layer position; `None` if the layer doesn't have a vector source.
pub(super) fn precompute_vector_layer_info(style: &Value) -> Vec<Option<VectorLayerInfo>> {
    let Some(root) = style.as_object() else {
        return vec![];
    };
    let Some(layers) = root.get("layers").and_then(Value::as_array) else {
        return vec![];
    };

    layers
        .iter()
        .map(|layer| {
            let (source, source_layer) = layer_source_info(layer)?;
            if !is_vector_source(style, source) {
                return None;
            }
            Some(VectorLayerInfo {
                source: source.to_string(),
                source_layer: source_layer.to_string(),
            })
        })
        .collect()
}

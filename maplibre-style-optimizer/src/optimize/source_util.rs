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

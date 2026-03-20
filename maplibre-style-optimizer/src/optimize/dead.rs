//! Remove unreferenced sources and layers whose filter is always false.

use std::collections::{HashMap, HashSet};

use serde_json::Value;

use super::expr::bool_literal;
use super::walk::StyleVisitor;

// ── Visitor ───────────────────────────────────────────────────────────────────

pub(crate) struct DeadEliminationVisitor;

impl StyleVisitor for DeadEliminationVisitor {
    fn visit_root(&mut self, root: &mut Value) {
        eliminate_dead_sources_and_layers(root);
    }
}

// ── Implementation ────────────────────────────────────────────────────────────

fn eliminate_dead_sources_and_layers(v: &mut Value) {
    let Some(root) = v.as_object_mut() else {
        return;
    };

    let Some(layers_val) = root.get_mut("layers") else {
        return;
    };
    let Some(layers) = layers_val.as_array_mut() else {
        return;
    };

    let mut to_drop: Vec<usize> = Vec::new();
    for (i, layer) in layers.iter().enumerate() {
        let Some(obj) = layer.as_object() else {
            continue;
        };
        if let Some(filt) = obj.get("filter")
            && filter_is_always_false(filt)
        {
            to_drop.push(i);
        }
        // Layers that never render (opacity 0 everywhere) are not implemented here.
    }

    if !to_drop.is_empty() {
        for i in to_drop.into_iter().rev() {
            layers.remove(i);
        }
    }

    let used = collect_used_sources(layers, &build_layer_index(layers));
    let Some(sources_val) = root.get_mut("sources") else {
        return;
    };
    let Some(sources) = sources_val.as_object_mut() else {
        return;
    };
    sources.retain(|id, _| used.contains(id));
}

fn build_layer_index(layers: &[Value]) -> HashMap<String, usize> {
    let mut m = HashMap::new();
    for (i, layer) in layers.iter().enumerate() {
        if let Some(obj) = layer.as_object()
            && let Some(id) = obj.get("id").and_then(|idx| idx.as_str())
        {
            m.insert(id.to_string(), i);
        }
    }
    m
}

fn resolve_source_for_layer(
    layer_idx: usize,
    layers: &[Value],
    by_id: &HashMap<String, usize>,
    visited: &mut HashSet<usize>,
) -> Option<String> {
    if !visited.insert(layer_idx) {
        return None;
    }
    let obj = layers.get(layer_idx)?.as_object()?;
    if let Some(r) = obj.get("ref").and_then(|v| v.as_str()) {
        let parent_idx = *by_id.get(r)?;
        return resolve_source_for_layer(parent_idx, layers, by_id, visited);
    }
    obj.get("source")
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

fn layer_type_str(
    layer_idx: usize,
    layers: &[Value],
    by_id: &HashMap<String, usize>,
) -> Option<String> {
    let obj = layers.get(layer_idx)?.as_object()?;
    if let Some(t) = obj.get("type").and_then(|v| v.as_str()) {
        return Some(t.to_string());
    }
    if let Some(r) = obj.get("ref").and_then(|v| v.as_str()) {
        let parent_idx = *by_id.get(r)?;
        return layer_type_str(parent_idx, layers, by_id);
    }
    None
}

fn collect_used_sources(layers: &[Value], by_id: &HashMap<String, usize>) -> HashSet<String> {
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

fn filter_is_always_false(f: &Value) -> bool {
    bool_literal(f) == Some(false)
}

//! Remove unreferenced sources and layers whose filter is always false.

use serde_json::Value;

use super::expr::bool_literal;
use super::source_util::{build_layer_index, collect_used_sources};
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
    sources.retain(|id, _| used.contains(id.as_str()));
}

fn filter_is_always_false(f: &Value) -> bool {
    bool_literal(f) == Some(false)
}

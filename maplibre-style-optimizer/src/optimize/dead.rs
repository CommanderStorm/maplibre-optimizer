//! Remove unreferenced sources and layers whose filter is always false.

use serde_json::Value;

use super::expr::bool_literal;
use super::source_util::{VectorLayerInfo, build_layer_index, collect_used_sources};
use super::walk::StyleVisitor;
use crate::stats::TileStatistics;

// ── Visitor ───────────────────────────────────────────────────────────────────

pub(crate) struct DeadEliminationVisitor<'a> {
    pub stats: Option<&'a TileStatistics>,
    pub layer_info: Option<&'a [Option<VectorLayerInfo>]>,
}

impl StyleVisitor for DeadEliminationVisitor<'_> {
    fn visit_root(&mut self, root: &mut Value) {
        eliminate_dead_sources_and_layers(root, self.stats, self.layer_info);
    }
}

// ── Implementation ────────────────────────────────────────────────────────────

fn eliminate_dead_sources_and_layers(
    v: &mut Value,
    stats: Option<&TileStatistics>,
    layer_info: Option<&[Option<VectorLayerInfo>]>,
) {
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
        // Original: filter is always false.
        if let Some(filt) = obj.get("filter")
            && filter_is_always_false(filt)
        {
            to_drop.push(i);
            continue;
        }
        // Stats-driven: geometry type mismatch.
        if let Some(stats) = stats
            && is_dead_by_geometry(i, layer, stats, layer_info)
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

/// Check if a layer is dead because its geometry type requirement cannot be satisfied
/// by the source-layer's actual geometry types.
fn is_dead_by_geometry(
    layer_index: usize,
    layer: &Value,
    stats: &TileStatistics,
    layer_info: Option<&[Option<VectorLayerInfo>]>,
) -> bool {
    let Some(infos) = layer_info else {
        return false;
    };
    let Some(Some(info)) = infos.get(layer_index) else {
        return false;
    };
    let Some(layer_stats) = stats.layer_stats(&info.source, &info.source_layer) else {
        return false;
    };
    let layer_type = layer
        .as_object()
        .and_then(|o| o.get("type"))
        .and_then(Value::as_str)
        .unwrap_or("");

    let gt = &layer_stats.geometry_types;
    match layer_type {
        "fill" | "fill-extrusion" => gt.polygon == 0,
        "circle" | "heatmap" => gt.point == 0,
        "line" => gt.linestring == 0 && gt.polygon == 0,
        "symbol" => gt.point == 0 && gt.linestring == 0,
        _ => false,
    }
}

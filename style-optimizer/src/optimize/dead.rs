//! Dead elimination pass operating on typed `MaplibreStyleSpecification`.

use std::collections::{HashMap, HashSet};

use maplibre_style_spec::spec::{AnyLayer, MaplibreStyleSpecification, TypedLayer};

use super::source_util::VectorLayerInfo;
use crate::stats::TileStatistics;

/// Remove layers with always-false filters and geometry-type mismatches,
/// then prune unused sources.
pub(crate) fn dead_elimination(
    style: &mut MaplibreStyleSpecification,
    stats: Option<&TileStatistics>,
    layer_info: Option<&[Option<VectorLayerInfo>]>,
) {
    let mut to_drop: Vec<usize> = Vec::new();
    for (i, layer) in style.layers.iter().enumerate() {
        if let AnyLayer::Typed(t) = layer {
            // Filter is always false.
            if let Some(ref filter) = t.common().filter
                && filter.is_always_false()
            {
                to_drop.push(i);
                continue;
            }
            // Stats-driven: geometry type mismatch.
            if let Some(stats) = stats
                && is_dead_by_geometry(i, t, stats, layer_info)
            {
                to_drop.push(i);
            }
        }
    }

    if !to_drop.is_empty() {
        for i in to_drop.into_iter().rev() {
            style.layers.remove(i);
        }
    }

    // Prune unused sources.
    let used = collect_used_sources(&style.layers);
    prune_sources(style, &used);
}

/// Prune sources not referenced by any layer.
pub(super) fn prune_sources(style: &mut MaplibreStyleSpecification, used: &HashSet<String>) {
    style.sources.0.retain(|id, _| used.contains(id.as_str()));
}

fn is_dead_by_geometry(
    layer_index: usize,
    layer: &TypedLayer,
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

    let gt = &layer_stats.geometry_types;
    match layer.layer_type() {
        "fill" | "fill-extrusion" => gt.polygon == 0,
        "circle" | "heatmap" => gt.point == 0,
        "line" => gt.linestring == 0 && gt.polygon == 0,
        "symbol" => gt.point == 0 && gt.linestring == 0,
        _ => false,
    }
}

pub(super) fn collect_used_sources(layers: &[AnyLayer]) -> HashSet<String> {
    let by_id: HashMap<&str, usize> = layers
        .iter()
        .enumerate()
        .map(|(i, l)| (l.id().as_str(), i))
        .collect();

    let mut used = HashSet::new();
    for layer in layers {
        match layer {
            AnyLayer::Typed(t) => {
                if t.layer_type() == "background" {
                    continue;
                }
                if let Some(src) = t.common().source.as_ref() {
                    used.insert(src.as_str().to_string());
                }
            }
            AnyLayer::Ref(r) => {
                // Follow ref chain to find source.
                if let Some(source) = resolve_ref_source(&r.r#ref, layers, &by_id) {
                    used.insert(source);
                }
            }
        }
    }
    used
}

fn resolve_ref_source(
    ref_id: &str,
    layers: &[AnyLayer],
    by_id: &HashMap<&str, usize>,
) -> Option<String> {
    let mut visited = HashSet::new();
    let mut current_ref = ref_id;
    loop {
        if !visited.insert(current_ref) {
            return None; // cycle
        }
        let idx = by_id.get(current_ref)?;
        match &layers[*idx] {
            AnyLayer::Typed(t) => {
                return t.common().source.as_ref().map(|s| s.as_str().to_string());
            }
            AnyLayer::Ref(r) => {
                current_ref = &r.r#ref;
            }
        }
    }
}

//! Typed optimization passes operating directly on `MaplibreStyleSpecification`.
//!
//! These passes use typed layer access instead of JSON string-matching,
//! providing compile-time safety for structural layer operations.

use std::collections::{HashMap, HashSet};

use maplibre_style_spec::spec::{AnyLayer, MaplibreStyleSpecification, TypedLayer};
use serde_json::Value;

use super::expr::bool_literal;
use super::source_util::VectorLayerInfo;
use crate::stats::TileStatistics;

// ── Strip Metadata (typed) ──────────────────────────────────────────────────

/// Remove `metadata` from the root and all layers.
pub(crate) fn strip_metadata_typed(style: &mut MaplibreStyleSpecification) {
    style.metadata = None;

    for layer in &mut style.layers {
        match layer {
            AnyLayer::Typed(t) => t.common_mut().metadata = None,
            AnyLayer::Ref(r) => r.metadata = None,
        }
    }
}

// ── Dead Elimination (typed) ─────────────────────────────────────────────────

/// Remove layers with always-false filters and geometry-type mismatches,
/// then prune unused sources.
pub(crate) fn dead_elimination_typed(
    style: &mut MaplibreStyleSpecification,
    stats: Option<&TileStatistics>,
    layer_info: Option<&[Option<VectorLayerInfo>]>,
) {
    let mut to_drop: Vec<usize> = Vec::new();
    for (i, layer) in style.layers.iter().enumerate() {
        if let AnyLayer::Typed(t) = layer {
            // Filter is always false.
            if let Some(ref filter) = t.common().filter {
                if filter_is_always_false(filter.as_value()) {
                    to_drop.push(i);
                    continue;
                }
            }
            // Stats-driven: geometry type mismatch.
            if let Some(stats) = stats {
                if is_dead_by_geometry_typed(i, t, stats, layer_info) {
                    to_drop.push(i);
                }
            }
        }
    }

    if !to_drop.is_empty() {
        for i in to_drop.into_iter().rev() {
            style.layers.remove(i);
        }
    }

    // Prune unused sources.
    let used = collect_used_sources_typed(&style.layers);
    prune_sources(style, &used);
}

fn filter_is_always_false(f: &Value) -> bool {
    bool_literal(f) == Some(false)
}

/// Prune unused sources by roundtripping through JSON.
fn prune_sources(style: &mut MaplibreStyleSpecification, used: &HashSet<String>) {
    if let Ok(mut sources_val) = serde_json::to_value(&style.sources) {
        if let Some(obj) = sources_val.as_object_mut() {
            obj.retain(|id, _| used.contains(id.as_str()));
        }
        if let Ok(new_sources) = serde_json::from_value(sources_val) {
            style.sources = new_sources;
        }
    }
}

fn is_dead_by_geometry_typed(
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

fn collect_used_sources_typed(layers: &[AnyLayer]) -> HashSet<String> {
    let by_id: HashMap<&str, usize> = layers
        .iter()
        .enumerate()
        .filter_map(|(i, l)| Some((l.id().as_str(), i)))
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

// ── Metadata Refinement (typed) ──────────────────────────────────────────────

/// Extract zoom bounds from filters and tighten minzoom/maxzoom.
pub(crate) fn metadata_refinement_typed(
    style: &mut MaplibreStyleSpecification,
    stats: Option<&TileStatistics>,
    layer_info: Option<&[Option<VectorLayerInfo>]>,
) {
    for (i, layer) in style.layers.iter_mut().enumerate() {
        let AnyLayer::Typed(typed) = layer else {
            continue;
        };
        refine_typed_layer(typed, i, stats, layer_info);
    }
}

fn refine_typed_layer(
    layer: &mut TypedLayer,
    layer_index: usize,
    stats: Option<&TileStatistics>,
    layer_info: Option<&[Option<VectorLayerInfo>]>,
) {
    let common = layer.common_mut();

    // Filter-based zoom extraction.
    if let Some(ref filter) = common.filter {
        let filter_val = filter.as_value();
        let (lb_raw, ub_raw) = super::metadata::zoom_bounds_from_expression(filter_val);
        let lb = lb_raw.map(|n| n.ceil());
        let ub = ub_raw.map(|n| n.floor());

        if let Some(bound) = lb {
            let cur = common.minzoom.as_ref().and_then(|m| m.as_f64());
            match cur {
                Some(c) if bound > c => {
                    common.minzoom = maplibre_style_spec::spec::LayerMinzoom::from_f64(bound);
                }
                None => {
                    common.minzoom = maplibre_style_spec::spec::LayerMinzoom::from_f64(bound);
                }
                _ => {}
            }
        }

        if let Some(bound) = ub {
            let cur = common.maxzoom.as_ref().and_then(|m| m.as_f64());
            let new_max = match cur {
                Some(c) => c.min(bound),
                None => bound,
            };
            common.maxzoom = maplibre_style_spec::spec::LayerMaxzoom::from_f64(new_max);
        }

        // Remove consumed zoom predicates from filter.
        let adopted_min = common.minzoom.as_ref().and_then(|m| m.as_f64());
        let adopted_max = common.maxzoom.as_ref().and_then(|m| m.as_f64());
        if adopted_min.is_some() || adopted_max.is_some() {
            if let Some(ref mut filter) = common.filter {
                super::metadata::remove_consumed_zoom_predicates(
                    filter.as_value_mut(),
                    adopted_min,
                    adopted_max,
                );
            }
        }
    }

    // Stats-driven zoom tightening.
    if let Some(stats) = stats
        && let Some(infos) = layer_info
        && let Some(Some(info)) = infos.get(layer_index)
        && let Some(layer_stats) = stats.layer_stats(&info.source, &info.source_layer)
        && !layer_stats.features_by_zoom.is_empty()
    {
        let common = layer.common_mut();
        let data_min = f64::from(*layer_stats.features_by_zoom.keys().next().unwrap());
        let data_max = f64::from(*layer_stats.features_by_zoom.keys().next_back().unwrap());

        let cur_min = common.minzoom.as_ref().and_then(|m| m.as_f64());
        if cur_min.is_none_or(|c| data_min > c) {
            common.minzoom = maplibre_style_spec::spec::LayerMinzoom::from_f64(data_min);
        }

        let cur_max = common.maxzoom.as_ref().and_then(|m| m.as_f64());
        if cur_max.is_none_or(|c| data_max < c) {
            common.maxzoom = maplibre_style_spec::spec::LayerMaxzoom::from_f64(data_max);
        }
    }
}

// ── Cleanup (typed) ──────────────────────────────────────────────────────────

/// Remove empty paint/layout, visibility:none layers, zero-opacity layers.
pub(crate) fn cleanup_typed(style: &mut MaplibreStyleSpecification) {
    // Collect referenced layer IDs so we never remove ref targets.
    let referenced_ids: HashSet<String> = style
        .layers
        .iter()
        .filter_map(|l| {
            if let AnyLayer::Ref(r) = l {
                Some(r.r#ref.clone())
            } else {
                None
            }
        })
        .collect();

    // Remove empty paint/layout on each layer (via JSON roundtrip on paint/layout).
    for layer in &mut style.layers {
        if let AnyLayer::Typed(t) = layer {
            cleanup_empty_paint_layout(t);
        }
    }

    // Remove invisible layers.
    let to_remove: Vec<usize> = style
        .layers
        .iter()
        .enumerate()
        .filter_map(|(i, layer)| {
            let id = layer.id().as_str();
            if referenced_ids.contains(id) {
                return None; // never remove a ref target
            }
            if is_invisible_typed(layer) {
                Some(i)
            } else {
                None
            }
        })
        .collect();

    if !to_remove.is_empty() {
        for i in to_remove.into_iter().rev() {
            style.layers.remove(i);
        }

        // Prune unused sources.
        let used = collect_used_sources_typed(&style.layers);
        prune_sources(style, &used);
    }
}

/// Remove empty paint/layout objects from a typed layer by checking if they're default.
fn cleanup_empty_paint_layout(layer: &mut TypedLayer) {
    // We check if the paint/layout, when serialized, would produce an empty JSON object.
    // This is simpler than matching every possible paint/layout struct.
    match layer {
        TypedLayer::Background { paint, layout, .. } => {
            check_empty_paint_layout_opt(paint, layout);
        }
        TypedLayer::Circle { paint, layout, .. } => {
            check_empty_paint_layout_opt(paint, layout);
        }
        TypedLayer::ColorRelief { paint, layout, .. } => {
            check_empty_paint_layout_opt(paint, layout);
        }
        TypedLayer::Fill { paint, layout, .. } => {
            check_empty_paint_layout_opt(paint, layout);
        }
        TypedLayer::FillExtrusion { paint, layout, .. } => {
            check_empty_paint_layout_opt(paint, layout);
        }
        TypedLayer::Heatmap { paint, layout, .. } => {
            check_empty_paint_layout_opt(paint, layout);
        }
        TypedLayer::Hillshade { paint, layout, .. } => {
            check_empty_paint_layout_opt(paint, layout);
        }
        TypedLayer::Line { paint, layout, .. } => {
            check_empty_paint_layout_opt(paint, layout);
        }
        TypedLayer::Raster { paint, layout, .. } => {
            check_empty_paint_layout_opt(paint, layout);
        }
        TypedLayer::Symbol { paint, layout, .. } => {
            check_empty_paint_layout_opt(paint, layout);
        }
    }
}

fn check_empty_paint_layout_opt<P: serde::Serialize, L: serde::Serialize>(
    paint: &mut Option<P>,
    layout: &mut Option<L>,
) {
    if let Some(p) = paint {
        if is_serialized_empty(p) {
            *paint = None;
        }
    }
    if let Some(l) = layout {
        if is_serialized_empty(l) {
            *layout = None;
        }
    }
}

fn is_serialized_empty(v: &impl serde::Serialize) -> bool {
    let Ok(val) = serde_json::to_value(v) else {
        return false;
    };
    val.as_object().is_some_and(serde_json::Map::is_empty)
}

fn is_invisible_typed(layer: &AnyLayer) -> bool {
    let AnyLayer::Typed(t) = layer else {
        return false;
    };

    // Check visibility via JSON serialization of layout (simpler than matching every layout type).
    let visibility_none = match t {
        TypedLayer::Fill {
            layout: Some(l), ..
        } => serde_json::to_value(l)
            .ok()
            .and_then(|v| v.get("visibility")?.as_str().map(|s| s == "none"))
            .unwrap_or(false),
        TypedLayer::Line {
            layout: Some(l), ..
        } => serde_json::to_value(l)
            .ok()
            .and_then(|v| v.get("visibility")?.as_str().map(|s| s == "none"))
            .unwrap_or(false),
        TypedLayer::Circle {
            layout: Some(l), ..
        } => serde_json::to_value(l)
            .ok()
            .and_then(|v| v.get("visibility")?.as_str().map(|s| s == "none"))
            .unwrap_or(false),
        TypedLayer::Symbol {
            layout: Some(l), ..
        } => serde_json::to_value(l)
            .ok()
            .and_then(|v| v.get("visibility")?.as_str().map(|s| s == "none"))
            .unwrap_or(false),
        TypedLayer::Raster {
            layout: Some(l), ..
        } => serde_json::to_value(l)
            .ok()
            .and_then(|v| v.get("visibility")?.as_str().map(|s| s == "none"))
            .unwrap_or(false),
        TypedLayer::Heatmap {
            layout: Some(l), ..
        } => serde_json::to_value(l)
            .ok()
            .and_then(|v| v.get("visibility")?.as_str().map(|s| s == "none"))
            .unwrap_or(false),
        TypedLayer::Hillshade {
            layout: Some(l), ..
        } => serde_json::to_value(l)
            .ok()
            .and_then(|v| v.get("visibility")?.as_str().map(|s| s == "none"))
            .unwrap_or(false),
        TypedLayer::FillExtrusion {
            layout: Some(l), ..
        } => serde_json::to_value(l)
            .ok()
            .and_then(|v| v.get("visibility")?.as_str().map(|s| s == "none"))
            .unwrap_or(false),
        TypedLayer::Background {
            layout: Some(l), ..
        } => serde_json::to_value(l)
            .ok()
            .and_then(|v| v.get("visibility")?.as_str().map(|s| s == "none"))
            .unwrap_or(false),
        _ => false,
    };
    if visibility_none {
        return true;
    }

    // Check zero opacity via JSON serialization of paint.
    let layer_type = t.layer_type();
    let opacity_prop = match layer_type {
        "fill" => Some("fill-opacity"),
        "line" => Some("line-opacity"),
        "circle" => Some("circle-opacity"),
        "fill-extrusion" => Some("fill-extrusion-opacity"),
        "raster" => Some("raster-opacity"),
        "heatmap" => Some("heatmap-opacity"),
        _ => None,
    };

    if let Some(prop) = opacity_prop {
        let paint_json = match t {
            TypedLayer::Fill { paint: Some(p), .. } => serde_json::to_value(p).ok(),
            TypedLayer::Line { paint: Some(p), .. } => serde_json::to_value(p).ok(),
            TypedLayer::Circle { paint: Some(p), .. } => serde_json::to_value(p).ok(),
            TypedLayer::FillExtrusion { paint: Some(p), .. } => serde_json::to_value(p).ok(),
            TypedLayer::Raster { paint: Some(p), .. } => serde_json::to_value(p).ok(),
            TypedLayer::Heatmap { paint: Some(p), .. } => serde_json::to_value(p).ok(),
            _ => None,
        };
        if let Some(pj) = paint_json {
            if pj.get(prop).and_then(Value::as_f64) == Some(0.0) {
                // Circle special case.
                if layer_type == "circle" {
                    let stroke_opacity = pj
                        .get("circle-stroke-opacity")
                        .and_then(Value::as_f64)
                        .unwrap_or(1.0);
                    let stroke_width = pj
                        .get("circle-stroke-width")
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0);
                    if stroke_opacity != 0.0 && stroke_width != 0.0 {
                        return false;
                    }
                }
                return true;
            }
        }
    }

    false
}

// ── Typed Vector Layer Info ─────────────────────────────────────────────────

/// Pre-compute vector layer info from typed layers.
pub(crate) fn precompute_vector_layer_info_typed(
    style: &MaplibreStyleSpecification,
) -> Vec<Option<VectorLayerInfo>> {
    style
        .layers
        .iter()
        .map(|layer| {
            let AnyLayer::Typed(t) = layer else {
                return None;
            };
            let common = t.common();
            let source = common.source.as_ref()?.as_str();
            let source_layer = common.source_layer.as_ref()?.as_str();

            // Check if it's a vector source.
            if !is_vector_source_typed(style, source) {
                return None;
            }

            Some(VectorLayerInfo {
                source: source.to_string(),
                source_layer: source_layer.to_string(),
            })
        })
        .collect()
}

fn is_vector_source_typed(style: &MaplibreStyleSpecification, source_name: &str) -> bool {
    serde_json::to_value(&style.sources)
        .ok()
        .and_then(|v| {
            v.as_object()?
                .get(source_name)?
                .as_object()?
                .get("type")?
                .as_str()
                .map(|s| s == "vector")
        })
        .unwrap_or(false)
}

//! Cleanup pass operating on typed `MaplibreStyleSpecification`.

use std::collections::HashSet;

use maplibre_style_spec::spec::{AnyLayer, MaplibreStyleSpecification, TypedLayer};
use serde_json::Value;

use super::dead::{collect_used_sources, prune_sources};

/// Remove empty paint/layout, visibility:none layers, zero-opacity layers.
pub(crate) fn cleanup(style: &mut MaplibreStyleSpecification) {
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
            if is_invisible(layer) {
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
        let used = collect_used_sources(&style.layers);
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
    if let Some(p) = paint
        && is_serialized_empty(p)
    {
        *paint = None;
    }
    if let Some(l) = layout
        && is_serialized_empty(l)
    {
        *layout = None;
    }
}

fn is_serialized_empty(v: &impl serde::Serialize) -> bool {
    let Ok(val) = serde_json::to_value(v) else {
        return false;
    };
    val.as_object().is_some_and(serde_json::Map::is_empty)
}

#[expect(clippy::too_many_lines)]
fn is_invisible(layer: &AnyLayer) -> bool {
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
        if let Some(pj) = paint_json
            && pj.get(prop).and_then(Value::as_f64) == Some(0.0)
        {
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

    false
}

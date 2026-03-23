//! Cleanup pass operating on typed `MaplibreStyleSpecification`.

use std::collections::HashSet;

use maplibre_style_spec::spec::{AnyLayer, MaplibreStyleSpecification, TypedLayer, Visibility};

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
            if is_invisible(layer) { Some(i) } else { None }
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

/// Extract the visibility from any layout variant.
fn layout_visibility(layer: &TypedLayer) -> Option<Visibility> {
    match layer {
        TypedLayer::Background {
            layout: Some(l), ..
        } => l.visibility,
        TypedLayer::Circle {
            layout: Some(l), ..
        } => l.visibility,
        TypedLayer::ColorRelief {
            layout: Some(l), ..
        } => l.visibility,
        TypedLayer::Fill {
            layout: Some(l), ..
        } => l.visibility,
        TypedLayer::FillExtrusion {
            layout: Some(l), ..
        } => l.visibility,
        TypedLayer::Heatmap {
            layout: Some(l), ..
        } => l.visibility,
        TypedLayer::Hillshade {
            layout: Some(l), ..
        } => l.visibility,
        TypedLayer::Line {
            layout: Some(l), ..
        } => l.visibility,
        TypedLayer::Raster {
            layout: Some(l), ..
        } => l.visibility,
        TypedLayer::Symbol {
            layout: Some(l), ..
        } => l.visibility,
        TypedLayer::Background { layout: None, .. }
        | TypedLayer::Circle { layout: None, .. }
        | TypedLayer::ColorRelief { layout: None, .. }
        | TypedLayer::Fill { layout: None, .. }
        | TypedLayer::FillExtrusion { layout: None, .. }
        | TypedLayer::Heatmap { layout: None, .. }
        | TypedLayer::Hillshade { layout: None, .. }
        | TypedLayer::Line { layout: None, .. }
        | TypedLayer::Raster { layout: None, .. }
        | TypedLayer::Symbol { layout: None, .. } => None,
    }
}

/// Extract the primary opacity as a literal f64, if present.
fn literal_opacity(layer: &TypedLayer) -> Option<f64> {
    match layer {
        TypedLayer::Background { paint: Some(p), .. } => p.background_opacity.as_ref()?.0.as_f64(),
        TypedLayer::Circle { paint: Some(p), .. } => p.circle_opacity.as_ref()?.0.as_f64(),
        TypedLayer::ColorRelief { paint: Some(p), .. } => {
            p.color_relief_opacity.as_ref()?.0.as_f64()
        }
        TypedLayer::Fill { paint: Some(p), .. } => p.fill_opacity.as_ref()?.0.as_f64(),
        TypedLayer::FillExtrusion { paint: Some(p), .. } => {
            p.fill_extrusion_opacity.as_ref()?.0.as_f64()
        }
        TypedLayer::Heatmap { paint: Some(p), .. } => p.heatmap_opacity.as_ref()?.0.as_f64(),
        TypedLayer::Line { paint: Some(p), .. } => p.line_opacity.as_ref()?.0.as_f64(),
        TypedLayer::Raster { paint: Some(p), .. } => p.raster_opacity.as_ref()?.0.as_f64(),
        TypedLayer::Symbol { paint: Some(p), .. } => {
            // Symbol is invisible only when both icon and text opacity are zero.
            let icon = p
                .icon_opacity
                .as_ref()
                .and_then(|o| o.0.as_f64())
                .unwrap_or(1.0);
            let text = p
                .text_opacity
                .as_ref()
                .and_then(|o| o.0.as_f64())
                .unwrap_or(1.0);
            if icon == 0.0 && text == 0.0 {
                Some(0.0)
            } else {
                None
            }
        }
        TypedLayer::Hillshade { .. }
        | TypedLayer::Background { paint: None, .. }
        | TypedLayer::Circle { paint: None, .. }
        | TypedLayer::ColorRelief { paint: None, .. }
        | TypedLayer::Fill { paint: None, .. }
        | TypedLayer::FillExtrusion { paint: None, .. }
        | TypedLayer::Heatmap { paint: None, .. }
        | TypedLayer::Line { paint: None, .. }
        | TypedLayer::Raster { paint: None, .. }
        | TypedLayer::Symbol { paint: None, .. } => None,
    }
}

fn is_invisible(layer: &AnyLayer) -> bool {
    let AnyLayer::Typed(t) = layer else {
        return false;
    };

    // Check visibility.
    if layout_visibility(t) == Some(Visibility::None) {
        return true;
    }

    // Check zero opacity.
    if literal_opacity(t) == Some(0.0) {
        // Circle special case: visible stroke keeps the layer alive.
        if let TypedLayer::Circle { paint: Some(p), .. } = t {
            let stroke_opacity = p
                .circle_stroke_opacity
                .as_ref()
                .and_then(|o| o.0.as_f64())
                .unwrap_or(1.0);
            let stroke_width = p
                .circle_stroke_width
                .as_ref()
                .and_then(|w| w.0.as_f64())
                .unwrap_or(0.0);
            if stroke_opacity != 0.0 && stroke_width != 0.0 {
                return false;
            }
        }
        return true;
    }

    false
}

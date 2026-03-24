//! Metadata refinement pass operating on typed `MaplibreStyleSpecification`.

use maplibre_style_spec::shared_expr::NumericExpression;
use maplibre_style_spec::spec::{AnyLayer, Boolean, MaplibreStyleSpecification, TypedLayer};

use super::source_util::VectorLayerInfo;
use super::zoom::visibility_minzoom_from_value;
use crate::stats::TileStatistics;

/// Extract zoom bounds from filters and tighten minzoom/maxzoom.
pub(crate) fn metadata_refinement(
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
        let filter_json = filter.to_json_value();
        let (lb_raw, ub_raw) = super::zoom::zoom_bounds_from_expression(&filter_json);
        let lb = lb_raw.map(f64::ceil);
        let ub = ub_raw.map(f64::floor);

        if let Some(bound) = lb {
            let cur = common
                .minzoom
                .as_ref()
                .and_then(maplibre_style_spec::spec::LayerMinzoom::as_f64);
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
            let cur = common
                .maxzoom
                .as_ref()
                .and_then(maplibre_style_spec::spec::LayerMaxzoom::as_f64);
            let new_max = match cur {
                Some(c) => c.min(bound),
                None => bound,
            };
            common.maxzoom = maplibre_style_spec::spec::LayerMaxzoom::from_f64(new_max);
        }

        // Remove consumed zoom predicates from filter.
        let adopted_min = common
            .minzoom
            .as_ref()
            .and_then(maplibre_style_spec::spec::LayerMinzoom::as_f64);
        let adopted_max = common
            .maxzoom
            .as_ref()
            .and_then(maplibre_style_spec::spec::LayerMaxzoom::as_f64);
        if (adopted_min.is_some() || adopted_max.is_some())
            && let Some(ref mut filter) = common.filter
        {
            let mut filter_json = filter.to_json_value();
            super::zoom::remove_consumed_zoom_predicates(
                &mut filter_json,
                adopted_min,
                adopted_max,
            );
            if let Some(new_filter) = Boolean::from_value(filter_json) {
                *filter = new_filter;
            }
        }
    }

    // Paint-based visibility minzoom.
    if let Some(paint_min) = paint_visibility_minzoom(layer)
        && paint_min != f64::INFINITY
    {
        let common = layer.common_mut();
        let cur = common
            .minzoom
            .as_ref()
            .and_then(maplibre_style_spec::spec::LayerMinzoom::as_f64);
        if cur.is_none_or(|c| paint_min > c) {
            common.minzoom = maplibre_style_spec::spec::LayerMinzoom::from_f64(paint_min);
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

        let cur_min = common
            .minzoom
            .as_ref()
            .and_then(maplibre_style_spec::spec::LayerMinzoom::as_f64);
        if cur_min.is_none_or(|c| data_min > c) {
            common.minzoom = maplibre_style_spec::spec::LayerMinzoom::from_f64(data_min);
        }

        let cur_max = common
            .maxzoom
            .as_ref()
            .and_then(maplibre_style_spec::spec::LayerMaxzoom::as_f64);
        if cur_max.is_none_or(|c| data_max < c) {
            common.maxzoom = maplibre_style_spec::spec::LayerMaxzoom::from_f64(data_max);
        }
    }
}

// ── Paint-based visibility analysis ─────────────────────────────────────────

/// Compute the minzoom from a single numeric paint property.
/// Returns `Some(zoom)` if the property evaluates to zero below that zoom.
fn prop_visibility_minzoom(prop: &NumericExpression) -> Option<f64> {
    // Fast path: literal number.
    #[allow(clippy::float_cmp)]
    if let Some(n) = prop.as_f64() {
        return if n == 0.0 { Some(f64::INFINITY) } else { None };
    }
    // Expr path: serialize and analyze.
    let json = serde_json::to_value(prop).ok()?;
    visibility_minzoom_from_value(&json)
}

/// Determine the zoom level at which a typed layer's paint properties first
/// produce a visible result.  Returns `None` when no constraint can be derived.
fn paint_visibility_minzoom(layer: &TypedLayer) -> Option<f64> {
    match layer {
        TypedLayer::Background { paint: Some(p), .. } => p
            .background_opacity
            .as_ref()
            .and_then(|o| prop_visibility_minzoom(&o.0)),
        TypedLayer::Circle { paint: Some(p), .. } => {
            let size = p
                .circle_radius
                .as_ref()
                .and_then(|r| prop_visibility_minzoom(&r.0));
            let opacity = p
                .circle_opacity
                .as_ref()
                .and_then(|o| prop_visibility_minzoom(&o.0));
            combine_size_opacity(size, opacity)
        }
        TypedLayer::ColorRelief { paint: Some(p), .. } => p
            .color_relief_opacity
            .as_ref()
            .and_then(|o| prop_visibility_minzoom(&o.0)),
        TypedLayer::Fill { paint: Some(p), .. } => p
            .fill_opacity
            .as_ref()
            .and_then(|o| prop_visibility_minzoom(&o.0)),
        TypedLayer::FillExtrusion { paint: Some(p), .. } => {
            let size = p
                .fill_extrusion_height
                .as_ref()
                .and_then(|h| prop_visibility_minzoom(&h.0));
            let opacity = p
                .fill_extrusion_opacity
                .as_ref()
                .and_then(|o| prop_visibility_minzoom(&o.0));
            combine_size_opacity(size, opacity)
        }
        TypedLayer::Heatmap { paint: Some(p), .. } => p
            .heatmap_opacity
            .as_ref()
            .and_then(|o| prop_visibility_minzoom(&o.0)),
        TypedLayer::Line { paint: Some(p), .. } => {
            let size = p
                .line_width
                .as_ref()
                .and_then(|w| prop_visibility_minzoom(&w.0));
            let opacity = p
                .line_opacity
                .as_ref()
                .and_then(|o| prop_visibility_minzoom(&o.0));
            combine_size_opacity(size, opacity)
        }
        TypedLayer::Raster { paint: Some(p), .. } => p
            .raster_opacity
            .as_ref()
            .and_then(|o| prop_visibility_minzoom(&o.0)),
        TypedLayer::Symbol { paint: Some(p), .. } => {
            // Symbol is visible if either text or icon is visible.
            let text_op = p
                .text_opacity
                .as_ref()
                .and_then(|o| prop_visibility_minzoom(&o.0));
            let icon_op = p
                .icon_opacity
                .as_ref()
                .and_then(|o| prop_visibility_minzoom(&o.0));
            // Visible when ANY sub-channel is visible → take min.
            match (text_op, icon_op) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (Some(a), None) | (None, Some(a)) => Some(a),
                (None, None) => None,
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

/// Combine size and opacity constraints.
/// Both must be non-zero for the layer to be visible → take `max`.
/// Either alone provides a constraint.  `None` means no constraint for that axis.
fn combine_size_opacity(size: Option<f64>, opacity: Option<f64>) -> Option<f64> {
    match (size, opacity) {
        (Some(s), Some(o)) => Some(s.max(o)),
        (Some(v), None) | (None, Some(v)) => Some(v),
        (None, None) => None,
    }
}

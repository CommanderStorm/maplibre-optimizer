//! Metadata refinement pass operating on typed `MaplibreStyleSpecification`.

use maplibre_style_spec::spec::{AnyLayer, LayerFilter, MaplibreStyleSpecification, TypedLayer};

use super::source_util::VectorLayerInfo;
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
            if let Some(new_filter) = LayerFilter::from_value(filter_json) {
                *filter = new_filter;
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

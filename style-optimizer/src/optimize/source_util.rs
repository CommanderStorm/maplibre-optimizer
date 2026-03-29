//! Shared utilities for vector layer info and source-level optimisations.

use std::collections::BTreeMap;

use maplibre_style_spec::spec::{AnyLayer, MaplibreStyleSpecification, Source};
use serde_json::Value;

/// Information about a layer's vector source, pre-computed for use by visitors.
#[derive(Clone, Debug)]
pub(crate) struct VectorLayerInfo {
    pub source: String,
    pub source_layer: String,
    /// The source's maxzoom (tiles at this zoom are overzoomed for higher zooms).
    pub source_maxzoom: Option<f64>,
}

/// Check if a source is a `"vector"` type by looking it up in the style root.
fn is_vector_source(style_root: &Value, source_name: &str) -> bool {
    style_root
        .as_object()
        .and_then(|r| r.get("sources"))
        .and_then(Value::as_object)
        .and_then(|s| s.get(source_name))
        .and_then(Value::as_object)
        .and_then(|s| s.get("type"))
        .and_then(Value::as_str)
        == Some("vector")
}

/// Extract `(source, source-layer)` from a layer object.
fn layer_source_info(layer: &Value) -> Option<(&str, &str)> {
    let obj = layer.as_object()?;
    let source = obj.get("source")?.as_str()?;
    let source_layer = obj.get("source-layer")?.as_str()?;
    Some((source, source_layer))
}

/// Pre-compute vector layer info for all layers in the style (JSON variant).
/// Returns a vec indexed by layer position; `None` if the layer doesn't have a vector source.
pub(crate) fn precompute_vector_layer_info(style: &Value) -> Vec<Option<VectorLayerInfo>> {
    let Some(root) = style.as_object() else {
        return vec![];
    };
    let Some(layers) = root.get("layers").and_then(Value::as_array) else {
        return vec![];
    };

    layers
        .iter()
        .map(|layer| {
            let (source, source_layer) = layer_source_info(layer)?;
            if !is_vector_source(style, source) {
                return None;
            }
            let source_maxzoom = root
                .get("sources")
                .and_then(Value::as_object)
                .and_then(|s| s.get(source))
                .and_then(Value::as_object)
                .and_then(|s| s.get("maxzoom"))
                .and_then(Value::as_f64);
            Some(VectorLayerInfo {
                source: source.to_string(),
                source_layer: source_layer.to_string(),
                source_maxzoom,
            })
        })
        .collect()
}

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
            let Some(Source::Vector(vector_source)) = style.sources.0.get(source) else {
                return None;
            };

            let source_maxzoom = vector_source
                .maxzoom
                .as_ref()
                .and_then(|m| serde_json::to_value(m).ok())
                .and_then(|v| v.as_f64());

            Some(VectorLayerInfo {
                source: source.to_string(),
                source_layer: source_layer.to_string(),
                source_maxzoom,
            })
        })
        .collect()
}

// ── Source zoom tightening ───────────────────────────────────────────────────

/// Effective zoom range for a tile-based source (returns `(minzoom, maxzoom)`).
/// Only applies to vector, raster, and raster-dem sources.
fn source_zoom_range(source: &Source) -> Option<(f64, f64)> {
    match source {
        Source::Vector(_) | Source::Raster(_) | Source::RasterDem(_) => {}
        _ => return None,
    }
    let obj = serde_json::to_value(source).ok()?;
    let lo = obj.get("minzoom").and_then(Value::as_f64).unwrap_or(0.0);
    let hi = obj.get("maxzoom").and_then(Value::as_f64).unwrap_or(22.0);
    Some((lo, hi))
}

/// Tighten source minzoom/maxzoom based on the effective zoom range of all
/// layers that reference each source.
///
/// After dead-layer elimination and metadata refinement, some sources may have
/// a wider zoom range than any of their referencing layers actually need.
/// Tightening prevents unnecessary tile fetches.
pub(crate) fn tighten_source_zoom_bounds(style: &mut MaplibreStyleSpecification) {
    // Collect effective zoom bounds per source from layer metadata.
    // For each source, track (min of layer minzooms, max of layer maxzooms).
    let mut source_bounds: BTreeMap<&str, (f64, f64)> = BTreeMap::new();

    for layer in &style.layers {
        let AnyLayer::Typed(t) = layer else {
            continue;
        };
        let common = t.common();
        let Some(source_name) = common
            .source
            .as_ref()
            .map(maplibre_style_spec::spec::LayerSource::as_str)
        else {
            continue;
        };

        let layer_min = common
            .minzoom
            .as_ref()
            .and_then(maplibre_style_spec::spec::LayerMinzoom::as_f64)
            .unwrap_or(0.0);
        let layer_max = common
            .maxzoom
            .as_ref()
            .and_then(maplibre_style_spec::spec::LayerMaxzoom::as_f64)
            .unwrap_or(24.0);

        source_bounds
            .entry(source_name)
            .and_modify(|(lo, hi)| {
                if layer_min < *lo {
                    *lo = layer_min;
                }
                if layer_max > *hi {
                    *hi = layer_max;
                }
            })
            .or_insert((layer_min, layer_max));
    }

    // Apply tightened bounds to each source.
    for (name, (eff_min, eff_max)) in &source_bounds {
        let Some(source) = style.sources.0.get_mut(*name) else {
            continue;
        };
        let Some((src_min, src_max)) = source_zoom_range(source) else {
            continue;
        };

        // Only tighten, never loosen.
        let new_min = eff_min.max(src_min);
        let new_max = eff_max.min(src_max);

        // Skip if no change.
        if (new_min - src_min).abs() < f64::EPSILON && (new_max - src_max).abs() < f64::EPSILON {
            continue;
        }

        // Modify source via JSON round-trip (constructors are crate-private).
        let Ok(mut src_json) = serde_json::to_value(&*source) else {
            continue;
        };
        let Some(obj) = src_json.as_object_mut() else {
            continue;
        };
        let mut changed = false;
        if new_min > src_min {
            obj.insert("minzoom".into(), Value::from(new_min));
            changed = true;
        }
        if new_max < src_max {
            obj.insert("maxzoom".into(), Value::from(new_max));
            changed = true;
        }
        if changed && let Ok(updated) = serde_json::from_value::<Source>(src_json) {
            *source = updated;
        }
    }
}

//! Shared utilities for vector layer info.

use maplibre_style_spec::spec::{AnyLayer, MaplibreStyleSpecification, Source};
use serde_json::Value;

/// Information about a layer's vector source, pre-computed for use by visitors.
#[derive(Clone, Debug)]
pub(crate) struct VectorLayerInfo {
    pub source: String,
    pub source_layer: String,
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
            Some(VectorLayerInfo {
                source: source.to_string(),
                source_layer: source_layer.to_string(),
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
    style
        .sources
        .0
        .get(source_name)
        .is_some_and(|s| matches!(s, Source::Vector { .. }))
}

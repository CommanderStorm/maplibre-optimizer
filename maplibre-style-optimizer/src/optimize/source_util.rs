//! Shared utilities for vector layer info.

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
pub(super) fn precompute_vector_layer_info(style: &Value) -> Vec<Option<VectorLayerInfo>> {
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

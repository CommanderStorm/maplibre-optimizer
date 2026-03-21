//! Metadata stripping (Pass 5) and structural cleanup (Pass 6).

use serde_json::Value;

use super::source_util::{build_layer_index, collect_used_sources};
use super::walk::StyleVisitor;

// ── Strip Metadata ─────────────────────────────────────────────────────────────

/// Remove `metadata` keys from the style root and from each layer.
pub(crate) struct StripMetadataVisitor;

impl StyleVisitor for StripMetadataVisitor {
    fn visit_layer(&mut self, _: usize, _: &str, layer: &mut Value) {
        if let Some(obj) = layer.as_object_mut() {
            obj.remove("metadata");
        }
    }

    fn visit_root(&mut self, root: &mut Value) {
        if let Some(obj) = root.as_object_mut() {
            obj.remove("metadata");
        }
    }
}

// ── Cleanup ────────────────────────────────────────────────────────────────────

/// Remove empty paint/layout objects, visibility:none layers, and zero-opacity layers.
pub(crate) struct CleanupVisitor;

impl StyleVisitor for CleanupVisitor {
    fn visit_layer(&mut self, _: usize, _: &str, layer: &mut Value) {
        remove_empty_paint_layout(layer);
    }

    fn visit_root(&mut self, root: &mut Value) {
        remove_invisible_layers(root);
    }
}

fn remove_empty_paint_layout(layer: &mut Value) {
    let Some(obj) = layer.as_object_mut() else {
        return;
    };
    for section in ["paint", "layout"] {
        if obj
            .get(section)
            .and_then(Value::as_object)
            .is_some_and(serde_json::Map::is_empty)
        {
            obj.remove(section);
        }
    }
}

/// Primary opacity property name for each layer type that has one.
fn primary_opacity_prop(layer_type: &str) -> Option<&'static str> {
    match layer_type {
        "fill" => Some("fill-opacity"),
        "line" => Some("line-opacity"),
        "circle" => Some("circle-opacity"),
        "fill-extrusion" => Some("fill-extrusion-opacity"),
        "raster" => Some("raster-opacity"),
        "heatmap" => Some("heatmap-opacity"),
        // symbol has separate icon-opacity and text-opacity — no single property suffices.
        _ => None,
    }
}

fn is_invisible_layer(layer: &Value) -> bool {
    let Some(obj) = layer.as_object() else {
        return false;
    };

    // Only match plain strings; expression-based visibility is data-dependent.
    if obj
        .get("layout")
        .and_then(Value::as_object)
        .and_then(|l| l.get("visibility"))
        .and_then(Value::as_str)
        == Some("none")
    {
        return true;
    }

    // Skip expression-valued opacities — they may vary by zoom/feature.
    let layer_type = obj.get("type").and_then(Value::as_str).unwrap_or("");
    let paint = obj.get("paint").and_then(Value::as_object);
    if let Some(prop) = primary_opacity_prop(layer_type)
        && paint
            .and_then(|p| p.get(prop))
            .and_then(Value::as_f64)
            == Some(0.0)
    {
        // Circle stroke renders independently of fill opacity.
        if layer_type == "circle" {
            let stroke_opacity = paint
                .and_then(|p| p.get("circle-stroke-opacity"))
                .and_then(Value::as_f64)
                .unwrap_or(1.0); // spec default
            let stroke_width = paint
                .and_then(|p| p.get("circle-stroke-width"))
                .and_then(Value::as_f64)
                .unwrap_or(0.0); // spec default
            if stroke_opacity != 0.0 && stroke_width != 0.0 {
                return false;
            }
        }
        return true;
    }

    false
}

fn remove_invisible_layers(root: &mut Value) {
    let Some(obj) = root.as_object_mut() else {
        return;
    };

    let Some(layers_arr) = obj.get("layers").and_then(Value::as_array) else {
        return;
    };

    let referenced_ids: std::collections::HashSet<&str> = layers_arr
        .iter()
        .filter_map(|l| l.as_object()?.get("ref")?.as_str())
        .collect();

    let to_remove: Vec<usize> = layers_arr
        .iter()
        .enumerate()
        .filter_map(|(i, layer)| {
            let id = layer.as_object()?.get("id")?.as_str()?;
            if referenced_ids.contains(id) {
                return None; // never remove a ref target
            }
            if is_invisible_layer(layer) {
                Some(i)
            } else {
                None
            }
        })
        .collect();

    if to_remove.is_empty() {
        return;
    }

    if let Some(layers) = obj.get_mut("layers").and_then(Value::as_array_mut) {
        for i in to_remove.into_iter().rev() {
            layers.remove(i);
        }
    }

    // Dropped layers may have been the only users of some sources.
    let Some(layers) = obj.get("layers").and_then(Value::as_array) else {
        return;
    };
    let by_id = build_layer_index(layers);
    let used = collect_used_sources(layers, &by_id);
    if let Some(sources) = obj.get_mut("sources").and_then(Value::as_object_mut) {
        sources.retain(|id, _| used.contains(id.as_str()));
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::optimize::walk::walk_style_mut;

    fn dummy_mir() -> maplibre_style_spec::mir::IntermediateSpec {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../upstream/src/reference/v8.json");
        crate::load_intermediate_spec_from_v8_path(&path).expect("v8.json")
    }

    #[test]
    fn strip_metadata_removes_root_and_layer_metadata() {
        let mir = dummy_mir();
        let mut v = json!({
            "version": 8,
            "metadata": { "maputnik:renderer": "mbgljs" },
            "layers": [{
                "id": "x",
                "type": "fill",
                "metadata": { "mapbox:group": "water" },
                "paint": { "fill-color": "#ff0000" }
            }]
        });
        walk_style_mut(&mut v, &mir, &mut StripMetadataVisitor);
        assert!(v.get("metadata").is_none(), "root metadata not removed");
        assert!(
            v["layers"][0].get("metadata").is_none(),
            "layer metadata not removed"
        );
        assert_eq!(
            v["layers"][0]["paint"]["fill-color"],
            json!("#ff0000"),
            "other properties preserved"
        );
    }

    #[test]
    fn cleanup_removes_empty_paint_layout() {
        let mir = dummy_mir();
        let mut v = json!({
            "layers": [{
                "id": "x",
                "type": "fill",
                "paint": {},
                "layout": {}
            }]
        });
        walk_style_mut(&mut v, &mir, &mut CleanupVisitor);
        let layer = &v["layers"][0];
        assert!(layer.get("paint").is_none(), "empty paint not removed");
        assert!(layer.get("layout").is_none(), "empty layout not removed");
    }

    #[test]
    fn cleanup_removes_visibility_none_layer() {
        let mir = dummy_mir();
        let mut v = json!({
            "version": 8,
            "sources": { "s": { "type": "vector", "url": "x" } },
            "layers": [{
                "id": "x",
                "type": "fill",
                "source": "s",
                "source-layer": "l",
                "layout": { "visibility": "none" }
            }]
        });
        walk_style_mut(&mut v, &mir, &mut CleanupVisitor);
        assert_eq!(v["layers"].as_array().unwrap().len(), 0);
        assert!(v["sources"].as_object().unwrap().is_empty());
    }

    #[test]
    fn cleanup_removes_zero_opacity_layer() {
        let mir = dummy_mir();
        let mut v = json!({
            "version": 8,
            "sources": { "s": { "type": "vector", "url": "x" } },
            "layers": [{
                "id": "x",
                "type": "fill",
                "source": "s",
                "source-layer": "l",
                "paint": { "fill-opacity": 0 }
            }]
        });
        walk_style_mut(&mut v, &mir, &mut CleanupVisitor);
        assert_eq!(v["layers"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn cleanup_preserves_circle_with_visible_stroke() {
        let mir = dummy_mir();
        let mut v = json!({
            "version": 8,
            "sources": { "s": { "type": "geojson", "data": { "type": "Point", "coordinates": [0, 0] } } },
            "layers": [{
                "id": "x",
                "type": "circle",
                "source": "s",
                "paint": { "circle-opacity": 0, "circle-stroke-width": 5 }
            }]
        });
        walk_style_mut(&mut v, &mir, &mut CleanupVisitor);
        // stroke is visible → layer must be kept
        assert_eq!(v["layers"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn cleanup_removes_circle_with_zero_opacity_and_no_stroke() {
        let mir = dummy_mir();
        let mut v = json!({
            "version": 8,
            "sources": { "s": { "type": "geojson", "data": { "type": "Point", "coordinates": [0, 0] } } },
            "layers": [{
                "id": "x",
                "type": "circle",
                "source": "s",
                "paint": { "circle-opacity": 0 }
            }]
        });
        walk_style_mut(&mut v, &mir, &mut CleanupVisitor);
        // no stroke (default width=0) → layer is invisible → removed
        assert_eq!(v["layers"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn cleanup_preserves_expression_opacity() {
        let mir = dummy_mir();
        let mut v = json!({
            "layers": [{
                "id": "x",
                "type": "fill",
                "paint": { "fill-opacity": ["interpolate", ["linear"], ["zoom"], 5, 0, 10, 1] }
            }]
        });
        walk_style_mut(&mut v, &mir, &mut CleanupVisitor);
        assert_eq!(v["layers"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn cleanup_does_not_remove_referenced_layer() {
        let mir = dummy_mir();
        let mut v = json!({
            "layers": [
                {
                    "id": "base",
                    "type": "fill",
                    "layout": { "visibility": "none" }
                },
                {
                    "id": "child",
                    "ref": "base"
                }
            ]
        });
        walk_style_mut(&mut v, &mir, &mut CleanupVisitor);
        // "base" is referenced by "child" → must not be removed
        assert_eq!(v["layers"].as_array().unwrap().len(), 2);
    }
}

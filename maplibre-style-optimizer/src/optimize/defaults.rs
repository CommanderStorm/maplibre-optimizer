//! Default property elimination (Pass 1).
//!
//! Removes paint/layout properties that equal their spec-defined default values.
//! Only strips plain scalars (numbers, strings, booleans) — never expressions.

use maplibre_style_spec::mir::{IntermediateSpec, PropertySection};
use serde_json::Value;

use super::walk::StyleVisitor;

// ── Visitor ───────────────────────────────────────────────────────────────────

pub(crate) struct StripDefaultsVisitor<'a> {
    pub mir: &'a IntermediateSpec,
}

impl StyleVisitor for StripDefaultsVisitor<'_> {
    fn visit_layer(&mut self, _: usize, layer_type: &str, layer: &mut Value) {
        strip_layer_defaults(layer, layer_type, self.mir);
    }
}

// ── Implementation ─────────────────────────────────────────────────────────────

/// A value is "plain" if it cannot be a `MapLibre` expression (arrays are expressions).
fn is_plain_value(v: &Value) -> bool {
    matches!(
        v,
        Value::Number(_) | Value::String(_) | Value::Bool(_) | Value::Null
    )
}

fn strip_section(
    layer_obj: &mut serde_json::Map<String, Value>,
    section_key: &str,
    section: PropertySection,
    layer_type: &str,
    mir: &IntermediateSpec,
) {
    let Some(section_val) = layer_obj.get_mut(section_key) else {
        return;
    };
    let Some(section_obj) = section_val.as_object_mut() else {
        return;
    };

    let to_remove: Vec<String> = section_obj
        .iter()
        .filter_map(|(prop, value)| {
            if !is_plain_value(value) {
                return None; // never strip expressions
            }
            let default = mir.layers.field_default(layer_type, section, prop)?;
            if value == default {
                Some(prop.clone())
            } else {
                None
            }
        })
        .collect();

    for prop in to_remove {
        section_obj.remove(&prop);
    }
}

fn strip_layer_defaults(layer: &mut Value, layer_type: &str, mir: &IntermediateSpec) {
    let Some(obj) = layer.as_object_mut() else {
        return;
    };

    strip_section(obj, "paint", PropertySection::Paint, layer_type, mir);
    strip_section(obj, "layout", PropertySection::Layout, layer_type, mir);

    // Remove empty sections produced by stripping.
    for section_key in ["paint", "layout"] {
        if obj
            .get(section_key)
            .and_then(Value::as_object)
            .is_some_and(serde_json::Map::is_empty)
        {
            obj.remove(section_key);
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::load_intermediate_spec_from_v8_path;
    use crate::optimize::walk::walk_style_mut;

    fn sample_mir() -> IntermediateSpec {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../upstream/src/reference/v8.json");
        load_intermediate_spec_from_v8_path(&path).expect("v8.json")
    }

    #[test]
    fn strips_fill_opacity_default() {
        let mir = sample_mir();
        let mut v = json!({
            "layers": [{
                "id": "x",
                "type": "fill",
                "paint": { "fill-opacity": 1, "fill-color": "#ff0000" }
            }]
        });
        walk_style_mut(&mut v, &mir, &mut StripDefaultsVisitor { mir: &mir });
        // fill-opacity: 1 is the default → removed
        assert!(v["layers"][0]["paint"].get("fill-opacity").is_none());
        // fill-color is not a default → kept
        assert_eq!(v["layers"][0]["paint"]["fill-color"], json!("#ff0000"));
    }

    #[test]
    fn strips_visibility_visible() {
        let mir = sample_mir();
        let mut v = json!({
            "layers": [{
                "id": "x",
                "type": "fill",
                "layout": { "visibility": "visible" }
            }]
        });
        walk_style_mut(&mut v, &mir, &mut StripDefaultsVisitor { mir: &mir });
        // After stripping, layout should be empty and then removed
        assert!(v["layers"][0].get("layout").is_none());
    }

    #[test]
    fn preserves_non_default_values() {
        let mir = sample_mir();
        let mut v = json!({
            "layers": [{
                "id": "x",
                "type": "fill",
                "paint": { "fill-opacity": 0.5 }
            }]
        });
        walk_style_mut(&mut v, &mir, &mut StripDefaultsVisitor { mir: &mir });
        assert_eq!(v["layers"][0]["paint"]["fill-opacity"], json!(0.5));
    }

    #[test]
    fn never_strips_expressions() {
        let mir = sample_mir();
        // fill-opacity: 1 is the default, but expressed as an expression
        let mut v = json!({
            "layers": [{
                "id": "x",
                "type": "fill",
                "paint": { "fill-opacity": ["literal", 1] }
            }]
        });
        walk_style_mut(&mut v, &mir, &mut StripDefaultsVisitor { mir: &mir });
        // Must NOT be stripped — it's an expression
        assert!(v["layers"][0]["paint"].get("fill-opacity").is_some());
    }

    #[test]
    fn removes_empty_paint_after_stripping() {
        let mir = sample_mir();
        let mut v = json!({
            "layers": [{
                "id": "x",
                "type": "line",
                "paint": { "line-width": 1, "line-opacity": 1 }
            }]
        });
        walk_style_mut(&mut v, &mir, &mut StripDefaultsVisitor { mir: &mir });
        // Both are defaults → paint becomes empty → removed
        assert!(v["layers"][0].get("paint").is_none());
    }
}

//! Schema-guided visitors for `MapLibre` style trees.
//!
//! Two visitor systems:
//! - [`StyleVisitor`] + [`walk_style_mut`]: JSON-based, used for property expression passes.
//! - [`TypedFilterVisitor`] + [`walk_typed_filters`]: typed, used for filter expression passes
//!   operating directly on [`Boolean`].

use std::collections::{HashMap, HashSet};

use maplibre_style_spec::mir::{MirLayerField, MirPropertySection, MirSpec};
use maplibre_style_spec::spec::{AnyLayer, Boolean, MaplibreStyleSpecification};
use serde_json::Value;

// ── Public types ──────────────────────────────────────────────────────────────

/// Context provided at each paint/layout property visit site.
#[allow(dead_code)]
pub(crate) struct PropertyContext<'a> {
    pub layer_index: usize,
    pub layer_type: &'a str,
    pub section: MirPropertySection,
    pub property_name: &'a str,
    pub field: &'a MirLayerField,
}

/// Visitor trait for schema-guided style tree walks.
///
/// All methods have default no-op implementations; implement only what you need.
///
/// Call order per layer: `visit_filter` → `visit_property` (for each paint/layout
/// property that is present) → `visit_layer`. After all layers: `visit_root`.
pub(crate) trait StyleVisitor {
    /// Called for each layer's `filter` expression (mutable).
    fn visit_filter(&mut self, layer_index: usize, layer_type: &str, filter: &mut Value) {
        let _ = (layer_index, layer_type, filter);
    }

    /// Called for each present paint/layout property value (mutable), with schema context.
    fn visit_property(&mut self, ctx: &PropertyContext<'_>, value: &mut Value) {
        let _ = (ctx, value);
    }

    /// Called for each layer object (mutable), after filter and properties.
    fn visit_layer(&mut self, layer_index: usize, layer_type: &str, layer: &mut Value) {
        let _ = (layer_index, layer_type, layer);
    }

    /// Called once for the root style object, after all layers.
    fn visit_root(&mut self, root: &mut Value) {
        let _ = root;
    }
}

// ── Driver ────────────────────────────────────────────────────────────────────

/// Walk a style JSON tree, calling visitor methods with schema context from MIR.
pub(crate) fn walk_style_mut(style: &mut Value, mir: &MirSpec, visitor: &mut impl StyleVisitor) {
    // Immutable pass: resolve every layer's effective type (following `ref` chains).
    let layer_types = collect_layer_types(style);

    {
        let Some(root) = style.as_object_mut() else {
            return;
        };
        let Some(layers_val) = root.get_mut("layers") else {
            return;
        };
        let Some(layers) = layers_val.as_array_mut() else {
            return;
        };

        for (i, layer) in layers.iter_mut().enumerate() {
            let Some(layer_type) = layer_types.get(i).and_then(|t| t.as_deref()) else {
                continue;
            };
            let type_def = mir.layers.layer_types.get(layer_type);

            // Visit filter and properties in an inner scope so the `obj` borrow is
            // released before we call `visit_layer` on the whole layer value.
            if let Some(obj) = layer.as_object_mut() {
                if let Some(filter) = obj.get_mut("filter") {
                    visitor.visit_filter(i, layer_type, filter);
                }

                if let Some(type_def) = type_def {
                    if let Some(paint_val) = obj.get_mut("paint")
                        && let Some(paint_obj) = paint_val.as_object_mut()
                    {
                        for (prop_name, field) in &type_def.paint {
                            if let Some(value) = paint_obj.get_mut(prop_name.as_str()) {
                                visitor.visit_property(
                                    &PropertyContext {
                                        layer_index: i,
                                        layer_type,
                                        section: MirPropertySection::Paint,
                                        property_name: prop_name,
                                        field,
                                    },
                                    value,
                                );
                            }
                        }
                    }

                    if let Some(layout_val) = obj.get_mut("layout")
                        && let Some(layout_obj) = layout_val.as_object_mut()
                    {
                        for (prop_name, field) in &type_def.layout {
                            if let Some(value) = layout_obj.get_mut(prop_name.as_str()) {
                                visitor.visit_property(
                                    &PropertyContext {
                                        layer_index: i,
                                        layer_type,
                                        section: MirPropertySection::Layout,
                                        property_name: prop_name,
                                        field,
                                    },
                                    value,
                                );
                            }
                        }
                    }
                }
            } // `obj` borrow released here

            visitor.visit_layer(i, layer_type, layer);
        }
    } // layers / root / style borrows released here

    visitor.visit_root(style);
}

// ── Layer-type resolution ─────────────────────────────────────────────────────

/// Collect the effective layer type for every layer in the style (following `ref` chains).
/// Returns `None` for layers whose type cannot be resolved.
pub(crate) fn collect_layer_types(style: &Value) -> Vec<Option<String>> {
    let Some(root) = style.as_object() else {
        return vec![];
    };
    let Some(layers) = root.get("layers").and_then(Value::as_array) else {
        return vec![];
    };

    let by_id: HashMap<String, usize> = layers
        .iter()
        .enumerate()
        .filter_map(|(i, l)| {
            let id = l.as_object()?.get("id")?.as_str()?;
            Some((id.to_string(), i))
        })
        .collect();

    (0..layers.len())
        .map(|i| resolve_layer_type(i, layers, &by_id))
        .collect()
}

fn resolve_layer_type(
    start: usize,
    layers: &[Value],
    by_id: &HashMap<String, usize>,
) -> Option<String> {
    let mut current = start;
    let mut visited = HashSet::new();
    loop {
        if !visited.insert(current) {
            return None; // cycle
        }
        let obj = layers.get(current)?.as_object()?;
        if let Some(t) = obj.get("type").and_then(Value::as_str) {
            return Some(t.to_string());
        }
        let r = obj.get("ref").and_then(Value::as_str)?;
        current = *by_id.get(r)?;
    }
}

// ── Typed filter visitor ─────────────────────────────────────────────────────

/// Visitor for typed filter passes operating directly on [`Boolean`].
pub(crate) trait TypedFilterVisitor {
    /// Called for each layer's filter. Return `true` if the filter was modified.
    fn visit_filter(&mut self, layer_index: usize, layer_type: &str, filter: &mut Boolean);
}

/// Walk all typed layers and visit their filters.
pub(crate) fn walk_typed_filters(
    style: &mut MaplibreStyleSpecification,
    visitor: &mut impl TypedFilterVisitor,
) {
    for (i, layer) in style.layers.iter_mut().enumerate() {
        let AnyLayer::Typed(t) = layer else {
            continue;
        };
        if t.common().filter.is_some() {
            let layer_type = t.layer_type().to_string();
            if let Some(ref mut filter) = t.common_mut().filter {
                visitor.visit_filter(i, &layer_type, filter);
            }
        }
    }
}

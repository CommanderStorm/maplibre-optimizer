//! Read-only analysis pass that collects how every tile data field is used across all layers.
//!
//! Walks expressions recursively to build [`FieldAnalysis`]: a map from
//! `(source, source_layer)` → property name → [`FieldUsage`].

use std::collections::{BTreeMap, BTreeSet};

use maplibre_style_spec::mir::IntermediateSpec;
use serde_json::Value;

use super::expr_util::{extract_json_literal, get_prop_name};
use super::source_util::VectorLayerInfo;
use super::walk::{PropertyContext, StyleVisitor, walk_style_mut};
use crate::advisory::SourceLayerKey;

/// How a single tile data field is used across the style.
#[derive(Debug, Clone)]
pub struct FieldUsage {
    /// Which layer indices reference this field.
    pub layer_indices: BTreeSet<usize>,
    /// Literal values compared against (`==`, match labels, `in` members).
    /// `None` if the field is used in an open-ended way (interpolate, concat, etc.).
    pub compared_values: Option<Vec<Value>>,
    /// Used in filter context.
    pub in_filter: bool,
    /// Used in paint or layout context.
    pub in_paint_layout: bool,
    /// Field feeds into a continuous/pass-through context.
    pub used_continuously: bool,
}

impl Default for FieldUsage {
    fn default() -> Self {
        Self {
            layer_indices: BTreeSet::new(),
            compared_values: Some(Vec::new()),
            in_filter: false,
            in_paint_layout: false,
            used_continuously: false,
        }
    }
}

/// `(source, source-layer)` → property name → [`FieldUsage`].
pub type FieldAnalysis = BTreeMap<SourceLayerKey, BTreeMap<String, FieldUsage>>;

/// Perform field analysis on a style JSON document.
///
/// The `layer_info` must be pre-computed via [`super::source_util::precompute_vector_layer_info`].
pub fn analyze_fields(
    style: &mut Value,
    mir: &IntermediateSpec,
    layer_info: &[Option<VectorLayerInfo>],
) -> FieldAnalysis {
    let mut visitor = FieldAnalysisVisitor {
        layer_info,
        result: BTreeMap::new(),
    };
    walk_style_mut(style, mir, &mut visitor);
    visitor.result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExprContext {
    Filter,
    PaintLayout,
}

struct FieldAnalysisVisitor<'a> {
    layer_info: &'a [Option<VectorLayerInfo>],
    result: FieldAnalysis,
}

impl FieldAnalysisVisitor<'_> {
    fn key_for_layer(&self, layer_index: usize) -> Option<SourceLayerKey> {
        self.layer_info
            .get(layer_index)?
            .as_ref()
            .map(|info| SourceLayerKey {
                source: info.source.clone(),
                source_layer: info.source_layer.clone(),
            })
    }

    fn analyze_expr(
        &mut self,
        expr: &Value,
        layer_index: usize,
        ctx: ExprContext,
        key: &SourceLayerKey,
    ) {
        let Value::Array(arr) = expr else { return };
        if arr.is_empty() {
            return;
        }
        let Some(op) = arr[0].as_str() else { return };

        match op {
            "get" if arr.len() == 2 => {
                if let Some(prop) = arr[1].as_str() {
                    self.record_reference(key, prop, layer_index, ctx);
                }
            }
            // ["get", prop, obj] — accessing object argument, skip
            "get" if arr.len() == 3 => {}

            "has" if arr.len() == 2 => {
                if let Some(prop) = arr[1].as_str() {
                    self.record_reference(key, prop, layer_index, ctx);
                }
            }

            "==" | "!=" if arr.len() == 3 => {
                // Check if one side is ["get", prop] and other is literal
                if let Some((prop, lit)) =
                    super::expr_util::extract_get_and_literal(&arr[1], &arr[2])
                {
                    self.record_reference(key, prop, layer_index, ctx);
                    self.add_compared_value(key, prop, lit);
                } else {
                    // Recurse into both sides
                    self.analyze_expr(&arr[1], layer_index, ctx, key);
                    self.analyze_expr(&arr[2], layer_index, ctx, key);
                }
            }

            "match" if arr.len() >= 4 => {
                // ["match", input, label1, out1, ..., fallback]
                let input = &arr[1];
                if let Some(prop) = get_prop_name(input) {
                    self.record_reference(key, prop, layer_index, ctx);
                    // Collect all labels (even indices starting at 2, excluding last which is
                    // fallback)
                    let label_output_pairs = arr.len() - 3; // subtract op, input, fallback
                    let pair_count = label_output_pairs / 2;
                    for i in 0..pair_count {
                        let label_idx = 2 + i * 2;
                        let label = &arr[label_idx];
                        // Labels can be single values or arrays of values
                        if let Some(values) = label.as_array() {
                            for v in values {
                                if let Some(lit) = extract_json_literal(v) {
                                    self.add_compared_value(key, prop, lit);
                                }
                            }
                        } else if let Some(lit) = extract_json_literal(label) {
                            self.add_compared_value(key, prop, lit);
                        }
                    }
                } else {
                    // Input isn't a simple get — recurse into everything
                    for child in &arr[1..] {
                        self.analyze_expr(child, layer_index, ctx, key);
                    }
                }
            }

            "in" if arr.len() == 3 => {
                let input = &arr[1];
                let values_expr = &arr[2];
                if let Some(prop) = get_prop_name(input) {
                    self.record_reference(key, prop, layer_index, ctx);
                    // Extract literal array members
                    if let Value::Array(lit_arr) = values_expr
                        && lit_arr.len() == 2
                        && lit_arr[0].as_str() == Some("literal")
                        && let Value::Array(members) = &lit_arr[1]
                    {
                        for m in members {
                            if let Some(lit) = extract_json_literal(m) {
                                self.add_compared_value(key, prop, lit);
                            }
                        }
                    }
                } else {
                    self.analyze_expr(input, layer_index, ctx, key);
                    self.analyze_expr(values_expr, layer_index, ctx, key);
                }
            }

            // Continuous/pass-through contexts (value passes through unmodified)
            "interpolate" | "interpolate-hcl" | "interpolate-lab" | "concat" | "to-string"
            | "to-number" | "to-boolean" | "format" | "image" | "number-format" | "coalesce" => {
                self.analyze_continuous_children(arr, layer_index, ctx, key);
            }

            "properties" => {
                // Mark ALL columns as referenced — we can't know which are used
                let fields = self.result.entry(key.clone()).or_default();
                // We can't enumerate all properties here — this is handled at advisory generation
                // by checking if any layer uses ["properties"]. Insert a sentinel.
                let usage = fields.entry("__all_properties__".to_string()).or_default();
                usage.layer_indices.insert(layer_index);
                usage.used_continuously = true;
                usage.compared_values = None;
                match ctx {
                    ExprContext::Filter => usage.in_filter = true,
                    ExprContext::PaintLayout => usage.in_paint_layout = true,
                }
            }

            // Generic recursion for all other operators
            _ => {
                for child in &arr[1..] {
                    self.analyze_expr(child, layer_index, ctx, key);
                }
            }
        }
    }

    fn analyze_continuous_children(
        &mut self,
        arr: &[Value],
        layer_index: usize,
        ctx: ExprContext,
        key: &SourceLayerKey,
    ) {
        for child in &arr[1..] {
            if let Some(prop) = get_prop_name(child) {
                self.record_reference(key, prop, layer_index, ctx);
                self.mark_continuous(key, prop);
            } else {
                self.analyze_expr(child, layer_index, ctx, key);
            }
        }
    }

    fn record_reference(
        &mut self,
        key: &SourceLayerKey,
        prop: &str,
        layer_index: usize,
        ctx: ExprContext,
    ) {
        let fields = self.result.entry(key.clone()).or_default();
        let usage = fields.entry(prop.to_string()).or_default();
        usage.layer_indices.insert(layer_index);
        match ctx {
            ExprContext::Filter => usage.in_filter = true,
            ExprContext::PaintLayout => usage.in_paint_layout = true,
        }
    }

    fn add_compared_value(&mut self, key: &SourceLayerKey, prop: &str, value: Value) {
        let fields = self.result.entry(key.clone()).or_default();
        let usage = fields.entry(prop.to_string()).or_default();
        if let Some(ref mut cv) = usage.compared_values
            && !cv.contains(&value)
        {
            cv.push(value);
        }
        // If compared_values is None, the field is already used_continuously — don't add back
    }

    fn mark_continuous(&mut self, key: &SourceLayerKey, prop: &str) {
        let fields = self.result.entry(key.clone()).or_default();
        let usage = fields.entry(prop.to_string()).or_default();
        usage.used_continuously = true;
        usage.compared_values = None;
    }
}

impl StyleVisitor for FieldAnalysisVisitor<'_> {
    fn visit_filter(&mut self, layer_index: usize, _layer_type: &str, filter: &mut Value) {
        let Some(key) = self.key_for_layer(layer_index) else {
            return;
        };
        self.analyze_expr(filter, layer_index, ExprContext::Filter, &key);
    }

    fn visit_property(&mut self, ctx: &PropertyContext<'_>, value: &mut Value) {
        let Some(key) = self.key_for_layer(ctx.layer_index) else {
            return;
        };
        self.analyze_expr(value, ctx.layer_index, ExprContext::PaintLayout, &key);
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use serde_json::json;

    use super::*;
    use crate::load_intermediate_spec_from_v8_path;

    fn sample_mir() -> IntermediateSpec {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../upstream/src/reference/v8.json");
        load_intermediate_spec_from_v8_path(&path).expect("v8.json")
    }

    fn make_layer_info(source: &str, source_layer: &str) -> Vec<Option<VectorLayerInfo>> {
        vec![Some(VectorLayerInfo {
            source: source.to_string(),
            source_layer: source_layer.to_string(),
        })]
    }

    #[test]
    fn basic_get_in_filter() {
        let mir = sample_mir();
        let mut style = json!({
            "version": 8,
            "sources": {"src": {"type": "vector"}},
            "layers": [{
                "id": "l",
                "type": "fill",
                "source": "src",
                "source-layer": "sl",
                "filter": ["==", ["get", "class"], "water"]
            }]
        });
        let info = make_layer_info("src", "sl");
        let analysis = analyze_fields(&mut style, &mir, &info);

        let key = SourceLayerKey {
            source: "src".into(),
            source_layer: "sl".into(),
        };
        let usage = &analysis[&key]["class"];
        assert!(usage.in_filter);
        assert!(!usage.in_paint_layout);
        assert!(!usage.used_continuously);
        assert!(usage.layer_indices.contains(&0));
        let cv = usage.compared_values.as_ref().unwrap();
        assert!(cv.contains(&json!("water")));
    }

    #[test]
    fn get_in_paint_marks_paint_layout() {
        let mir = sample_mir();
        let mut style = json!({
            "version": 8,
            "sources": {"src": {"type": "vector"}},
            "layers": [{
                "id": "l",
                "type": "fill",
                "source": "src",
                "source-layer": "sl",
                "paint": {
                    "fill-color": ["match", ["get", "class"], "water", "#00f", "#ccc"]
                }
            }]
        });
        let info = make_layer_info("src", "sl");
        let analysis = analyze_fields(&mut style, &mir, &info);

        let key = SourceLayerKey {
            source: "src".into(),
            source_layer: "sl".into(),
        };
        let usage = &analysis[&key]["class"];
        assert!(!usage.in_filter);
        assert!(usage.in_paint_layout);
        let cv = usage.compared_values.as_ref().unwrap();
        assert!(cv.contains(&json!("water")));
    }

    #[test]
    fn interpolate_marks_continuous() {
        let mir = sample_mir();
        let mut style = json!({
            "version": 8,
            "sources": {"src": {"type": "vector"}},
            "layers": [{
                "id": "l",
                "type": "fill",
                "source": "src",
                "source-layer": "sl",
                "paint": {
                    "fill-opacity": ["interpolate", ["linear"], ["get", "rank"], 0, 0.0, 10, 1.0]
                }
            }]
        });
        let info = make_layer_info("src", "sl");
        let analysis = analyze_fields(&mut style, &mir, &info);

        let key = SourceLayerKey {
            source: "src".into(),
            source_layer: "sl".into(),
        };
        let usage = &analysis[&key]["rank"];
        assert!(usage.used_continuously);
        assert!(usage.compared_values.is_none());
    }

    #[test]
    fn match_with_array_labels() {
        let mir = sample_mir();
        let mut style = json!({
            "version": 8,
            "sources": {"src": {"type": "vector"}},
            "layers": [{
                "id": "l",
                "type": "fill",
                "source": "src",
                "source-layer": "sl",
                "filter": ["match", ["get", "class"], ["water", "river"], true, false]
            }]
        });
        let info = make_layer_info("src", "sl");
        let analysis = analyze_fields(&mut style, &mir, &info);

        let key = SourceLayerKey {
            source: "src".into(),
            source_layer: "sl".into(),
        };
        let cv = analysis[&key]["class"].compared_values.as_ref().unwrap();
        assert!(cv.contains(&json!("water")));
        assert!(cv.contains(&json!("river")));
    }

    #[test]
    fn in_expr_collects_members() {
        let mir = sample_mir();
        let mut style = json!({
            "version": 8,
            "sources": {"src": {"type": "vector"}},
            "layers": [{
                "id": "l",
                "type": "fill",
                "source": "src",
                "source-layer": "sl",
                "filter": ["in", ["get", "class"], ["literal", ["water", "river"]]]
            }]
        });
        let info = make_layer_info("src", "sl");
        let analysis = analyze_fields(&mut style, &mir, &info);

        let key = SourceLayerKey {
            source: "src".into(),
            source_layer: "sl".into(),
        };
        let cv = analysis[&key]["class"].compared_values.as_ref().unwrap();
        assert!(cv.contains(&json!("water")));
        assert!(cv.contains(&json!("river")));
    }

    #[test]
    fn three_element_get_is_skipped() {
        let mir = sample_mir();
        let mut style = json!({
            "version": 8,
            "sources": {"src": {"type": "vector"}},
            "layers": [{
                "id": "l",
                "type": "fill",
                "source": "src",
                "source-layer": "sl",
                "filter": ["==", ["get", "prop", ["literal", {}]], "val"]
            }]
        });
        let info = make_layer_info("src", "sl");
        let analysis = analyze_fields(&mut style, &mir, &info);

        let key = SourceLayerKey {
            source: "src".into(),
            source_layer: "sl".into(),
        };
        // Should not record "prop" since it's a 3-element get
        assert!(analysis.get(&key).is_none() || !analysis[&key].contains_key("prop"));
    }

    #[test]
    fn has_records_reference_no_values() {
        let mir = sample_mir();
        let mut style = json!({
            "version": 8,
            "sources": {"src": {"type": "vector"}},
            "layers": [{
                "id": "l",
                "type": "fill",
                "source": "src",
                "source-layer": "sl",
                "filter": ["has", "class"]
            }]
        });
        let info = make_layer_info("src", "sl");
        let analysis = analyze_fields(&mut style, &mir, &info);

        let key = SourceLayerKey {
            source: "src".into(),
            source_layer: "sl".into(),
        };
        let usage = &analysis[&key]["class"];
        assert!(usage.in_filter);
        // compared_values should be Some(empty set) since "has" doesn't compare values
        let cv = usage.compared_values.as_ref().unwrap();
        assert!(cv.is_empty());
    }

    #[test]
    fn coalesce_marks_continuous() {
        let mir = sample_mir();
        let mut style = json!({
            "version": 8,
            "sources": {"src": {"type": "vector"}},
            "layers": [{
                "id": "l",
                "type": "fill",
                "source": "src",
                "source-layer": "sl",
                "paint": {
                    "fill-color": ["coalesce", ["get", "color"], "#000"]
                }
            }]
        });
        let info = make_layer_info("src", "sl");
        let analysis = analyze_fields(&mut style, &mir, &info);

        let key = SourceLayerKey {
            source: "src".into(),
            source_layer: "sl".into(),
        };
        let usage = &analysis[&key]["color"];
        assert!(usage.used_continuously);
        assert!(usage.compared_values.is_none());
    }
}

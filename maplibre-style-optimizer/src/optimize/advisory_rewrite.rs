//! Expression rewriting to match a [`DataRewriteAdvisory`]'s string→integer encodings.
//!
//! For each [`EncodeStringAdvisory`], this pass rewrites comparison literals in
//! `==`, `!=`, `match`, and `in` expressions to use the integer encoding.

use std::collections::BTreeMap;

use maplibre_style_spec::mir::IntermediateSpec;
use serde_json::Value;

use super::expr_util::get_prop_name;
use super::source_util::VectorLayerInfo;
use super::walk::{PropertyContext, StyleVisitor, walk_style_mut};
use crate::advisory::{DataRewriteAdvisory, EncodeStringAdvisory, SourceLayerKey};

/// Apply advisory-driven expression rewrites to a style JSON.
pub fn apply_advisory(
    style: &mut Value,
    mir: &IntermediateSpec,
    layer_info: &[Option<VectorLayerInfo>],
    advisory: &DataRewriteAdvisory,
) {
    // Build a lookup: (source, source-layer, property) → EncodeStringAdvisory
    let mut encodings: BTreeMap<(SourceLayerKey, String), &EncodeStringAdvisory> = BTreeMap::new();
    for (key, sl_advisory) in &advisory.rewrites {
        for enc in &sl_advisory.encode_strings {
            encodings.insert((key.clone(), enc.property.clone()), enc);
        }
    }

    if encodings.is_empty() {
        return;
    }

    let mut visitor = AdvisoryRewriteVisitor {
        layer_info,
        encodings,
    };
    walk_style_mut(style, mir, &mut visitor);
}

struct AdvisoryRewriteVisitor<'a> {
    layer_info: &'a [Option<VectorLayerInfo>],
    encodings: BTreeMap<(SourceLayerKey, String), &'a EncodeStringAdvisory>,
}

impl AdvisoryRewriteVisitor<'_> {
    fn key_for_layer(&self, layer_index: usize) -> Option<SourceLayerKey> {
        self.layer_info
            .get(layer_index)?
            .as_ref()
            .map(|info| SourceLayerKey {
                source: info.source.clone(),
                source_layer: info.source_layer.clone(),
            })
    }

    fn lookup_encoding(&self, key: &SourceLayerKey, prop: &str) -> Option<&EncodeStringAdvisory> {
        self.encodings
            .get(&(key.clone(), prop.to_string()))
            .copied()
    }

    fn rewrite_expr(&self, expr: &mut Value, key: &SourceLayerKey) {
        let Value::Array(arr) = expr else { return };
        if arr.is_empty() {
            return;
        }
        let Some(op) = arr[0].as_str().map(String::from) else {
            return;
        };

        match op.as_str() {
            "==" | "!=" if arr.len() == 3 => {
                self.rewrite_binary_comparison(arr, key);
            }
            "match" if arr.len() >= 4 => {
                self.rewrite_match(arr, key);
            }
            "in" if arr.len() == 3 => {
                self.rewrite_in(arr, key);
            }
            _ => {
                // Recurse into sub-expressions
                for child in arr.iter_mut().skip(1) {
                    self.rewrite_expr(child, key);
                }
            }
        }
    }

    /// Rewrite `["==" | "!=", ["get", prop], literal]` or vice versa.
    fn rewrite_binary_comparison(&self, arr: &mut [Value], key: &SourceLayerKey) {
        let (prop_idx, lit_idx) = if get_prop_name(&arr[1]).is_some() {
            (1, 2)
        } else if get_prop_name(&arr[2]).is_some() {
            (2, 1)
        } else {
            // Neither side is a get — recurse
            for child in arr.iter_mut().skip(1) {
                self.rewrite_expr(child, key);
            }
            return;
        };

        let prop = match get_prop_name(&arr[prop_idx]) {
            Some(p) => p.to_string(),
            None => return,
        };

        let Some(enc) = self.lookup_encoding(key, &prop) else {
            // No encoding for this property — recurse into the literal side
            self.rewrite_expr(&mut arr[lit_idx], key);
            return;
        };

        if let Some(s) = arr[lit_idx].as_str()
            && let Some(&int_val) = enc.mapping.get(s)
        {
            arr[lit_idx] = Value::from(int_val);
        }
    }

    /// Rewrite `["match", ["get", prop], label1, out1, ..., fallback]`.
    fn rewrite_match(&self, arr: &mut [Value], key: &SourceLayerKey) {
        let input = &arr[1];
        let Some(prop) = get_prop_name(input).map(String::from) else {
            // Input isn't a simple get — recurse into everything
            for child in arr.iter_mut().skip(1) {
                self.rewrite_expr(child, key);
            }
            return;
        };

        let Some(enc) = self.lookup_encoding(key, &prop) else {
            // No encoding — recurse into output expressions
            for child in arr.iter_mut().skip(2) {
                self.rewrite_expr(child, key);
            }
            return;
        };

        // Rewrite labels (even indices starting at 2, excluding last fallback)
        let label_output_pairs = arr.len() - 3;
        let pair_count = label_output_pairs / 2;
        for i in 0..pair_count {
            let label_idx = 2 + i * 2;
            let output_idx = label_idx + 1;

            rewrite_match_label(&mut arr[label_idx], enc);
            self.rewrite_expr(&mut arr[output_idx], key);
        }

        // Recurse into fallback (last element)
        if let Some(fb) = arr.last_mut() {
            self.rewrite_expr(fb, key);
        }
    }

    /// Rewrite `["in", ["get", prop], ["literal", [v1, v2, ...]]]`.
    fn rewrite_in(&self, arr: &mut [Value], key: &SourceLayerKey) {
        let input = &arr[1];
        let Some(prop) = get_prop_name(input).map(String::from) else {
            for child in arr.iter_mut().skip(1) {
                self.rewrite_expr(child, key);
            }
            return;
        };

        let Some(enc) = self.lookup_encoding(key, &prop) else {
            return;
        };

        // arr[2] should be ["literal", [...]]
        if let Value::Array(lit_arr) = &mut arr[2]
            && lit_arr.len() == 2
            && lit_arr[0].as_str() == Some("literal")
            && let Value::Array(members) = &mut lit_arr[1]
        {
            for m in members.iter_mut() {
                if let Some(s) = m.as_str().map(String::from)
                    && let Some(&int_val) = enc.mapping.get(&s)
                {
                    *m = Value::from(int_val);
                }
            }
        }
    }
}

/// Rewrite a match label: can be a single value or an array of values.
fn rewrite_match_label(label: &mut Value, enc: &EncodeStringAdvisory) {
    match label {
        Value::String(s) => {
            if let Some(&int_val) = enc.mapping.get(s.as_str()) {
                *label = Value::from(int_val);
            }
        }
        Value::Array(values) => {
            for v in values.iter_mut() {
                if let Some(s) = v.as_str().map(String::from)
                    && let Some(&int_val) = enc.mapping.get(&s)
                {
                    *v = Value::from(int_val);
                }
            }
        }
        _ => {}
    }
}

impl StyleVisitor for AdvisoryRewriteVisitor<'_> {
    fn visit_filter(&mut self, layer_index: usize, _layer_type: &str, filter: &mut Value) {
        let Some(key) = self.key_for_layer(layer_index) else {
            return;
        };
        self.rewrite_expr(filter, &key);
    }

    fn visit_property(&mut self, ctx: &PropertyContext<'_>, value: &mut Value) {
        let Some(key) = self.key_for_layer(ctx.layer_index) else {
            return;
        };
        self.rewrite_expr(value, &key);
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use indexmap::IndexMap;
    use serde_json::json;

    use super::*;
    use crate::advisory::{SourceLayerAdvisory, SourceLayerKey};
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

    fn make_advisory(
        source: &str,
        source_layer: &str,
        prop: &str,
        values: &[(&str, i64)],
        sentinel: Option<i64>,
    ) -> DataRewriteAdvisory {
        let mut mapping = IndexMap::new();
        for (s, i) in values {
            mapping.insert((*s).to_string(), *i);
        }
        let mut advisory = DataRewriteAdvisory::default();
        let mut sl = SourceLayerAdvisory::default();
        sl.encode_strings.push(EncodeStringAdvisory {
            property: prop.to_string(),
            mapping,
            unmapped_sentinel: sentinel,
        });
        advisory.rewrites.insert(
            SourceLayerKey {
                source: source.into(),
                source_layer: source_layer.into(),
            },
            sl,
        );
        advisory
    }

    #[test]
    fn rewrite_eq() {
        let mir = sample_mir();
        let info = make_layer_info("src", "sl");
        let advisory = make_advisory("src", "sl", "class", &[("water", 0), ("forest", 1)], None);

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

        apply_advisory(&mut style, &mir, &info, &advisory);
        assert_eq!(
            style["layers"][0]["filter"],
            json!(["==", ["get", "class"], 0])
        );
    }

    #[test]
    fn rewrite_neq() {
        let mir = sample_mir();
        let info = make_layer_info("src", "sl");
        let advisory = make_advisory("src", "sl", "class", &[("water", 0)], None);

        let mut style = json!({
            "version": 8,
            "sources": {"src": {"type": "vector"}},
            "layers": [{
                "id": "l",
                "type": "fill",
                "source": "src",
                "source-layer": "sl",
                "filter": ["!=", ["get", "class"], "water"]
            }]
        });

        apply_advisory(&mut style, &mir, &info, &advisory);
        assert_eq!(
            style["layers"][0]["filter"],
            json!(["!=", ["get", "class"], 0])
        );
    }

    #[test]
    fn rewrite_match_labels() {
        let mir = sample_mir();
        let info = make_layer_info("src", "sl");
        let advisory = make_advisory(
            "src",
            "sl",
            "class",
            &[("water", 0), ("forest", 1)],
            Some(2),
        );

        let mut style = json!({
            "version": 8,
            "sources": {"src": {"type": "vector"}},
            "layers": [{
                "id": "l",
                "type": "fill",
                "source": "src",
                "source-layer": "sl",
                "paint": {
                    "fill-color": ["match", ["get", "class"],
                        "water", "#00f",
                        "forest", "#0f0",
                        "#ccc"
                    ]
                }
            }]
        });

        apply_advisory(&mut style, &mir, &info, &advisory);
        let paint = &style["layers"][0]["paint"]["fill-color"];
        assert_eq!(
            *paint,
            json!(["match", ["get", "class"], 0, "#00f", 1, "#0f0", "#ccc"])
        );
    }

    #[test]
    fn rewrite_match_array_labels() {
        let mir = sample_mir();
        let info = make_layer_info("src", "sl");
        let advisory = make_advisory("src", "sl", "class", &[("water", 0), ("forest", 1)], None);

        let mut style = json!({
            "version": 8,
            "sources": {"src": {"type": "vector"}},
            "layers": [{
                "id": "l",
                "type": "fill",
                "source": "src",
                "source-layer": "sl",
                "filter": ["match", ["get", "class"], ["water", "forest"], true, false]
            }]
        });

        apply_advisory(&mut style, &mir, &info, &advisory);
        let filter = &style["layers"][0]["filter"];
        assert_eq!(
            *filter,
            json!(["match", ["get", "class"], [0, 1], true, false])
        );
    }

    #[test]
    fn rewrite_in_members() {
        let mir = sample_mir();
        let info = make_layer_info("src", "sl");
        let advisory = make_advisory("src", "sl", "class", &[("water", 0), ("forest", 1)], None);

        let mut style = json!({
            "version": 8,
            "sources": {"src": {"type": "vector"}},
            "layers": [{
                "id": "l",
                "type": "fill",
                "source": "src",
                "source-layer": "sl",
                "filter": ["in", ["get", "class"], ["literal", ["water", "forest"]]]
            }]
        });

        apply_advisory(&mut style, &mir, &info, &advisory);
        assert_eq!(
            style["layers"][0]["filter"],
            json!(["in", ["get", "class"], ["literal", [0, 1]]])
        );
    }

    #[test]
    fn no_rewrite_for_unmapped_property() {
        let mir = sample_mir();
        let info = make_layer_info("src", "sl");
        let advisory = make_advisory("src", "sl", "class", &[("water", 0)], None);

        let mut style = json!({
            "version": 8,
            "sources": {"src": {"type": "vector"}},
            "layers": [{
                "id": "l",
                "type": "fill",
                "source": "src",
                "source-layer": "sl",
                "filter": ["==", ["get", "other_prop"], "something"]
            }]
        });

        let original = style.clone();
        apply_advisory(&mut style, &mir, &info, &advisory);
        assert_eq!(style, original);
    }

    #[test]
    fn idempotent_double_apply() {
        let mir = sample_mir();
        let info = make_layer_info("src", "sl");
        let advisory = make_advisory("src", "sl", "class", &[("water", 0), ("forest", 1)], None);

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

        apply_advisory(&mut style, &mir, &info, &advisory);
        let after_first = style.clone();
        apply_advisory(&mut style, &mir, &info, &advisory);
        assert_eq!(style, after_first);
    }
}

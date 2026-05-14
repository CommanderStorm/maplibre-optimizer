//! Layer merging: collapse adjacent same-type/source/source-layer layers into
//! fewer layers with data-driven `case`/`match` expressions and synthesised sort-keys.
//!
//! **Phase 1** — literal-only properties, identical zoom, no existing sort-key.
//! **Phase 2** — `match` optimisation when all filters are `["==", ["get", P], L]`.
//! **Phase 3** — zoom-tolerant merging: layers with different minzoom/maxzoom are
//!   merged by wrapping each sub-layer's filter contribution with zoom guards.

use maplibre_style_spec::mir::types::MirType;
use maplibre_style_spec::mir::{MirPropertySection, MirSpec};
use serde_json::Value;

/// Layer types that support a sort-key layout property.
fn sort_key_name(layer_type: &str) -> Option<&'static str> {
    match layer_type {
        "fill" => Some("fill-sort-key"),
        "line" => Some("line-sort-key"),
        "circle" => Some("circle-sort-key"),
        "symbol" => Some("symbol-sort-key"),
        _ => None,
    }
}

/// Merge adjacent layers that share type, source, source-layer, and zoom range.
///
/// Synthesises `case`/`match` expressions for differing paint/layout properties
/// and a sort-key to preserve inter-layer draw order.
pub fn layer_merge(v: &mut Value, mir: &MirSpec) {
    let Some(layers) = v.get_mut("layers").and_then(Value::as_array_mut) else {
        return;
    };

    let groups = find_merge_groups(layers, mir);
    if groups.is_empty() {
        return;
    }

    let old_layers = std::mem::take(layers);
    let mut i = 0;

    for &(start, end) in &groups {
        while i < start {
            layers.push(old_layers[i].clone());
            i += 1;
        }
        layers.push(build_merged_layer(&old_layers[start..end], mir));
        i = end;
    }
    while i < old_layers.len() {
        layers.push(old_layers[i].clone());
        i += 1;
    }
}

// ── Grouping ─────────────────────────────────────────────────────────────────

/// Returns `(start, end)` pairs of mergeable adjacent layer runs.
fn find_merge_groups(layers: &[Value], mir: &MirSpec) -> Vec<(usize, usize)> {
    let mut groups = Vec::new();
    let mut i = 0;

    while i < layers.len() {
        if !is_merge_candidate(&layers[i]) {
            i += 1;
            continue;
        }

        let start = i;
        i += 1;

        while i < layers.len()
            && is_merge_candidate(&layers[i])
            && same_group_key(&layers[start], &layers[i])
        {
            i += 1;
        }

        let layer_type = layers[start]["type"].as_str().expect("validated");
        if i - start >= 2 && differing_props_are_mergeable(&layers[start..i], layer_type, mir) {
            groups.push((start, i));
        }
    }

    groups
}

/// A layer is a merge candidate if it is a typed layer whose type supports
/// sort-key, has a filter, and has no existing sort-key property.
fn is_merge_candidate(layer: &Value) -> bool {
    let Some(lt) = layer.get("type").and_then(Value::as_str) else {
        return false;
    };
    // Symbol layers cannot be merged: MapLibre performs collision/overlap
    // detection per-layer, so combining symbol layers changes which labels
    // can suppress each other.
    if lt == "symbol" {
        return false;
    }
    let Some(sk) = sort_key_name(lt) else {
        return false;
    };
    // Must have a non-trivial filter (skip absent, null, and dead `false` filters).
    let dominated = |f: &Value| f.is_null() || *f == Value::Bool(false);
    if layer.get("filter").is_none_or(dominated) {
        return false;
    }
    // Must not already have a sort-key.
    !has_layout_key(layer, sk)
}

fn same_group_key(a: &Value, b: &Value) -> bool {
    a.get("type") == b.get("type")
        && a.get("source") == b.get("source")
        && a.get("source-layer") == b.get("source-layer")
}

fn has_layout_key(layer: &Value, key: &str) -> bool {
    layer
        .get("layout")
        .and_then(Value::as_object)
        .is_some_and(|o| o.contains_key(key))
}

/// Every differing paint/layout property across the group must be mergeable.
///
/// A property is mergeable if either:
/// - It is uniform across all layers (same value everywhere, including arrays)
/// - Its MIR field has `expression.feature == true`, meaning it supports
///   feature-dependent expressions and can be wrapped in `case`/`match`
///
/// Properties whose MIR field lacks feature-expression support (camera-only
/// properties like `line-dasharray`, `*-translate`) must be uniform to merge.
fn differing_props_are_mergeable(layers: &[Value], layer_type: &str, mir: &MirSpec) -> bool {
    for (section, mir_section) in [
        ("paint", MirPropertySection::Paint),
        ("layout", MirPropertySection::Layout),
    ] {
        let props = collect_property_names(layers, section);
        for prop in &props {
            if prop == "visibility" {
                continue;
            }
            let values: Vec<Option<&Value>> = layers
                .iter()
                .map(|l| l.get(section).and_then(|s| s.get(prop.as_str())))
                .collect();
            let first = values.iter().flatten().next();
            let uniform = first.is_some_and(|f| values.iter().all(|v| v.is_some_and(|x| x == *f)));
            if uniform {
                continue;
            }
            // Property differs — check if the field supports feature-driven expressions.
            let field = mir.layers.field_for(layer_type, mir_section, prop);
            let is_feature_driven = field
                .and_then(|f| f.expression.as_ref())
                .is_some_and(|e| e.feature);
            if !is_feature_driven {
                return false;
            }
            // Array-typed properties (e.g. `line-dasharray`, `*-translate`,
            // `*-offset`) are classified as feature-driven in the spec but
            // MapLibre GL JS's expression compiler cannot actually evaluate
            // complex `case`/`match` expressions that return arrays — the
            // rendering loop hangs on styles that try. Treat any array-valued
            // property as non-mergeable when values differ.
            if field.is_some_and(|f| is_array_type(&f.r#type)) {
                return false;
            }
            // Values containing zoom-dependent expressions (interpolate/step
            // over ["zoom"]) cannot be placed inside case/match arms — zoom
            // interpolation must be at the top level of a property expression.
            if values
                .iter()
                .flatten()
                .any(|v| super::zoom::is_zoom_ramp(v))
            {
                return false;
            }
        }
    }
    true
}

/// Return true for MIR types whose values are always JSON arrays. Feature-
/// driven `case`/`match` expressions returning arrays are nominally allowed
/// by the spec but not actually supported by `MapLibre GL JS`'s runtime, so
/// layer merging must treat these conservatively as non-mergeable.
fn is_array_type(ty: &MirType) -> bool {
    matches!(
        ty,
        MirType::Array { .. }
            | MirType::NumberArray { .. }
            | MirType::ColorArray
            | MirType::VariableAnchorOffsetCollection
    )
}

fn collect_property_names(layers: &[Value], section: &str) -> Vec<String> {
    let mut names = Vec::new();
    for l in layers {
        if let Some(obj) = l.get(section).and_then(Value::as_object) {
            for k in obj.keys() {
                if !names.contains(k) {
                    names.push(k.clone());
                }
            }
        }
    }
    names
}

// ── Match-pattern detection (Phase 2) ────────────────────────────────────────

struct MatchPattern {
    property: String,
    labels: Vec<Value>,
}

/// Detect whether every filter is `["==", ["get", P], L]` (or reversed) on the
/// same property with unique literal labels.
fn detect_match_pattern(layers: &[Value]) -> Option<MatchPattern> {
    let mut prop: Option<String> = None;
    let mut labels = Vec::with_capacity(layers.len());

    for l in layers {
        let filter = l.get("filter")?.as_array()?;
        if filter.len() != 3 || filter[0].as_str()? != "==" {
            return None;
        }

        let (get_expr, literal) = if is_get_expr(&filter[1]) {
            (&filter[1], &filter[2])
        } else if is_get_expr(&filter[2]) {
            (&filter[2], &filter[1])
        } else {
            return None;
        };

        if !(literal.is_string() || literal.is_number()) {
            return None;
        }

        let p = get_expr.as_array()?[1].as_str()?;
        match &prop {
            None => prop = Some(p.to_owned()),
            Some(existing) if existing == p => {}
            _ => return None,
        }

        labels.push(literal.clone());
    }

    // Labels must be unique (duplicate labels → fall back to `case`).
    for (i, a) in labels.iter().enumerate() {
        if labels[i + 1..].contains(a) {
            return None;
        }
    }

    Some(MatchPattern {
        property: prop?,
        labels,
    })
}

fn is_get_expr(v: &Value) -> bool {
    v.as_array()
        .is_some_and(|a| a.len() == 2 && a[0].as_str() == Some("get") && a[1].is_string())
}

// ── Zoom range helpers ───────────────────────────────────────────────────────

/// Effective zoom range for a layer (defaults: minzoom=0, maxzoom=24).
fn layer_zoom_range(layer: &Value) -> (f64, f64) {
    let min = layer.get("minzoom").and_then(Value::as_f64).unwrap_or(0.0);
    let max = layer.get("maxzoom").and_then(Value::as_f64).unwrap_or(24.0);
    (min, max)
}

/// Wrap a filter expression with zoom guards when this sub-layer's zoom range
/// `(lo, hi)` is narrower than the merged group range `(group_min, group_max)`.
/// Returns the original filter unmodified if no guard is needed.
fn wrap_filter_with_zoom_guard(
    filter: &Value,
    lo: f64,
    hi: f64,
    group_min: f64,
    group_max: f64,
) -> Value {
    let needs_lo = lo > group_min;
    let needs_hi = hi < group_max;

    if !needs_lo && !needs_hi {
        return filter.clone();
    }

    let mut all_args = vec![Value::String("all".into())];
    let zoom_expr = || Value::Array(vec![Value::String("zoom".into())]);

    if needs_lo {
        all_args.push(Value::Array(vec![
            Value::String(">=".into()),
            zoom_expr(),
            Value::from(lo),
        ]));
    }
    if needs_hi {
        all_args.push(Value::Array(vec![
            Value::String("<".into()),
            zoom_expr(),
            Value::from(hi),
        ]));
    }

    all_args.push(filter.clone());
    Value::Array(all_args)
}

// ── Merged-layer construction ────────────────────────────────────────────────

#[expect(clippy::too_many_lines)]
fn build_merged_layer(layers: &[Value], mir: &MirSpec) -> Value {
    let layer_type = layers[0]["type"].as_str().expect("validated in grouping");
    let sort_key_prop = sort_key_name(layer_type).expect("validated in grouping");
    let n = layers.len();

    // Precompute zoom ranges once to avoid repeated JSON lookups.
    let zoom_ranges: Vec<(f64, f64)> = layers.iter().map(layer_zoom_range).collect();
    let group_min = zoom_ranges
        .iter()
        .map(|&(lo, _)| lo)
        .fold(f64::MAX, f64::min);
    let group_max = zoom_ranges
        .iter()
        .map(|&(_, hi)| hi)
        .fold(f64::MIN, f64::max);
    let any_zoom_differs = zoom_ranges
        .iter()
        .any(|&(lo, hi)| lo > group_min || hi < group_max);

    // Match-pattern detection only works when zoom ranges are uniform
    // (zoom guards would break the simple match-on-property pattern).
    let match_pat = if any_zoom_differs {
        None
    } else {
        detect_match_pattern(layers)
    };

    let mut merged = layers[0].clone();
    let obj = merged.as_object_mut().expect("layer is an object");

    // ── Merged zoom range ────────────────────────────────────────────────────
    if group_min > 0.0 {
        obj.insert("minzoom".into(), Value::from(group_min));
    } else {
        obj.remove("minzoom");
    }
    if group_max < 24.0 {
        obj.insert("maxzoom".into(), Value::from(group_max));
    } else {
        obj.remove("maxzoom");
    }

    // ── Merged filter ────────────────────────────────────────────────────────
    obj.insert(
        "filter".into(),
        build_merged_filter(
            layers,
            match_pat.as_ref(),
            &zoom_ranges,
            group_min,
            group_max,
        ),
    );

    // ── Paint & layout properties ────────────────────────────────────────────
    for section in ["paint", "layout"] {
        let mir_section = if section == "paint" {
            MirPropertySection::Paint
        } else {
            MirPropertySection::Layout
        };
        let props = collect_property_names(layers, section);

        let section_obj = obj
            .entry(section)
            .or_insert_with(|| Value::Object(serde_json::Map::new()));
        let section_map = section_obj.as_object_mut().expect("section is an object");

        for prop in &props {
            if prop == "visibility" {
                continue;
            }

            let values: Vec<Option<&Value>> = layers
                .iter()
                .map(|l| l.get(section).and_then(|s| s.get(prop.as_str())))
                .collect();
            let first = values.iter().flatten().next();
            let uniform = first.is_some_and(|f| values.iter().all(|v| v.is_some_and(|x| x == *f)));

            if uniform {
                if let Some(&val) = first {
                    section_map.insert(prop.clone(), val.clone());
                }
                continue;
            }

            let default = mir
                .layers
                .field_default(layer_type, mir_section, prop)
                .cloned()
                .unwrap_or(Value::Null);

            section_map.insert(
                prop.clone(),
                build_property_expr(
                    layers,
                    &values,
                    &default,
                    match_pat.as_ref(),
                    &zoom_ranges,
                    group_min,
                    group_max,
                ),
            );
        }
    }

    // ── Sort-key synthesis ───────────────────────────────────────────────────
    let layout = obj
        .entry("layout")
        .or_insert_with(|| Value::Object(serde_json::Map::new()));
    layout.as_object_mut().expect("layout is an object").insert(
        sort_key_prop.into(),
        build_sort_key_expr(
            layers,
            n,
            match_pat.as_ref(),
            &zoom_ranges,
            group_min,
            group_max,
        ),
    );

    merged
}

fn build_merged_filter(
    layers: &[Value],
    match_pat: Option<&MatchPattern>,
    zoom_ranges: &[(f64, f64)],
    group_min: f64,
    group_max: f64,
) -> Value {
    if let Some(pat) = match_pat {
        // ["match", ["get", P], [L0, L1, …], true, false]
        Value::Array(vec![
            Value::String("match".into()),
            Value::Array(vec![
                Value::String("get".into()),
                Value::String(pat.property.clone()),
            ]),
            Value::Array(pat.labels.clone()),
            Value::Bool(true),
            Value::Bool(false),
        ])
    } else {
        let mut args = vec![Value::String("any".into())];
        for (l, &(lo, hi)) in layers.iter().zip(zoom_ranges) {
            if let Some(f) = l.get("filter") {
                args.push(wrap_filter_with_zoom_guard(f, lo, hi, group_min, group_max));
            }
        }
        Value::Array(args)
    }
}

/// Build a `case` or `match` expression for a property that differs across layers.
///
/// Uses reverse priority (top-most layer first) for `case` so that overlapping
/// filters get "last painter wins" semantics.
fn build_property_expr(
    layers: &[Value],
    values: &[Option<&Value>],
    default: &Value,
    match_pat: Option<&MatchPattern>,
    zoom_ranges: &[(f64, f64)],
    group_min: f64,
    group_max: f64,
) -> Value {
    let n = layers.len();

    if let Some(pat) = match_pat {
        // ["match", ["get", P], L0, V0, L1, V1, …, default]
        let mut args = vec![
            Value::String("match".into()),
            Value::Array(vec![
                Value::String("get".into()),
                Value::String(pat.property.clone()),
            ]),
        ];
        for (label, value) in pat.labels.iter().zip(values) {
            args.push(label.clone());
            args.push(value.cloned().unwrap_or_else(|| default.clone()));
        }
        args.push(default.clone());
        Value::Array(args)
    } else {
        // ["case", filter_top, val_top, …, filter_bot, val_bot, default]
        let mut args = vec![Value::String("case".into())];
        for i in (0..n).rev() {
            let filter = layers[i].get("filter").expect("validated");
            let (lo, hi) = zoom_ranges[i];
            args.push(wrap_filter_with_zoom_guard(
                filter, lo, hi, group_min, group_max,
            ));
            args.push(values[i].cloned().unwrap_or_else(|| default.clone()));
        }
        args.push(default.clone());
        Value::Array(args)
    }
}

fn build_sort_key_expr(
    layers: &[Value],
    n: usize,
    match_pat: Option<&MatchPattern>,
    zoom_ranges: &[(f64, f64)],
    group_min: f64,
    group_max: f64,
) -> Value {
    if let Some(pat) = match_pat {
        // ["match", ["get", P], L0, 0, L1, 1, …, -1]
        let mut args = vec![
            Value::String("match".into()),
            Value::Array(vec![
                Value::String("get".into()),
                Value::String(pat.property.clone()),
            ]),
        ];
        for (i, label) in pat.labels.iter().enumerate() {
            args.push(label.clone());
            args.push(Value::from(i as u64));
        }
        args.push(Value::from(-1_i64));
        Value::Array(args)
    } else {
        // ["case", filter_top, n-1, …, filter_bot, 0, -1]
        let mut args = vec![Value::String("case".into())];
        for i in (0..n).rev() {
            let filter = layers[i].get("filter").expect("validated");
            let (lo, hi) = zoom_ranges[i];
            args.push(wrap_filter_with_zoom_guard(
                filter, lo, hi, group_min, group_max,
            ));
            args.push(Value::from(i as u64));
        }
        args.push(Value::from(-1_i64));
        Value::Array(args)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::path::Path;

    use serde_json::json;

    use super::*;
    use crate::load_intermediate_spec_from_v8_path;

    fn mir() -> MirSpec {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../upstream/src/reference/v8.json");
        load_intermediate_spec_from_v8_path(&path).expect("v8.json")
    }

    #[test]
    fn merge_two_fill_layers_with_match() {
        let mir = mir();
        let mut v = json!({
            "version": 8,
            "sources": {"s": {"type": "vector"}},
            "layers": [
                {
                    "id": "park",
                    "type": "fill",
                    "source": "s",
                    "source-layer": "landuse",
                    "filter": ["==", ["get", "class"], "park"],
                    "paint": {"fill-color": "green", "fill-opacity": 0.5}
                },
                {
                    "id": "wood",
                    "type": "fill",
                    "source": "s",
                    "source-layer": "landuse",
                    "filter": ["==", ["get", "class"], "wood"],
                    "paint": {"fill-color": "darkgreen", "fill-opacity": 0.5}
                }
            ]
        });
        layer_merge(&mut v, &mir);
        insta::assert_yaml_snapshot!(v, @r##"
        layers:
          - filter:
              - match
              - - get
                - class
              - - park
                - wood
              - true
              - false
            id: park
            layout:
              fill-sort-key:
                - match
                - - get
                  - class
                - park
                - 0
                - wood
                - 1
                - -1
            paint:
              fill-color:
                - match
                - - get
                  - class
                - park
                - green
                - wood
                - darkgreen
                - "#000000"
              fill-opacity: 0.5
            source: s
            source-layer: landuse
            type: fill
        sources:
          s:
            type: vector
        version: 8
        "##);
    }

    #[test]
    fn merge_with_case_for_complex_filters() {
        let mir = mir();
        let mut v = json!({
            "version": 8,
            "sources": {"s": {"type": "vector"}},
            "layers": [
                {
                    "id": "a",
                    "type": "line",
                    "source": "s",
                    "source-layer": "road",
                    "filter": ["all", ["==", ["get", "class"], "motorway"], ["has", "name"]],
                    "paint": {"line-color": "red"}
                },
                {
                    "id": "b",
                    "type": "line",
                    "source": "s",
                    "source-layer": "road",
                    "filter": ["==", ["get", "class"], "trunk"],
                    "paint": {"line-color": "orange"}
                }
            ]
        });
        layer_merge(&mut v, &mir);
        insta::assert_yaml_snapshot!(v, @r##"
        layers:
          - filter:
              - any
              - - all
                - - "=="
                  - - get
                    - class
                  - motorway
                - - has
                  - name
              - - "=="
                - - get
                  - class
                - trunk
            id: a
            layout:
              line-sort-key:
                - case
                - - "=="
                  - - get
                    - class
                  - trunk
                - 1
                - - all
                  - - "=="
                    - - get
                      - class
                    - motorway
                  - - has
                    - name
                - 0
                - -1
            paint:
              line-color:
                - case
                - - "=="
                  - - get
                    - class
                  - trunk
                - orange
                - - all
                  - - "=="
                    - - get
                      - class
                    - motorway
                  - - has
                    - name
                - red
                - "#000000"
            source: s
            source-layer: road
            type: line
        sources:
          s:
            type: vector
        version: 8
        "##);
    }

    #[test]
    fn no_merge_for_unsupported_type() {
        let mir = mir();
        let mut v = json!({
            "version": 8,
            "sources": {"s": {"type": "vector"}},
            "layers": [
                {
                    "id": "a", "type": "fill-extrusion", "source": "s",
                    "source-layer": "building",
                    "filter": ["==", ["get", "type"], "A"],
                    "paint": {"fill-extrusion-color": "#aaa"}
                },
                {
                    "id": "b", "type": "fill-extrusion", "source": "s",
                    "source-layer": "building",
                    "filter": ["==", ["get", "type"], "B"],
                    "paint": {"fill-extrusion-color": "#bbb"}
                }
            ]
        });
        let original = v.clone();
        layer_merge(&mut v, &mir);
        assert_eq!(v, original);
    }

    #[test]
    fn no_merge_when_sort_key_exists() {
        let mir = mir();
        let mut v = json!({
            "version": 8,
            "sources": {"s": {"type": "vector"}},
            "layers": [
                {
                    "id": "a", "type": "line", "source": "s",
                    "source-layer": "road",
                    "filter": ["==", ["get", "class"], "x"],
                    "paint": {"line-color": "#f00"},
                    "layout": {"line-sort-key": 1}
                },
                {
                    "id": "b", "type": "line", "source": "s",
                    "source-layer": "road",
                    "filter": ["==", ["get", "class"], "y"],
                    "paint": {"line-color": "#0f0"}
                }
            ]
        });
        let original = v.clone();
        layer_merge(&mut v, &mir);
        assert_eq!(v, original);
    }

    #[test]
    fn merge_with_differing_zoom_adds_guards() {
        let mir = mir();
        let mut v = json!({
            "version": 8,
            "sources": {"s": {"type": "vector"}},
            "layers": [
                {
                    "id": "a", "type": "fill", "source": "s",
                    "source-layer": "land", "minzoom": 5,
                    "filter": ["==", ["get", "t"], "a"],
                    "paint": {"fill-color": "#aaa"}
                },
                {
                    "id": "b", "type": "fill", "source": "s",
                    "source-layer": "land", "minzoom": 10,
                    "filter": ["==", ["get", "t"], "b"],
                    "paint": {"fill-color": "#bbb"}
                }
            ]
        });
        layer_merge(&mut v, &mir);
        // Layers are now merged despite differing zoom.
        assert_eq!(v["layers"].as_array().unwrap().len(), 1);
        let merged = &v["layers"][0];
        // Merged minzoom is the minimum (5).
        assert_eq!(merged["minzoom"], 5.0);
        // Uses case (not match) because zoom guards break the match pattern.
        assert_eq!(merged["filter"][0], "any");
        // Layer b's filter arm is wrapped with a zoom guard.
        // ["any", filter_a, ["all", [">=", ["zoom"], 10], filter_b]]
        let arm_b = &merged["filter"][2];
        assert_eq!(arm_b[0], "all");
        assert_eq!(arm_b[1][0], ">=");
    }

    #[test]
    fn no_merge_without_filter() {
        let mir = mir();
        let mut v = json!({
            "version": 8,
            "sources": {"s": {"type": "vector"}},
            "layers": [
                {
                    "id": "a", "type": "fill", "source": "s",
                    "source-layer": "land",
                    "filter": ["==", ["get", "t"], "a"],
                    "paint": {"fill-color": "#aaa"}
                },
                {
                    "id": "b", "type": "fill", "source": "s",
                    "source-layer": "land",
                    "paint": {"fill-color": "#bbb"}
                }
            ]
        });
        let original = v.clone();
        layer_merge(&mut v, &mir);
        assert_eq!(v, original);
    }

    #[test]
    fn skip_merge_when_zoom_expression_in_differing_property() {
        let mir = mir();
        let mut v = json!({
            "version": 8,
            "sources": {"s": {"type": "vector"}},
            "layers": [
                {
                    "id": "a", "type": "line", "source": "s",
                    "source-layer": "road",
                    "filter": ["==", ["get", "class"], "x"],
                    "paint": {"line-color": ["interpolate", ["linear"], ["zoom"], 5, "#aaa", 10, "#bbb"]}
                },
                {
                    "id": "b", "type": "line", "source": "s",
                    "source-layer": "road",
                    "filter": ["==", ["get", "class"], "y"],
                    "paint": {"line-color": "#ccc"}
                }
            ]
        });
        layer_merge(&mut v, &mir);
        // Zoom expressions cannot be nested inside case/match arms, so layers
        // with differing zoom-dependent property values must not be merged.
        assert_eq!(v["layers"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn merge_group_with_feature_driven_literal_property() {
        let mir = mir();
        let mut v = json!({
            "version": 8,
            "sources": {"s": {"type": "vector"}},
            "layers": [
                {
                    "id": "a", "type": "line", "source": "s",
                    "source-layer": "road",
                    "filter": ["==", ["get", "class"], "x"],
                    "paint": {"line-color": "#aaa"}
                },
                {
                    "id": "b", "type": "line", "source": "s",
                    "source-layer": "road",
                    "filter": ["==", ["get", "class"], "y"],
                    "paint": {"line-color": "#ccc"}
                }
            ]
        });
        layer_merge(&mut v, &mir);
        // line-color differs but values are literals, so merge is valid.
        assert_eq!(v["layers"].as_array().unwrap().len(), 1);
        let merged = &v["layers"][0];
        assert_eq!(merged["filter"][0], "match");
        assert_eq!(merged["paint"]["line-color"][0], "match");
    }

    #[test]
    fn skip_group_with_camera_only_differing_property() {
        let mir = mir();
        // line-translate is camera-only (feature: false) — differing values must block merge.
        let mut v = json!({
            "version": 8,
            "sources": {"s": {"type": "vector"}},
            "layers": [
                {
                    "id": "a", "type": "line", "source": "s",
                    "source-layer": "road",
                    "filter": ["==", ["get", "class"], "x"],
                    "paint": {"line-translate": [1, 0]}
                },
                {
                    "id": "b", "type": "line", "source": "s",
                    "source-layer": "road",
                    "filter": ["==", ["get", "class"], "y"],
                    "paint": {"line-translate": [0, 1]}
                }
            ]
        });
        let original = v.clone();
        layer_merge(&mut v, &mir);
        assert_eq!(v, original);
    }

    #[test]
    fn skip_group_with_differing_array_property() {
        // line-dasharray is nominally feature-driven in the spec
        // (parameters include "feature") but it is a cross-faded-data-driven
        // array property. MapLibre GL JS cannot actually evaluate `case` /
        // `match` expressions that return arrays for line-dasharray — the
        // renderer hangs. Layer merge must refuse this group.
        let mir = mir();
        let mut v = json!({
            "version": 8,
            "sources": {"s": {"type": "vector"}},
            "layers": [
                {
                    "id": "a", "type": "line", "source": "s",
                    "source-layer": "road",
                    "filter": ["==", ["get", "class"], "rail"],
                    "paint": {"line-dasharray": [0.2, 8]}
                },
                {
                    "id": "b", "type": "line", "source": "s",
                    "source-layer": "road",
                    "filter": ["==", ["get", "class"], "transit"],
                    "paint": {"line-dasharray": [1, 2]}
                }
            ]
        });
        let original = v.clone();
        layer_merge(&mut v, &mir);
        assert_eq!(v, original);
    }

    #[test]
    fn property_absent_on_some_layers_uses_default() {
        let mir = mir();
        let mut v = json!({
            "version": 8,
            "sources": {"s": {"type": "vector"}},
            "layers": [
                {
                    "id": "a", "type": "fill", "source": "s",
                    "source-layer": "land",
                    "filter": ["==", ["get", "t"], "a"],
                    "paint": {"fill-color": "#ff0000", "fill-opacity": 0.5}
                },
                {
                    "id": "b", "type": "fill", "source": "s",
                    "source-layer": "land",
                    "filter": ["==", ["get", "t"], "b"],
                    "paint": {"fill-color": "#0000ff"}
                }
            ]
        });
        layer_merge(&mut v, &mir);
        // fill-opacity absent from layer b → spec default (1)
        let merged = &v["layers"][0];
        let opacity = &merged["paint"]["fill-opacity"];
        assert_eq!(
            opacity,
            &json!(["match", ["get", "t"], "a", 0.5, "b", 1, 1]),
        );
    }

    #[test]
    fn non_adjacent_layers_not_merged() {
        let mir = mir();
        let mut v = json!({
            "version": 8,
            "sources": {"s": {"type": "vector"}},
            "layers": [
                {
                    "id": "fill-a", "type": "fill", "source": "s",
                    "source-layer": "land",
                    "filter": ["==", ["get", "t"], "a"],
                    "paint": {"fill-color": "#aaa"}
                },
                {
                    "id": "line-x", "type": "line", "source": "s",
                    "source-layer": "road",
                    "filter": ["==", ["get", "class"], "x"],
                    "paint": {"line-color": "#000"}
                },
                {
                    "id": "fill-b", "type": "fill", "source": "s",
                    "source-layer": "land",
                    "filter": ["==", ["get", "t"], "b"],
                    "paint": {"fill-color": "#bbb"}
                }
            ]
        });
        let original = v.clone();
        layer_merge(&mut v, &mir);
        assert_eq!(v, original);
    }

    #[test]
    fn merge_three_fill_layers_with_match() {
        let mir = mir();
        let mut v = json!({
            "version": 8,
            "sources": {"s": {"type": "vector"}},
            "layers": [
                {
                    "id": "park", "type": "fill", "source": "s",
                    "source-layer": "land",
                    "filter": ["==", ["get", "class"], "park"],
                    "paint": {"fill-color": "#0f0", "fill-opacity": 0.9}
                },
                {
                    "id": "forest", "type": "fill", "source": "s",
                    "source-layer": "land",
                    "filter": ["==", ["get", "class"], "forest"],
                    "paint": {"fill-color": "#0a0", "fill-opacity": 0.7}
                },
                {
                    "id": "grass", "type": "fill", "source": "s",
                    "source-layer": "land",
                    "filter": ["==", ["get", "class"], "grass"],
                    "paint": {"fill-color": "#0d0", "fill-opacity": 0.5}
                }
            ]
        });
        layer_merge(&mut v, &mir);
        assert_eq!(v["layers"].as_array().unwrap().len(), 1);
        let merged = &v["layers"][0];
        // Uses match (all filters are ["==", ["get", "class"], L]).
        assert_eq!(merged["filter"][0], "match");
        assert_eq!(merged["layout"]["fill-sort-key"][0], "match");
        assert_eq!(merged["paint"]["fill-color"][0], "match");
        assert_eq!(merged["paint"]["fill-opacity"][0], "match");
    }

    #[test]
    fn symbol_layers_are_not_merged() {
        let mir = mir();
        let mut v = json!({
            "version": 8,
            "sources": {"s": {"type": "vector"}},
            "layers": [
                {
                    "id": "city", "type": "symbol", "source": "s",
                    "source-layer": "place",
                    "filter": ["==", ["get", "class"], "city"],
                    "layout": {"text-size": 16},
                    "paint": {"text-color": "#111"}
                },
                {
                    "id": "town", "type": "symbol", "source": "s",
                    "source-layer": "place",
                    "filter": ["==", ["get", "class"], "town"],
                    "layout": {"text-size": 14},
                    "paint": {"text-color": "#333"}
                }
            ]
        });
        layer_merge(&mut v, &mir);
        // Symbol layers must not be merged — collision detection is per-layer.
        assert_eq!(v["layers"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn duplicate_match_labels_fall_back_to_case() {
        let mir = mir();
        let mut v = json!({
            "version": 8,
            "sources": {"s": {"type": "vector"}},
            "layers": [
                {
                    "id": "a", "type": "fill", "source": "s",
                    "source-layer": "land",
                    "filter": ["==", ["get", "t"], "park"],
                    "paint": {"fill-color": "#0f0"}
                },
                {
                    "id": "b", "type": "fill", "source": "s",
                    "source-layer": "land",
                    "filter": ["==", ["get", "t"], "park"],
                    "paint": {"fill-color": "#0a0"}
                }
            ]
        });
        layer_merge(&mut v, &mir);
        let merged = &v["layers"][0];
        // Duplicate labels → case, not match.
        assert_eq!(merged["filter"][0], "any");
        assert_eq!(merged["paint"]["fill-color"][0], "case");
    }

    #[test]
    fn layers_pass_through_outside_groups() {
        let mir = mir();
        let mut v = json!({
            "version": 8,
            "sources": {"s": {"type": "vector"}},
            "layers": [
                {"id": "bg", "type": "background", "paint": {"background-color": "#fff"}},
                {
                    "id": "a", "type": "fill", "source": "s",
                    "source-layer": "land",
                    "filter": ["==", ["get", "t"], "a"],
                    "paint": {"fill-color": "#aaa"}
                },
                {
                    "id": "b", "type": "fill", "source": "s",
                    "source-layer": "land",
                    "filter": ["==", ["get", "t"], "b"],
                    "paint": {"fill-color": "#bbb"}
                },
                {"id": "top", "type": "background", "paint": {"background-color": "#000"}}
            ]
        });
        layer_merge(&mut v, &mir);

        insta::assert_yaml_snapshot!(v, @r##"
        layers:
          - id: bg
            paint:
              background-color: "#fff"
            type: background
          - filter:
              - match
              - - get
                - t
              - - a
                - b
              - true
              - false
            id: a
            layout:
              fill-sort-key:
                - match
                - - get
                  - t
                - a
                - 0
                - b
                - 1
                - -1
            paint:
              fill-color:
                - match
                - - get
                  - t
                - a
                - "#aaa"
                - b
                - "#bbb"
                - "#000000"
            source: s
            source-layer: land
            type: fill
          - id: top
            paint:
              background-color: "#000"
            type: background
        sources:
          s:
            type: vector
        version: 8
        "##);
    }

    #[test]
    #[expect(clippy::too_many_lines)]
    fn zoom_tolerant_merge_three_line_layers() {
        let mir = mir();
        let mut v = json!({
            "version": 8,
            "sources": {"s": {"type": "vector"}},
            "layers": [
                {
                    "id": "motorway", "type": "line", "source": "s",
                    "source-layer": "road", "minzoom": 4, "maxzoom": 22,
                    "filter": ["==", ["get", "class"], "motorway"],
                    "paint": {"line-color": "red", "line-width": 5}
                },
                {
                    "id": "trunk", "type": "line", "source": "s",
                    "source-layer": "road", "minzoom": 7, "maxzoom": 22,
                    "filter": ["==", ["get", "class"], "trunk"],
                    "paint": {"line-color": "orange", "line-width": 3}
                },
                {
                    "id": "primary", "type": "line", "source": "s",
                    "source-layer": "road", "minzoom": 10, "maxzoom": 22,
                    "filter": ["==", ["get", "class"], "primary"],
                    "paint": {"line-color": "yellow", "line-width": 2}
                }
            ]
        });
        layer_merge(&mut v, &mir);
        insta::assert_yaml_snapshot!(v, @r##"
        layers:
          - filter:
              - any
              - - "=="
                - - get
                  - class
                - motorway
              - - all
                - - ">="
                  - - zoom
                  - 7
                - - "=="
                  - - get
                    - class
                  - trunk
              - - all
                - - ">="
                  - - zoom
                  - 10
                - - "=="
                  - - get
                    - class
                  - primary
            id: motorway
            layout:
              line-sort-key:
                - case
                - - all
                  - - ">="
                    - - zoom
                    - 10
                  - - "=="
                    - - get
                      - class
                    - primary
                - 2
                - - all
                  - - ">="
                    - - zoom
                    - 7
                  - - "=="
                    - - get
                      - class
                    - trunk
                - 1
                - - "=="
                  - - get
                    - class
                  - motorway
                - 0
                - -1
            maxzoom: 22
            minzoom: 4
            paint:
              line-color:
                - case
                - - all
                  - - ">="
                    - - zoom
                    - 10
                  - - "=="
                    - - get
                      - class
                    - primary
                - yellow
                - - all
                  - - ">="
                    - - zoom
                    - 7
                  - - "=="
                    - - get
                      - class
                    - trunk
                - orange
                - - "=="
                  - - get
                    - class
                  - motorway
                - red
                - "#000000"
              line-width:
                - case
                - - all
                  - - ">="
                    - - zoom
                    - 10
                  - - "=="
                    - - get
                      - class
                    - primary
                - 2
                - - all
                  - - ">="
                    - - zoom
                    - 7
                  - - "=="
                    - - get
                      - class
                    - trunk
                - 3
                - - "=="
                  - - get
                    - class
                  - motorway
                - 5
                - 1
            source: s
            source-layer: road
            type: line
        sources:
          s:
            type: vector
        version: 8
        "##);
    }

    #[test]
    fn zoom_tolerant_merge_maxzoom_guard() {
        let mir = mir();
        let mut v = json!({
            "version": 8,
            "sources": {"s": {"type": "vector"}},
            "layers": [
                {
                    "id": "a", "type": "fill", "source": "s",
                    "source-layer": "land",
                    "filter": ["==", ["get", "t"], "a"],
                    "paint": {"fill-color": "#aaa"}
                },
                {
                    "id": "b", "type": "fill", "source": "s",
                    "source-layer": "land", "maxzoom": 14,
                    "filter": ["==", ["get", "t"], "b"],
                    "paint": {"fill-color": "#bbb"}
                }
            ]
        });
        layer_merge(&mut v, &mir);
        assert_eq!(v["layers"].as_array().unwrap().len(), 1);
        let merged = &v["layers"][0];
        // No maxzoom on merged (24 is default, omitted).
        assert!(merged.get("maxzoom").is_none());
        // Layer b's arm has a maxzoom guard.
        let arm_b = &merged["filter"][2];
        assert_eq!(arm_b[0], "all");
        assert_eq!(arm_b[1][0], "<");
        assert_eq!(arm_b[1][2], 14.0);
    }

    #[test]
    fn same_zoom_still_uses_match_pattern() {
        let mir = mir();
        let mut v = json!({
            "version": 8,
            "sources": {"s": {"type": "vector"}},
            "layers": [
                {
                    "id": "a", "type": "fill", "source": "s",
                    "source-layer": "land", "minzoom": 5,
                    "filter": ["==", ["get", "t"], "a"],
                    "paint": {"fill-color": "#aaa"}
                },
                {
                    "id": "b", "type": "fill", "source": "s",
                    "source-layer": "land", "minzoom": 5,
                    "filter": ["==", ["get", "t"], "b"],
                    "paint": {"fill-color": "#bbb"}
                }
            ]
        });
        layer_merge(&mut v, &mir);
        assert_eq!(v["layers"].as_array().unwrap().len(), 1);
        let merged = &v["layers"][0];
        // Same zoom → match pattern still applies.
        assert_eq!(merged["filter"][0], "match");
    }
}

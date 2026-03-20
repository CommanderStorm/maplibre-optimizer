//! Cross-field and spec rules that are not expressible in serde types alone.

use std::collections::{BTreeMap, HashSet};
use std::sync::OnceLock;

use serde_json::{Map, Value};

use crate::decoder::StyleReference;
use crate::mir::types::{IntermediateType, MirField};
use crate::mir::{
    IntermediateLayerField, IntermediateLayers, IntermediateNamedType, IntermediateRootPrimitives,
    IntermediateSpec,
};
use crate::spec::MaplibreStyleSpecification;

#[derive(Debug)]
struct MirValidationTables {
    /// JSON paint/layout property name → optional min/max for scalar numeric leaves.
    layer_number_bounds: BTreeMap<String, (Option<f64>, Option<f64>)>,
    root_center_len: Option<usize>,
    light_allowed_keys: HashSet<String>,
}

static MIR_VALIDATION: OnceLock<MirValidationTables> = OnceLock::new();

fn mir_tables() -> &'static MirValidationTables {
    MIR_VALIDATION.get_or_init(|| {
        let reference: StyleReference =
            serde_json::from_str(include_str!("../../upstream/src/reference/v8.json"))
                .expect("parse v8.json for validation tables");
        let spec = IntermediateSpec::from(reference);
        MirValidationTables {
            layer_number_bounds: layer_number_bounds(&spec.layers),
            root_center_len: root_center_expected_len(&spec.root),
            light_allowed_keys: light_allowed_keys(&spec.named_types),
        }
    })
}

fn merge_num_bounds(
    map: &mut BTreeMap<String, (Option<f64>, Option<f64>)>,
    name: &str,
    min: Option<f64>,
    max: Option<f64>,
) {
    let merge_lo = |a: Option<f64>, b: Option<f64>| -> Option<f64> {
        match (a, b) {
            (Some(x), Some(y)) => Some(x.max(y)),
            (Some(x), None) | (None, Some(x)) => Some(x),
            (None, None) => None,
        }
    };
    let merge_hi = |a: Option<f64>, b: Option<f64>| -> Option<f64> {
        match (a, b) {
            (Some(x), Some(y)) => Some(x.min(y)),
            (Some(x), None) | (None, Some(x)) => Some(x),
            (None, None) => None,
        }
    };
    map.entry(name.to_string())
        .and_modify(|(cur_min, cur_max)| {
            *cur_min = merge_lo(*cur_min, min);
            *cur_max = merge_hi(*cur_max, max);
        })
        .or_insert((min, max));
}

fn merge_fields_bounds(
    map: &mut BTreeMap<String, (Option<f64>, Option<f64>)>,
    fields: &BTreeMap<String, IntermediateLayerField>,
) {
    for (spec_name, f) in fields {
        if let IntermediateType::Number { min, max } = &f.r#type {
            merge_num_bounds(map, spec_name, *min, *max);
        }
    }
}

fn layer_number_bounds(
    layers: &IntermediateLayers,
) -> BTreeMap<String, (Option<f64>, Option<f64>)> {
    let mut out = BTreeMap::new();
    for lt in layers.layer_types.values() {
        merge_fields_bounds(&mut out, &lt.paint);
        merge_fields_bounds(&mut out, &lt.layout);
    }
    out
}

fn root_center_expected_len(root: &IntermediateRootPrimitives) -> Option<usize> {
    let field = root.0.get("center")?;
    match field {
        MirField::Array(a) => a.length,
        _ => None,
    }
}

fn light_allowed_keys(named: &BTreeMap<String, IntermediateNamedType>) -> HashSet<String> {
    let mut out = match named.get("light") {
        Some(IntermediateNamedType::Struct(fields)) => {
            fields.iter().map(|f| f.meta().spec_name.clone()).collect()
        }
        _ => HashSet::new(),
    };
    if out.is_empty() {
        // v8 always defines `light`; keep a fallback for trimmed test corpora.
        for k in [
            "anchor",
            "color",
            "intensity",
            "position",
            "color-use-theme",
        ] {
            out.insert(k.to_string());
        }
    }
    out
}

/// Run the same JSON-tree checks as [`parse_and_validate_style`] without deserializing into
/// [`MaplibreStyleSpecification`] (avoids dropping root keys not yet modeled in `spec.rs`).
pub fn validate_style_value(v: &Value) -> Result<(), String> {
    let obj = v
        .as_object()
        .ok_or_else(|| "style: object expected".to_string())?;
    validate_from_root_object(obj)
}

/// Deserialize a style JSON string and run validation on the **parsed JSON tree** (before lossy
/// deserialization drops unknown nested keys), then decode into [`MaplibreStyleSpecification`].
pub fn parse_and_validate_style(json: &str) -> Result<MaplibreStyleSpecification, String> {
    let v: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    validate_style_value(&v)?;
    serde_json::from_value(v).map_err(|e| e.to_string())
}

/// Best-effort validation on a decoded style. Prefer [`parse_and_validate_style`] on raw JSON
/// so checks see keys serde may have skipped on nested objects.
pub fn validate_style(style: &MaplibreStyleSpecification) -> Result<(), String> {
    if let Ok(v) = serde_json::to_value(style)
        && let Some(obj) = v.as_object()
    {
        validate_from_root_object(obj)?;
    }
    Ok(())
}

fn validate_from_root_object(style: &Map<String, Value>) -> Result<(), String> {
    validate_root_object(style)?;
    validate_sources_object(&style.get("sources").cloned().unwrap_or(Value::Null))?;
    validate_layers_value(&style.get("layers").cloned().unwrap_or(Value::Null))?;
    validate_glyphs_root(style.get("glyphs"))?;
    validate_light_value(style.get("light"))?;
    validate_terrain_value(style.get("terrain"))?;
    validate_pitch_value(style.get("pitch"))?;
    Ok(())
}

fn validate_root_object(style: &Map<String, Value>) -> Result<(), String> {
    if style.contains_key("constants") {
        return Err("constants: constants have been deprecated as of v8".to_string());
    }
    if let Some(center) = style.get("center")
        && let Some(a) = center.as_array()
    {
        let expected = mir_tables().root_center_len.unwrap_or(2);
        if a.len() != expected {
            return Err(format!(
                "center: array length {expected} expected, length {} found",
                a.len()
            ));
        }
    }
    Ok(())
}

fn validate_pitch_value(v: Option<&Value>) -> Result<(), String> {
    let Some(p) = v else {
        return Ok(());
    };
    if !p.is_number() {
        return Err("pitch: number expected, string found".to_string());
    }
    Ok(())
}

fn validate_sources_object(sources: &Value) -> Result<(), String> {
    let Some(obj) = sources.as_object() else {
        if sources.is_null() {
            return Ok(());
        }
        return Err("sources: object expected".to_string());
    };
    for (id, src) in obj {
        let Some(so) = src.as_object() else {
            return Err(format!("sources.{id}: object expected"));
        };
        if !so.contains_key("type") {
            return Err(format!("sources.{id}: \"type\" is required"));
        }
    }
    Ok(())
}

fn validate_layers_value(layers: &Value) -> Result<(), String> {
    let Some(arr) = layers.as_array() else {
        return Ok(());
    };
    for (i, layer) in arr.iter().enumerate() {
        let Some(obj) = layer.as_object() else {
            return Err(format!("layers[{i}]: object expected"));
        };
        let has_type = obj.get("type").map(|t| !t.is_null()).unwrap_or(false);
        let has_ref = obj.get("ref").map(|t| !t.is_null()).unwrap_or(false);
        if !has_type && !has_ref {
            return Err(format!(
                "layers[{i}]: either \"type\" or \"ref\" is required"
            ));
        }
        if let Some(f) = obj.get("filter")
            && !f.is_array()
        {
            return Err(format!("layers[{i}].filter: array expected, object found"));
        }
        validate_layer_paint_layout(i, obj)?;
    }
    Ok(())
}

fn validate_layer_paint_layout(idx: usize, layer: &Map<String, Value>) -> Result<(), String> {
    let bounds = &mir_tables().layer_number_bounds;
    if let Some(Value::Object(paint)) = layer.get("paint") {
        validate_numeric_layer_section(idx, "paint", paint, bounds)?;
        validate_colors_in_paint(idx, paint)?;
        validate_functions_base(idx, paint)?;
    }
    if let Some(Value::Object(layout)) = layer.get("layout") {
        validate_numeric_layer_section(idx, "layout", layout, bounds)?;
        validate_layout_format_and_font(idx, layout)?;
    }
    Ok(())
}

fn validate_functions_base(idx: usize, paint: &Map<String, Value>) -> Result<(), String> {
    if let Some(lw) = paint.get("line-width")
        && let Some(m) = lw.as_object()
        && let Some(b) = m.get("base")
        && !b.is_number()
    {
        return Err(format!(
            "layers[{idx}].paint.line-width.base: number expected, string found"
        ));
    }
    Ok(())
}

fn validate_numeric_layer_section(
    idx: usize,
    section: &str,
    obj: &Map<String, Value>,
    bounds: &BTreeMap<String, (Option<f64>, Option<f64>)>,
) -> Result<(), String> {
    for (key, v) in obj {
        let Some(n) = v.as_f64() else {
            continue;
        };
        let Some(&(min_b, max_b)) = bounds.get(key.as_str()) else {
            continue;
        };
        if let Some(lo) = min_b
            && n < lo
        {
            return Err(format!(
                "layers[{idx}].{section}.{key}: {n} is less than the minimum value {lo}",
            ));
        }
        if let Some(hi) = max_b
            && n > hi
        {
            return Err(format!(
                "layers[{idx}].{section}.{key}: {n} is greater than the maximum value {hi}",
            ));
        }
    }
    Ok(())
}

fn validate_colors_in_paint(idx: usize, paint: &Map<String, Value>) -> Result<(), String> {
    for (key, label) in [
        ("fill-color", "fill-color"),
        ("fill-outline-color", "fill-outline-color"),
    ] {
        if let Some(fc) = paint.get(key) {
            if fc.is_array() {
                let first = fc.as_array().and_then(|a| a.first());
                let is_color_expr = matches!(
                    first.and_then(|x| x.as_str()),
                    Some("rgb" | "rgba" | "hsl" | "to-color")
                );
                if !is_color_expr {
                    return Err(format!(
                        "layers[{idx}].paint.{label}: color expected, array found"
                    ));
                }
            }
            if let Some(s) = fc.as_str()
                && color::parse_color(s).is_err()
            {
                return Err(format!(
                    "layers[{idx}].paint.{label}: color expected, \"{s}\" found"
                ));
            }
            if let Value::Object(m) = fc
                && let Some(Value::Array(stops)) = m.get("stops")
            {
                for (si, stop) in stops.iter().enumerate() {
                    let Some(pair) = stop.as_array() else {
                        continue;
                    };
                    let c = pair.get(1);
                    if let Some(Value::String(s)) = c
                        && color::parse_color(s).is_err()
                    {
                        return Err(format!(
                            "layers[{idx}].paint.{label}.stops[{si}][1]: color expected, \"{s}\" found"
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_layout_format_and_font(idx: usize, layout: &Map<String, Value>) -> Result<(), String> {
    if let Some(tf) = layout.get("text-field") {
        validate_text_field_format(idx, tf)?;
    }
    if let Some(tf) = layout.get("text-font") {
        validate_text_font_data_expression(idx, tf)?;
    }
    Ok(())
}

fn validate_text_field_format(idx: usize, v: &Value) -> Result<(), String> {
    let arr = match v {
        Value::Array(a) => a,
        _ => return Ok(()),
    };
    if arr.first().and_then(|x| x.as_str()) != Some("format") {
        return Ok(());
    }
    if arr.len() < 2 {
        return Err(format!(
            "layers[{idx}].layout.text-field: First argument must be an image or text section."
        ));
    }
    Ok(())
}

fn validate_text_font_data_expression(idx: usize, v: &Value) -> Result<(), String> {
    if contains_disallowed_literal_in_expression(v) {
        return Err(format!(
            "layers[{idx}].layout.text-font: Invalid data expression for \"text-font\" (literals within the expression)."
        ));
    }
    Ok(())
}

fn contains_disallowed_literal_in_expression(v: &Value) -> bool {
    let Some(arr) = v.as_array() else {
        return false;
    };
    if arr.is_empty() {
        return false;
    }
    if arr[0].as_str() == Some("literal") {
        return true;
    }
    arr.iter().any(contains_disallowed_literal_in_expression)
}

fn validate_glyphs_root(glyphs: Option<&Value>) -> Result<(), String> {
    let Some(g) = glyphs else {
        return Ok(());
    };
    if !g.is_string() {
        return Err("glyphs: string expected, boolean found".to_string());
    }
    let s = g.as_str().unwrap_or("");
    if !s.contains("{fontstack}") {
        return Err(r#"glyphs: "glyphs" url must include a "{fontstack}" token"#.to_string());
    }
    if !s.contains("{range}") {
        return Err(r#"glyphs: "glyphs" url must include a "{range}" token"#.to_string());
    }
    Ok(())
}

fn validate_light_value(light: Option<&Value>) -> Result<(), String> {
    let Some(l) = light else {
        return Ok(());
    };
    let Some(obj) = l.as_object() else {
        return Ok(());
    };
    let allowed = &mir_tables().light_allowed_keys;
    for key in obj.keys() {
        if !allowed.contains(key.as_str()) {
            return Err(format!(r#"light: unknown property "{key}""#));
        }
    }
    if let Some(a) = obj.get("anchor")
        && !a.is_string()
    {
        return Err(format!(
            "light.anchor: expected one of [map, viewport], {:?} found",
            a
        ));
    }
    if let Some(c) = obj.get("color") {
        if let Some(s) = c.as_str()
            && color::parse_color(s).is_err()
        {
            return Err(format!("light.color: color expected, \"{s}\" found"));
        }
        if !c.is_string() && !c.is_array() {
            return Err("light.color: color expected".to_string());
        }
    }
    Ok(())
}

fn validate_terrain_value(terrain: Option<&Value>) -> Result<(), String> {
    let Some(t) = terrain else {
        return Ok(());
    };
    let Some(obj) = t.as_object() else {
        return Ok(());
    };
    if let Some(s) = obj.get("source")
        && !s.is_string()
    {
        return Err("terrain.source: string expected, number found".to_string());
    }
    Ok(())
}

//! Cross-field and spec rules that are not expressible in serde types alone.

use serde_json::{Map, Value};

use crate::spec::MaplibreStyleSpecification;

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
    if let Ok(v) = serde_json::to_value(style) {
        if let Some(obj) = v.as_object() {
            validate_from_root_object(obj)?;
        }
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
    if let Some(center) = style.get("center") {
        if let Some(a) = center.as_array() {
            if a.len() != 2 {
                return Err(format!(
                    "center: array length 2 expected, length {} found",
                    a.len()
                ));
            }
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
        if let Some(f) = obj.get("filter") {
            if !f.is_array() {
                return Err(format!("layers[{i}].filter: array expected, object found"));
            }
        }
        validate_layer_paint_layout(i, obj)?;
    }
    Ok(())
}

fn validate_layer_paint_layout(idx: usize, layer: &Map<String, Value>) -> Result<(), String> {
    if let Some(Value::Object(paint)) = layer.get("paint") {
        validate_numeric_paint(idx, "paint", paint)?;
        validate_colors_in_paint(idx, paint)?;
        validate_functions_base(idx, paint)?;
    }
    if let Some(Value::Object(layout)) = layer.get("layout") {
        validate_layout_format_and_font(idx, layout)?;
    }
    Ok(())
}

fn validate_functions_base(idx: usize, paint: &Map<String, Value>) -> Result<(), String> {
    if let Some(lw) = paint.get("line-width") {
        if let Some(m) = lw.as_object() {
            if let Some(b) = m.get("base") {
                if !b.is_number() {
                    return Err(format!(
                        "layers[{idx}].paint.line-width.base: number expected, string found"
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_numeric_paint(
    idx: usize,
    section: &str,
    paint: &Map<String, Value>,
) -> Result<(), String> {
    let check_ge_0 = |name: &str| -> Result<(), String> {
        if let Some(v) = paint.get(name) {
            if let Some(n) = v.as_f64() {
                if n < 0.0 {
                    return Err(format!(
                        "layers[{idx}].{section}.{name}: {n} is less than the minimum value 0",
                    ));
                }
            }
        }
        Ok(())
    };
    check_ge_0("circle-radius")?;
    check_ge_0("fill-opacity")?;
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
            if let Some(s) = fc.as_str() {
                if s == "__proto__" {
                    return Err(format!(
                        "layers[{idx}].paint.{label}: color expected, \"__proto__\" found"
                    ));
                }
                if color::parse_color(s).is_err() {
                    return Err(format!(
                        "layers[{idx}].paint.{label}: color expected, \"{s}\" found"
                    ));
                }
            }
            if let Value::Object(m) = fc {
                if let Some(Value::Array(stops)) = m.get("stops") {
                    for (si, stop) in stops.iter().enumerate() {
                        let Some(pair) = stop.as_array() else {
                            continue;
                        };
                        let c = pair.get(1);
                        if let Some(Value::String(s)) = c {
                            if s == "valueOf" || s == "__proto__" {
                                return Err(format!(
                                    "layers[{idx}].paint.{label}.stops[{si}][1]: color expected, \"{s}\" found"
                                ));
                            }
                        }
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
    Ok(())
}

fn validate_light_value(light: Option<&Value>) -> Result<(), String> {
    let Some(l) = light else {
        return Ok(());
    };
    let Some(obj) = l.as_object() else {
        return Ok(());
    };
    for key in obj.keys() {
        let allowed = matches!(
            key.as_str(),
            "anchor" | "color" | "intensity" | "position" | "color-use-theme"
        );
        if !allowed {
            return Err(format!(r#"light: unknown property "{key}""#));
        }
    }
    if let Some(a) = obj.get("anchor") {
        if !a.is_string() {
            return Err(format!(
                "light.anchor: expected one of [map, viewport], {:?} found",
                a
            ));
        }
    }
    if let Some(c) = obj.get("color") {
        if let Some(s) = c.as_str() {
            if s == "__proto__" {
                return Err("light.color: color expected, \"__proto__\" found".to_string());
            }
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
    if let Some(s) = obj.get("source") {
        if !s.is_string() {
            return Err("terrain.source: string expected, number found".to_string());
        }
    }
    Ok(())
}

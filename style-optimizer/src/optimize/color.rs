//! Color minification: shorten verbose CSS color strings to their shortest form.

use serde_json::Value;

use super::walk::{PropertyContext, StyleVisitor};

// ── Public visitor ───────────────────────────────────────────────────────────

pub(crate) struct MinifyColorsVisitor;

impl StyleVisitor for MinifyColorsVisitor {
    fn visit_filter(&mut self, _: usize, _: &str, filter: &mut Value) {
        minify_colors_in_expr(filter);
    }

    fn visit_property(&mut self, ctx: &PropertyContext<'_>, value: &mut Value) {
        if ctx.field.r#type.is_color() {
            minify_color_value(value);
        } else {
            // Expressions in non-color properties may still contain color literals.
            minify_colors_in_expr(value);
        }
    }
}

// ── Recursive expression walker ──────────────────────────────────────────────

fn minify_colors_in_expr(v: &mut Value) {
    match v {
        Value::String(s) => {
            if let Some(short) = minify_color_string(s) {
                *s = short;
            }
        }
        Value::Array(arr) => {
            for child in arr.iter_mut() {
                minify_colors_in_expr(child);
            }
        }
        Value::Object(map) => {
            for child in map.values_mut() {
                minify_colors_in_expr(child);
            }
        }
        _ => {}
    }
}

fn minify_color_value(v: &mut Value) {
    match v {
        Value::String(s) => {
            if let Some(short) = minify_color_string(s) {
                *s = short;
            }
        }
        Value::Array(_) | Value::Object(_) => minify_colors_in_expr(v),
        _ => {}
    }
}

// ── Color parsing and minification ───────────────────────────────────────────

/// Try to shorten a CSS color string. Returns `None` if already shortest or not a color.
#[expect(clippy::many_single_char_names)]
pub(crate) fn minify_color_string(s: &str) -> Option<String> {
    let (r, g, b, a) = parse_color(s)?;
    let short = format_shortest(r, g, b, a);
    if short.len() < s.len() {
        Some(short)
    } else {
        None
    }
}

/// Parse a CSS color string into RGBA (0–255 for rgb, 0.0–1.0 for alpha).
#[expect(clippy::many_single_char_names)]
fn parse_color(s: &str) -> Option<(u8, u8, u8, f64)> {
    let s = s.trim();

    if let Some(hex) = s.strip_prefix('#') {
        return parse_hex(hex);
    }

    if let Some(inner) = strip_func(s, "rgba") {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() != 4 {
            return None;
        }
        let r = parse_channel(parts[0])?;
        let g = parse_channel(parts[1])?;
        let b = parse_channel(parts[2])?;
        let a = parts[3].trim().parse::<f64>().ok()?;
        if !(0.0..=1.0).contains(&a) {
            return None;
        }
        return Some((r, g, b, a));
    }

    if let Some(inner) = strip_func(s, "rgb") {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() != 3 {
            return None;
        }
        let r = parse_channel(parts[0])?;
        let g = parse_channel(parts[1])?;
        let b = parse_channel(parts[2])?;
        return Some((r, g, b, 1.0));
    }

    if let Some(inner) = strip_func(s, "hsla") {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() != 4 {
            return None;
        }
        let h = parts[0].trim().parse::<f64>().ok()?;
        let s_pct = parse_percent(parts[1])?;
        let l = parse_percent(parts[2])?;
        let a = parts[3].trim().parse::<f64>().ok()?;
        if !(0.0..=1.0).contains(&a) {
            return None;
        }
        let (r, g, b) = hsl_to_rgb(h, s_pct, l);
        return Some((r, g, b, a));
    }

    if let Some(inner) = strip_func(s, "hsl") {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() != 3 {
            return None;
        }
        let h = parts[0].trim().parse::<f64>().ok()?;
        let s_pct = parse_percent(parts[1])?;
        let l = parse_percent(parts[2])?;
        let (r, g, b) = hsl_to_rgb(h, s_pct, l);
        return Some((r, g, b, 1.0));
    }

    named_color(s)
}

fn strip_func<'a>(s: &'a str, name: &str) -> Option<&'a str> {
    let s = s.strip_prefix(name)?;
    let s = s.strip_prefix('(')?;
    let s = s.strip_suffix(')')?;
    Some(s)
}

fn parse_channel(s: &str) -> Option<u8> {
    let s = s.trim();
    let n: f64 = s.parse().ok()?;
    if !n.is_finite() {
        return None;
    }
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    Some(n.round().clamp(0.0, 255.0) as u8)
}

fn parse_percent(s: &str) -> Option<f64> {
    let s = s.trim().strip_suffix('%')?;
    let n: f64 = s.parse().ok()?;
    Some(n / 100.0)
}

fn parse_hex(hex: &str) -> Option<(u8, u8, u8, f64)> {
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            Some((r, g, b, 1.0))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r, g, b, 1.0))
        }
        _ => None,
    }
}

/// Format color in the shortest representation.
fn format_shortest(r: u8, g: u8, b: u8, a: f64) -> String {
    if (a - 1.0).abs() < f64::EPSILON {
        // Full opacity: use hex.
        if r.is_multiple_of(17) && g.is_multiple_of(17) && b.is_multiple_of(17) {
            // Can use short hex #rgb.
            format!("#{:x}{:x}{:x}", r / 17, g / 17, b / 17)
        } else {
            format!("#{r:02x}{g:02x}{b:02x}")
        }
    } else {
        // Partial opacity: rgba is the shortest standard form.
        // Try to avoid trailing zeros in alpha.
        let a_str = format_alpha(a);
        format!("rgba({r},{g},{b},{a_str})")
    }
}

fn format_alpha(a: f64) -> String {
    // Use up to 2 decimal places, stripping trailing zeros.
    let s = format!("{a:.2}");
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_string()
}

/// Convert HSL to RGB (all inputs in 0..1 range for s/l, degrees for h).
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names
)]
fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let h = ((h % 360.0) + 360.0) % 360.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r1, g1, b1) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r1 + m) * 255.0).round() as u8,
        ((g1 + m) * 255.0).round() as u8,
        ((b1 + m) * 255.0).round() as u8,
    )
}

/// A small subset of CSS named colors that are shorter than or equal to their hex form.
fn named_color(s: &str) -> Option<(u8, u8, u8, f64)> {
    // Only parse named colors that are commonly used in styles; we just need to
    // recognize them so we can convert to hex if shorter.
    Some(match s {
        "black" => (0, 0, 0, 1.0),
        "white" => (255, 255, 255, 1.0),
        "red" => (255, 0, 0, 1.0),
        "green" => (0, 128, 0, 1.0),
        "blue" => (0, 0, 255, 1.0),
        "yellow" => (255, 255, 0, 1.0),
        "cyan" | "aqua" => (0, 255, 255, 1.0),
        "magenta" | "fuchsia" => (255, 0, 255, 1.0),
        "silver" => (192, 192, 192, 1.0),
        "gray" | "grey" => (128, 128, 128, 1.0),
        "maroon" => (128, 0, 0, 1.0),
        "olive" => (128, 128, 0, 1.0),
        "navy" => (0, 0, 128, 1.0),
        "purple" => (128, 0, 128, 1.0),
        "teal" => (0, 128, 128, 1.0),
        "transparent" => (0, 0, 0, 0.0),
        "orange" => (255, 165, 0, 1.0),
        "pink" => (255, 192, 203, 1.0),
        "coral" => (255, 127, 80, 1.0),
        "tomato" => (255, 99, 71, 1.0),
        _ => return None,
    })
}

// ── MirType helper ───────────────────────────────────────────────────────────

trait MirTypeExt {
    fn is_color(&self) -> bool;
}

impl MirTypeExt for maplibre_style_spec::mir::types::MirType {
    fn is_color(&self) -> bool {
        matches!(self, Self::Color)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgba_full_opacity_to_hex() {
        assert_eq!(
            minify_color_string("rgba(255,255,255,1)"),
            Some("#fff".to_string())
        );
        assert_eq!(
            minify_color_string("rgba(0,0,0,1)"),
            Some("#000".to_string())
        );
        assert_eq!(
            minify_color_string("rgba(255, 255, 255, 1)"),
            Some("#fff".to_string())
        );
    }

    #[test]
    fn rgb_to_hex() {
        assert_eq!(minify_color_string("rgb(0,0,0)"), Some("#000".to_string()));
        assert_eq!(
            minify_color_string("rgb(158,189,255)"),
            Some("#9ebdff".to_string())
        );
    }

    #[test]
    fn hsl_to_hex() {
        assert_eq!(
            minify_color_string("hsl(0, 0%, 100%)"),
            Some("#fff".to_string())
        );
        assert_eq!(
            minify_color_string("hsl(0, 0%, 0%)"),
            Some("#000".to_string())
        );
    }

    #[test]
    fn hsla_partial_opacity() {
        let result = minify_color_string("hsla(35, 6%, 79%, 0.32)");
        assert!(result.is_some());
        let s = result.unwrap();
        assert!(s.starts_with("rgba("));
        assert!(s.len() < "hsla(35, 6%, 79%, 0.32)".len());
    }

    #[test]
    fn hex_already_short() {
        assert_eq!(minify_color_string("#fff"), None);
        assert_eq!(minify_color_string("#000"), None);
    }

    #[test]
    fn hex_long_to_short() {
        assert_eq!(minify_color_string("#ffffff"), Some("#fff".to_string()));
        assert_eq!(minify_color_string("#aabbcc"), Some("#abc".to_string()));
    }

    #[test]
    fn hex_long_not_shortenable() {
        // #9ebdff has no short form — already 7 chars.
        assert_eq!(minify_color_string("#9ebdff"), None);
    }

    #[test]
    fn partial_opacity_stays_rgba() {
        // Already shortest form — no change.
        assert_eq!(minify_color_string("rgba(255,0,0,0.5)"), None);
        // Verbose form with spaces gets shortened.
        assert_eq!(
            minify_color_string("rgba(255, 0, 0, 0.5)"),
            Some("rgba(255,0,0,0.5)".to_string())
        );
    }

    #[test]
    fn named_to_hex_when_shorter() {
        // "transparent" (11 chars) → "rgba(0,0,0,0)" (14 chars) — not shorter, keep.
        assert_eq!(minify_color_string("transparent"), None);
        // "red" (3 chars) → "#f00" (4 chars) — not shorter, keep.
        assert_eq!(minify_color_string("red"), None);
        // "tomato" (6 chars) → "#ff6347" (7 chars) — not shorter, keep.
        assert_eq!(minify_color_string("tomato"), None);
    }
}

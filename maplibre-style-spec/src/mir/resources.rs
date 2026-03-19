/// Holds spec-level description of the `glyphs` root field — a URL template for loading
/// signed-distance-field glyph sets used for text rendering.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IntermediateFontResources {
    /// URL template pattern for loading SDF glyph sets (e.g. `{fontstack}/{range}.pbf`)
    pub url_template: Option<String>,
}

/// Holds spec-level description of the `sprite` root field — a URL or array of sprite objects
/// used for icon/pattern rendering.  Uses `serde_json::Value` because the field accepts either
/// a plain string URL or an array of `{id, url}` objects.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IntermediateSpriteResources {
    pub url: Option<serde_json::Value>,
}

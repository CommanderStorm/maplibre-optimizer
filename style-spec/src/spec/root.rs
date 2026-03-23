#![allow(clippy::large_enum_variant)]
#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use crate::{array_prop, boolean_prop, color_prop, formatted_prop, numeric_prop, string_prop};

/// This is a Maplibre Style Specification
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct MaplibreStyleSpecification {
    /// Default bearing, in degrees. The bearing is the compass direction that is "up"; for example, a bearing of 90° orients the map so that east is up. This value will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    ///
    /// Range: every 360
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bearing: Option<RootBearing>,
    /// Default map center in longitude and latitude.  The style center will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub center: Option<RootCenter>,
    /// Default map center altitude in meters above sea level. The style center altitude defines the altitude where the camera is looking at and will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    #[serde(rename = "centerAltitude")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub center_altitude: Option<RootCenterAltitude>,
    /// The `font-faces` property can be used to specify what font files to use for rendering text. Font faces contain information needed to render complex texts such as [Devanagari](https://en.wikipedia.org/wiki/Devanagari), [Khmer](https://en.wikipedia.org/wiki/Khmer_script) among many others.<h2>Unicode range</h2>The optional `unicode-range` property can be used to only use a particular font file for characters within the specified unicode range(s). Its value should be an array of strings, each indicating a start and end of a unicode range, similar to the [CSS descriptor with the same name](https://developer.mozilla.org/en-US/docs/Web/CSS/@font-face/unicode-range). This allows specifying multiple non-consecutive unicode ranges. When not specified, the default value is `U+0-10FFFF`, meaning the font file will be used for all unicode characters.
    ///
    /// Refer to the [Unicode Character Code Charts](https://www.unicode.org/charts/) to see ranges for scripts supported by Unicode. To see what unicode code-points are available in a font, use a tool like [FontDrop](https://fontdrop.info/).
    ///
    /// <h2>Font Resolution</h2>For every name in a symbol layer’s [`text-font`](./layers.md/#text-font) array, characters are matched if they are covered one of the by the font files in the corresponding entry of the `font-faces` map. Any still-unmatched characters then fall back to the [`glyphs`](./glyphs.md) URL if provided.
    ///
    /// <h2>Supported Fonts</h2>What type of fonts are supported is implementation-defined. Unsupported fonts are ignored.
    #[serde(rename = "font-faces")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_faces: Option<RootFontFaces>,
    /// The global light source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub light: Option<RootLight>,
    /// Arbitrary properties useful to track with the stylesheet, but do not influence rendering. Properties should be prefixed to avoid collisions, like 'maplibre:'.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<RootMetadata>,
    /// A human-readable name for the style.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<RootName>,
    /// Default pitch, in degrees. Zero is perpendicular to the surface, for a look straight down at the map, while a greater value like 60 looks ahead towards the horizon. The style pitch will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pitch: Option<RootPitch>,
    /// The projection configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projection: Option<RootProjection>,
    /// Default roll, in degrees. The roll angle is measured counterclockwise about the camera boresight. The style roll will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roll: Option<RootRoll>,
    /// The map's sky configuration. **Note:** this definition is still experimental and is under development in maplibre-gl-js.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sky: Option<RootSky>,
    /// Sources state which data the map should display. Specify the type of source with the `type` property. Adding a source isn't enough to make data appear on the map because sources don't contain styling details like color or width. Layers refer to a source and give it a visual representation. This makes it possible to style the same source in different ways, like differentiating between types of roads in a highways layer.
    ///
    /// Tiled sources (vector and raster) must specify their details according to the [TileJSON specification](https://github.com/mapbox/tilejson-spec).
    pub sources: RootSources,
    /// An object used to define default values when using the [`global-state`](https://maplibre.org/maplibre-style-spec/expressions/#global-state) expression.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<RootState>,
    /// The terrain configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terrain: Option<RootTerrain>,
    /// A global transition definition to use as a default across properties, to be used for timing transitions between one value and the next when no property-specific transition is set. Collision-based symbol fading is controlled independently of the style's `transition` property.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transition: Option<RootTransition>,
    /// Style specification version number. Must be 8.
    pub version: RootVersion,
    /// Default zoom level.  The style zoom will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zoom: Option<RootZoom>,
    /// Layers will be drawn in the order of this array.
    #[serde(default)]
    pub layers: Vec<AnyLayer>,
}

/// Default bearing, in degrees. The bearing is the compass direction that is "up"; for example, a bearing of 90° orients the map so that east is up. This value will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
///
/// Range: every 360
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RootBearing(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for RootBearing {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod test {
    use super::*;

    #[test]
    fn test_example_root_bearing_decodes() {
        let example = serde_json::json!(29);
        let _ = serde_json::from_value::<RootBearing>(example).expect("example should decode");
    }

    #[test]
    fn test_example_root_center_decodes() {
        let example = serde_json::json!([-73.9749, 40.7736]);
        let _ = serde_json::from_value::<RootCenter>(example).expect("example should decode");
    }

    #[test]
    fn test_example_root_center_altitude_decodes() {
        let example = serde_json::json!(123.4);
        let _ =
            serde_json::from_value::<RootCenterAltitude>(example).expect("example should decode");
    }

    #[test]
    fn test_example_root_font_faces_decodes() {
        let example = serde_json::json!({"Noto Sans Regular":[{"unicode-range":["U+1780-17FF"],"url":"https://cdn.jsdelivr.net/gh/notofonts/notofonts.github.io/fonts/NotoSansKhmer/hinted/ttf/NotoSansKhmer-Regular.ttf"},{"unicode-range":["U+0900-097F"],"url":"https://cdn.jsdelivr.net/gh/notofonts/notofonts.github.io/fonts/NotoSansDevanagari/hinted/ttf/NotoSansDevanagari-Regular.ttf"},{"unicode-range":["U+1000-109F"],"url":"https://cdn.jsdelivr.net/gh/notofonts/notofonts.github.io/fonts/NotoSansMyanmar/hinted/ttf/NotoSansMyanmar-Regular.ttf"},{"unicode-range":["U+1200-137F"],"url":"https://cdn.jsdelivr.net/gh/notofonts/notofonts.github.io/fonts/NotoSansEthiopic/hinted/ttf/NotoSansEthiopic-Regular.ttf"}],"Unifont":"https://ftp.gnu.org/gnu/unifont/unifont-15.0.01/unifont-15.0.01.ttf"});
        let _ = serde_json::from_value::<RootFontFaces>(example).expect("example should decode");
    }

    #[test]
    fn test_example_root_light_decodes() {
        let example = serde_json::json!({"anchor":"viewport","color":"white","intensity":0.4});
        let _ = serde_json::from_value::<RootLight>(example).expect("example should decode");
    }

    #[test]
    fn test_example_root_metadata_decodes() {
        let example = serde_json::json!({"example:object":{"Boolean":false,"Number":2,"String":"one"},"styleeditor:comment":"Style generated 1677776383","styleeditor:slimmode":true,"styleeditor:version":"3.14.159265"});
        let _ = serde_json::from_value::<RootMetadata>(example).expect("example should decode");
    }

    #[test]
    fn test_example_root_name_decodes() {
        let example = serde_json::json!("Bright");
        let _ = serde_json::from_value::<RootName>(example).expect("example should decode");
    }

    #[test]
    fn test_example_root_pitch_decodes() {
        let example = serde_json::json!(50);
        let _ = serde_json::from_value::<RootPitch>(example).expect("example should decode");
    }

    #[test]
    fn test_example_root_projection_decodes() {
        let example = serde_json::json!({"type":["interpolate",["linear"],["zoom"],10,"vertical-perspective",12,"mercator"]});
        let _ = serde_json::from_value::<RootProjection>(example).expect("example should decode");
    }

    #[test]
    fn test_example_root_roll_decodes() {
        let example = serde_json::json!(45);
        let _ = serde_json::from_value::<RootRoll>(example).expect("example should decode");
    }

    #[test]
    fn test_example_root_sky_decodes() {
        let example = serde_json::json!({"atmosphere-blend":["interpolate",["linear"],["zoom"],0,1,10,1,12,0],"fog-color":"#0000ff","fog-ground-blend":0.5,"horizon-color":"#ffffff","horizon-fog-blend":0.5,"sky-color":"#199EF3","sky-horizon-blend":0.5});
        let _ = serde_json::from_value::<RootSky>(example).expect("example should decode");
    }

    #[test]
    fn test_example_root_sources_decodes() {
        let example = serde_json::json!({"maplibre-demotiles":{"type":"vector","url":"https://demotiles.maplibre.org/tiles/tiles.json"},"maplibre-streets":{"maxzoom":14,"tiles":["http://a.example.com/tiles/{z}/{x}/{y}.pbf","http://b.example.com/tiles/{z}/{x}/{y}.pbf"],"type":"vector"},"maplibre-tilejson":{"type":"vector","url":"http://api.example.com/tilejson.json"},"wms-imagery":{"tileSize":256,"tiles":["http://a.example.com/wms?bbox={bbox-epsg-3857}&format=image/png&service=WMS&version=1.1.1&request=GetMap&srs=EPSG:3857&width=256&height=256&layers=example"],"type":"raster"}});
        let _ = serde_json::from_value::<RootSources>(example).expect("example should decode");
    }

    #[test]
    fn test_example_root_state_decodes() {
        let example = serde_json::json!({"chargerType":{"default":["CCS","CHAdeMO","Type2"]},"minPreferredChargingSpeed":{"default":50}});
        let _ = serde_json::from_value::<RootState>(example).expect("example should decode");
    }

    #[test]
    fn test_example_root_terrain_decodes() {
        let example = serde_json::json!({"exaggeration":0.5,"source":"raster-dem-source"});
        let _ = serde_json::from_value::<RootTerrain>(example).expect("example should decode");
    }

    #[test]
    fn test_example_root_transition_decodes() {
        let example = serde_json::json!({"delay":0,"duration":300});
        let _ = serde_json::from_value::<RootTransition>(example).expect("example should decode");
    }

    #[test]
    fn test_example_root_version_decodes() {
        let example = serde_json::json!(8);
        let _ = serde_json::from_value::<RootVersion>(example).expect("example should decode");
    }

    #[test]
    fn test_example_root_zoom_decodes() {
        let example = serde_json::json!(12.5);
        let _ = serde_json::from_value::<RootZoom>(example).expect("example should decode");
    }
}

/// Default map center in longitude and latitude.  The style center will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RootCenter(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_2_json_number))]
     Box<[serde_json::Number; 2]>,
);

/// Default map center altitude in meters above sea level. The style center altitude defines the altitude where the camera is looking at and will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RootCenterAltitude(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

/// Font file URL and the unicode-range at which it can be used
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FontWithRange {
    /// URL the font can retrieved under
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_url))]
    pub url: url::Url,
    /// Unicode range(s) where this font applies (CSS `unicode-range` semantics)
    #[serde(rename = "unicode-range")]
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unicode_range: Vec<std::string::String>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FontFace {
    /// A single global font file URL
    Url(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_url))]
        url::Url,
    ),
    /// Load different fonts depending on the unicode range
    FontRange(Vec<FontWithRange>),
}

impl serde::Serialize for FontFace {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Url(v) => v.serialize(serializer),
            Self::FontRange(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for FontFace {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <url::Url as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Url(v)),
            Err(e) => errors.push(("Url", e.to_string())),
        }
        match <Vec<FontWithRange> as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::FontRange(v)),
            Err(e) => errors.push(("FontRange", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "FontFace: no variant matched. Expected Url(url::Url) | FontRange(Vec<FontWithRange>). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The `font-faces` property can be used to specify what font files to use for rendering text. Font faces contain information needed to render complex texts such as [Devanagari](https://en.wikipedia.org/wiki/Devanagari), [Khmer](https://en.wikipedia.org/wiki/Khmer_script) among many others.<h2>Unicode range</h2>The optional `unicode-range` property can be used to only use a particular font file for characters within the specified unicode range(s). Its value should be an array of strings, each indicating a start and end of a unicode range, similar to the [CSS descriptor with the same name](https://developer.mozilla.org/en-US/docs/Web/CSS/@font-face/unicode-range). This allows specifying multiple non-consecutive unicode ranges. When not specified, the default value is `U+0-10FFFF`, meaning the font file will be used for all unicode characters.
///
/// Refer to the [Unicode Character Code Charts](https://www.unicode.org/charts/) to see ranges for scripts supported by Unicode. To see what unicode code-points are available in a font, use a tool like [FontDrop](https://fontdrop.info/).
///
/// <h2>Font Resolution</h2>For every name in a symbol layer’s [`text-font`](./layers.md/#text-font) array, characters are matched if they are covered one of the by the font files in the corresponding entry of the `font-faces` map. Any still-unmatched characters then fall back to the [`glyphs`](./glyphs.md) URL if provided.
///
/// <h2>Supported Fonts</h2>What type of fonts are supported is implementation-defined. Unsupported fonts are ignored.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RootFontFaces(std::collections::BTreeMap<std::string::String, FontFace>);

/// The global light source.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RootLight(pub Light);

/// Arbitrary properties useful to track with the stylesheet, but do not influence rendering. Properties should be prefixed to avoid collisions, like 'maplibre:'.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RootMetadata(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
    serde_json::Value,
);

/// A human-readable name for the style.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RootName(std::string::String);

/// Default pitch, in degrees. Zero is perpendicular to the surface, for a look straight down at the map, while a greater value like 60 looks ahead towards the horizon. The style pitch will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RootPitch(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for RootPitch {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The projection configuration
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RootProjection(pub Projection);

/// Default roll, in degrees. The roll angle is measured counterclockwise about the camera boresight. The style roll will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RootRoll(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for RootRoll {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The map's sky configuration. **Note:** this definition is still experimental and is under development in maplibre-gl-js.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RootSky(pub Sky);

/// Sources state which data the map should display. Specify the type of source with the `type` property. Adding a source isn't enough to make data appear on the map because sources don't contain styling details like color or width. Layers refer to a source and give it a visual representation. This makes it possible to style the same source in different ways, like differentiating between types of roads in a highways layer.
///
/// Tiled sources (vector and raster) must specify their details according to the [TileJSON specification](https://github.com/mapbox/tilejson-spec).
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RootSources(pub std::collections::BTreeMap<std::string::String, Source>);

/// An object used to define default values when using the [`global-state`](https://maplibre.org/maplibre-style-spec/expressions/#global-state) expression.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RootState(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
    serde_json::Value,
);

impl Default for RootState {
    fn default() -> Self {
        Self(serde_json::json!({}))
    }
}

/// The terrain configuration.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RootTerrain(pub Terrain);

/// A global transition definition to use as a default across properties, to be used for timing transitions between one value and the next when no property-specific transition is set. Collision-based symbol fading is controlled independently of the style's `transition` property.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RootTransition(pub Transition);

/// Style specification version number. Must be 8.
#[derive(
    serde_repr::Serialize_repr, serde_repr::Deserialize_repr, PartialEq, Eq, Debug, Clone, Copy,
)]
#[repr(u8)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum RootVersion {
    Eight = 8,
}

/// Default zoom level.  The style zoom will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RootZoom(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

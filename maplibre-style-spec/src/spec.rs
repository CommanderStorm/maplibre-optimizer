/// This is a Maplibre Style Specification
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct MaplibreStyleSpecification {
    /// Default bearing, in degrees. The bearing is the compass direction that is "up"; for example, a bearing of 90° orients the map so that east is up. This value will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    pub bearing: RootBearing,
    /// Default map center in longitude and latitude.  The style center will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    pub center: RootCenter,
    /// Default map center altitude in meters above sea level. The style center altitude defines the altitude where the camera is looking at and will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    #[serde(rename = "centerAltitude")]
    pub center_altitude: RootCenterAltitude,
    /// The `font-faces` property can be used to specify what font files to use for rendering text. Font faces contain information needed to render complex texts such as [Devanagari](https://en.wikipedia.org/wiki/Devanagari), [Khmer](https://en.wikipedia.org/wiki/Khmer_script) among many others.<h2>Unicode range</h2>The optional `unicode-range` property can be used to only use a particular font file for characters within the specified unicode range(s). Its value should be an array of strings, each indicating a start and end of a unicode range, similar to the [CSS descriptor with the same name](https://developer.mozilla.org/en-US/docs/Web/CSS/@font-face/unicode-range). This allows specifying multiple non-consecutive unicode ranges. When not specified, the default value is `U+0-10FFFF`, meaning the font file will be used for all unicode characters.
    ///
    /// Refer to the [Unicode Character Code Charts](https://www.unicode.org/charts/) to see ranges for scripts supported by Unicode. To see what unicode code-points are available in a font, use a tool like [FontDrop](https://fontdrop.info/).
    ///
    /// <h2>Font Resolution</h2>For every name in a symbol layer’s [`text-font`](./layers.md/#text-font) array, characters are matched if they are covered one of the by the font files in the corresponding entry of the `font-faces` map. Any still-unmatched characters then fall back to the [`glyphs`](./glyphs.md) URL if provided.
    ///
    /// <h2>Supported Fonts</h2>What type of fonts are supported is implementation-defined. Unsupported fonts are ignored.
    #[serde(rename = "font-faces")]
    pub font_faces: RootFontFaces,
    /// A URL template for loading signed-distance-field glyph sets in PBF format.
    ///
    /// If this property is set, any text in the `text-field` layout property is displayed in the font stack named by the `text-font` layout property based on glyphs located at the URL specified by this property. Otherwise, font faces will be determined by the `text-font` property based on the local environment.
    ///
    /// The URL must include:
    ///
    ///  - `{fontstack}` - When requesting glyphs, this token is replaced with a comma separated list of fonts from a font stack specified in the `text-font` property of a symbol layer.
    ///
    ///  - `{range}` - When requesting glyphs, this token is replaced with a range of 256 Unicode code points. For example, to load glyphs for the Unicode Basic Latin and Basic Latin-1 Supplement blocks, the range would be 0-255. The actual ranges that are loaded are determined at runtime based on what text needs to be displayed.
    ///
    /// The URL must be absolute, containing the [scheme, authority and path components](https://en.wikipedia.org/wiki/URL#Syntax).
    pub glyphs: RootGlyphs,
    /// A style's `layers` property lists all the layers available in that style. The type of layer is specified by the `type` property, and must be one of `background`, `fill`, `line`, `symbol`, `raster`, `circle`, `fill-extrusion`, `heatmap`, `hillshade`, `color-relief`.
    ///
    /// Except for layers of the `background` type, each layer needs to refer to a source. Layers take the data that they get from a source, optionally filter features, and then define how those features are styled.
    pub layers: RootLayers,
    /// The global light source.
    pub light: RootLight,
    /// Arbitrary properties useful to track with the stylesheet, but do not influence rendering. Properties should be prefixed to avoid collisions, like 'maplibre:'.
    pub metadata: RootMetadata,
    /// A human-readable name for the style.
    pub name: RootName,
    /// Default pitch, in degrees. Zero is perpendicular to the surface, for a look straight down at the map, while a greater value like 60 looks ahead towards the horizon. The style pitch will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    pub pitch: RootPitch,
    /// The projection configuration
    pub projection: RootProjection,
    /// Default roll, in degrees. The roll angle is measured counterclockwise about the camera boresight. The style roll will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    pub roll: RootRoll,
    /// The map's sky configuration. **Note:** this definition is still experimental and is under development in maplibre-gl-js.
    pub sky: RootSky,
    /// Sources state which data the map should display. Specify the type of source with the `type` property. Adding a source isn't enough to make data appear on the map because sources don't contain styling details like color or width. Layers refer to a source and give it a visual representation. This makes it possible to style the same source in different ways, like differentiating between types of roads in a highways layer.
    ///
    /// Tiled sources (vector and raster) must specify their details according to the [TileJSON specification](https://github.com/mapbox/tilejson-spec).
    pub sources: RootSources,
    /// An array of `{id: 'my-sprite', url: 'https://example.com/sprite'}` objects. Each object should represent a unique URL to load a sprite from and and a unique ID to use as a prefix when referencing images from that sprite (i.e. 'my-sprite:image'). All the URLs are internally extended to load both .json and .png files. If the `id` field is equal to 'default', the prefix is omitted (just 'image' instead of 'default:image'). All the IDs and URLs must be unique. For backwards compatibility, instead of an array, one can also provide a single string that represent a URL to load the sprite from. The images in this case won't be prefixed.
    pub sprite: RootSprite,
    /// An object used to define default values when using the [`global-state`](https://maplibre.org/maplibre-style-spec/expressions/#global-state) expression.
    pub state: RootState,
    /// The terrain configuration.
    pub terrain: RootTerrain,
    /// A global transition definition to use as a default across properties, to be used for timing transitions between one value and the next when no property-specific transition is set. Collision-based symbol fading is controlled independently of the style's `transition` property.
    pub transition: RootTransition,
    /// Style specification version number. Must be 8.
    pub version: RootVersion,
    /// Default zoom level.  The style zoom will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    pub zoom: RootZoom,
}

/// Default bearing, in degrees. The bearing is the compass direction that is "up"; for example, a bearing of 90° orients the map so that east is up. This value will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
///
/// Range: every 360
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct RootBearing(serde_json::Number);

impl Default for RootBearing {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

#[cfg(test)]
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
    fn test_example_root_glyphs_decodes() {
        let example =
            serde_json::json!("https://demotiles.maplibre.org/font/{fontstack}/{range}.pbf");
        let _ = serde_json::from_value::<RootGlyphs>(example).expect("example should decode");
    }

    #[test]
    fn test_example_root_layers_decodes() {
        let example = serde_json::json!([{"id":"coastline","paint":{"line-color":"#198EC8"},"source":"maplibre","source-layer":"countries","type":"line"}]);
        let _ = serde_json::from_value::<RootLayers>(example).expect("example should decode");
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
    fn test_example_root_sprite_decodes() {
        let example =
            serde_json::json!("https://demotiles.maplibre.org/styles/osm-bright-gl-style/sprite");
        let _ = serde_json::from_value::<RootSprite>(example).expect("example should decode");
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

    #[test]
    fn test_example_layer_metadata_decodes() {
        let example = serde_json::json!({"source:comment":"Hydrology FCCODE 460 - Narrow wash"});
        let _ = serde_json::from_value::<LayerMetadata>(example).expect("example should decode");
    }

    #[test]
    fn test_example_light_anchor_decodes() {
        let example = serde_json::json!("map");
        let _ = serde_json::from_value::<LightAnchor>(example).expect("example should decode");
    }

    #[test]
    fn test_example_light_position_decodes() {
        let example = serde_json::json!([1.5, 90, 80]);
        let _ = serde_json::from_value::<LightPosition>(example).expect("example should decode");
    }

    #[test]
    fn test_example_paint_color_relief_color_relief_color_decodes() {
        let example = serde_json::json!([
            "interpolate",
            ["linear"],
            ["elevation"],
            0,
            "black",
            8849,
            "white"
        ]);
        let _ = serde_json::from_value::<PaintColorReliefColorReliefColor>(example)
            .expect("example should decode");
    }
}

/// Default map center in longitude and latitude.  The style center will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct RootCenter(Vec<serde_json::Number>);

/// Default map center altitude in meters above sea level. The style center altitude defines the altitude where the camera is looking at and will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct RootCenterAltitude(serde_json::Number);

/// The `font-faces` property can be used to specify what font files to use for rendering text. Font faces contain information needed to render complex texts such as [Devanagari](https://en.wikipedia.org/wiki/Devanagari), [Khmer](https://en.wikipedia.org/wiki/Khmer_script) among many others.<h2>Unicode range</h2>The optional `unicode-range` property can be used to only use a particular font file for characters within the specified unicode range(s). Its value should be an array of strings, each indicating a start and end of a unicode range, similar to the [CSS descriptor with the same name](https://developer.mozilla.org/en-US/docs/Web/CSS/@font-face/unicode-range). This allows specifying multiple non-consecutive unicode ranges. When not specified, the default value is `U+0-10FFFF`, meaning the font file will be used for all unicode characters.
///
/// Refer to the [Unicode Character Code Charts](https://www.unicode.org/charts/) to see ranges for scripts supported by Unicode. To see what unicode code-points are available in a font, use a tool like [FontDrop](https://fontdrop.info/).
///
/// <h2>Font Resolution</h2>For every name in a symbol layer’s [`text-font`](./layers.md/#text-font) array, characters are matched if they are covered one of the by the font files in the corresponding entry of the `font-faces` map. Any still-unmatched characters then fall back to the [`glyphs`](./glyphs.md) URL if provided.
///
/// <h2>Supported Fonts</h2>What type of fonts are supported is implementation-defined. Unsupported fonts are ignored.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct RootFontFaces(Vec<FontFaces>);

/// A URL template for loading signed-distance-field glyph sets in PBF format.
///
/// If this property is set, any text in the `text-field` layout property is displayed in the font stack named by the `text-font` layout property based on glyphs located at the URL specified by this property. Otherwise, font faces will be determined by the `text-font` property based on the local environment.
///
/// The URL must include:
///
///  - `{fontstack}` - When requesting glyphs, this token is replaced with a comma separated list of fonts from a font stack specified in the `text-font` property of a symbol layer.
///
///  - `{range}` - When requesting glyphs, this token is replaced with a range of 256 Unicode code points. For example, to load glyphs for the Unicode Basic Latin and Basic Latin-1 Supplement blocks, the range would be 0-255. The actual ranges that are loaded are determined at runtime based on what text needs to be displayed.
///
/// The URL must be absolute, containing the [scheme, authority and path components](https://en.wikipedia.org/wiki/URL#Syntax).
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct RootGlyphs(String);

/// A style's `layers` property lists all the layers available in that style. The type of layer is specified by the `type` property, and must be one of `background`, `fill`, `line`, `symbol`, `raster`, `circle`, `fill-extrusion`, `heatmap`, `hillshade`, `color-relief`.
///
/// Except for layers of the `background` type, each layer needs to refer to a source. Layers take the data that they get from a source, optionally filter features, and then define how those features are styled.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct RootLayers(Vec<Layer>);

/// The global light source.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct RootLight(Light);

/// Arbitrary properties useful to track with the stylesheet, but do not influence rendering. Properties should be prefixed to avoid collisions, like 'maplibre:'.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct RootMetadata(serde_json::Value);

/// A human-readable name for the style.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct RootName(String);

/// Default pitch, in degrees. Zero is perpendicular to the surface, for a look straight down at the map, while a greater value like 60 looks ahead towards the horizon. The style pitch will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct RootPitch(serde_json::Number);

impl Default for RootPitch {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The projection configuration
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct RootProjection(Projection);

/// Default roll, in degrees. The roll angle is measured counterclockwise about the camera boresight. The style roll will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct RootRoll(serde_json::Number);

impl Default for RootRoll {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The map's sky configuration. **Note:** this definition is still experimental and is under development in maplibre-gl-js.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct RootSky(Sky);

/// Sources state which data the map should display. Specify the type of source with the `type` property. Adding a source isn't enough to make data appear on the map because sources don't contain styling details like color or width. Layers refer to a source and give it a visual representation. This makes it possible to style the same source in different ways, like differentiating between types of roads in a highways layer.
///
/// Tiled sources (vector and raster) must specify their details according to the [TileJSON specification](https://github.com/mapbox/tilejson-spec).
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct RootSources(Sources);

/// URL where the sprite can be loaded from.
///
/// This is equivalent to the following multiple sprite definition:
///
/// ```json
/// {
///         "id": "default",
///         "url": "https://example2.com/anotherurl"
/// }
/// ```
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SpriteUrlAndId {
    id: String,
    url: url::Url,
}

/// An array of `{id: 'my-sprite', url: 'https://example.com/sprite'}` objects. Each object should represent a unique URL to load a sprite from and and a unique ID to use as a prefix when referencing images from that sprite (i.e. 'my-sprite:image'). All the URLs are internally extended to load both .json and .png files. If the `id` field is equal to 'default', the prefix is omitted (just 'image' instead of 'default:image'). All the IDs and URLs must be unique. For backwards compatibility, instead of an array, one can also provide a single string that represent a URL to load the sprite from. The images in this case won't be prefixed.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub enum RootSprite {
    /// URL where the sprite can be loaded from
    Url(url::Url),
    /// Array of `{ id: ..., url: ... }` pairs to load multiple sprites
    Multiple(Vec<SpriteUrlAndId>),
}

/// An object used to define default values when using the [`global-state`](https://maplibre.org/maplibre-style-spec/expressions/#global-state) expression.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct RootState(serde_json::Value);

impl Default for RootState {
    fn default() -> Self {
        Self(serde_json::json!({}))
    }
}

/// The terrain configuration.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct RootTerrain(Terrain);

/// A global transition definition to use as a default across properties, to be used for timing transitions between one value and the next when no property-specific transition is set. Collision-based symbol fading is controlled independently of the style's `transition` property.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "transition not implemented"]
struct RootTransition(serde_json::Value);

/// Style specification version number. Must be 8.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
#[repr(u8)]
pub enum RootVersion {
    Eight = 8,
}

/// Default zoom level.  The style zoom will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct RootZoom(serde_json::Number);

/// An expression defines a function that can be used for data-driven style properties or feature filters.
///
/// Range: 1..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct Expression(Vec<serde_json::Value>);

#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum ExpressionName {
    /// Logical negation. Returns `true` if the input is `false`, and `false` if the input is `true`.
    ///
    ///  - [Create and style clusters](https://maplibre.org/maplibre-gl-js/docs/examples/create-and-style-clusters/)
    #[serde(rename = "!")]
    Not,
    /// Returns `true` if the input values are not equal, `false` otherwise. The comparison is strictly typed: values of different runtime types are always considered unequal. Cases where the types are known to be different at parse time are considered invalid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    #[serde(rename = "!=")]
    NotEqual,
    /// Returns the remainder after integer division of the first input by the second.
    #[serde(rename = "%")]
    Percentage,
    /// Returns the product of the inputs.
    #[serde(rename = "*")]
    Star,
    /// Returns the sum of the inputs.
    #[serde(rename = "+")]
    Plus,
    /// For two inputs, returns the result of subtracting the second input from the first. For a single input, returns the result of subtracting it from 0.
    #[serde(rename = "-")]
    Minus,
    /// Returns the result of floating point division of the first input by the second.
    ///
    ///  - [Visualize population density](https://maplibre.org/maplibre-gl-js/docs/examples/visualize-population-density/)
    #[serde(rename = "/")]
    Slash,
    /// Returns `true` if the first input is strictly less than the second, `false` otherwise. The arguments are required to be either both strings or both numbers; if during evaluation they are not, expression evaluation produces an error. Cases where this constraint is known not to hold at parse time are considered in valid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    #[serde(rename = "<")]
    Less,
    /// Returns `true` if the first input is less than or equal to the second, `false` otherwise. The arguments are required to be either both strings or both numbers; if during evaluation they are not, expression evaluation produces an error. Cases where this constraint is known not to hold at parse time are considered in valid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    #[serde(rename = "<=")]
    LessEqual,
    /// Returns `true` if the input values are equal, `false` otherwise. The comparison is strictly typed: values of different runtime types are always considered unequal. Cases where the types are known to be different at parse time are considered invalid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    ///
    ///  - [Add multiple geometries from one GeoJSON source](https://maplibre.org/maplibre-gl-js/docs/examples/multiple-geometries/)
    ///
    ///  - [Create a time slider](https://maplibre.org/maplibre-gl-js/docs/examples/timeline-animation/)
    ///
    ///  - [Display buildings in 3D](https://maplibre.org/maplibre-gl-js/docs/examples/display-buildings-in-3d/)
    ///
    ///  - [Filter symbols by toggling a list](https://maplibre.org/maplibre-gl-js/docs/examples/filter-symbols-by-toggling-a-list/)
    #[serde(rename = "==")]
    EqualEqual,
    /// Returns `true` if the first input is strictly greater than the second, `false` otherwise. The arguments are required to be either both strings or both numbers; if during evaluation they are not, expression evaluation produces an error. Cases where this constraint is known not to hold at parse time are considered in valid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    #[serde(rename = ">")]
    Greater,
    /// Returns `true` if the first input is greater than or equal to the second, `false` otherwise. The arguments are required to be either both strings or both numbers; if during evaluation they are not, expression evaluation produces an error. Cases where this constraint is known not to hold at parse time are considered in valid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    #[serde(rename = ">=")]
    GreaterEqual,
    /// Returns the result of raising the first input to the power specified by the second.
    #[serde(rename = "^")]
    Power,
    /// Returns the absolute value of the input.
    #[serde(rename = "abs")]
    Absolute,
    /// Gets the value of a cluster property accumulated so far. Can only be used in the `clusterProperties` option of a clustered GeoJSON source.
    #[serde(rename = "accumulated")]
    Accumulated,
    /// Returns the arccosine of the input.
    #[serde(rename = "acos")]
    Arccosine,
    /// Returns `true` if all the inputs are `true`, `false` otherwise. The inputs are evaluated in order, and evaluation is short-circuiting: once an input expression evaluates to `false`, the result is `false` and no further input expressions are evaluated.
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    #[serde(rename = "all")]
    All,
    /// Returns `true` if any of the inputs are `true`, `false` otherwise. The inputs are evaluated in order, and evaluation is short-circuiting: once an input expression evaluates to `true`, the result is `true` and no further input expressions are evaluated.
    #[serde(rename = "any")]
    Any,
    /// Asserts that the input is an array (optionally with a specific item type and length). If, when the input expression is evaluated, it is not of the asserted type or length, then this assertion will cause the whole expression to be aborted.
    #[serde(rename = "array")]
    Array,
    /// Returns the arcsine of the input.
    #[serde(rename = "asin")]
    Asin,
    /// Retrieves an item from an array.
    #[serde(rename = "at")]
    At,
    /// Returns the arctangent of the input.
    #[serde(rename = "atan")]
    Atan,
    /// Asserts that the input value is a boolean. If multiple values are provided, each one is evaluated in order until a boolean is obtained. If none of the inputs are booleans, the expression is an error.
    ///
    ///  - [Create a hover effect](https://maplibre.org/maplibre-gl-js/docs/examples/create-a-hover-effect/)
    #[serde(rename = "boolean")]
    Boolean,
    /// Selects the first output whose corresponding test condition evaluates to true, or the fallback value otherwise.
    ///
    ///  - [Create a hover effect](https://maplibre.org/maplibre-gl-js/docs/examples/create-a-hover-effect/)
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    #[serde(rename = "case")]
    Case,
    /// Returns the smallest integer that is greater than or equal to the input.
    #[serde(rename = "ceil")]
    Ceil,
    /// Evaluates each expression in turn until the first non-null value is obtained, and returns that value.
    ///
    ///  - [Use a fallback image](https://maplibre.org/maplibre-gl-js/docs/examples/use-a-fallback-image/)
    #[serde(rename = "coalesce")]
    Coalesce,
    /// Returns a `collator` for use in locale-dependent comparison operations. The `case-sensitive` and `diacritic-sensitive` options default to `false`. The `locale` argument specifies the IETF language tag of the locale to use. If none is provided, the default locale is used. If the requested locale is not available, the `collator` will use a system-defined fallback locale. Use `resolved-locale` to test the results of locale fallback behavior.
    #[serde(rename = "collator")]
    Collator,
    /// Returns a `string` consisting of the concatenation of the inputs. Each input is converted to a string as if by `to-string`.
    ///
    ///  - [Add a generated icon to the map](https://maplibre.org/maplibre-gl-js/docs/examples/add-a-generated-icon-to-the-map/)
    ///
    ///  - [Create a time slider](https://maplibre.org/maplibre-gl-js/docs/examples/create-a-time-slider/)
    ///
    ///  - [Use a fallback image](https://maplibre.org/maplibre-gl-js/docs/examples/fallback-image/)
    ///
    ///  - [Variable label placement](https://maplibre.org/maplibre-gl-js/docs/examples/variable-label-placement/)
    #[serde(rename = "concat")]
    Concat,
    /// Returns the cosine of the input.
    #[serde(rename = "cos")]
    Cos,
    /// Returns the shortest distance in meters between the evaluated feature and the input geometry. The input value can be a valid GeoJSON of type `Point`, `MultiPoint`, `LineString`, `MultiLineString`, `Polygon`, `MultiPolygon`, `Feature`, or `FeatureCollection`. Distance values returned may vary in precision due to loss in precision from encoding geometries, particularly below zoom level 13.
    #[serde(rename = "distance")]
    Distance,
    /// Returns the input string converted to lowercase. Follows the Unicode Default Case Conversion algorithm and the locale-insensitive case mappings in the Unicode Character Database.
    ///
    ///  - [Change the case of labels](https://maplibre.org/maplibre-gl-js/docs/examples/change-case-of-labels/)
    #[serde(rename = "downcase")]
    Downcase,
    /// Returns the mathematical constant e.
    #[serde(rename = "e")]
    E,
    /// Gets the elevation of a pixel (in meters above the vertical datum reference of the `raster-dem` tiles) from a `raster-dem` source. Can only be used in the `color-relief-color` property of a `color-relief` layer.
    #[serde(rename = "elevation")]
    Elevation,
    /// Retrieves a property value from the current feature's state. Returns null if the requested property is not present on the feature's state. A feature's state is not part of the GeoJSON or vector tile data, and must be set programmatically on each feature. When `source.promoteId` is not provided, features are identified by their `id` attribute, which must be an integer or a string that can be cast to an integer. When `source.promoteId` is provided, features are identified by their `promoteId` property, which may be a number, string, or any primitive data type. Note that ["feature-state"] can only be used with paint properties that support data-driven styling.
    ///
    ///  - [Create a hover effect](https://maplibre.org/maplibre-gl-js/docs/examples/create-a-hover-effect/)
    #[serde(rename = "feature-state")]
    FeatureState,
    /// Returns the largest integer that is less than or equal to the input.
    #[serde(rename = "floor")]
    Floor,
    /// Returns a `formatted` string for displaying mixed-format text in the `text-field` property. The input may contain a string literal or expression, including an [`'image'`](#image) expression. Strings may be followed by a style override object that supports the following properties:
    ///
    /// - `"text-font"`: Overrides the font stack specified by the root layout property.
    ///
    /// - `"text-color"`: Overrides the color specified by the root paint property.
    ///
    /// - `"font-scale"`: Applies a scaling factor on `text-size` as specified by the root layout property.
    ///
    /// - `"vertical-align"`: Aligns vertically text section or image in relation to the row it belongs to. Possible values are:
    /// 	- `"bottom"` *default*: align the bottom of this section with the bottom of other sections.
    /// <img alt="Visual representation of bottom alignment" src="https://github.com/user-attachments/assets/0474a2fd-a4b2-417c-9187-7a13a28695bc"/>
    /// 	- `"center"`: align the center of this section with the center of other sections.
    /// <img alt="Visual representation of center alignment" src="https://github.com/user-attachments/assets/92237455-be6d-4c5d-b8f6-8127effc1950"/>
    /// 	- `"top"`: align the top of this section with the top of other sections.
    /// <img alt="Visual representation of top alignment" src="https://github.com/user-attachments/assets/45dccb28-d977-4abb-a006-4ea9792b7c53"/>
    /// 	- Refer to [the design proposal](https://github.com/maplibre/maplibre-style-spec/issues/832) for more details.
    ///
    ///  - [Change the case of labels](https://maplibre.org/maplibre-gl-js/docs/examples/change-case-of-labels/)
    ///
    ///  - [Display and style rich text labels](https://maplibre.org/maplibre-gl-js/docs/examples/display-and-style-rich-text-labels/)
    #[serde(rename = "format")]
    Format,
    /// Returns the feature's simple geometry type: `Point`, `LineString`, or `Polygon`. `MultiPoint`, `MultiLineString`, and `MultiPolygon` are returned as `Point`, `LineString`, and `Polygon`, respectively.
    #[serde(rename = "geometry-type")]
    GeometryType,
    /// Retrieves a property value from the current feature's properties, or from another object if a second argument is provided. Returns null if the requested property is missing.
    ///
    ///  - [Change the case of labels](https://maplibre.org/maplibre-gl-js/docs/examples/change-case-of-labels/)
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    ///
    ///  - [Extrude polygons for 3D indoor mapping](https://maplibre.org/maplibre-gl-js/docs/examples/extrude-polygons-for-3d-indoor-mapping/)
    #[serde(rename = "get")]
    Get,
    /// Retrieves a property value from global state that can be set with platform-specific APIs. Defaults can be provided using the [`state`](https://maplibre.org/maplibre-style-spec/root/#state) root property. Returns `null` if no value nor default value is set for the retrieved property.
    #[serde(rename = "global-state")]
    GlobalState,
    /// Tests for the presence of a property value in the current feature's properties, or from another object if a second argument is provided.
    ///
    ///  - [Create and style clusters](https://maplibre.org/maplibre-gl-js/docs/examples/create-and-style-clusters/)
    #[serde(rename = "has")]
    Has,
    /// Gets the kernel density estimation of a pixel in a heatmap layer, which is a relative measure of how many data points are crowded around a particular pixel. Can only be used in the `heatmap-color` property.
    #[serde(rename = "heatmap-density")]
    HeatmapDensity,
    /// Gets the feature's id, if it has one.
    #[serde(rename = "id")]
    Id,
    /// Returns an `image` type for use in `icon-image`, `*-pattern` entries and as a section in the `format` expression. If set, the `image` argument will check that the requested image exists in the style and will return either the resolved image name or `null`, depending on whether or not the image is currently in the style. This validation process is synchronous and requires the image to have been added to the style before requesting it in the `image` argument.
    ///
    ///  - [Use a fallback image](https://maplibre.org/maplibre-gl-js/docs/examples/use-a-fallback-image/)
    #[serde(rename = "image")]
    Image,
    /// Determines whether an item exists in an array or a substring exists in a string.
    ///
    ///  - [Measure distances](https://maplibre.org/maplibre-gl-js/docs/examples/measure-distances/)
    #[serde(rename = "in")]
    In,
    /// Returns the first position at which an item can be found in an array or a substring can be found in a string, or `-1` if the input cannot be found. Accepts an optional index from where to begin the search. In a string, a UTF-16 surrogate pair counts as a single position.
    #[serde(rename = "index-of")]
    IndexOf,
    /// Produces continuous, smooth results by interpolating between pairs of input and output values ("stops"). The `input` may be any numeric expression (e.g., `["get", "population"]`). Stop inputs must be numeric literals in strictly ascending order. The output type must be `number`, `array<number>`, `color`, `array<color>`, or `projection`.
    ///
    /// Interpolation types:
    ///
    /// - `["linear"]`, or an expression returning one of those types: Interpolates linearly between the pair of stops just less than and just greater than the input.
    ///
    /// - `["exponential", base]`: Interpolates exponentially between the stops just less than and just greater than the input. `base` controls the rate at which the output increases: higher values make the output increase more towards the high end of the range. With values close to 1 the output increases linearly.
    ///
    /// - `["cubic-bezier", x1, y1, x2, y2]`: Interpolates using the cubic bezier curve defined by the given control points.
    ///
    ///  - [Animate map camera around a point](https://maplibre.org/maplibre-gl-js/docs/examples/animate-camera-around-point/)
    ///
    ///  - [Change building color based on zoom level](https://maplibre.org/maplibre-gl-js/docs/examples/change-building-color-based-on-zoom-level/)
    ///
    ///  - [Create a heatmap layer](https://maplibre.org/maplibre-gl-js/docs/examples/heatmap-layer/)
    ///
    ///  - [Visualize population density](https://maplibre.org/maplibre-gl-js/docs/examples/visualize-population-density/)
    #[serde(rename = "interpolate")]
    Interpolate,
    /// Produces continuous, smooth results by interpolating between pairs of input and output values ("stops"). Works like `interpolate`, but the output type must be `color` or `array<color>`, and the interpolation is performed in the Hue-Chroma-Luminance color space.
    #[serde(rename = "interpolate-hcl")]
    InterpolateHcl,
    /// Produces continuous, smooth results by interpolating between pairs of input and output values ("stops"). Works like `interpolate`, but the output type must be `color` or `array<color>`, and the interpolation is performed in the CIELAB color space.
    #[serde(rename = "interpolate-lab")]
    InterpolateLab,
    /// Returns `true` if the input string is expected to render legibly. Returns `false` if the input string contains sections that cannot be rendered without potential loss of meaning (e.g. Indic scripts that require complex text shaping, or right-to-left scripts if the `mapbox-gl-rtl-text` plugin is not in use in MapLibre GL JS).
    #[serde(rename = "is-supported-script")]
    IsSupportedScript,
    /// Gets the length of an array or string. In a string, a UTF-16 surrogate pair counts as a single position.
    #[serde(rename = "length")]
    Length,
    /// Binds expressions to named variables, which can then be referenced in the result expression using `["var", "variable_name"]`.
    ///
    ///  - [Visualize population density](https://maplibre.org/maplibre-gl-js/docs/examples/visualize-population-density/)
    #[serde(rename = "let")]
    Let,
    /// Gets the progress along a gradient line. Can only be used in the `line-gradient` property.
    #[serde(rename = "line-progress")]
    LineProgress,
    /// Provides a literal array or object value.
    ///
    ///  - [Display and style rich text labels](https://maplibre.org/maplibre-gl-js/docs/examples/display-and-style-rich-text-labels/)
    #[serde(rename = "literal")]
    Literal,
    /// Returns the natural logarithm of the input.
    #[serde(rename = "ln")]
    Ln,
    /// Returns the mathematical constant ln(2).
    #[serde(rename = "ln2")]
    Ln2,
    /// Returns the base-ten logarithm of the input.
    #[serde(rename = "log10")]
    Log10,
    /// Returns the base-two logarithm of the input.
    #[serde(rename = "log2")]
    Log2,
    /// Selects the output whose label value matches the input value, or the fallback value if no match is found. The input can be any expression (e.g. `["get", "building_type"]`). Each label must be either:
    ///
    ///  - a single literal value; or
    ///
    ///  - an array of literal values, whose values must be all strings or all numbers (e.g. `[100, 101]` or `["c", "b"]`). The input matches if any of the values in the array matches, similar to the `"in"` operator.
    ///
    /// Each label must be unique. If the input type does not match the type of the labels, the result will be the fallback value.
    #[serde(rename = "match")]
    Match,
    /// Returns the maximum value of the inputs.
    #[serde(rename = "max")]
    Max,
    /// Returns the minimum value of the inputs.
    #[serde(rename = "min")]
    Min,
    /// Asserts that the input value is a number. If multiple values are provided, each one is evaluated in order until a number is obtained. If none of the inputs are numbers, the expression is an error.
    #[serde(rename = "number")]
    Number,
    /// Converts the input number into a string representation using the providing formatting rules. If set, the `locale` argument specifies the locale to use, as a BCP 47 language tag. If set, the `currency` argument specifies an ISO 4217 code to use for currency-style formatting. If set, the `min-fraction-digits` and `max-fraction-digits` arguments specify the minimum and maximum number of fractional digits to include.
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    #[serde(rename = "number-format")]
    NumberFormat,
    /// Asserts that the input value is an object. If multiple values are provided, each one is evaluated in order until an object is obtained. If none of the inputs are objects, the expression is an error.
    #[serde(rename = "object")]
    Object,
    /// Returns the mathematical constant pi.
    #[serde(rename = "pi")]
    Pi,
    /// Gets the feature properties object.  Note that in some cases, it may be more efficient to use ["get", "property_name"] directly.
    #[serde(rename = "properties")]
    Properties,
    /// Returns the IETF language tag of the locale being used by the provided `collator`. This can be used to determine the default system locale, or to determine if a requested locale was successfully loaded.
    #[serde(rename = "resolved-locale")]
    ResolvedLocale,
    /// Creates a color value from red, green, and blue components, which must range between 0 and 255, and an alpha component of 1. If any component is out of range, the expression is an error.
    #[serde(rename = "rgb")]
    Rgb,
    /// Creates a color value from red, green, blue components, which must range between 0 and 255, and an alpha component which must range between 0 and 1. If any component is out of range, the expression is an error.
    #[serde(rename = "rgba")]
    Rgba,
    /// Rounds the input to the nearest integer. Halfway values are rounded away from zero. For example, `["round", -1.5]` evaluates to -2.
    #[serde(rename = "round")]
    Round,
    /// Returns the sine of the input.
    #[serde(rename = "sin")]
    Sin,
    /// Returns a subarray from an array or a substring from a string from a specified start index, or between a start index and an end index if set. The return value is inclusive of the start index but not of the end index. In a string, a UTF-16 surrogate pair counts as a single position.
    #[serde(rename = "slice")]
    Slice,
    /// Returns the square root of the input.
    #[serde(rename = "sqrt")]
    Sqrt,
    /// Produces discrete, stepped results by evaluating a piecewise-constant function defined by pairs of input and output values ("stops"). The `input` may be any numeric expression (e.g., `["get", "population"]`). Stop inputs must be numeric literals in strictly ascending order.
    ///
    /// Returns the output value of the stop just less than the input, or the first output if the input is less than the first stop.
    ///
    ///  - [Create and style clusters](https://maplibre.org/maplibre-gl-js/docs/examples/create-and-style-clusters/)
    #[serde(rename = "step")]
    Step,
    /// Asserts that the input value is a string. If multiple values are provided, each one is evaluated in order until a string is obtained. If none of the inputs are strings, the expression is an error.
    #[serde(rename = "string")]
    String,
    /// Returns the tangent of the input.
    #[serde(rename = "tan")]
    Tan,
    /// Converts the input value to a boolean. The result is `false` when the input is an empty string, 0, `false`, `null`, or `NaN`; otherwise it is `true`.
    #[serde(rename = "to-boolean")]
    ToBoolean,
    /// Converts the input value to a color. If multiple values are provided, each one is evaluated in order until the first successful conversion is obtained. If none of the inputs can be converted, the expression is an error.
    ///
    ///  - [Visualize population density](https://maplibre.org/maplibre-gl-js/docs/examples/visualize-population-density/)
    #[serde(rename = "to-color")]
    ToColor,
    /// Converts the input value to a number, if possible. If the input is `null` or `false`, the result is 0. If the input is `true`, the result is 1. If the input is a string, it is converted to a number as specified by the ["ToNumber Applied to the String Type" algorithm](https://tc39.github.io/ecma262/#sec-tonumber-applied-to-the-string-type) of the ECMAScript Language Specification. If multiple values are provided, each one is evaluated in order until the first successful conversion is obtained. If none of the inputs can be converted, the expression is an error.
    #[serde(rename = "to-number")]
    ToNumber,
    /// Returns a four-element array containing the input color's red, green, blue, and alpha components, in that order.
    #[serde(rename = "to-rgba")]
    ToRgba,
    /// Converts the input value to a string. If the input is `null`, the result is `""`. If the input is a boolean, the result is `"true"` or `"false"`. If the input is a number, it is converted to a string as specified by the ["NumberToString" algorithm](https://tc39.github.io/ecma262/#sec-tostring-applied-to-the-number-type) of the ECMAScript Language Specification. If the input is a color, it is converted to a string of the form `"rgba(r,g,b,a)"`, where `r`, `g`, and `b` are numerals ranging from 0 to 255, and `a` ranges from 0 to 1. Otherwise, the input is converted to a string in the format specified by the [`JSON.stringify`](https://tc39.github.io/ecma262/#sec-json.stringify) function of the ECMAScript Language Specification.
    ///
    ///  - [Create a time slider](https://maplibre.org/maplibre-gl-js/docs/examples/create-a-time-slider/)
    #[serde(rename = "to-string")]
    ToString,
    /// Returns a string describing the type of the given value.
    #[serde(rename = "typeof")]
    Typeof,
    /// Returns the input string converted to uppercase. Follows the Unicode Default Case Conversion algorithm and the locale-insensitive case mappings in the Unicode Character Database.
    ///
    ///  - [Change the case of labels](https://maplibre.org/maplibre-gl-js/docs/examples/change-case-of-labels/)
    #[serde(rename = "upcase")]
    Upcase,
    /// References variable bound using `let`.
    ///
    ///  - [Visualize population density](https://maplibre.org/maplibre-gl-js/docs/examples/visualize-population-density/)
    #[serde(rename = "var")]
    Var,
    /// Returns `true` if the evaluated feature is fully contained inside a boundary of the input geometry, `false` otherwise. The input value can be a valid GeoJSON of type `Polygon`, `MultiPolygon`, `Feature`, or `FeatureCollection`. Supported features for evaluation:
    ///
    /// - `Point`: Returns `false` if a point is on the boundary or falls outside the boundary.
    ///
    /// - `LineString`: Returns `false` if any part of a line falls outside the boundary, the line intersects the boundary, or a line's endpoint is on the boundary.
    #[serde(rename = "within")]
    Within,
    /// Gets the current zoom level.  Note that in style layout and paint properties, ["zoom"] may only appear as the input to a top-level "step" or "interpolate" expression.
    #[serde(rename = "zoom")]
    Zoom,
}

/// A filter selects specific features from a layer.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct Filter(Vec<serde_json::Value>);

/// The filter operator.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum FilterOperator {
    /// `["!=", key, value]` inequality: `feature[key] ≠ value`
    #[serde(rename = "!=")]
    NotEqual,
    /// `["!has", key]` `feature[key]` does not exist
    #[serde(rename = "!has")]
    NotHas,
    /// `["!in", key, v0, ..., vn]` set exclusion: `feature[key] ∉ {v0, ..., vn}`
    #[serde(rename = "!in")]
    NotIn,
    /// `["<", key, value]` less than: `feature[key] < value`
    #[serde(rename = "<")]
    Less,
    /// `["<=", key, value]` less than or equal: `feature[key] ≤ value`
    #[serde(rename = "<=")]
    LessEqual,
    /// `["==", key, value]` equality: `feature[key] = value`
    #[serde(rename = "==")]
    EqualEqual,
    /// `[">", key, value]` greater than: `feature[key] > value`
    #[serde(rename = ">")]
    Greater,
    /// `[">=", key, value]` greater than or equal: `feature[key] ≥ value`
    #[serde(rename = ">=")]
    GreaterEqual,
    /// `["all", f0, ..., fn]` logical `AND`: `f0 ∧ ... ∧ fn`
    #[serde(rename = "all")]
    All,
    /// `["any", f0, ..., fn]` logical `OR`: `f0 ∨ ... ∨ fn`
    #[serde(rename = "any")]
    Any,
    /// `["has", key]` `feature[key]` exists
    #[serde(rename = "has")]
    Has,
    /// `["in", key, v0, ..., vn]` set inclusion: `feature[key] ∈ {v0, ..., vn}`
    #[serde(rename = "in")]
    In,
    /// `["none", f0, ..., fn]` logical `NOR`: `¬f0 ∧ ... ∧ ¬fn`
    #[serde(rename = "none")]
    None,
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Function {
    /// The exponential base of the interpolation curve. It controls the rate at which the result increases. Higher values make the result increase more towards the high end of the range. With `1` the stops are interpolated linearly.
    pub base: FunctionBase,
    /// The color space in which colors interpolated. Interpolating colors in perceptual color spaces like LAB and HCL tend to produce color ramps that look more consistent and produce colors that can be differentiated more easily than those interpolated in RGB space.
    #[serde(rename = "colorSpace")]
    pub color_space: FunctionColorSpace,
    /// A value to serve as a fallback function result when a value isn't otherwise available. It is used in the following circumstances:
    ///
    /// * In categorical functions, when the feature value does not match any of the stop domain values.
    ///
    /// * In property and zoom-and-property functions, when a feature does not contain a value for the specified property.
    ///
    /// * In identity functions, when the feature value is not valid for the style property (for example, if the function is being used for a `circle-color` property but the feature property value is not a string or not a valid color).
    ///
    /// * In interval or exponential property and zoom-and-property functions, when the feature value is not numeric.
    ///
    /// If no default is provided, the style property's default is used in these circumstances.
    pub default: FunctionDefault,
    /// An expression.
    pub expression: FunctionExpression,
    /// The name of a feature property to use as the function input.
    pub property: FunctionProperty,
    /// An array of stops.
    pub stops: FunctionStops,
    /// The interpolation strategy to use in function evaluation.
    #[serde(rename = "type")]
    pub r#type: FunctionType,
}

/// The exponential base of the interpolation curve. It controls the rate at which the result increases. Higher values make the result increase more towards the high end of the range. With `1` the stops are interpolated linearly.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct FunctionBase(serde_json::Number);

impl Default for FunctionBase {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The color space in which colors interpolated. Interpolating colors in perceptual color spaces like LAB and HCL tend to produce color ramps that look more consistent and produce colors that can be differentiated more easily than those interpolated in RGB space.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum FunctionColorSpace {
    /// Use the HCL color space to interpolate color values, interpolating the Hue, Chroma, and Luminance channels individually.
    #[serde(rename = "hcl")]
    Hcl,
    /// Use the LAB color space to interpolate color values.
    #[serde(rename = "lab")]
    Lab,
    /// Use the RGB color space to interpolate color values
    #[serde(rename = "rgb")]
    Rgb,
}

impl Default for FunctionColorSpace {
    fn default() -> Self {
        Self::Rgb
    }
}

/// A value to serve as a fallback function result when a value isn't otherwise available. It is used in the following circumstances:
///
/// * In categorical functions, when the feature value does not match any of the stop domain values.
///
/// * In property and zoom-and-property functions, when a feature does not contain a value for the specified property.
///
/// * In identity functions, when the feature value is not valid for the style property (for example, if the function is being used for a `circle-color` property but the feature property value is not a string or not a valid color).
///
/// * In interval or exponential property and zoom-and-property functions, when the feature value is not numeric.
///
/// If no default is provided, the style property's default is used in these circumstances.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct FunctionDefault(serde_json::Value);

/// An expression.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "expression not implemented"]
struct FunctionExpression(serde_json::Value);

/// The name of a feature property to use as the function input.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct FunctionProperty(String);

impl Default for FunctionProperty {
    fn default() -> Self {
        Self("$zoom".to_string())
    }
}

/// An array of stops.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct FunctionStops(Vec<FunctionStop>);

/// The interpolation strategy to use in function evaluation.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum FunctionType {
    /// Return the output value of the stop equal to the function input.
    #[serde(rename = "categorical")]
    Categorical,
    /// Generate an output by interpolating between stops just less than and just greater than the function input.
    #[serde(rename = "exponential")]
    Exponential,
    /// Return the input value as the output value.
    #[serde(rename = "identity")]
    Identity,
    /// Return the output value of the stop just less than the function input.
    #[serde(rename = "interval")]
    Interval,
}

impl Default for FunctionType {
    fn default() -> Self {
        Self::Exponential
    }
}

/// FunctionStopValue Values
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum FunctionStopValue {
    Zero(serde_json::Number),
    One(color::DynamicColor),
}

/// Zoom level and value pair.
///
/// Range: 0..=24
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct FunctionStop(Box<[FunctionStopValue; 2]>);

/// The geometry type for the filter to select.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum GeometryType {
    /// Filter to line geometries.
    LineString,
    /// Filter to point geometries.
    Point,
    /// Filter to polygon geometries.
    Polygon,
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Layer {
    /// A expression specifying conditions on source features. Only features that match the filter are displayed. Zoom expressions in filters are only evaluated at integer zoom levels. The `feature-state` expression is not supported in filter expressions.
    pub filter: LayerFilter,
    /// Unique layer name.
    pub id: LayerId,
    /// Layout properties for the layer.
    pub layout: LayerLayout,
    /// The maximum zoom level for the layer. At zoom levels equal to or greater than the maxzoom, the layer will be hidden.
    pub maxzoom: LayerMaxzoom,
    /// Arbitrary properties useful to track with the layer, but do not influence rendering. Properties should be prefixed to avoid collisions, like 'maplibre:'.
    pub metadata: LayerMetadata,
    /// The minimum zoom level for the layer. At zoom levels less than the minzoom, the layer will be hidden.
    pub minzoom: LayerMinzoom,
    /// Default paint properties for this layer.
    pub paint: LayerPaint,
    /// Name of a source description to be used for this layer. Required for all layer types except `background`.
    pub source: LayerSource,
    /// Layer to use from a vector tile source. Required for vector tile sources; prohibited for all other source types, including GeoJSON sources.
    #[serde(rename = "source-layer")]
    pub source_layer: LayerSourceLayer,
    /// Rendering type of this layer.
    #[serde(rename = "type")]
    pub r#type: LayerType,
}

/// A expression specifying conditions on source features. Only features that match the filter are displayed. Zoom expressions in filters are only evaluated at integer zoom levels. The `feature-state` expression is not supported in filter expressions.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "filter not implemented"]
struct LayerFilter(serde_json::Value);

/// Unique layer name.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct LayerId(String);

/// Layout properties for the layer.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct LayerLayout(Layout);

/// The maximum zoom level for the layer. At zoom levels equal to or greater than the maxzoom, the layer will be hidden.
///
/// Range: 0..=24
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayerMaxzoom(serde_json::Number);

/// Arbitrary properties useful to track with the layer, but do not influence rendering. Properties should be prefixed to avoid collisions, like 'maplibre:'.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct LayerMetadata(serde_json::Value);

/// The minimum zoom level for the layer. At zoom levels less than the minzoom, the layer will be hidden.
///
/// Range: 0..=24
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayerMinzoom(serde_json::Number);

/// Default paint properties for this layer.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "paint not implemented"]
struct LayerPaint(serde_json::Value);

/// Name of a source description to be used for this layer. Required for all layer types except `background`.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct LayerSource(String);

/// Layer to use from a vector tile source. Required for vector tile sources; prohibited for all other source types, including GeoJSON sources.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct LayerSourceLayer(String);

/// Rendering type of this layer.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayerType {
    /// The background color or pattern of the map.
    #[serde(rename = "background")]
    Background,
    /// A filled circle.
    #[serde(rename = "circle")]
    Circle,
    /// Client-side elevation coloring based on DEM data. The implementation supports Mapbox Terrain RGB, Mapzen Terrarium tiles and custom encodings.
    #[serde(rename = "color-relief")]
    ColorRelief,
    /// A filled polygon with an optional stroked border.
    #[serde(rename = "fill")]
    Fill,
    /// An extruded (3D) polygon.
    #[serde(rename = "fill-extrusion")]
    FillExtrusion,
    /// A heatmap.
    #[serde(rename = "heatmap")]
    Heatmap,
    /// Client-side hillshading visualization based on DEM data. The implementation supports Mapbox Terrain RGB, Mapzen Terrarium tiles and custom encodings.
    #[serde(rename = "hillshade")]
    Hillshade,
    /// A stroked line.
    #[serde(rename = "line")]
    Line,
    /// Raster map textures such as satellite imagery.
    #[serde(rename = "raster")]
    Raster,
    /// An icon or a text label.
    #[serde(rename = "symbol")]
    Symbol,
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum Layout {
    LayoutFill(LayoutFill),
    LayoutLine(LayoutLine),
    LayoutCircle(LayoutCircle),
    LayoutHeatmap(LayoutHeatmap),
    LayoutFillExtrusion(LayoutFillExtrusion),
    LayoutSymbol(LayoutSymbol),
    LayoutRaster(LayoutRaster),
    LayoutHillshade(LayoutHillshade),
    LayoutColorRelief(LayoutColorRelief),
    LayoutBackground(LayoutBackground),
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutBackground {
    /// Whether this layer is displayed.
    pub visibility: LayoutBackgroundVisibility,
}

/// Whether this layer is displayed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutBackgroundVisibility {
    /// The layer is not shown.
    #[serde(rename = "none")]
    None,
    /// The layer is shown.
    #[serde(rename = "visible")]
    Visible,
}

impl Default for LayoutBackgroundVisibility {
    fn default() -> Self {
        Self::Visible
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutCircle {
    /// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
    #[serde(rename = "circle-sort-key")]
    pub circle_sort_key: LayoutCircleCircleSortKey,
    /// Whether this layer is displayed.
    pub visibility: LayoutCircleVisibility,
}

/// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutCircleCircleSortKey(serde_json::Number);

/// Whether this layer is displayed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutCircleVisibility {
    /// The layer is not shown.
    #[serde(rename = "none")]
    None,
    /// The layer is shown.
    #[serde(rename = "visible")]
    Visible,
}

impl Default for LayoutCircleVisibility {
    fn default() -> Self {
        Self::Visible
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutColorRelief {
    /// Whether this layer is displayed.
    pub visibility: LayoutColorReliefVisibility,
}

/// Whether this layer is displayed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutColorReliefVisibility {
    /// The layer is not shown.
    #[serde(rename = "none")]
    None,
    /// The layer is shown.
    #[serde(rename = "visible")]
    Visible,
}

impl Default for LayoutColorReliefVisibility {
    fn default() -> Self {
        Self::Visible
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutFill {
    /// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
    #[serde(rename = "fill-sort-key")]
    pub fill_sort_key: LayoutFillFillSortKey,
    /// Whether this layer is displayed.
    pub visibility: LayoutFillVisibility,
}

/// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutFillFillSortKey(serde_json::Number);

/// Whether this layer is displayed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutFillVisibility {
    /// The layer is not shown.
    #[serde(rename = "none")]
    None,
    /// The layer is shown.
    #[serde(rename = "visible")]
    Visible,
}

impl Default for LayoutFillVisibility {
    fn default() -> Self {
        Self::Visible
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutFillExtrusion {
    /// Whether this layer is displayed.
    pub visibility: LayoutFillExtrusionVisibility,
}

/// Whether this layer is displayed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutFillExtrusionVisibility {
    /// The layer is not shown.
    #[serde(rename = "none")]
    None,
    /// The layer is shown.
    #[serde(rename = "visible")]
    Visible,
}

impl Default for LayoutFillExtrusionVisibility {
    fn default() -> Self {
        Self::Visible
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutHeatmap {
    /// Whether this layer is displayed.
    pub visibility: LayoutHeatmapVisibility,
}

/// Whether this layer is displayed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutHeatmapVisibility {
    /// The layer is not shown.
    #[serde(rename = "none")]
    None,
    /// The layer is shown.
    #[serde(rename = "visible")]
    Visible,
}

impl Default for LayoutHeatmapVisibility {
    fn default() -> Self {
        Self::Visible
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutHillshade {
    /// Whether this layer is displayed.
    pub visibility: LayoutHillshadeVisibility,
}

/// Whether this layer is displayed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutHillshadeVisibility {
    /// The layer is not shown.
    #[serde(rename = "none")]
    None,
    /// The layer is shown.
    #[serde(rename = "visible")]
    Visible,
}

impl Default for LayoutHillshadeVisibility {
    fn default() -> Self {
        Self::Visible
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutLine {
    /// The display of line endings.
    #[serde(rename = "line-cap")]
    pub line_cap: LayoutLineLineCap,
    /// The display of lines when joining.
    #[serde(rename = "line-join")]
    pub line_join: LayoutLineLineJoin,
    /// Used to automatically convert miter joins to bevel joins for sharp angles.
    #[serde(rename = "line-miter-limit")]
    pub line_miter_limit: LayoutLineLineMiterLimit,
    /// Used to automatically convert round joins to miter joins for shallow angles.
    #[serde(rename = "line-round-limit")]
    pub line_round_limit: LayoutLineLineRoundLimit,
    /// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
    #[serde(rename = "line-sort-key")]
    pub line_sort_key: LayoutLineLineSortKey,
    /// Whether this layer is displayed.
    pub visibility: LayoutLineVisibility,
}

/// The display of line endings.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutLineLineCap {
    /// A cap with a squared-off end which is drawn to the exact endpoint of the line.
    #[serde(rename = "butt")]
    Butt,
    /// A cap with a rounded end which is drawn beyond the endpoint of the line at a radius of one-half of the line's width and centered on the endpoint of the line.
    #[serde(rename = "round")]
    Round,
    /// A cap with a squared-off end which is drawn beyond the endpoint of the line at a distance of one-half of the line's width.
    #[serde(rename = "square")]
    Square,
}

impl Default for LayoutLineLineCap {
    fn default() -> Self {
        Self::Butt
    }
}

/// The display of lines when joining.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutLineLineJoin {
    /// A join with a squared-off end which is drawn beyond the endpoint of the line at a distance of one-half of the line's width.
    #[serde(rename = "bevel")]
    Bevel,
    /// A join with a sharp, angled corner which is drawn with the outer sides beyond the endpoint of the path until they meet.
    #[serde(rename = "miter")]
    Miter,
    /// A join with a rounded end which is drawn beyond the endpoint of the line at a radius of one-half of the line's width and centered on the endpoint of the line.
    #[serde(rename = "round")]
    Round,
}

impl Default for LayoutLineLineJoin {
    fn default() -> Self {
        Self::Miter
    }
}

/// Used to automatically convert miter joins to bevel joins for sharp angles.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutLineLineMiterLimit(serde_json::Number);

impl Default for LayoutLineLineMiterLimit {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(2)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Used to automatically convert round joins to miter joins for shallow angles.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutLineLineRoundLimit(serde_json::Number);

impl Default for LayoutLineLineRoundLimit {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(1.05)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutLineLineSortKey(serde_json::Number);

/// Whether this layer is displayed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutLineVisibility {
    /// The layer is not shown.
    #[serde(rename = "none")]
    None,
    /// The layer is shown.
    #[serde(rename = "visible")]
    Visible,
}

impl Default for LayoutLineVisibility {
    fn default() -> Self {
        Self::Visible
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutRaster {
    /// Whether this layer is displayed.
    pub visibility: LayoutRasterVisibility,
}

/// Whether this layer is displayed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutRasterVisibility {
    /// The layer is not shown.
    #[serde(rename = "none")]
    None,
    /// The layer is shown.
    #[serde(rename = "visible")]
    Visible,
}

impl Default for LayoutRasterVisibility {
    fn default() -> Self {
        Self::Visible
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutSymbol {
    /// If true, the icon will be visible even if it collides with other previously drawn symbols.
    #[serde(rename = "icon-allow-overlap")]
    pub icon_allow_overlap: LayoutSymbolIconAllowOverlap,
    /// Part of the icon placed closest to the anchor.
    #[serde(rename = "icon-anchor")]
    pub icon_anchor: LayoutSymbolIconAnchor,
    /// If true, other symbols can be visible even if they collide with the icon.
    #[serde(rename = "icon-ignore-placement")]
    pub icon_ignore_placement: LayoutSymbolIconIgnorePlacement,
    /// Name of image in sprite to use for drawing an image background.
    #[serde(rename = "icon-image")]
    pub icon_image: LayoutSymbolIconImage,
    /// If true, the icon may be flipped to prevent it from being rendered upside-down.
    #[serde(rename = "icon-keep-upright")]
    pub icon_keep_upright: LayoutSymbolIconKeepUpright,
    /// Offset distance of icon from its anchor. Positive values indicate right and down, while negative values indicate left and up. Each component is multiplied by the value of `icon-size` to obtain the final offset in pixels. When combined with `icon-rotate` the offset will be as if the rotated direction was up.
    #[serde(rename = "icon-offset")]
    pub icon_offset: LayoutSymbolIconOffset,
    /// If true, text will display without their corresponding icons when the icon collides with other symbols and the text does not.
    #[serde(rename = "icon-optional")]
    pub icon_optional: LayoutSymbolIconOptional,
    /// Allows for control over whether to show an icon when it overlaps other symbols on the map. If `icon-overlap` is not set, `icon-allow-overlap` is used instead.
    #[serde(rename = "icon-overlap")]
    pub icon_overlap: LayoutSymbolIconOverlap,
    /// Size of additional area round the icon bounding box used for detecting symbol collisions.
    #[serde(rename = "icon-padding")]
    pub icon_padding: LayoutSymbolIconPadding,
    /// Orientation of icon when map is pitched.
    #[serde(rename = "icon-pitch-alignment")]
    pub icon_pitch_alignment: LayoutSymbolIconPitchAlignment,
    /// Rotates the icon clockwise.
    #[serde(rename = "icon-rotate")]
    pub icon_rotate: LayoutSymbolIconRotate,
    /// In combination with `symbol-placement`, determines the rotation behavior of icons.
    #[serde(rename = "icon-rotation-alignment")]
    pub icon_rotation_alignment: LayoutSymbolIconRotationAlignment,
    /// Scales the original size of the icon by the provided factor. The new pixel size of the image will be the original pixel size multiplied by `icon-size`. 1 is the original size; 3 triples the size of the image.
    #[serde(rename = "icon-size")]
    pub icon_size: LayoutSymbolIconSize,
    /// Scales the icon to fit around the associated text.
    #[serde(rename = "icon-text-fit")]
    pub icon_text_fit: LayoutSymbolIconTextFit,
    /// Size of the additional area added to dimensions determined by `icon-text-fit`, in clockwise order: top, right, bottom, left.
    #[serde(rename = "icon-text-fit-padding")]
    pub icon_text_fit_padding: LayoutSymbolIconTextFitPadding,
    /// If true, the symbols will not cross tile edges to avoid mutual collisions. Recommended in layers that don't have enough padding in the vector tile to prevent collisions, or if it is a point symbol layer placed after a line symbol layer. When using a client that supports global collision detection, like MapLibre GL JS version 0.42.0 or greater, enabling this property is not needed to prevent clipped labels at tile boundaries.
    #[serde(rename = "symbol-avoid-edges")]
    pub symbol_avoid_edges: LayoutSymbolSymbolAvoidEdges,
    /// Label placement relative to its geometry.
    #[serde(rename = "symbol-placement")]
    pub symbol_placement: LayoutSymbolSymbolPlacement,
    /// Sorts features in ascending order based on this value. Features with lower sort keys are drawn and placed first.  When `icon-allow-overlap` or `text-allow-overlap` is `false`, features with a lower sort key will have priority during placement. When `icon-allow-overlap` or `text-allow-overlap` is set to `true`, features with a higher sort key will overlap over features with a lower sort key.
    #[serde(rename = "symbol-sort-key")]
    pub symbol_sort_key: LayoutSymbolSymbolSortKey,
    /// Distance between two symbol anchors.
    #[serde(rename = "symbol-spacing")]
    pub symbol_spacing: LayoutSymbolSymbolSpacing,
    /// Determines whether overlapping symbols in the same layer are rendered in the order that they appear in the data source or by their y-position relative to the viewport. To control the order and prioritization of symbols otherwise, use `symbol-sort-key`.
    #[serde(rename = "symbol-z-order")]
    pub symbol_z_order: LayoutSymbolSymbolZOrder,
    /// If true, the text will be visible even if it collides with other previously drawn symbols.
    #[serde(rename = "text-allow-overlap")]
    pub text_allow_overlap: LayoutSymbolTextAllowOverlap,
    /// Part of the text placed closest to the anchor.
    #[serde(rename = "text-anchor")]
    pub text_anchor: LayoutSymbolTextAnchor,
    /// Value to use for a text label. If a plain `string` is provided, it will be treated as a `formatted` with default/inherited formatting options.
    #[serde(rename = "text-field")]
    pub text_field: LayoutSymbolTextField,
    /// Fonts to use for displaying text. If the `glyphs` root property is specified, this array is joined together and interpreted as a font stack name. Otherwise, it is interpreted as a cascading fallback list of local font names.
    #[serde(rename = "text-font")]
    pub text_font: LayoutSymbolTextFont,
    /// If true, other symbols can be visible even if they collide with the text.
    #[serde(rename = "text-ignore-placement")]
    pub text_ignore_placement: LayoutSymbolTextIgnorePlacement,
    /// Text justification options.
    #[serde(rename = "text-justify")]
    pub text_justify: LayoutSymbolTextJustify,
    /// If true, the text may be flipped vertically to prevent it from being rendered upside-down.
    #[serde(rename = "text-keep-upright")]
    pub text_keep_upright: LayoutSymbolTextKeepUpright,
    /// Text tracking amount.
    #[serde(rename = "text-letter-spacing")]
    pub text_letter_spacing: LayoutSymbolTextLetterSpacing,
    /// Text leading value for multi-line text.
    #[serde(rename = "text-line-height")]
    pub text_line_height: LayoutSymbolTextLineHeight,
    /// Maximum angle change between adjacent characters.
    #[serde(rename = "text-max-angle")]
    pub text_max_angle: LayoutSymbolTextMaxAngle,
    /// The maximum line width for text wrapping.
    #[serde(rename = "text-max-width")]
    pub text_max_width: LayoutSymbolTextMaxWidth,
    /// Offset distance of text from its anchor. Positive values indicate right and down, while negative values indicate left and up. If used with text-variable-anchor, input values will be taken as absolute values. Offsets along the x- and y-axis will be applied automatically based on the anchor position.
    #[serde(rename = "text-offset")]
    pub text_offset: LayoutSymbolTextOffset,
    /// If true, icons will display without their corresponding text when the text collides with other symbols and the icon does not.
    #[serde(rename = "text-optional")]
    pub text_optional: LayoutSymbolTextOptional,
    /// Allows for control over whether to show symbol text when it overlaps other symbols on the map. If `text-overlap` is not set, `text-allow-overlap` is used instead
    #[serde(rename = "text-overlap")]
    pub text_overlap: LayoutSymbolTextOverlap,
    /// Size of the additional area around the text bounding box used for detecting symbol collisions.
    #[serde(rename = "text-padding")]
    pub text_padding: LayoutSymbolTextPadding,
    /// Orientation of text when map is pitched.
    #[serde(rename = "text-pitch-alignment")]
    pub text_pitch_alignment: LayoutSymbolTextPitchAlignment,
    /// Radial offset of text, in the direction of the symbol's anchor. Useful in combination with `text-variable-anchor`, which defaults to using the two-dimensional `text-offset` if present.
    #[serde(rename = "text-radial-offset")]
    pub text_radial_offset: LayoutSymbolTextRadialOffset,
    /// Rotates the text clockwise.
    #[serde(rename = "text-rotate")]
    pub text_rotate: LayoutSymbolTextRotate,
    /// In combination with `symbol-placement`, determines the rotation behavior of the individual glyphs forming the text.
    #[serde(rename = "text-rotation-alignment")]
    pub text_rotation_alignment: LayoutSymbolTextRotationAlignment,
    /// Font size.
    #[serde(rename = "text-size")]
    pub text_size: LayoutSymbolTextSize,
    /// Specifies how to capitalize text, similar to the CSS `text-transform` property.
    #[serde(rename = "text-transform")]
    pub text_transform: LayoutSymbolTextTransform,
    /// To increase the chance of placing high-priority labels on the map, you can provide an array of `text-anchor` locations: the renderer will attempt to place the label at each location, in order, before moving onto the next label. Use `text-justify: auto` to choose justification based on anchor position. To apply an offset, use the `text-radial-offset` or the two-dimensional `text-offset`.
    #[serde(rename = "text-variable-anchor")]
    pub text_variable_anchor: LayoutSymbolTextVariableAnchor,
    /// To increase the chance of placing high-priority labels on the map, you can provide an array of `text-anchor` locations, each paired with an offset value. The renderer will attempt to place the label at each location, in order, before moving on to the next location+offset. Use `text-justify: auto` to choose justification based on anchor position.
    ///
    ///  The length of the array must be even, and must alternate between enum and point entries. i.e., each anchor location must be accompanied by a point, and that point defines the offset when the corresponding anchor location is used. Positive offset values indicate right and down, while negative values indicate left and up. Anchor locations may repeat, allowing the renderer to try multiple offsets to try and place a label using the same anchor.
    ///
    ///  When present, this property takes precedence over `text-anchor`, `text-variable-anchor`, `text-offset`, and `text-radial-offset`.
    ///
    ///  ```json
    ///
    ///  { "text-variable-anchor-offset": ["top", [0, 4], "left", [3,0], "bottom", [1, 1]] }
    ///
    ///  ```
    ///
    ///  When the renderer chooses the `top` anchor, `[0, 4]` will be used for `text-offset`; the text will be shifted down by 4 ems.
    ///
    ///  When the renderer chooses the `left` anchor, `[3, 0]` will be used for `text-offset`; the text will be shifted right by 3 ems.
    #[serde(rename = "text-variable-anchor-offset")]
    pub text_variable_anchor_offset: LayoutSymbolTextVariableAnchorOffset,
    /// The property allows control over a symbol's orientation. Note that the property values act as a hint, so that a symbol whose language doesn’t support the provided orientation will be laid out in its natural orientation. Example: English point symbol will be rendered horizontally even if array value contains single 'vertical' enum value. The order of elements in an array define priority order for the placement of an orientation variant.
    #[serde(rename = "text-writing-mode")]
    pub text_writing_mode: LayoutSymbolTextWritingMode,
    /// Whether this layer is displayed.
    pub visibility: LayoutSymbolVisibility,
}

/// If true, the icon will be visible even if it collides with other previously drawn symbols.
#[derive(serde::Deserialize, PartialEq, Debug, Clone, Copy)]
struct LayoutSymbolIconAllowOverlap(bool);

impl Default for LayoutSymbolIconAllowOverlap {
    fn default() -> Self {
        Self(false)
    }
}

/// Part of the icon placed closest to the anchor.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutSymbolIconAnchor {
    /// The bottom of the icon is placed closest to the anchor.
    #[serde(rename = "bottom")]
    Bottom,
    /// The bottom left corner of the icon is placed closest to the anchor.
    #[serde(rename = "bottom-left")]
    BottomLeft,
    /// The bottom right corner of the icon is placed closest to the anchor.
    #[serde(rename = "bottom-right")]
    BottomRight,
    /// The center of the icon is placed closest to the anchor.
    #[serde(rename = "center")]
    Center,
    /// The left side of the icon is placed closest to the anchor.
    #[serde(rename = "left")]
    Left,
    /// The right side of the icon is placed closest to the anchor.
    #[serde(rename = "right")]
    Right,
    /// The top of the icon is placed closest to the anchor.
    #[serde(rename = "top")]
    Top,
    /// The top left corner of the icon is placed closest to the anchor.
    #[serde(rename = "top-left")]
    TopLeft,
    /// The top right corner of the icon is placed closest to the anchor.
    #[serde(rename = "top-right")]
    TopRight,
}

impl Default for LayoutSymbolIconAnchor {
    fn default() -> Self {
        Self::Center
    }
}

/// If true, other symbols can be visible even if they collide with the icon.
#[derive(serde::Deserialize, PartialEq, Debug, Clone, Copy)]
struct LayoutSymbolIconIgnorePlacement(bool);

impl Default for LayoutSymbolIconIgnorePlacement {
    fn default() -> Self {
        Self(false)
    }
}

/// Name of image in sprite to use for drawing an image background.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "resolved_image not implemented"]
struct LayoutSymbolIconImage(serde_json::Value);

/// If true, the icon may be flipped to prevent it from being rendered upside-down.
#[derive(serde::Deserialize, PartialEq, Debug, Clone, Copy)]
struct LayoutSymbolIconKeepUpright(bool);

impl Default for LayoutSymbolIconKeepUpright {
    fn default() -> Self {
        Self(false)
    }
}

/// Offset distance of icon from its anchor. Positive values indicate right and down, while negative values indicate left and up. Each component is multiplied by the value of `icon-size` to obtain the final offset in pixels. When combined with `icon-rotate` the offset will be as if the rotated direction was up.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct LayoutSymbolIconOffset(Box<[serde_json::Number; 2]>);

impl Default for LayoutSymbolIconOffset {
    fn default() -> Self {
        Self(Box::new([
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        ]))
    }
}

/// If true, text will display without their corresponding icons when the icon collides with other symbols and the text does not.
#[derive(serde::Deserialize, PartialEq, Debug, Clone, Copy)]
struct LayoutSymbolIconOptional(bool);

impl Default for LayoutSymbolIconOptional {
    fn default() -> Self {
        Self(false)
    }
}

/// Allows for control over whether to show an icon when it overlaps other symbols on the map. If `icon-overlap` is not set, `icon-allow-overlap` is used instead.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutSymbolIconOverlap {
    /// The icon will be visible even if it collides with any other previously drawn symbol.
    #[serde(rename = "always")]
    Always,
    /// If the icon collides with another previously drawn symbol, the overlap mode for that symbol is checked. If the previous symbol was placed using `never` overlap mode, the new icon is hidden. If the previous symbol was placed using `always` or `cooperative` overlap mode, the new icon is visible.
    #[serde(rename = "cooperative")]
    Cooperative,
    /// The icon will be hidden if it collides with any other previously drawn symbol.
    #[serde(rename = "never")]
    Never,
}

/// Size of additional area round the icon bounding box used for detecting symbol collisions.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum LayoutSymbolIconPadding {
    /// A single value applies to all four sides.
    ///
    /// Only avaliable for backwards compatibility.
    #[deprecated = "Please see [`Self::One`] instead"]
    Unwrapped(serde_json::Number),
    /// A single value applies to all four sides
    One(Box<[serde_json::Number; 1]>),
    /// two values apply to `[top/bottom, left/right]`
    Two(Box<[serde_json::Number; 2]>),
    /// three values apply to `[top, left/right, bottom]`
    Three(Box<[serde_json::Number; 3]>),
    /// four values apply to `[top, right, bottom, left]`
    Four(Box<[serde_json::Number; 4]>),
}

impl Default for LayoutSymbolIconPadding {
    fn default() -> Self {
        Self::One(Box::new([2.into()]))
    }
}

/// Orientation of icon when map is pitched.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutSymbolIconPitchAlignment {
    /// Automatically matches the value of `icon-rotation-alignment`.
    #[serde(rename = "auto")]
    Auto,
    /// The icon is aligned to the plane of the map.
    #[serde(rename = "map")]
    Map,
    /// The icon is aligned to the plane of the viewport.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for LayoutSymbolIconPitchAlignment {
    fn default() -> Self {
        Self::Auto
    }
}

/// Rotates the icon clockwise.
///
/// Range: every 360
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutSymbolIconRotate(serde_json::Number);

impl Default for LayoutSymbolIconRotate {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// In combination with `symbol-placement`, determines the rotation behavior of icons.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutSymbolIconRotationAlignment {
    /// When `symbol-placement` is set to `point`, this is equivalent to `viewport`. When `symbol-placement` is set to `line` or `line-center`, this is equivalent to `map`.
    #[serde(rename = "auto")]
    Auto,
    /// When `symbol-placement` is set to `point`, aligns icons east-west. When `symbol-placement` is set to `line` or `line-center`, aligns icon x-axes with the line.
    #[serde(rename = "map")]
    Map,
    /// Produces icons whose x-axes are aligned with the x-axis of the viewport, regardless of the value of `symbol-placement`.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for LayoutSymbolIconRotationAlignment {
    fn default() -> Self {
        Self::Auto
    }
}

/// Scales the original size of the icon by the provided factor. The new pixel size of the image will be the original pixel size multiplied by `icon-size`. 1 is the original size; 3 triples the size of the image.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutSymbolIconSize(serde_json::Number);

impl Default for LayoutSymbolIconSize {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Scales the icon to fit around the associated text.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutSymbolIconTextFit {
    /// The icon is scaled in both x- and y-dimensions.
    #[serde(rename = "both")]
    Both,
    /// The icon is scaled in the y-dimension to fit the height of the text.
    #[serde(rename = "height")]
    Height,
    /// The icon is displayed at its intrinsic aspect ratio.
    #[serde(rename = "none")]
    None,
    /// The icon is scaled in the x-dimension to fit the width of the text.
    #[serde(rename = "width")]
    Width,
}

impl Default for LayoutSymbolIconTextFit {
    fn default() -> Self {
        Self::None
    }
}

/// Size of the additional area added to dimensions determined by `icon-text-fit`, in clockwise order: top, right, bottom, left.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct LayoutSymbolIconTextFitPadding(Box<[serde_json::Number; 4]>);

impl Default for LayoutSymbolIconTextFitPadding {
    fn default() -> Self {
        Self(Box::new([
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        ]))
    }
}

/// If true, the symbols will not cross tile edges to avoid mutual collisions. Recommended in layers that don't have enough padding in the vector tile to prevent collisions, or if it is a point symbol layer placed after a line symbol layer. When using a client that supports global collision detection, like MapLibre GL JS version 0.42.0 or greater, enabling this property is not needed to prevent clipped labels at tile boundaries.
#[derive(serde::Deserialize, PartialEq, Debug, Clone, Copy)]
struct LayoutSymbolSymbolAvoidEdges(bool);

impl Default for LayoutSymbolSymbolAvoidEdges {
    fn default() -> Self {
        Self(false)
    }
}

/// Label placement relative to its geometry.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutSymbolSymbolPlacement {
    /// The label is placed along the line of the geometry. Can only be used on `LineString` and `Polygon` geometries.
    #[serde(rename = "line")]
    Line,
    /// The label is placed at the center of the line of the geometry. Can only be used on `LineString` and `Polygon` geometries. Note that a single feature in a vector tile may contain multiple line geometries.
    #[serde(rename = "line-center")]
    LineCenter,
    /// The label is placed at the point where the geometry is located.
    #[serde(rename = "point")]
    Point,
}

impl Default for LayoutSymbolSymbolPlacement {
    fn default() -> Self {
        Self::Point
    }
}

/// Sorts features in ascending order based on this value. Features with lower sort keys are drawn and placed first.  When `icon-allow-overlap` or `text-allow-overlap` is `false`, features with a lower sort key will have priority during placement. When `icon-allow-overlap` or `text-allow-overlap` is set to `true`, features with a higher sort key will overlap over features with a lower sort key.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutSymbolSymbolSortKey(serde_json::Number);

/// Distance between two symbol anchors.
///
/// Range: 1..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutSymbolSymbolSpacing(serde_json::Number);

impl Default for LayoutSymbolSymbolSpacing {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(250)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Determines whether overlapping symbols in the same layer are rendered in the order that they appear in the data source or by their y-position relative to the viewport. To control the order and prioritization of symbols otherwise, use `symbol-sort-key`.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutSymbolSymbolZOrder {
    /// Sorts symbols by `symbol-sort-key` if set. Otherwise, sorts symbols by their y-position relative to the viewport if `icon-allow-overlap` or `text-allow-overlap` is set to `true` or `icon-ignore-placement` or `text-ignore-placement` is `false`.
    #[serde(rename = "auto")]
    Auto,
    /// Sorts symbols by `symbol-sort-key` if set. Otherwise, no sorting is applied; symbols are rendered in the same order as the source data.
    #[serde(rename = "source")]
    Source,
    /// Sorts symbols by their y-position relative to the viewport if `icon-allow-overlap` or `text-allow-overlap` is set to `true` or `icon-ignore-placement` or `text-ignore-placement` is `false`.
    #[serde(rename = "viewport-y")]
    ViewportY,
}

impl Default for LayoutSymbolSymbolZOrder {
    fn default() -> Self {
        Self::Auto
    }
}

/// If true, the text will be visible even if it collides with other previously drawn symbols.
#[derive(serde::Deserialize, PartialEq, Debug, Clone, Copy)]
struct LayoutSymbolTextAllowOverlap(bool);

impl Default for LayoutSymbolTextAllowOverlap {
    fn default() -> Self {
        Self(false)
    }
}

/// Part of the text placed closest to the anchor.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutSymbolTextAnchor {
    /// The bottom of the text is placed closest to the anchor.
    #[serde(rename = "bottom")]
    Bottom,
    /// The bottom left corner of the text is placed closest to the anchor.
    #[serde(rename = "bottom-left")]
    BottomLeft,
    /// The bottom right corner of the text is placed closest to the anchor.
    #[serde(rename = "bottom-right")]
    BottomRight,
    /// The center of the text is placed closest to the anchor.
    #[serde(rename = "center")]
    Center,
    /// The left side of the text is placed closest to the anchor.
    #[serde(rename = "left")]
    Left,
    /// The right side of the text is placed closest to the anchor.
    #[serde(rename = "right")]
    Right,
    /// The top of the text is placed closest to the anchor.
    #[serde(rename = "top")]
    Top,
    /// The top left corner of the text is placed closest to the anchor.
    #[serde(rename = "top-left")]
    TopLeft,
    /// The top right corner of the text is placed closest to the anchor.
    #[serde(rename = "top-right")]
    TopRight,
}

impl Default for LayoutSymbolTextAnchor {
    fn default() -> Self {
        Self::Center
    }
}

/// Value to use for a text label. If a plain `string` is provided, it will be treated as a `formatted` with default/inherited formatting options.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct LayoutSymbolTextField(String);

impl Default for LayoutSymbolTextField {
    fn default() -> Self {
        Self("".to_string())
    }
}

/// Fonts to use for displaying text. If the `glyphs` root property is specified, this array is joined together and interpreted as a font stack name. Otherwise, it is interpreted as a cascading fallback list of local font names.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct LayoutSymbolTextFont(Vec<String>);

impl Default for LayoutSymbolTextFont {
    fn default() -> Self {
        Self(Vec::from([
            "Open Sans Regular".to_string(),
            "Arial Unicode MS Regular".to_string(),
        ]))
    }
}

/// If true, other symbols can be visible even if they collide with the text.
#[derive(serde::Deserialize, PartialEq, Debug, Clone, Copy)]
struct LayoutSymbolTextIgnorePlacement(bool);

impl Default for LayoutSymbolTextIgnorePlacement {
    fn default() -> Self {
        Self(false)
    }
}

/// Text justification options.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutSymbolTextJustify {
    /// The text is aligned towards the anchor position.
    #[serde(rename = "auto")]
    Auto,
    /// The text is centered.
    #[serde(rename = "center")]
    Center,
    /// The text is aligned to the left.
    #[serde(rename = "left")]
    Left,
    /// The text is aligned to the right.
    #[serde(rename = "right")]
    Right,
}

impl Default for LayoutSymbolTextJustify {
    fn default() -> Self {
        Self::Center
    }
}

/// If true, the text may be flipped vertically to prevent it from being rendered upside-down.
#[derive(serde::Deserialize, PartialEq, Debug, Clone, Copy)]
struct LayoutSymbolTextKeepUpright(bool);

impl Default for LayoutSymbolTextKeepUpright {
    fn default() -> Self {
        Self(true)
    }
}

/// Text tracking amount.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutSymbolTextLetterSpacing(serde_json::Number);

impl Default for LayoutSymbolTextLetterSpacing {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Text leading value for multi-line text.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutSymbolTextLineHeight(serde_json::Number);

impl Default for LayoutSymbolTextLineHeight {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(1.2)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Maximum angle change between adjacent characters.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutSymbolTextMaxAngle(serde_json::Number);

impl Default for LayoutSymbolTextMaxAngle {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(45)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The maximum line width for text wrapping.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutSymbolTextMaxWidth(serde_json::Number);

impl Default for LayoutSymbolTextMaxWidth {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(10)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Offset distance of text from its anchor. Positive values indicate right and down, while negative values indicate left and up. If used with text-variable-anchor, input values will be taken as absolute values. Offsets along the x- and y-axis will be applied automatically based on the anchor position.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct LayoutSymbolTextOffset(Box<[serde_json::Number; 2]>);

impl Default for LayoutSymbolTextOffset {
    fn default() -> Self {
        Self(Box::new([
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        ]))
    }
}

/// If true, icons will display without their corresponding text when the text collides with other symbols and the icon does not.
#[derive(serde::Deserialize, PartialEq, Debug, Clone, Copy)]
struct LayoutSymbolTextOptional(bool);

impl Default for LayoutSymbolTextOptional {
    fn default() -> Self {
        Self(false)
    }
}

/// Allows for control over whether to show symbol text when it overlaps other symbols on the map. If `text-overlap` is not set, `text-allow-overlap` is used instead
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutSymbolTextOverlap {
    /// The text will be visible even if it collides with any other previously drawn symbol.
    #[serde(rename = "always")]
    Always,
    /// If the text collides with another previously drawn symbol, the overlap mode for that symbol is checked. If the previous symbol was placed using `never` overlap mode, the new text is hidden. If the previous symbol was placed using `always` or `cooperative` overlap mode, the new text is visible.
    #[serde(rename = "cooperative")]
    Cooperative,
    /// The text will be hidden if it collides with any other previously drawn symbol.
    #[serde(rename = "never")]
    Never,
}

/// Size of the additional area around the text bounding box used for detecting symbol collisions.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutSymbolTextPadding(serde_json::Number);

impl Default for LayoutSymbolTextPadding {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(2)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Orientation of text when map is pitched.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutSymbolTextPitchAlignment {
    /// Automatically matches the value of `text-rotation-alignment`.
    #[serde(rename = "auto")]
    Auto,
    /// The text is aligned to the plane of the map.
    #[serde(rename = "map")]
    Map,
    /// The text is aligned to the plane of the viewport.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for LayoutSymbolTextPitchAlignment {
    fn default() -> Self {
        Self::Auto
    }
}

/// Radial offset of text, in the direction of the symbol's anchor. Useful in combination with `text-variable-anchor`, which defaults to using the two-dimensional `text-offset` if present.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutSymbolTextRadialOffset(serde_json::Number);

impl Default for LayoutSymbolTextRadialOffset {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Rotates the text clockwise.
///
/// Range: every 360
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutSymbolTextRotate(serde_json::Number);

impl Default for LayoutSymbolTextRotate {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// In combination with `symbol-placement`, determines the rotation behavior of the individual glyphs forming the text.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutSymbolTextRotationAlignment {
    /// When `symbol-placement` is set to `point`, this is equivalent to `viewport`. When `symbol-placement` is set to `line` or `line-center`, this is equivalent to `map`.
    #[serde(rename = "auto")]
    Auto,
    /// When `symbol-placement` is set to `point`, aligns text east-west. When `symbol-placement` is set to `line` or `line-center`, aligns text x-axes with the line.
    #[serde(rename = "map")]
    Map,
    /// Produces glyphs whose x-axes are aligned with the x-axis of the viewport, regardless of the value of `symbol-placement`.
    #[serde(rename = "viewport")]
    Viewport,
    /// When `symbol-placement` is set to `point`, aligns text to the x-axis of the viewport. When `symbol-placement` is set to `line` or `line-center`, aligns glyphs to the x-axis of the viewport and places them along the line.
    #[serde(rename = "viewport-glyph")]
    ViewportGlyph,
}

impl Default for LayoutSymbolTextRotationAlignment {
    fn default() -> Self {
        Self::Auto
    }
}

/// Font size.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutSymbolTextSize(serde_json::Number);

impl Default for LayoutSymbolTextSize {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(16)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Specifies how to capitalize text, similar to the CSS `text-transform` property.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutSymbolTextTransform {
    /// Forces all letters to be displayed in lowercase.
    #[serde(rename = "lowercase")]
    Lowercase,
    /// The text is not altered.
    #[serde(rename = "none")]
    None,
    /// Forces all letters to be displayed in uppercase.
    #[serde(rename = "uppercase")]
    Uppercase,
}

impl Default for LayoutSymbolTextTransform {
    fn default() -> Self {
        Self::None
    }
}

/// To increase the chance of placing high-priority labels on the map, you can provide an array of `text-anchor` locations: the renderer will attempt to place the label at each location, in order, before moving onto the next label. Use `text-justify: auto` to choose justification based on anchor position. To apply an offset, use the `text-radial-offset` or the two-dimensional `text-offset`.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutSymbolTextVariableAnchorValue {
    /// The bottom of the text is placed closest to the anchor.
    #[serde(rename = "bottom")]
    Bottom,
    /// The bottom left corner of the text is placed closest to the anchor.
    #[serde(rename = "bottom-left")]
    BottomLeft,
    /// The bottom right corner of the text is placed closest to the anchor.
    #[serde(rename = "bottom-right")]
    BottomRight,
    /// The center of the text is placed closest to the anchor.
    #[serde(rename = "center")]
    Center,
    /// The left side of the text is placed closest to the anchor.
    #[serde(rename = "left")]
    Left,
    /// The right side of the text is placed closest to the anchor.
    #[serde(rename = "right")]
    Right,
    /// The top of the text is placed closest to the anchor.
    #[serde(rename = "top")]
    Top,
    /// The top left corner of the text is placed closest to the anchor.
    #[serde(rename = "top-left")]
    TopLeft,
    /// The top right corner of the text is placed closest to the anchor.
    #[serde(rename = "top-right")]
    TopRight,
}

/// To increase the chance of placing high-priority labels on the map, you can provide an array of `text-anchor` locations: the renderer will attempt to place the label at each location, in order, before moving onto the next label. Use `text-justify: auto` to choose justification based on anchor position. To apply an offset, use the `text-radial-offset` or the two-dimensional `text-offset`.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct LayoutSymbolTextVariableAnchor(Vec<LayoutSymbolTextVariableAnchorValue>);

/// To increase the chance of placing high-priority labels on the map, you can provide an array of `text-anchor` locations, each paired with an offset value. The renderer will attempt to place the label at each location, in order, before moving on to the next location+offset. Use `text-justify: auto` to choose justification based on anchor position.
///
///  The length of the array must be even, and must alternate between enum and point entries. i.e., each anchor location must be accompanied by a point, and that point defines the offset when the corresponding anchor location is used. Positive offset values indicate right and down, while negative values indicate left and up. Anchor locations may repeat, allowing the renderer to try multiple offsets to try and place a label using the same anchor.
///
///  When present, this property takes precedence over `text-anchor`, `text-variable-anchor`, `text-offset`, and `text-radial-offset`.
///
///  ```json
///
///  { "text-variable-anchor-offset": ["top", [0, 4], "left", [3,0], "bottom", [1, 1]] }
///
///  ```
///
///  When the renderer chooses the `top` anchor, `[0, 4]` will be used for `text-offset`; the text will be shifted down by 4 ems.
///
///  When the renderer chooses the `left` anchor, `[3, 0]` will be used for `text-offset`; the text will be shifted right by 3 ems.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "var_anchor not implemented"]
struct LayoutSymbolTextVariableAnchorOffset(serde_json::Value);

/// The property allows control over a symbol's orientation. Note that the property values act as a hint, so that a symbol whose language doesn’t support the provided orientation will be laid out in its natural orientation. Example: English point symbol will be rendered horizontally even if array value contains single 'vertical' enum value. The order of elements in an array define priority order for the placement of an orientation variant.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutSymbolTextWritingModeValue {
    /// If a text's language supports horizontal writing mode, symbols with point placement would be laid out horizontally.
    #[serde(rename = "horizontal")]
    Horizontal,
    /// If a text's language supports vertical writing mode, symbols with point placement would be laid out vertically.
    #[serde(rename = "vertical")]
    Vertical,
}

/// The property allows control over a symbol's orientation. Note that the property values act as a hint, so that a symbol whose language doesn’t support the provided orientation will be laid out in its natural orientation. Example: English point symbol will be rendered horizontally even if array value contains single 'vertical' enum value. The order of elements in an array define priority order for the placement of an orientation variant.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct LayoutSymbolTextWritingMode(Vec<LayoutSymbolTextWritingModeValue>);

/// Whether this layer is displayed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LayoutSymbolVisibility {
    /// The layer is not shown.
    #[serde(rename = "none")]
    None,
    /// The layer is shown.
    #[serde(rename = "visible")]
    Visible,
}

impl Default for LayoutSymbolVisibility {
    fn default() -> Self {
        Self::Visible
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Light {
    /// Whether extruded geometries are lit relative to the map or viewport.
    pub anchor: LightAnchor,
    /// Color tint for lighting extruded geometries.
    pub color: LightColor,
    /// Intensity of lighting (on a scale from 0 to 1). Higher numbers will present as more extreme contrast.
    pub intensity: LightIntensity,
    /// Position of the light source relative to lit (extruded) geometries, in [r radial coordinate, a azimuthal angle, p polar angle] where r indicates the distance from the center of the base of an object to its light, a indicates the position of the light relative to 0° (0° when `light.anchor` is set to `viewport` corresponds to the top of the viewport, or 0° when `light.anchor` is set to `map` corresponds to due north, and degrees proceed clockwise), and p indicates the height of the light (from 0°, directly above, to 180°, directly below).
    pub position: LightPosition,
}

/// Whether extruded geometries are lit relative to the map or viewport.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LightAnchor {
    /// The position of the light source is aligned to the rotation of the map.
    #[serde(rename = "map")]
    Map,
    /// The position of the light source is aligned to the rotation of the viewport.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for LightAnchor {
    fn default() -> Self {
        Self::Viewport
    }
}

/// Color tint for lighting extruded geometries.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct LightColor(color::DynamicColor);

impl Default for LightColor {
    fn default() -> Self {
        Self(color::parse_color("#ffffff").expect("Invalid color specified as the default value"))
    }
}

/// Intensity of lighting (on a scale from 0 to 1). Higher numbers will present as more extreme contrast.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LightIntensity(serde_json::Number);

impl Default for LightIntensity {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(0.5)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Position of the light source relative to lit (extruded) geometries, in [r radial coordinate, a azimuthal angle, p polar angle] where r indicates the distance from the center of the base of an object to its light, a indicates the position of the light relative to 0° (0° when `light.anchor` is set to `viewport` corresponds to the top of the viewport, or 0° when `light.anchor` is set to `map` corresponds to due north, and degrees proceed clockwise), and p indicates the height of the light (from 0°, directly above, to 180°, directly below).
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct LightPosition(Box<[serde_json::Number; 3]>);

impl Default for LightPosition {
    fn default() -> Self {
        Self(Box::new([
            serde_json::Number::from_f64(1.15)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_i128(210)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_i128(30)
                .expect("the number is serialised from a number and is thus always valid"),
        ]))
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum Paint {
    PaintFill(PaintFill),
    PaintLine(PaintLine),
    PaintCircle(PaintCircle),
    PaintHeatmap(PaintHeatmap),
    PaintFillExtrusion(PaintFillExtrusion),
    PaintSymbol(PaintSymbol),
    PaintRaster(PaintRaster),
    PaintHillshade(PaintHillshade),
    PaintColorRelief(PaintColorRelief),
    PaintBackground(PaintBackground),
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintBackground {
    /// The color with which the background will be drawn.
    #[serde(rename = "background-color")]
    pub background_color: PaintBackgroundBackgroundColor,
    /// The opacity at which the background will be drawn.
    #[serde(rename = "background-opacity")]
    pub background_opacity: PaintBackgroundBackgroundOpacity,
    /// Name of image in sprite to use for drawing an image background. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
    #[serde(rename = "background-pattern")]
    pub background_pattern: PaintBackgroundBackgroundPattern,
}

/// The color with which the background will be drawn.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct PaintBackgroundBackgroundColor(color::DynamicColor);

impl Default for PaintBackgroundBackgroundColor {
    fn default() -> Self {
        Self(color::parse_color("#000000").expect("Invalid color specified as the default value"))
    }
}

/// The opacity at which the background will be drawn.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintBackgroundBackgroundOpacity(serde_json::Number);

impl Default for PaintBackgroundBackgroundOpacity {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Name of image in sprite to use for drawing an image background. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "resolved_image not implemented"]
struct PaintBackgroundBackgroundPattern(serde_json::Value);

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintCircle {
    /// Amount to blur the circle. 1 blurs the circle such that only the centerpoint is full opacity.
    #[serde(rename = "circle-blur")]
    pub circle_blur: PaintCircleCircleBlur,
    /// The fill color of the circle.
    #[serde(rename = "circle-color")]
    pub circle_color: PaintCircleCircleColor,
    /// The opacity at which the circle will be drawn.
    #[serde(rename = "circle-opacity")]
    pub circle_opacity: PaintCircleCircleOpacity,
    /// Orientation of circle when map is pitched.
    #[serde(rename = "circle-pitch-alignment")]
    pub circle_pitch_alignment: PaintCircleCirclePitchAlignment,
    /// Controls the scaling behavior of the circle when the map is pitched.
    #[serde(rename = "circle-pitch-scale")]
    pub circle_pitch_scale: PaintCircleCirclePitchScale,
    /// Circle radius.
    #[serde(rename = "circle-radius")]
    pub circle_radius: PaintCircleCircleRadius,
    /// The stroke color of the circle.
    #[serde(rename = "circle-stroke-color")]
    pub circle_stroke_color: PaintCircleCircleStrokeColor,
    /// The opacity of the circle's stroke.
    #[serde(rename = "circle-stroke-opacity")]
    pub circle_stroke_opacity: PaintCircleCircleStrokeOpacity,
    /// The width of the circle's stroke. Strokes are placed outside of the `circle-radius`.
    #[serde(rename = "circle-stroke-width")]
    pub circle_stroke_width: PaintCircleCircleStrokeWidth,
    /// The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.
    #[serde(rename = "circle-translate")]
    pub circle_translate: PaintCircleCircleTranslate,
    /// Controls the frame of reference for `circle-translate`.
    #[serde(rename = "circle-translate-anchor")]
    pub circle_translate_anchor: PaintCircleCircleTranslateAnchor,
}

/// Amount to blur the circle. 1 blurs the circle such that only the centerpoint is full opacity.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintCircleCircleBlur(serde_json::Number);

impl Default for PaintCircleCircleBlur {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The fill color of the circle.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct PaintCircleCircleColor(color::DynamicColor);

impl Default for PaintCircleCircleColor {
    fn default() -> Self {
        Self(color::parse_color("#000000").expect("Invalid color specified as the default value"))
    }
}

/// The opacity at which the circle will be drawn.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintCircleCircleOpacity(serde_json::Number);

impl Default for PaintCircleCircleOpacity {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Orientation of circle when map is pitched.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum PaintCircleCirclePitchAlignment {
    /// The circle is aligned to the plane of the map.
    #[serde(rename = "map")]
    Map,
    /// The circle is aligned to the plane of the viewport.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for PaintCircleCirclePitchAlignment {
    fn default() -> Self {
        Self::Viewport
    }
}

/// Controls the scaling behavior of the circle when the map is pitched.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum PaintCircleCirclePitchScale {
    /// Circles are scaled according to their apparent distance to the camera.
    #[serde(rename = "map")]
    Map,
    /// Circles are not scaled.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for PaintCircleCirclePitchScale {
    fn default() -> Self {
        Self::Map
    }
}

/// Circle radius.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintCircleCircleRadius(serde_json::Number);

impl Default for PaintCircleCircleRadius {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(5)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The stroke color of the circle.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct PaintCircleCircleStrokeColor(color::DynamicColor);

impl Default for PaintCircleCircleStrokeColor {
    fn default() -> Self {
        Self(color::parse_color("#000000").expect("Invalid color specified as the default value"))
    }
}

/// The opacity of the circle's stroke.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintCircleCircleStrokeOpacity(serde_json::Number);

impl Default for PaintCircleCircleStrokeOpacity {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The width of the circle's stroke. Strokes are placed outside of the `circle-radius`.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintCircleCircleStrokeWidth(serde_json::Number);

impl Default for PaintCircleCircleStrokeWidth {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct PaintCircleCircleTranslate(Box<[serde_json::Number; 2]>);

impl Default for PaintCircleCircleTranslate {
    fn default() -> Self {
        Self(Box::new([
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        ]))
    }
}

/// Controls the frame of reference for `circle-translate`.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum PaintCircleCircleTranslateAnchor {
    /// The circle is translated relative to the map.
    #[serde(rename = "map")]
    Map,
    /// The circle is translated relative to the viewport.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for PaintCircleCircleTranslateAnchor {
    fn default() -> Self {
        Self::Map
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintColorRelief {
    /// Defines the color of each pixel based on its elevation. Should be an expression that uses `["elevation"]` as input.
    #[serde(rename = "color-relief-color")]
    pub color_relief_color: PaintColorReliefColorReliefColor,
    /// The opacity at which the color-relief will be drawn.
    #[serde(rename = "color-relief-opacity")]
    pub color_relief_opacity: PaintColorReliefColorReliefOpacity,
}

/// Defines the color of each pixel based on its elevation. Should be an expression that uses `["elevation"]` as input.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct PaintColorReliefColorReliefColor(color::DynamicColor);

/// The opacity at which the color-relief will be drawn.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintColorReliefColorReliefOpacity(serde_json::Number);

impl Default for PaintColorReliefColorReliefOpacity {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintFill {
    /// Whether or not the fill should be antialiased.
    #[serde(rename = "fill-antialias")]
    pub fill_antialias: PaintFillFillAntialias,
    /// The color of the filled part of this layer. This color can be specified as `rgba` with an alpha component and the color's opacity will not affect the opacity of the 1px stroke, if it is used.
    #[serde(rename = "fill-color")]
    pub fill_color: PaintFillFillColor,
    /// The opacity of the entire fill layer. In contrast to the `fill-color`, this value will also affect the 1px stroke around the fill, if the stroke is used.
    #[serde(rename = "fill-opacity")]
    pub fill_opacity: PaintFillFillOpacity,
    /// The outline color of the fill. Matches the value of `fill-color` if unspecified.
    #[serde(rename = "fill-outline-color")]
    pub fill_outline_color: PaintFillFillOutlineColor,
    /// Name of image in sprite to use for drawing image fills. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
    #[serde(rename = "fill-pattern")]
    pub fill_pattern: PaintFillFillPattern,
    /// The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.
    #[serde(rename = "fill-translate")]
    pub fill_translate: PaintFillFillTranslate,
    /// Controls the frame of reference for `fill-translate`.
    #[serde(rename = "fill-translate-anchor")]
    pub fill_translate_anchor: PaintFillFillTranslateAnchor,
}

/// Whether or not the fill should be antialiased.
#[derive(serde::Deserialize, PartialEq, Debug, Clone, Copy)]
struct PaintFillFillAntialias(bool);

impl Default for PaintFillFillAntialias {
    fn default() -> Self {
        Self(true)
    }
}

/// The color of the filled part of this layer. This color can be specified as `rgba` with an alpha component and the color's opacity will not affect the opacity of the 1px stroke, if it is used.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct PaintFillFillColor(color::DynamicColor);

impl Default for PaintFillFillColor {
    fn default() -> Self {
        Self(color::parse_color("#000000").expect("Invalid color specified as the default value"))
    }
}

/// The opacity of the entire fill layer. In contrast to the `fill-color`, this value will also affect the 1px stroke around the fill, if the stroke is used.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintFillFillOpacity(serde_json::Number);

impl Default for PaintFillFillOpacity {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The outline color of the fill. Matches the value of `fill-color` if unspecified.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct PaintFillFillOutlineColor(color::DynamicColor);

/// Name of image in sprite to use for drawing image fills. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "resolved_image not implemented"]
struct PaintFillFillPattern(serde_json::Value);

/// The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct PaintFillFillTranslate(Box<[serde_json::Number; 2]>);

impl Default for PaintFillFillTranslate {
    fn default() -> Self {
        Self(Box::new([
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        ]))
    }
}

/// Controls the frame of reference for `fill-translate`.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum PaintFillFillTranslateAnchor {
    /// The fill is translated relative to the map.
    #[serde(rename = "map")]
    Map,
    /// The fill is translated relative to the viewport.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for PaintFillFillTranslateAnchor {
    fn default() -> Self {
        Self::Map
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintFillExtrusion {
    /// The height with which to extrude the base of this layer. Must be less than or equal to `fill-extrusion-height`.
    #[serde(rename = "fill-extrusion-base")]
    pub fill_extrusion_base: PaintFillExtrusionFillExtrusionBase,
    /// The base color of the extruded fill. The extrusion's surfaces will be shaded differently based on this color in combination with the root `light` settings. If this color is specified as `rgba` with an alpha component, the alpha component will be ignored; use `fill-extrusion-opacity` to set layer opacity.
    #[serde(rename = "fill-extrusion-color")]
    pub fill_extrusion_color: PaintFillExtrusionFillExtrusionColor,
    /// The height with which to extrude this layer.
    #[serde(rename = "fill-extrusion-height")]
    pub fill_extrusion_height: PaintFillExtrusionFillExtrusionHeight,
    /// The opacity of the entire fill extrusion layer. This is rendered on a per-layer, not per-feature, basis, and data-driven styling is not available.
    #[serde(rename = "fill-extrusion-opacity")]
    pub fill_extrusion_opacity: PaintFillExtrusionFillExtrusionOpacity,
    /// Name of image in sprite to use for drawing images on extruded fills. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
    #[serde(rename = "fill-extrusion-pattern")]
    pub fill_extrusion_pattern: PaintFillExtrusionFillExtrusionPattern,
    /// The geometry's offset. Values are [x, y] where negatives indicate left and up (on the flat plane), respectively.
    #[serde(rename = "fill-extrusion-translate")]
    pub fill_extrusion_translate: PaintFillExtrusionFillExtrusionTranslate,
    /// Controls the frame of reference for `fill-extrusion-translate`.
    #[serde(rename = "fill-extrusion-translate-anchor")]
    pub fill_extrusion_translate_anchor: PaintFillExtrusionFillExtrusionTranslateAnchor,
    /// Whether to apply a vertical gradient to the sides of a fill-extrusion layer. If true, sides will be shaded slightly darker farther down.
    #[serde(rename = "fill-extrusion-vertical-gradient")]
    pub fill_extrusion_vertical_gradient: PaintFillExtrusionFillExtrusionVerticalGradient,
}

/// The height with which to extrude the base of this layer. Must be less than or equal to `fill-extrusion-height`.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintFillExtrusionFillExtrusionBase(serde_json::Number);

impl Default for PaintFillExtrusionFillExtrusionBase {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The base color of the extruded fill. The extrusion's surfaces will be shaded differently based on this color in combination with the root `light` settings. If this color is specified as `rgba` with an alpha component, the alpha component will be ignored; use `fill-extrusion-opacity` to set layer opacity.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct PaintFillExtrusionFillExtrusionColor(color::DynamicColor);

impl Default for PaintFillExtrusionFillExtrusionColor {
    fn default() -> Self {
        Self(color::parse_color("#000000").expect("Invalid color specified as the default value"))
    }
}

/// The height with which to extrude this layer.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintFillExtrusionFillExtrusionHeight(serde_json::Number);

impl Default for PaintFillExtrusionFillExtrusionHeight {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The opacity of the entire fill extrusion layer. This is rendered on a per-layer, not per-feature, basis, and data-driven styling is not available.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintFillExtrusionFillExtrusionOpacity(serde_json::Number);

impl Default for PaintFillExtrusionFillExtrusionOpacity {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Name of image in sprite to use for drawing images on extruded fills. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "resolved_image not implemented"]
struct PaintFillExtrusionFillExtrusionPattern(serde_json::Value);

/// The geometry's offset. Values are [x, y] where negatives indicate left and up (on the flat plane), respectively.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct PaintFillExtrusionFillExtrusionTranslate(Box<[serde_json::Number; 2]>);

impl Default for PaintFillExtrusionFillExtrusionTranslate {
    fn default() -> Self {
        Self(Box::new([
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        ]))
    }
}

/// Controls the frame of reference for `fill-extrusion-translate`.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum PaintFillExtrusionFillExtrusionTranslateAnchor {
    /// The fill extrusion is translated relative to the map.
    #[serde(rename = "map")]
    Map,
    /// The fill extrusion is translated relative to the viewport.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for PaintFillExtrusionFillExtrusionTranslateAnchor {
    fn default() -> Self {
        Self::Map
    }
}

/// Whether to apply a vertical gradient to the sides of a fill-extrusion layer. If true, sides will be shaded slightly darker farther down.
#[derive(serde::Deserialize, PartialEq, Debug, Clone, Copy)]
struct PaintFillExtrusionFillExtrusionVerticalGradient(bool);

impl Default for PaintFillExtrusionFillExtrusionVerticalGradient {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintHeatmap {
    /// Defines the color of each pixel based on its density value in a heatmap.  Should be an expression that uses `["heatmap-density"]` as input.
    #[serde(rename = "heatmap-color")]
    pub heatmap_color: PaintHeatmapHeatmapColor,
    /// Similar to `heatmap-weight` but controls the intensity of the heatmap globally. Primarily used for adjusting the heatmap based on zoom level.
    #[serde(rename = "heatmap-intensity")]
    pub heatmap_intensity: PaintHeatmapHeatmapIntensity,
    /// The global opacity at which the heatmap layer will be drawn.
    #[serde(rename = "heatmap-opacity")]
    pub heatmap_opacity: PaintHeatmapHeatmapOpacity,
    /// Radius of influence of one heatmap point in pixels. Increasing the value makes the heatmap smoother, but less detailed.
    #[serde(rename = "heatmap-radius")]
    pub heatmap_radius: PaintHeatmapHeatmapRadius,
    /// A measure of how much an individual point contributes to the heatmap. A value of 10 would be equivalent to having 10 points of weight 1 in the same spot. Especially useful when combined with clustering.
    #[serde(rename = "heatmap-weight")]
    pub heatmap_weight: PaintHeatmapHeatmapWeight,
}

/// Defines the color of each pixel based on its density value in a heatmap.  Should be an expression that uses `["heatmap-density"]` as input.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct PaintHeatmapHeatmapColor(color::DynamicColor);

impl Default for PaintHeatmapHeatmapColor {
    fn default() -> Self {
        todo!("needs expressions to be expressed in the style spec")
    }
}

/// Similar to `heatmap-weight` but controls the intensity of the heatmap globally. Primarily used for adjusting the heatmap based on zoom level.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintHeatmapHeatmapIntensity(serde_json::Number);

impl Default for PaintHeatmapHeatmapIntensity {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The global opacity at which the heatmap layer will be drawn.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintHeatmapHeatmapOpacity(serde_json::Number);

impl Default for PaintHeatmapHeatmapOpacity {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Radius of influence of one heatmap point in pixels. Increasing the value makes the heatmap smoother, but less detailed.
///
/// Range: 1..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintHeatmapHeatmapRadius(serde_json::Number);

impl Default for PaintHeatmapHeatmapRadius {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(30)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// A measure of how much an individual point contributes to the heatmap. A value of 10 would be equivalent to having 10 points of weight 1 in the same spot. Especially useful when combined with clustering.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintHeatmapHeatmapWeight(serde_json::Number);

impl Default for PaintHeatmapHeatmapWeight {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintHillshade {
    /// The shading color used to accentuate rugged terrain like sharp cliffs and gorges.
    #[serde(rename = "hillshade-accent-color")]
    pub hillshade_accent_color: PaintHillshadeHillshadeAccentColor,
    /// Intensity of the hillshade
    #[serde(rename = "hillshade-exaggeration")]
    pub hillshade_exaggeration: PaintHillshadeHillshadeExaggeration,
    /// The shading color of areas that faces towards the light source(s). Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
    #[serde(rename = "hillshade-highlight-color")]
    pub hillshade_highlight_color: PaintHillshadeHillshadeHighlightColor,
    /// The altitude of the light source(s) used to generate the hillshading with 0 as sunset and 90 as noon. Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
    #[serde(rename = "hillshade-illumination-altitude")]
    pub hillshade_illumination_altitude: PaintHillshadeHillshadeIlluminationAltitude,
    /// Direction of light source when map is rotated.
    #[serde(rename = "hillshade-illumination-anchor")]
    pub hillshade_illumination_anchor: PaintHillshadeHillshadeIlluminationAnchor,
    /// The direction of the light source(s) used to generate the hillshading with 0 as the top of the viewport if `hillshade-illumination-anchor` is set to `viewport` and due north if `hillshade-illumination-anchor` is set to `map`. Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
    #[serde(rename = "hillshade-illumination-direction")]
    pub hillshade_illumination_direction: PaintHillshadeHillshadeIlluminationDirection,
    /// The hillshade algorithm to use, one of `standard`, `basic`, `combined`, `igor`, or `multidirectional`. ![image](assets/hillshade_methods.png)
    #[serde(rename = "hillshade-method")]
    pub hillshade_method: PaintHillshadeHillshadeMethod,
    /// The shading color of areas that face away from the light source(s). Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
    #[serde(rename = "hillshade-shadow-color")]
    pub hillshade_shadow_color: PaintHillshadeHillshadeShadowColor,
}

/// The shading color used to accentuate rugged terrain like sharp cliffs and gorges.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct PaintHillshadeHillshadeAccentColor(color::DynamicColor);

impl Default for PaintHillshadeHillshadeAccentColor {
    fn default() -> Self {
        Self(color::parse_color("#000000").expect("Invalid color specified as the default value"))
    }
}

/// Intensity of the hillshade
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintHillshadeHillshadeExaggeration(serde_json::Number);

impl Default for PaintHillshadeHillshadeExaggeration {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(0.5)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The shading color of areas that faces towards the light source(s). Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum PaintHillshadeHillshadeHighlightColor {
    /// A color
    One(color::DynamicColor),
    /// A set of colors
    Multiple(Vec<color::DynamicColor>),
}

impl Default for PaintHillshadeHillshadeHighlightColor {
    fn default() -> Self {
        Self::One(
            color::parse_color("#FFFFFF").expect("Invalid color specified as the default value"),
        )
    }
}

/// The altitude of the light source(s) used to generate the hillshading with 0 as sunset and 90 as noon. Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
///
/// Range: 0..=90
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum PaintHillshadeHillshadeIlluminationAltitude {
    One(serde_json::Number),
    Many(Vec<serde_json::Number>),
}

impl Default for PaintHillshadeHillshadeIlluminationAltitude {
    fn default() -> Self {
        Self::One(
            serde_json::Number::from_i128(45)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Direction of light source when map is rotated.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum PaintHillshadeHillshadeIlluminationAnchor {
    /// The hillshade illumination is relative to the north direction.
    #[serde(rename = "map")]
    Map,
    /// The hillshade illumination is relative to the top of the viewport.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for PaintHillshadeHillshadeIlluminationAnchor {
    fn default() -> Self {
        Self::Viewport
    }
}

/// The direction of the light source(s) used to generate the hillshading with 0 as the top of the viewport if `hillshade-illumination-anchor` is set to `viewport` and due north if `hillshade-illumination-anchor` is set to `map`. Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
///
/// Range: 0..=359
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum PaintHillshadeHillshadeIlluminationDirection {
    One(serde_json::Number),
    Many(Vec<serde_json::Number>),
}

impl Default for PaintHillshadeHillshadeIlluminationDirection {
    fn default() -> Self {
        Self::One(
            serde_json::Number::from_i128(335)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The hillshade algorithm to use, one of `standard`, `basic`, `combined`, `igor`, or `multidirectional`. ![image](assets/hillshade_methods.png)
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum PaintHillshadeHillshadeMethod {
    /// Basic hillshade. Uses a simple physics model where the reflected light intensity is proportional to the cosine of the angle between the incident light and the surface normal. Similar to GDAL's `gdaldem` default algorithm.
    #[serde(rename = "basic")]
    Basic,
    /// Hillshade algorithm whose intensity scales with slope. Similar to GDAL's `gdaldem` with `-combined` option.
    #[serde(rename = "combined")]
    Combined,
    /// Hillshade algorithm which tries to minimize effects on other map features beneath. Similar to GDAL's `gdaldem` with `-igor` option.
    #[serde(rename = "igor")]
    Igor,
    /// Hillshade with multiple illumination directions. Uses the basic hillshade model with multiple independent light sources.
    #[serde(rename = "multidirectional")]
    Multidirectional,
    /// The legacy hillshade method.
    #[serde(rename = "standard")]
    Standard,
}

impl Default for PaintHillshadeHillshadeMethod {
    fn default() -> Self {
        Self::Standard
    }
}

/// The shading color of areas that face away from the light source(s). Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum PaintHillshadeHillshadeShadowColor {
    /// A color
    One(color::DynamicColor),
    /// A set of colors
    Multiple(Vec<color::DynamicColor>),
}

impl Default for PaintHillshadeHillshadeShadowColor {
    fn default() -> Self {
        Self::One(
            color::parse_color("#000000").expect("Invalid color specified as the default value"),
        )
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintLine {
    /// Blur applied to the line, in pixels.
    #[serde(rename = "line-blur")]
    pub line_blur: PaintLineLineBlur,
    /// The color with which the line will be drawn.
    #[serde(rename = "line-color")]
    pub line_color: PaintLineLineColor,
    /// Specifies the lengths of the alternating dashes and gaps that form the dash pattern. The lengths are later scaled by the line width. To convert a dash length to pixels, multiply the length by the current line width. GeoJSON sources with `lineMetrics: true` specified won't render dashed lines to the expected scale. Zoom-dependent expressions will be evaluated only at integer zoom levels. The only way to create an array value is using `["literal", [...]]`; arrays cannot be read from or derived from feature properties.
    #[serde(rename = "line-dasharray")]
    pub line_dasharray: PaintLineLineDasharray,
    /// Draws a line casing outside of a line's actual path. Value indicates the width of the inner gap.
    #[serde(rename = "line-gap-width")]
    pub line_gap_width: PaintLineLineGapWidth,
    /// Defines a gradient with which to color a line feature. Can only be used with GeoJSON sources that specify `"lineMetrics": true`.
    #[serde(rename = "line-gradient")]
    pub line_gradient: PaintLineLineGradient,
    /// The line's offset. For linear features, a positive value offsets the line to the right, relative to the direction of the line, and a negative value to the left. For polygon features, a positive value results in an inset, and a negative value results in an outset.
    #[serde(rename = "line-offset")]
    pub line_offset: PaintLineLineOffset,
    /// The opacity at which the line will be drawn.
    #[serde(rename = "line-opacity")]
    pub line_opacity: PaintLineLineOpacity,
    /// Name of image in sprite to use for drawing image lines. For seamless patterns, image width must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
    #[serde(rename = "line-pattern")]
    pub line_pattern: PaintLineLinePattern,
    /// The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.
    #[serde(rename = "line-translate")]
    pub line_translate: PaintLineLineTranslate,
    /// Controls the frame of reference for `line-translate`.
    #[serde(rename = "line-translate-anchor")]
    pub line_translate_anchor: PaintLineLineTranslateAnchor,
    /// Stroke thickness.
    #[serde(rename = "line-width")]
    pub line_width: PaintLineLineWidth,
}

/// Blur applied to the line, in pixels.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintLineLineBlur(serde_json::Number);

impl Default for PaintLineLineBlur {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The color with which the line will be drawn.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct PaintLineLineColor(color::DynamicColor);

impl Default for PaintLineLineColor {
    fn default() -> Self {
        Self(color::parse_color("#000000").expect("Invalid color specified as the default value"))
    }
}

/// Specifies the lengths of the alternating dashes and gaps that form the dash pattern. The lengths are later scaled by the line width. To convert a dash length to pixels, multiply the length by the current line width. GeoJSON sources with `lineMetrics: true` specified won't render dashed lines to the expected scale. Zoom-dependent expressions will be evaluated only at integer zoom levels. The only way to create an array value is using `["literal", [...]]`; arrays cannot be read from or derived from feature properties.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct PaintLineLineDasharray(Vec<serde_json::Number>);

/// Draws a line casing outside of a line's actual path. Value indicates the width of the inner gap.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintLineLineGapWidth(serde_json::Number);

impl Default for PaintLineLineGapWidth {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Defines a gradient with which to color a line feature. Can only be used with GeoJSON sources that specify `"lineMetrics": true`.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct PaintLineLineGradient(color::DynamicColor);

/// The line's offset. For linear features, a positive value offsets the line to the right, relative to the direction of the line, and a negative value to the left. For polygon features, a positive value results in an inset, and a negative value results in an outset.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintLineLineOffset(serde_json::Number);

impl Default for PaintLineLineOffset {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The opacity at which the line will be drawn.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintLineLineOpacity(serde_json::Number);

impl Default for PaintLineLineOpacity {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Name of image in sprite to use for drawing image lines. For seamless patterns, image width must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "resolved_image not implemented"]
struct PaintLineLinePattern(serde_json::Value);

/// The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct PaintLineLineTranslate(Box<[serde_json::Number; 2]>);

impl Default for PaintLineLineTranslate {
    fn default() -> Self {
        Self(Box::new([
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        ]))
    }
}

/// Controls the frame of reference for `line-translate`.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum PaintLineLineTranslateAnchor {
    /// The line is translated relative to the map.
    #[serde(rename = "map")]
    Map,
    /// The line is translated relative to the viewport.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for PaintLineLineTranslateAnchor {
    fn default() -> Self {
        Self::Map
    }
}

/// Stroke thickness.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintLineLineWidth(serde_json::Number);

impl Default for PaintLineLineWidth {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintRaster {
    /// Increase or reduce the brightness of the image. The value is the maximum brightness.
    #[serde(rename = "raster-brightness-max")]
    pub raster_brightness_max: PaintRasterRasterBrightnessMax,
    /// Increase or reduce the brightness of the image. The value is the minimum brightness.
    #[serde(rename = "raster-brightness-min")]
    pub raster_brightness_min: PaintRasterRasterBrightnessMin,
    /// Increase or reduce the contrast of the image.
    #[serde(rename = "raster-contrast")]
    pub raster_contrast: PaintRasterRasterContrast,
    /// Fade duration when a new tile is added, or when a video is started or its coordinates are updated.
    #[serde(rename = "raster-fade-duration")]
    pub raster_fade_duration: PaintRasterRasterFadeDuration,
    /// Rotates hues around the color wheel.
    #[serde(rename = "raster-hue-rotate")]
    pub raster_hue_rotate: PaintRasterRasterHueRotate,
    /// The opacity at which the image will be drawn.
    #[serde(rename = "raster-opacity")]
    pub raster_opacity: PaintRasterRasterOpacity,
    /// The resampling/interpolation method to use for overscaling, also known as texture magnification filter
    #[serde(rename = "raster-resampling")]
    pub raster_resampling: PaintRasterRasterResampling,
    /// Increase or reduce the saturation of the image.
    #[serde(rename = "raster-saturation")]
    pub raster_saturation: PaintRasterRasterSaturation,
}

/// Increase or reduce the brightness of the image. The value is the maximum brightness.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintRasterRasterBrightnessMax(serde_json::Number);

impl Default for PaintRasterRasterBrightnessMax {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Increase or reduce the brightness of the image. The value is the minimum brightness.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintRasterRasterBrightnessMin(serde_json::Number);

impl Default for PaintRasterRasterBrightnessMin {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Increase or reduce the contrast of the image.
///
/// Range: -1..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintRasterRasterContrast(serde_json::Number);

impl Default for PaintRasterRasterContrast {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Fade duration when a new tile is added, or when a video is started or its coordinates are updated.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintRasterRasterFadeDuration(serde_json::Number);

impl Default for PaintRasterRasterFadeDuration {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(300)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Rotates hues around the color wheel.
///
/// Range: every 360
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintRasterRasterHueRotate(serde_json::Number);

impl Default for PaintRasterRasterHueRotate {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The opacity at which the image will be drawn.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintRasterRasterOpacity(serde_json::Number);

impl Default for PaintRasterRasterOpacity {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The resampling/interpolation method to use for overscaling, also known as texture magnification filter
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum PaintRasterRasterResampling {
    /// (Bi)linear filtering interpolates pixel values using the weighted average of the four closest original source pixels creating a smooth but blurry look when overscaled
    #[serde(rename = "linear")]
    Linear,
    /// Nearest neighbor filtering interpolates pixel values using the nearest original source pixel creating a sharp but pixelated look when overscaled
    #[serde(rename = "nearest")]
    Nearest,
}

impl Default for PaintRasterRasterResampling {
    fn default() -> Self {
        Self::Linear
    }
}

/// Increase or reduce the saturation of the image.
///
/// Range: -1..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintRasterRasterSaturation(serde_json::Number);

impl Default for PaintRasterRasterSaturation {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintSymbol {
    /// The color of the icon. This can only be used with SDF icons.
    #[serde(rename = "icon-color")]
    pub icon_color: PaintSymbolIconColor,
    /// Fade out the halo towards the outside.
    #[serde(rename = "icon-halo-blur")]
    pub icon_halo_blur: PaintSymbolIconHaloBlur,
    /// The color of the icon's halo. Icon halos can only be used with SDF icons.
    #[serde(rename = "icon-halo-color")]
    pub icon_halo_color: PaintSymbolIconHaloColor,
    /// Distance of halo to the icon outline.
    ///
    /// The unit is in pixels only for SDF sprites that were created with a blur radius of 8, multiplied by the display density. I.e., the radius needs to be 16 for `@2x` sprites, etc.
    #[serde(rename = "icon-halo-width")]
    pub icon_halo_width: PaintSymbolIconHaloWidth,
    /// The opacity at which the icon will be drawn.
    #[serde(rename = "icon-opacity")]
    pub icon_opacity: PaintSymbolIconOpacity,
    /// Distance that the icon's anchor is moved from its original placement. Positive values indicate right and down, while negative values indicate left and up.
    #[serde(rename = "icon-translate")]
    pub icon_translate: PaintSymbolIconTranslate,
    /// Controls the frame of reference for `icon-translate`.
    #[serde(rename = "icon-translate-anchor")]
    pub icon_translate_anchor: PaintSymbolIconTranslateAnchor,
    /// The color with which the text will be drawn.
    #[serde(rename = "text-color")]
    pub text_color: PaintSymbolTextColor,
    /// The halo's fadeout distance towards the outside.
    #[serde(rename = "text-halo-blur")]
    pub text_halo_blur: PaintSymbolTextHaloBlur,
    /// The color of the text's halo, which helps it stand out from backgrounds.
    #[serde(rename = "text-halo-color")]
    pub text_halo_color: PaintSymbolTextHaloColor,
    /// Distance of halo to the font outline. Max text halo width is 1/4 of the font-size.
    #[serde(rename = "text-halo-width")]
    pub text_halo_width: PaintSymbolTextHaloWidth,
    /// The opacity at which the text will be drawn.
    #[serde(rename = "text-opacity")]
    pub text_opacity: PaintSymbolTextOpacity,
    /// Distance that the text's anchor is moved from its original placement. Positive values indicate right and down, while negative values indicate left and up.
    #[serde(rename = "text-translate")]
    pub text_translate: PaintSymbolTextTranslate,
    /// Controls the frame of reference for `text-translate`.
    #[serde(rename = "text-translate-anchor")]
    pub text_translate_anchor: PaintSymbolTextTranslateAnchor,
}

/// The color of the icon. This can only be used with SDF icons.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct PaintSymbolIconColor(color::DynamicColor);

impl Default for PaintSymbolIconColor {
    fn default() -> Self {
        Self(color::parse_color("#000000").expect("Invalid color specified as the default value"))
    }
}

/// Fade out the halo towards the outside.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintSymbolIconHaloBlur(serde_json::Number);

impl Default for PaintSymbolIconHaloBlur {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The color of the icon's halo. Icon halos can only be used with SDF icons.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct PaintSymbolIconHaloColor(color::DynamicColor);

impl Default for PaintSymbolIconHaloColor {
    fn default() -> Self {
        Self(
            color::parse_color("rgba(0, 0, 0, 0)")
                .expect("Invalid color specified as the default value"),
        )
    }
}

/// Distance of halo to the icon outline.
///
/// The unit is in pixels only for SDF sprites that were created with a blur radius of 8, multiplied by the display density. I.e., the radius needs to be 16 for `@2x` sprites, etc.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintSymbolIconHaloWidth(serde_json::Number);

impl Default for PaintSymbolIconHaloWidth {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The opacity at which the icon will be drawn.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintSymbolIconOpacity(serde_json::Number);

impl Default for PaintSymbolIconOpacity {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Distance that the icon's anchor is moved from its original placement. Positive values indicate right and down, while negative values indicate left and up.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct PaintSymbolIconTranslate(Box<[serde_json::Number; 2]>);

impl Default for PaintSymbolIconTranslate {
    fn default() -> Self {
        Self(Box::new([
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        ]))
    }
}

/// Controls the frame of reference for `icon-translate`.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum PaintSymbolIconTranslateAnchor {
    /// Icons are translated relative to the map.
    #[serde(rename = "map")]
    Map,
    /// Icons are translated relative to the viewport.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for PaintSymbolIconTranslateAnchor {
    fn default() -> Self {
        Self::Map
    }
}

/// The color with which the text will be drawn.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct PaintSymbolTextColor(color::DynamicColor);

impl Default for PaintSymbolTextColor {
    fn default() -> Self {
        Self(color::parse_color("#000000").expect("Invalid color specified as the default value"))
    }
}

/// The halo's fadeout distance towards the outside.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintSymbolTextHaloBlur(serde_json::Number);

impl Default for PaintSymbolTextHaloBlur {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The color of the text's halo, which helps it stand out from backgrounds.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct PaintSymbolTextHaloColor(color::DynamicColor);

impl Default for PaintSymbolTextHaloColor {
    fn default() -> Self {
        Self(
            color::parse_color("rgba(0, 0, 0, 0)")
                .expect("Invalid color specified as the default value"),
        )
    }
}

/// Distance of halo to the font outline. Max text halo width is 1/4 of the font-size.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintSymbolTextHaloWidth(serde_json::Number);

impl Default for PaintSymbolTextHaloWidth {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The opacity at which the text will be drawn.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintSymbolTextOpacity(serde_json::Number);

impl Default for PaintSymbolTextOpacity {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Distance that the text's anchor is moved from its original placement. Positive values indicate right and down, while negative values indicate left and up.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct PaintSymbolTextTranslate(Box<[serde_json::Number; 2]>);

impl Default for PaintSymbolTextTranslate {
    fn default() -> Self {
        Self(Box::new([
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        ]))
    }
}

/// Controls the frame of reference for `text-translate`.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum PaintSymbolTextTranslateAnchor {
    /// The text is translated relative to the map.
    #[serde(rename = "map")]
    Map,
    /// The text is translated relative to the viewport.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for PaintSymbolTextTranslateAnchor {
    fn default() -> Self {
        Self::Map
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Projection {
    /// The projection definition type. Can be specified as a string, a transition state, or an expression.
    #[serde(rename = "type")]
    pub r#type: ProjectionType,
}

/// Available Projections
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum AvailableProjections {
    /// [Web Mercator projection](https://en.wikipedia.org/wiki/Web_Mercator_projection)
    #[serde(rename = "mercator")]
    Mercator,
    /// [Vertical Perspective projection](https://en.wikipedia.org/wiki/General_Perspective_projection)
    #[serde(rename = "vertical-perspective")]
    VerticalPerspective,
}

/// The projection definition type. Can be specified as a string, a transition state, or an expression.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(untagged)]
pub enum ProjectionType {
    /// Preset for the Globe projection
    #[serde(rename = "globe")]
    Globe,
    /// Preset for the Globe projection
    Raw(AvailableProjections),
    /// Preset for the Equirectangular projection
    CameraExpression(Vec<CameraExpression<AvailableProjections>>),
}

impl Default for ProjectionType {
    fn default() -> Self {
        Self::Raw(AvailableProjections::Mercator)
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PromoteId(std::collections::BTreeMap<String, InnerPromoteId>);

/// A name of a feature property to use as ID for feature state.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct InnerPromoteId(String);

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Sky {
    /// How to blend the atmosphere. Where 1 is visible atmosphere and 0 is hidden. It is best to interpolate this expression when using globe projection.
    #[serde(rename = "atmosphere-blend")]
    pub atmosphere_blend: SkyAtmosphereBlend,
    /// The base color for the fog. Requires 3D terrain.
    #[serde(rename = "fog-color")]
    pub fog_color: SkyFogColor,
    /// How to blend the fog over the 3D terrain. Where 0 is the map center and 1 is the horizon.
    #[serde(rename = "fog-ground-blend")]
    pub fog_ground_blend: SkyFogGroundBlend,
    /// The base color at the horizon.
    #[serde(rename = "horizon-color")]
    pub horizon_color: SkyHorizonColor,
    /// How to blend the fog color and the horizon color. Where 0 is using the horizon color only and 1 is using the fog color only.
    #[serde(rename = "horizon-fog-blend")]
    pub horizon_fog_blend: SkyHorizonFogBlend,
    /// The base color for the sky.
    #[serde(rename = "sky-color")]
    pub sky_color: SkySkyColor,
    /// How to blend the sky color and the horizon color. Where 1 is blending the color at the middle of the sky and 0 is not blending at all and using the sky color only.
    #[serde(rename = "sky-horizon-blend")]
    pub sky_horizon_blend: SkySkyHorizonBlend,
}

/// How to blend the atmosphere. Where 1 is visible atmosphere and 0 is hidden. It is best to interpolate this expression when using globe projection.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SkyAtmosphereBlend(serde_json::Number);

impl Default for SkyAtmosphereBlend {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(0.8)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The base color for the fog. Requires 3D terrain.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SkyFogColor(color::DynamicColor);

impl Default for SkyFogColor {
    fn default() -> Self {
        Self(color::parse_color("#ffffff").expect("Invalid color specified as the default value"))
    }
}

/// How to blend the fog over the 3D terrain. Where 0 is the map center and 1 is the horizon.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SkyFogGroundBlend(serde_json::Number);

impl Default for SkyFogGroundBlend {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(0.5)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The base color at the horizon.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SkyHorizonColor(color::DynamicColor);

impl Default for SkyHorizonColor {
    fn default() -> Self {
        Self(color::parse_color("#ffffff").expect("Invalid color specified as the default value"))
    }
}

/// How to blend the fog color and the horizon color. Where 0 is using the horizon color only and 1 is using the fog color only.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SkyHorizonFogBlend(serde_json::Number);

impl Default for SkyHorizonFogBlend {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(0.8)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The base color for the sky.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SkySkyColor(color::DynamicColor);

impl Default for SkySkyColor {
    fn default() -> Self {
        Self(color::parse_color("#88C6FC").expect("Invalid color specified as the default value"))
    }
}

/// How to blend the sky color and the horizon color. Where 1 is blending the color at the middle of the sky and 0 is not blending at all and using the sky color only.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SkySkyHorizonBlend(serde_json::Number);

impl Default for SkySkyHorizonBlend {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(0.8)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum Source {
    SourceVector(SourceVector),
    SourceRaster(SourceRaster),
    SourceRasterDem(SourceRasterDem),
    SourceGeojson(SourceGeojson),
    SourceVideo(SourceVideo),
    SourceImage(SourceImage),
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceGeojson {
    /// Contains an attribution to be displayed when the map is shown to a user.
    pub attribution: SourceGeojsonAttribution,
    /// Size of the tile buffer on each side. A value of 0 produces no buffer. A value of 512 produces a buffer as wide as the tile itself. Larger values produce fewer rendering artifacts near tile edges and slower performance.
    pub buffer: SourceGeojsonBuffer,
    /// If the data is a collection of point features, setting this to true clusters the points by radius into groups. Cluster groups become new `Point` features in the source with additional properties:
    ///
    ///  * `cluster` Is `true` if the point is a cluster
    ///
    ///  * `cluster_id` A unique id for the cluster to be used in conjunction with the [cluster inspection methods](https://maplibre.org/maplibre-gl-js/docs/API/classes/GeoJSONSource/#getclusterexpansionzoom)
    ///
    ///  * `point_count` Number of original points grouped into this cluster
    ///
    ///  * `point_count_abbreviated` An abbreviated point count
    pub cluster: SourceGeojsonCluster,
    /// Max zoom on which to cluster points if clustering is enabled. Defaults to one zoom less than maxzoom (so that last zoom features are not clustered). Clusters are re-evaluated at integer zoom levels so setting clusterMaxZoom to 14 means the clusters will be displayed until z15.
    #[serde(rename = "clusterMaxZoom")]
    pub cluster_max_zoom: SourceGeojsonClusterMaxZoom,
    /// Minimum number of points necessary to form a cluster if clustering is enabled. Defaults to `2`.
    #[serde(rename = "clusterMinPoints")]
    pub cluster_min_points: SourceGeojsonClusterMinPoints,
    /// An object defining custom properties on the generated clusters if clustering is enabled, aggregating values from clustered points. Has the form `{"property_name": [operator, map_expression]}`. `operator` is any expression function that accepts at least 2 operands (e.g. `"+"` or `"max"`) — it accumulates the property value from clusters/points the cluster contains; `map_expression` produces the value of a single point.
    ///
    /// Example: `{"sum": ["+", ["get", "scalerank"]]}`.
    ///
    /// For more advanced use cases, in place of `operator`, you can use a custom reduce expression that references a special `["accumulated"]` value, e.g.:
    ///
    /// `{"sum": [["+", ["accumulated"], ["get", "sum"]], ["get", "scalerank"]]}`
    #[serde(rename = "clusterProperties")]
    pub cluster_properties: SourceGeojsonClusterProperties,
    /// Radius of each cluster if clustering is enabled. A value of 512 indicates a radius equal to the width of a tile.
    #[serde(rename = "clusterRadius")]
    pub cluster_radius: SourceGeojsonClusterRadius,
    /// A URL to a GeoJSON file, or inline GeoJSON.
    pub data: SourceGeojsonData,
    /// An expression for filtering features prior to processing them for rendering.
    pub filter: SourceGeojsonFilter,
    /// Whether to generate ids for the geojson features. When enabled, the `feature.id` property will be auto assigned based on its index in the `features` array, over-writing any previous values.
    #[serde(rename = "generateId")]
    pub generate_id: SourceGeojsonGenerateId,
    /// Whether to calculate line distance metrics. This is required for line layers that specify `line-gradient` values.
    #[serde(rename = "lineMetrics")]
    pub line_metrics: SourceGeojsonLineMetrics,
    /// Maximum zoom level at which to create vector tiles (higher means greater detail at high zoom levels).
    pub maxzoom: SourceGeojsonMaxzoom,
    /// A property to use as a feature id (for feature state). Either a property name, or an object of the form `{<sourceLayer>: <propertyName>}`.
    #[serde(rename = "promoteId")]
    pub promote_id: SourceGeojsonPromoteId,
    /// Douglas-Peucker simplification tolerance (higher means simpler geometries and faster performance).
    pub tolerance: SourceGeojsonTolerance,
    /// The data type of the GeoJSON source.
    #[serde(rename = "type")]
    pub r#type: SourceGeojsonType,
}

/// Contains an attribution to be displayed when the map is shown to a user.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceGeojsonAttribution(String);

/// Size of the tile buffer on each side. A value of 0 produces no buffer. A value of 512 produces a buffer as wide as the tile itself. Larger values produce fewer rendering artifacts near tile edges and slower performance.
///
/// Range: 0..=512
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceGeojsonBuffer(serde_json::Number);

impl Default for SourceGeojsonBuffer {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(128)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// If the data is a collection of point features, setting this to true clusters the points by radius into groups. Cluster groups become new `Point` features in the source with additional properties:
///
///  * `cluster` Is `true` if the point is a cluster
///
///  * `cluster_id` A unique id for the cluster to be used in conjunction with the [cluster inspection methods](https://maplibre.org/maplibre-gl-js/docs/API/classes/GeoJSONSource/#getclusterexpansionzoom)
///
///  * `point_count` Number of original points grouped into this cluster
///
///  * `point_count_abbreviated` An abbreviated point count
#[derive(serde::Deserialize, PartialEq, Debug, Clone, Copy)]
struct SourceGeojsonCluster(bool);

impl Default for SourceGeojsonCluster {
    fn default() -> Self {
        Self(false)
    }
}

/// Max zoom on which to cluster points if clustering is enabled. Defaults to one zoom less than maxzoom (so that last zoom features are not clustered). Clusters are re-evaluated at integer zoom levels so setting clusterMaxZoom to 14 means the clusters will be displayed until z15.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceGeojsonClusterMaxZoom(serde_json::Number);

/// Minimum number of points necessary to form a cluster if clustering is enabled. Defaults to `2`.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceGeojsonClusterMinPoints(serde_json::Number);

/// An object defining custom properties on the generated clusters if clustering is enabled, aggregating values from clustered points. Has the form `{"property_name": [operator, map_expression]}`. `operator` is any expression function that accepts at least 2 operands (e.g. `"+"` or `"max"`) — it accumulates the property value from clusters/points the cluster contains; `map_expression` produces the value of a single point.
///
/// Example: `{"sum": ["+", ["get", "scalerank"]]}`.
///
/// For more advanced use cases, in place of `operator`, you can use a custom reduce expression that references a special `["accumulated"]` value, e.g.:
///
/// `{"sum": [["+", ["accumulated"], ["get", "sum"]], ["get", "scalerank"]]}`
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceGeojsonClusterProperties(serde_json::Value);

/// Radius of each cluster if clustering is enabled. A value of 512 indicates a radius equal to the width of a tile.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceGeojsonClusterRadius(serde_json::Number);

impl Default for SourceGeojsonClusterRadius {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(50)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// A URL to a GeoJSON file, or inline GeoJSON.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceGeojsonData(serde_json::Value);

/// An expression for filtering features prior to processing them for rendering.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceGeojsonFilter(serde_json::Value);

/// Whether to generate ids for the geojson features. When enabled, the `feature.id` property will be auto assigned based on its index in the `features` array, over-writing any previous values.
#[derive(serde::Deserialize, PartialEq, Debug, Clone, Copy)]
struct SourceGeojsonGenerateId(bool);

impl Default for SourceGeojsonGenerateId {
    fn default() -> Self {
        Self(false)
    }
}

/// Whether to calculate line distance metrics. This is required for line layers that specify `line-gradient` values.
#[derive(serde::Deserialize, PartialEq, Debug, Clone, Copy)]
struct SourceGeojsonLineMetrics(bool);

impl Default for SourceGeojsonLineMetrics {
    fn default() -> Self {
        Self(false)
    }
}

/// Maximum zoom level at which to create vector tiles (higher means greater detail at high zoom levels).
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceGeojsonMaxzoom(serde_json::Number);

impl Default for SourceGeojsonMaxzoom {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(18)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// A property to use as a feature id (for feature state). Either a property name, or an object of the form `{<sourceLayer>: <propertyName>}`.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceGeojsonPromoteId(String);

/// Douglas-Peucker simplification tolerance (higher means simpler geometries and faster performance).
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceGeojsonTolerance(serde_json::Number);

impl Default for SourceGeojsonTolerance {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(0.375)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The data type of the GeoJSON source.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum SourceGeojsonType {
    /// A GeoJSON data source.
    #[serde(rename = "geojson")]
    Geojson,
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceImage {
    /// Corners of image specified in longitude, latitude pairs.
    pub coordinates: SourceImageCoordinates,
    /// The data type of the image source.
    #[serde(rename = "type")]
    pub r#type: SourceImageType,
    /// URL that points to an image.
    pub url: SourceImageUrl,
}

/// A single longitude, latitude pair.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceImageCoordinatesValue(Box<[serde_json::Number; 2]>);

/// Corners of image specified in longitude, latitude pairs.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceImageCoordinates(Box<[SourceImageCoordinatesValue; 4]>);

/// The data type of the image source.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum SourceImageType {
    /// An image data source.
    #[serde(rename = "image")]
    Image,
}

/// URL that points to an image.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceImageUrl(String);

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceRaster {
    /// Other keys to configure the data source.
    #[serde(flatten)]
    pub star: std::collections::BTreeMap<String, SourceRasterStar>,
    /// Contains an attribution to be displayed when the map is shown to a user.
    pub attribution: SourceRasterAttribution,
    /// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
    pub bounds: SourceRasterBounds,
    /// Maximum zoom level for which tiles are available, as in the TileJSON spec. Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
    pub maxzoom: SourceRasterMaxzoom,
    /// Minimum zoom level for which tiles are available, as in the TileJSON spec.
    pub minzoom: SourceRasterMinzoom,
    /// Influences the y direction of the tile coordinates. The global-mercator (aka Spherical Mercator) profile is assumed.
    pub scheme: SourceRasterScheme,
    /// The minimum visual size to display tiles for this layer. Only configurable for raster layers.
    #[serde(rename = "tileSize")]
    pub tile_size: SourceRasterTileSize,
    /// An array of one or more tile source URLs, as in the TileJSON spec.
    pub tiles: SourceRasterTiles,
    /// The type of the source.
    #[serde(rename = "type")]
    pub r#type: SourceRasterType,
    /// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
    pub url: SourceRasterUrl,
    /// A setting to determine whether a source's tiles are cached locally.
    pub volatile: SourceRasterVolatile,
}

/// Other keys to configure the data source.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceRasterStar(serde_json::Value);

/// Contains an attribution to be displayed when the map is shown to a user.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceRasterAttribution(String);

/// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceRasterBounds(Box<[serde_json::Number; 4]>);

impl Default for SourceRasterBounds {
    fn default() -> Self {
        Self(Box::new([
            serde_json::Number::from_i128(-180)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_f64(-85.051129)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_i128(180)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_f64(85.051129)
                .expect("the number is serialised from a number and is thus always valid"),
        ]))
    }
}

/// Maximum zoom level for which tiles are available, as in the TileJSON spec. Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceRasterMaxzoom(serde_json::Number);

impl Default for SourceRasterMaxzoom {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(22)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Minimum zoom level for which tiles are available, as in the TileJSON spec.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceRasterMinzoom(serde_json::Number);

impl Default for SourceRasterMinzoom {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Influences the y direction of the tile coordinates. The global-mercator (aka Spherical Mercator) profile is assumed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum SourceRasterScheme {
    /// OSGeo spec scheme.
    #[serde(rename = "tms")]
    Tms,
    /// Slippy map tilenames scheme.
    #[serde(rename = "xyz")]
    Xyz,
}

impl Default for SourceRasterScheme {
    fn default() -> Self {
        Self::Xyz
    }
}

/// The minimum visual size to display tiles for this layer. Only configurable for raster layers.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceRasterTileSize(serde_json::Number);

impl Default for SourceRasterTileSize {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(512)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// An array of one or more tile source URLs, as in the TileJSON spec.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceRasterTiles(Vec<String>);

/// The type of the source.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum SourceRasterType {
    /// A raster tile source.
    #[serde(rename = "raster")]
    Raster,
}

/// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceRasterUrl(String);

/// A setting to determine whether a source's tiles are cached locally.
#[derive(serde::Deserialize, PartialEq, Debug, Clone, Copy)]
struct SourceRasterVolatile(bool);

impl Default for SourceRasterVolatile {
    fn default() -> Self {
        Self(false)
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceRasterDem {
    /// Other keys to configure the data source.
    #[serde(flatten)]
    pub star: std::collections::BTreeMap<String, SourceRasterDemStar>,
    /// Contains an attribution to be displayed when the map is shown to a user.
    pub attribution: SourceRasterDemAttribution,
    /// Value that will be added to the encoding mix when decoding. Only used on custom encodings.
    #[serde(rename = "baseShift")]
    pub base_shift: SourceRasterDemBaseShift,
    /// Value that will be multiplied by the blue channel value when decoding. Only used on custom encodings.
    #[serde(rename = "blueFactor")]
    pub blue_factor: SourceRasterDemBlueFactor,
    /// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
    pub bounds: SourceRasterDemBounds,
    /// The encoding used by this source. Mapbox Terrain RGB is used by default.
    pub encoding: SourceRasterDemEncoding,
    /// Value that will be multiplied by the green channel value when decoding. Only used on custom encodings.
    #[serde(rename = "greenFactor")]
    pub green_factor: SourceRasterDemGreenFactor,
    /// Maximum zoom level for which tiles are available, as in the TileJSON spec. Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
    pub maxzoom: SourceRasterDemMaxzoom,
    /// Minimum zoom level for which tiles are available, as in the TileJSON spec.
    pub minzoom: SourceRasterDemMinzoom,
    /// Value that will be multiplied by the red channel value when decoding. Only used on custom encodings.
    #[serde(rename = "redFactor")]
    pub red_factor: SourceRasterDemRedFactor,
    /// The minimum visual size to display tiles for this layer. Only configurable for raster layers.
    #[serde(rename = "tileSize")]
    pub tile_size: SourceRasterDemTileSize,
    /// An array of one or more tile source URLs, as in the TileJSON spec.
    pub tiles: SourceRasterDemTiles,
    /// The type of the source.
    #[serde(rename = "type")]
    pub r#type: SourceRasterDemType,
    /// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
    pub url: SourceRasterDemUrl,
    /// A setting to determine whether a source's tiles are cached locally.
    pub volatile: SourceRasterDemVolatile,
}

/// Other keys to configure the data source.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceRasterDemStar(serde_json::Value);

/// Contains an attribution to be displayed when the map is shown to a user.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceRasterDemAttribution(String);

/// Value that will be added to the encoding mix when decoding. Only used on custom encodings.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceRasterDemBaseShift(serde_json::Number);

impl Default for SourceRasterDemBaseShift {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(0.0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Value that will be multiplied by the blue channel value when decoding. Only used on custom encodings.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceRasterDemBlueFactor(serde_json::Number);

impl Default for SourceRasterDemBlueFactor {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(1.0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceRasterDemBounds(Box<[serde_json::Number; 4]>);

impl Default for SourceRasterDemBounds {
    fn default() -> Self {
        Self(Box::new([
            serde_json::Number::from_i128(-180)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_f64(-85.051129)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_i128(180)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_f64(85.051129)
                .expect("the number is serialised from a number and is thus always valid"),
        ]))
    }
}

/// The encoding used by this source. Mapbox Terrain RGB is used by default.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum SourceRasterDemEncoding {
    /// Decodes tiles using the redFactor, blueFactor, greenFactor, baseShift parameters.
    #[serde(rename = "custom")]
    Custom,
    /// Mapbox Terrain RGB tiles. See https://www.mapbox.com/help/access-elevation-data/#mapbox-terrain-rgb for more info.
    #[serde(rename = "mapbox")]
    Mapbox,
    /// Terrarium format PNG tiles. See https://aws.amazon.com/es/public-datasets/terrain/ for more info.
    #[serde(rename = "terrarium")]
    Terrarium,
}

impl Default for SourceRasterDemEncoding {
    fn default() -> Self {
        Self::Mapbox
    }
}

/// Value that will be multiplied by the green channel value when decoding. Only used on custom encodings.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceRasterDemGreenFactor(serde_json::Number);

impl Default for SourceRasterDemGreenFactor {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(1.0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Maximum zoom level for which tiles are available, as in the TileJSON spec. Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceRasterDemMaxzoom(serde_json::Number);

impl Default for SourceRasterDemMaxzoom {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(22)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Minimum zoom level for which tiles are available, as in the TileJSON spec.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceRasterDemMinzoom(serde_json::Number);

impl Default for SourceRasterDemMinzoom {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Value that will be multiplied by the red channel value when decoding. Only used on custom encodings.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceRasterDemRedFactor(serde_json::Number);

impl Default for SourceRasterDemRedFactor {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(1.0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The minimum visual size to display tiles for this layer. Only configurable for raster layers.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceRasterDemTileSize(serde_json::Number);

impl Default for SourceRasterDemTileSize {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(512)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// An array of one or more tile source URLs, as in the TileJSON spec.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceRasterDemTiles(Vec<String>);

/// The type of the source.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum SourceRasterDemType {
    /// A RGB-encoded raster DEM source
    #[serde(rename = "raster-dem")]
    RasterDem,
}

/// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceRasterDemUrl(String);

/// A setting to determine whether a source's tiles are cached locally.
#[derive(serde::Deserialize, PartialEq, Debug, Clone, Copy)]
struct SourceRasterDemVolatile(bool);

impl Default for SourceRasterDemVolatile {
    fn default() -> Self {
        Self(false)
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceVector {
    /// Other keys to configure the data source.
    #[serde(flatten)]
    pub star: std::collections::BTreeMap<String, SourceVectorStar>,
    /// Contains an attribution to be displayed when the map is shown to a user.
    pub attribution: SourceVectorAttribution,
    /// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
    pub bounds: SourceVectorBounds,
    /// The encoding used by this source. Mapbox Vector Tiles encoding is used by default.
    pub encoding: SourceVectorEncoding,
    /// Maximum zoom level for which tiles are available, as in the TileJSON spec. Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
    pub maxzoom: SourceVectorMaxzoom,
    /// Minimum zoom level for which tiles are available, as in the TileJSON spec.
    pub minzoom: SourceVectorMinzoom,
    /// A property to use as a feature id (for feature state). Either a property name, or an object of the form `{<sourceLayer>: <propertyName>}`. If specified as a string for a vector tile source, the same property is used across all its source layers.
    #[serde(rename = "promoteId")]
    pub promote_id: SourceVectorPromoteId,
    /// Influences the y direction of the tile coordinates. The global-mercator (aka Spherical Mercator) profile is assumed.
    pub scheme: SourceVectorScheme,
    /// An array of one or more tile source URLs, as in the TileJSON spec.
    pub tiles: SourceVectorTiles,
    /// The type of the source.
    #[serde(rename = "type")]
    pub r#type: SourceVectorType,
    /// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
    pub url: SourceVectorUrl,
    /// A setting to determine whether a source's tiles are cached locally.
    pub volatile: SourceVectorVolatile,
}

/// Other keys to configure the data source.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceVectorStar(serde_json::Value);

/// Contains an attribution to be displayed when the map is shown to a user.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceVectorAttribution(String);

/// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceVectorBounds(Box<[serde_json::Number; 4]>);

impl Default for SourceVectorBounds {
    fn default() -> Self {
        Self(Box::new([
            serde_json::Number::from_i128(-180)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_f64(-85.051129)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_i128(180)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_f64(85.051129)
                .expect("the number is serialised from a number and is thus always valid"),
        ]))
    }
}

/// The encoding used by this source. Mapbox Vector Tiles encoding is used by default.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum SourceVectorEncoding {
    /// MapLibre Vector Tiles. See https://github.com/maplibre/maplibre-tile-spec for more info.
    #[serde(rename = "mlt")]
    Mlt,
    /// Mapbox Vector Tiles. See http://github.com/mapbox/vector-tile-spec for more info.
    #[serde(rename = "mvt")]
    Mvt,
}

impl Default for SourceVectorEncoding {
    fn default() -> Self {
        Self::Mvt
    }
}

/// Maximum zoom level for which tiles are available, as in the TileJSON spec. Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceVectorMaxzoom(serde_json::Number);

impl Default for SourceVectorMaxzoom {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(22)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Minimum zoom level for which tiles are available, as in the TileJSON spec.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceVectorMinzoom(serde_json::Number);

impl Default for SourceVectorMinzoom {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// A property to use as a feature id (for feature state). Either a property name, or an object of the form `{<sourceLayer>: <propertyName>}`. If specified as a string for a vector tile source, the same property is used across all its source layers.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceVectorPromoteId(String);

/// Influences the y direction of the tile coordinates. The global-mercator (aka Spherical Mercator) profile is assumed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum SourceVectorScheme {
    /// OSGeo spec scheme.
    #[serde(rename = "tms")]
    Tms,
    /// Slippy map tilenames scheme.
    #[serde(rename = "xyz")]
    Xyz,
}

impl Default for SourceVectorScheme {
    fn default() -> Self {
        Self::Xyz
    }
}

/// An array of one or more tile source URLs, as in the TileJSON spec.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceVectorTiles(Vec<String>);

/// The type of the source.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum SourceVectorType {
    /// A vector tile source.
    #[serde(rename = "vector")]
    Vector,
}

/// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceVectorUrl(String);

/// A setting to determine whether a source's tiles are cached locally.
#[derive(serde::Deserialize, PartialEq, Debug, Clone, Copy)]
struct SourceVectorVolatile(bool);

impl Default for SourceVectorVolatile {
    fn default() -> Self {
        Self(false)
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceVideo {
    /// Corners of video specified in longitude, latitude pairs.
    pub coordinates: SourceVideoCoordinates,
    /// The data type of the video source.
    #[serde(rename = "type")]
    pub r#type: SourceVideoType,
    /// URLs to video content in order of preferred format.
    pub urls: SourceVideoUrls,
}

/// A single longitude, latitude pair.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceVideoCoordinatesValue(Box<[serde_json::Number; 2]>);

/// Corners of video specified in longitude, latitude pairs.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceVideoCoordinates(Box<[SourceVideoCoordinatesValue; 4]>);

/// The data type of the video source.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum SourceVideoType {
    /// A video data source.
    #[serde(rename = "video")]
    Video,
}

/// URLs to video content in order of preferred format.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SourceVideoUrls(Vec<String>);

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Sources(std::collections::BTreeMap<String, InnerSources>);

/// Specification of a data source. For vector and raster sources, either TileJSON or a URL to a TileJSON must be provided. For image and video sources, a URL must be provided. For GeoJSON sources, a URL or inline GeoJSON must be provided.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct InnerSources(Source);

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Terrain {
    /// The exaggeration of the terrain - how high it will look.
    pub exaggeration: TerrainExaggeration,
    /// The source for the terrain data.
    pub source: TerrainSource,
}

/// The exaggeration of the terrain - how high it will look.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct TerrainExaggeration(serde_json::Number);

impl Default for TerrainExaggeration {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(1.0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The source for the terrain data.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct TerrainSource(String);

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Transition {
    /// Length of time before a transition begins.
    pub delay: TransitionDelay,
    /// Time allotted for transitions to complete.
    pub duration: TransitionDuration,
}

/// Length of time before a transition begins.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct TransitionDelay(serde_json::Number);

impl Default for TransitionDelay {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Time allotted for transitions to complete.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct TransitionDuration(serde_json::Number);

impl Default for TransitionDuration {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(300)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

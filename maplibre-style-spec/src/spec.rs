/// JSON number in an expression position
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct NumberLiteral(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

/// JSON string in an expression position
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct StringLiteral(std::string::String);

/// GeoJSON object literal
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeoJSONObjectLiteral(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_geojson))]
    geojson::GeoJson,
);

/// JSON object literal
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct JSONObjectLiteral(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
    serde_json::Value,
);

/// JSON array literal
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct JSONArrayLiteral(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_vec_json_value))]
    Vec<serde_json::Value>,
);

/// Array whose elements are string literals (e.g. match labels)
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct ArrayOfStringLiteral(Vec<StringLiteral>);

/// Array whose elements are number literals (e.g. match labels)
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct ArrayOfNumberLiteral(Vec<NumberLiteral>);

/// This is a Maplibre Style Specification
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct MaplibreStyleSpecification {
    /// Default bearing, in degrees. The bearing is the compass direction that is "up"; for example, a bearing of 90° orients the map so that east is up. This value will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    ///
    /// Range: every 360
    pub bearing: Option<RootBearing>,
    /// Default map center in longitude and latitude.  The style center will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    pub center: Option<RootCenter>,
    /// Default map center altitude in meters above sea level. The style center altitude defines the altitude where the camera is looking at and will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    #[serde(rename = "centerAltitude")]
    pub center_altitude: Option<RootCenterAltitude>,
    /// The `font-faces` property can be used to specify what font files to use for rendering text. Font faces contain information needed to render complex texts such as [Devanagari](https://en.wikipedia.org/wiki/Devanagari), [Khmer](https://en.wikipedia.org/wiki/Khmer_script) among many others.<h2>Unicode range</h2>The optional `unicode-range` property can be used to only use a particular font file for characters within the specified unicode range(s). Its value should be an array of strings, each indicating a start and end of a unicode range, similar to the [CSS descriptor with the same name](https://developer.mozilla.org/en-US/docs/Web/CSS/@font-face/unicode-range). This allows specifying multiple non-consecutive unicode ranges. When not specified, the default value is `U+0-10FFFF`, meaning the font file will be used for all unicode characters.
    ///
    /// Refer to the [Unicode Character Code Charts](https://www.unicode.org/charts/) to see ranges for scripts supported by Unicode. To see what unicode code-points are available in a font, use a tool like [FontDrop](https://fontdrop.info/).
    ///
    /// <h2>Font Resolution</h2>For every name in a symbol layer’s [`text-font`](./layers.md/#text-font) array, characters are matched if they are covered one of the by the font files in the corresponding entry of the `font-faces` map. Any still-unmatched characters then fall back to the [`glyphs`](./glyphs.md) URL if provided.
    ///
    /// <h2>Supported Fonts</h2>What type of fonts are supported is implementation-defined. Unsupported fonts are ignored.
    #[serde(rename = "font-faces")]
    pub font_faces: Option<RootFontFaces>,
    /// The global light source.
    pub light: Option<RootLight>,
    /// Arbitrary properties useful to track with the stylesheet, but do not influence rendering. Properties should be prefixed to avoid collisions, like 'maplibre:'.
    pub metadata: Option<RootMetadata>,
    /// A human-readable name for the style.
    pub name: Option<RootName>,
    /// Default pitch, in degrees. Zero is perpendicular to the surface, for a look straight down at the map, while a greater value like 60 looks ahead towards the horizon. The style pitch will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    pub pitch: Option<RootPitch>,
    /// The projection configuration
    pub projection: Option<RootProjection>,
    /// Default roll, in degrees. The roll angle is measured counterclockwise about the camera boresight. The style roll will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    pub roll: Option<RootRoll>,
    /// The map's sky configuration. **Note:** this definition is still experimental and is under development in maplibre-gl-js.
    pub sky: Option<RootSky>,
    /// Sources state which data the map should display. Specify the type of source with the `type` property. Adding a source isn't enough to make data appear on the map because sources don't contain styling details like color or width. Layers refer to a source and give it a visual representation. This makes it possible to style the same source in different ways, like differentiating between types of roads in a highways layer.
    ///
    /// Tiled sources (vector and raster) must specify their details according to the [TileJSON specification](https://github.com/mapbox/tilejson-spec).
    pub sources: RootSources,
    /// An object used to define default values when using the [`global-state`](https://maplibre.org/maplibre-style-spec/expressions/#global-state) expression.
    pub state: Option<RootState>,
    /// The terrain configuration.
    pub terrain: Option<RootTerrain>,
    /// A global transition definition to use as a default across properties, to be used for timing transitions between one value and the next when no property-specific transition is set. Collision-based symbol fading is controlled independently of the style's `transition` property.
    pub transition: Option<RootTransition>,
    /// Style specification version number. Must be 8.
    pub version: RootVersion,
    /// Default zoom level.  The style zoom will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    pub zoom: Option<RootZoom>,
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

    #[rstest::rstest]
    #[case::t_cubic_bezier(serde_json::json!(["cubic-bezier",2,3,2,3]))]
    #[case::t_exponential(serde_json::json!(["exponential",2]))]
    #[case::t_linear(serde_json::json!(["linear"]))]
    fn test_example_interpolation_name_decodes(#[case] example: serde_json::Value) {
        let _ =
            serde_json::from_value::<InterpolationName>(example).expect("example should decode");
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

    #[rstest::rstest]
    #[case::t_accumulated(serde_json::json!(["accumulated"]))]
    #[case::t_at(serde_json::json!(["at",1,["literal",["a","b","c"]]]))]
    #[case::t_case(serde_json::json!(["case",["boolean",["feature-state","hover"],false],1,0.5]))]
    #[case::t_coalesce(serde_json::json!(["coalesce",["image",["concat",["get","icon"],"_15"]],["image","marker_15"]]))]
    #[case::t_feature_state(serde_json::json!(["feature-state","hover"]))]
    #[case::t_get(serde_json::json!(["get","someProperty"]))]
    #[case::t_global_state(serde_json::json!(["global-state","someProperty"]))]
    #[case::t_id(serde_json::json!(["id"]))]
    #[case::t_let(serde_json::json!(["let","someNumber",500,["interpolate",["linear"],["var","someNumber"],274,"#edf8e9",1551,"#006d2c"]]))]
    #[case::t_match(serde_json::json!(["match",["get","building_type"],"residential","#f00","commercial","#0f0","#000"]))]
    #[case::t_step(serde_json::json!(["step",["get","point_count"],20,100,30,750,40]))]
    #[case::t_var(serde_json::json!(["var","density"]))]
    fn test_example_any_decodes(#[case] example: serde_json::Value) {
        let _ = serde_json::from_value::<Any>(example).expect("example should decode");
    }

    #[rstest::rstest]
    #[case::t_literal(serde_json::json!(["literal",["DIN Offc Pro Italic","Arial Unicode MS Regular"]]))]
    #[case::t_slice(serde_json::json!(["slice",["get","name"],0,3]))]
    #[case::t_to_rgba(serde_json::json!(["to-rgba","#ff0000"]))]
    fn test_example_array_decodes(#[case] example: serde_json::Value) {
        let _ = serde_json::from_value::<Array>(example).expect("example should decode");
    }

    #[rstest::rstest]
    #[case::t_array(serde_json::json!(["array","string",3,["literal",["a","b","c"]]]))]
    fn test_example_array_less_type_length_greater_decodes(#[case] example: serde_json::Value) {
        let _ = serde_json::from_value::<ArrayLessTypeLengthGreater>(example)
            .expect("example should decode");
    }

    #[rstest::rstest]
    #[case::t_not(serde_json::json!(["!",["has","point_count"]]))]
    #[case::t_not_equal(serde_json::json!(["!=","cluster",true]))]
    #[case::t_less(serde_json::json!(["<",["get","mag"],2]))]
    #[case::t_less_equal(serde_json::json!(["<=",["get","mag"],6]))]
    #[case::t_equal_equal(serde_json::json!(["==","$type","Polygon"]))]
    #[case::t_greater(serde_json::json!([">",["get","mag"],2]))]
    #[case::t_greater_equal(serde_json::json!([">=",["get","mag"],6]))]
    #[case::t_all(serde_json::json!(["all",[">=",["get","mag"],4],["<",["get","mag"],5]]))]
    #[case::t_any(serde_json::json!(["any",[">=",["get","mag"],4],["<",["get","mag"],5]]))]
    #[case::t_boolean(serde_json::json!(["boolean",["feature-state","hover"],false]))]
    #[case::t_has(serde_json::json!(["has","someProperty"]))]
    #[case::t_in(serde_json::json!(["in","$type","Point"]))]
    #[case::t_is_supported_script(serde_json::json!(["is-supported-script","दिल्ली"]))]
    #[case::t_to_boolean(serde_json::json!(["to-boolean","someProperty"]))]
    #[case::t_within(serde_json::json!(["within",{"coordinates":[[[0,0],[0,5],[5,5],[5,0],[0,0]]],"type":"Polygon"}]))]
    fn test_example_boolean_decodes(#[case] example: serde_json::Value) {
        let _ = serde_json::from_value::<Boolean>(example).expect("example should decode");
    }

    #[rstest::rstest]
    #[case::t_collator(serde_json::json!(["collator",{"case-sensitive":true,"diacritic-sensitive":true,"locale":"fr"}]))]
    fn test_example_collator_decodes(#[case] example: serde_json::Value) {
        let _ = serde_json::from_value::<Collator>(example).expect("example should decode");
    }

    #[rstest::rstest]
    #[case::t_rgb(serde_json::json!(["rgb",255,0,0]))]
    #[case::t_rgba(serde_json::json!(["rgba",255,0,0,1]))]
    #[case::t_to_color(serde_json::json!(["to-color","#edf8e9"]))]
    fn test_example_color_decodes(#[case] example: serde_json::Value) {
        let _ = serde_json::from_value::<Color>(example).expect("example should decode");
    }

    #[rstest::rstest]
    #[case::t_interpolate_hcl(serde_json::json!(["interpolate-hcl",["linear"],["zoom"],15,"#f00",15.05,"#00f"]))]
    #[case::t_interpolate_lab(serde_json::json!(["interpolate-lab",["linear"],["zoom"],15,"#f00",15.05,"#00f"]))]
    fn test_example_color_or_array_of_color_decodes(#[case] example: serde_json::Value) {
        let _ =
            serde_json::from_value::<ColorOrArrayOfColor>(example).expect("example should decode");
    }

    #[rstest::rstest]
    #[case::t_format(serde_json::json!(["format",["upcase",["get","FacilityName"]],{"font-scale":0.8},"\n\n",{},["downcase",["get","Comments"]],{"font-scale":0.6,"vertical-align":"center"}]))]
    fn test_example_formatted_decodes(#[case] example: serde_json::Value) {
        let _ = serde_json::from_value::<Formatted>(example).expect("example should decode");
    }

    #[rstest::rstest]
    #[case::t_image(serde_json::json!(["image","marker_15"]))]
    fn test_example_image_decodes(#[case] example: serde_json::Value) {
        let _ = serde_json::from_value::<Image>(example).expect("example should decode");
    }

    #[rstest::rstest]
    #[case::t_percentage(serde_json::json!(["%",10,3]))]
    #[case::t_star(serde_json::json!(["*",2,3]))]
    #[case::t_plus(serde_json::json!(["+",2,3]))]
    #[case::t_minus(serde_json::json!(["-",10]))]
    #[case::t_slash(serde_json::json!(["/",["get","population"],["get","sq-km"]]))]
    #[case::t_power(serde_json::json!(["^",2,3]))]
    #[case::t_absolute(serde_json::json!(["abs",-1.5]))]
    #[case::t_arccosine(serde_json::json!(["acos",1]))]
    #[case::t_asin(serde_json::json!(["asin",1]))]
    #[case::t_atan(serde_json::json!(["atan",1]))]
    #[case::t_ceil(serde_json::json!(["ceil",1.5]))]
    #[case::t_cos(serde_json::json!(["cos",1]))]
    #[case::t_distance(serde_json::json!(["distance",{"coordinates":[0,0],"type":"Point"}]))]
    #[case::t_e(serde_json::json!(["e"]))]
    #[case::t_elevation(serde_json::json!(["elevation"]))]
    #[case::t_floor(serde_json::json!(["floor",1.5]))]
    #[case::t_heatmap_density(serde_json::json!(["heatmap-density"]))]
    #[case::t_index_of(serde_json::json!(["index-of","foo",["baz","bar","hello","foo","world"]]))]
    #[case::t_length(serde_json::json!(["length",["get","myArray"]]))]
    #[case::t_line_progress(serde_json::json!(["line-progress"]))]
    #[case::t_ln(serde_json::json!(["ln",8]))]
    #[case::t_ln2(serde_json::json!(["ln2"]))]
    #[case::t_log10(serde_json::json!(["log10",8]))]
    #[case::t_log2(serde_json::json!(["log2",8]))]
    #[case::t_max(serde_json::json!(["max",1,2]))]
    #[case::t_min(serde_json::json!(["min",1,2]))]
    #[case::t_number(serde_json::json!(["number",["get","population"]]))]
    #[case::t_pi(serde_json::json!(["pi"]))]
    #[case::t_round(serde_json::json!(["round",1.5]))]
    #[case::t_sin(serde_json::json!(["sin",1]))]
    #[case::t_sqrt(serde_json::json!(["sqrt",9]))]
    #[case::t_tan(serde_json::json!(["tan",1]))]
    #[case::t_to_number(serde_json::json!(["to-number","someProperty"]))]
    fn test_example_number_decodes(#[case] example: serde_json::Value) {
        let _ = serde_json::from_value::<Number>(example).expect("example should decode");
    }

    #[rstest::rstest]
    #[case::t_interpolate(serde_json::json!(["interpolate",["linear"],["zoom"],15,0,15.05,["get","height"]]))]
    fn test_example_number_or_array_of_number_or_color_or_array_of_color_or_projection_decodes(
        #[case] example: serde_json::Value,
    ) {
        let _ = serde_json::from_value::<NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection>(
            example,
        )
        .expect("example should decode");
    }

    #[rstest::rstest]
    #[case::t_literal(serde_json::json!(["literal",["DIN Offc Pro Italic","Arial Unicode MS Regular"]]))]
    #[case::t_object(serde_json::json!(["object",["get","some-property"]]))]
    #[case::t_properties(serde_json::json!(["properties"]))]
    fn test_example_object_decodes(#[case] example: serde_json::Value) {
        let _ = serde_json::from_value::<Object>(example).expect("example should decode");
    }

    #[rstest::rstest]
    #[case::t_concat(serde_json::json!(["concat","square-rgb-",["get","color"]]))]
    #[case::t_downcase(serde_json::json!(["downcase",["get","name"]]))]
    #[case::t_number_format(serde_json::json!(["number-format",["get","mag"],{"max-fraction-digits":1,"min-fraction-digits":1}]))]
    #[case::t_resolved_locale(serde_json::json!(["resolved-locale",["collator",{"case-sensitive":true,"diacritic-sensitive":false,"locale":"de"}]]))]
    #[case::t_slice(serde_json::json!(["slice",["get","name"],0,3]))]
    #[case::t_string(serde_json::json!(["string",["get","name"]]))]
    #[case::t_to_string(serde_json::json!(["to-string",["get","mag"]]))]
    #[case::t_typeof(serde_json::json!(["typeof",["get","name"]]))]
    #[case::t_upcase(serde_json::json!(["upcase",["get","name"]]))]
    fn test_example_string_decodes(#[case] example: serde_json::Value) {
        let _ = serde_json::from_value::<String>(example).expect("example should decode");
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

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone)]
#[serde(untagged)]
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
pub struct RootLight(Light);

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
pub struct RootProjection(Projection);

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
pub struct RootSky(Sky);

/// Sources state which data the map should display. Specify the type of source with the `type` property. Adding a source isn't enough to make data appear on the map because sources don't contain styling details like color or width. Layers refer to a source and give it a visual representation. This makes it possible to style the same source in different ways, like differentiating between types of roads in a highways layer.
///
/// Tiled sources (vector and raster) must specify their details according to the [TileJSON specification](https://github.com/mapbox/tilejson-spec).
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RootSources(std::collections::BTreeMap<std::string::String, Source>);

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
pub struct RootTerrain(Terrain);

/// A global transition definition to use as a default across properties, to be used for timing transitions between one value and the next when no property-specific transition is set. Collision-based symbol fading is controlled independently of the style's `transition` property.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RootTransition(Transition);

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

/// A filter selects specific features from a layer.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct Filter(bool);

/// The filter operator.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
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

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct Function {
    /// The exponential base of the interpolation curve. It controls the rate at which the result increases. Higher values make the result increase more towards the high end of the range. With `1` the stops are interpolated linearly.
    ///
    /// Range: 0..
    pub base: Option<FunctionBase>,
    /// The color space in which colors interpolated. Interpolating colors in perceptual color spaces like LAB and HCL tend to produce color ramps that look more consistent and produce colors that can be differentiated more easily than those interpolated in RGB space.
    #[serde(rename = "colorSpace")]
    pub color_space: Option<FunctionColorSpace>,
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
    pub default: Option<FunctionDefault>,
    /// An expression.
    pub expression: Option<FunctionExpression>,
    /// The name of a feature property to use as the function input.
    pub property: Option<FunctionProperty>,
    /// An array of stops.
    pub stops: Option<FunctionStops>,
    /// The interpolation strategy to use in function evaluation.
    #[serde(rename = "type")]
    pub r#type: Option<FunctionType>,
}

/// The exponential base of the interpolation curve. It controls the rate at which the result increases. Higher values make the result increase more towards the high end of the range. With `1` the stops are interpolated linearly.
///
/// Range: 0..
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FunctionBase(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for FunctionBase {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The color space in which colors interpolated. Interpolating colors in perceptual color spaces like LAB and HCL tend to produce color ramps that look more consistent and produce colors that can be differentiated more easily than those interpolated in RGB space.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FunctionColorSpace {
    /// Use the HCL color space to interpolate color values, interpolating the Hue, Chroma, and Luminance channels individually.
    #[serde(rename = "hcl")]
    Hcl,
    /// Use the LAB color space to interpolate color values.
    #[serde(rename = "lab")]
    Lab,
    /// Use the RGB color space to interpolate color values
    #[serde(rename = "rgb")]
    #[default]
    Rgb,
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
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FunctionDefault(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
    serde_json::Value,
);

/// An expression.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FunctionExpression(Any);

/// The name of a feature property to use as the function input.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FunctionProperty(std::string::String);

impl Default for FunctionProperty {
    fn default() -> Self {
        Self("$zoom".to_string())
    }
}

/// An array of stops.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FunctionStops(Vec<FunctionStop>);

/// The interpolation strategy to use in function evaluation.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FunctionType {
    /// Return the output value of the stop equal to the function input.
    #[serde(rename = "categorical")]
    Categorical,
    /// Generate an output by interpolating between stops just less than and just greater than the function input.
    #[serde(rename = "exponential")]
    #[default]
    Exponential,
    /// Return the input value as the output value.
    #[serde(rename = "identity")]
    Identity,
    /// Return the output value of the stop just less than the function input.
    #[serde(rename = "interval")]
    Interval,
}

/// FunctionStopValue Values
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FunctionStopValue {
    Zero(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
    One(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_dynamic_color))]
         color::DynamicColor,
    ),
}

/// Zoom level and value pair.
///
/// Range: 0..=24
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FunctionStop(Box<[FunctionStopValue; 2]>);

/// The geometry type for the filter to select.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum GeometryType {
    /// Filter to line geometries.
    LineString,
    /// Filter to point geometries.
    Point,
    /// Filter to polygon geometries.
    Polygon,
}

/// An interpolation defines how to transition between items. The first element of an interpolation array is a string naming the interpolation operator, e.g. `"linear"` or `"exponential"`. Elements that follow (if any) are the _arguments_ to the interpolation.
///
/// Range: 1..
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct Interpolation(InterpolationName);

/// First element in an interpolation array. May be followed by a number of arguments.
#[derive(serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum InterpolationName {
    /// Interpolates using the cubic bézier curve defined by the given control points.
    CubicBezier(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Interpolates exponentially between the stops just less than and just greater than the input.
    Exponential(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Interpolates linearly between the pair of stops just less than and just greater than the input
    Linear,
}

impl<'de> serde::Deserialize<'de> for InterpolationName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(InterpolationNameVisitor)
    }
}

/// Visitor for deserializing the syntax enum [`InterpolationName`]
struct InterpolationNameVisitor;

impl<'de> serde::de::Visitor<'de> for InterpolationNameVisitor {
    type Value = InterpolationName;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an InterpolationName expression (example: [\"cubic-bezier\",2,3,2,3])")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        /// Reads the next element from the sequence or reports a missing field error.
        #[allow(dead_code)]
        fn visit_seq_field<'de, A, T>(seq: &mut A, name: &'static str) -> Result<T, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
            T: serde::Deserialize<'de>,
        {
            seq.next_element()?
                .ok_or_else(|| serde::de::Error::missing_field(name))
        }

        // First element: operator string
        let op: std::string::String = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::custom("missing operator"))?;
        match op.as_str() {
            "cubic-bezier" => {
                let x1 = visit_seq_field(&mut seq, "x1")?;
                let y1 = visit_seq_field(&mut seq, "y1")?;
                let x2 = visit_seq_field(&mut seq, "x2")?;
                let y2 = visit_seq_field(&mut seq, "y2")?;
                Ok(InterpolationName::CubicBezier(x1, y1, x2, y2))
            }
            "exponential" => {
                let base = visit_seq_field(&mut seq, "base")?;
                Ok(InterpolationName::Exponential(base))
            }
            "linear" => Ok(InterpolationName::Linear),
            _ => Err(serde::de::Error::unknown_variant(
                &op,
                &["cubic-bezier", "exponential", "linear"],
            )),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct Light {
    /// Whether extruded geometries are lit relative to the map or viewport.
    pub anchor: Option<LightAnchor>,
    /// Color tint for lighting extruded geometries.
    pub color: Option<LightColor>,
    /// Intensity of lighting (on a scale from 0 to 1). Higher numbers will present as more extreme contrast.
    ///
    /// Range: 0..=1
    pub intensity: Option<LightIntensity>,
    /// Position of the light source relative to lit (extruded) geometries, in [r radial coordinate, a azimuthal angle, p polar angle] where r indicates the distance from the center of the base of an object to its light, a indicates the position of the light relative to 0° (0° when `light.anchor` is set to `viewport` corresponds to the top of the viewport, or 0° when `light.anchor` is set to `map` corresponds to due north, and degrees proceed clockwise), and p indicates the height of the light (from 0°, directly above, to 180°, directly below).
    pub position: Option<LightPosition>,
}

/// Whether extruded geometries are lit relative to the map or viewport.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LightAnchor {
    /// The position of the light source is aligned to the rotation of the map.
    #[serde(rename = "map")]
    Map,
    /// The position of the light source is aligned to the rotation of the viewport.
    #[serde(rename = "viewport")]
    #[default]
    Viewport,
}

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LightColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

/// Color tint for lighting extruded geometries.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LightColor {
    Expr(LightColorExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl Default for LightColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("#ffffff"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LightIntensityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Intensity of lighting (on a scale from 0 to 1). Higher numbers will present as more extreme contrast.
///
/// Range: 0..=1
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LightIntensity {
    Expr(LightIntensityExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for LightIntensity {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_f64(0.5)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Position of the light source relative to lit (extruded) geometries, in [r radial coordinate, a azimuthal angle, p polar angle] where r indicates the distance from the center of the base of an object to its light, a indicates the position of the light relative to 0° (0° when `light.anchor` is set to `viewport` corresponds to the top of the viewport, or 0° when `light.anchor` is set to `map` corresponds to due north, and degrees proceed clockwise), and p indicates the height of the light (from 0°, directly above, to 180°, directly below).
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct LightPosition(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_3_json_number))]
     Box<[serde_json::Number; 3]>,
);

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

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct Projection {
    /// The projection definition type. Can be specified as a string, a transition state, or an expression.
    #[serde(rename = "type")]
    pub r#type: Option<ProjectionType>,
}

/// The projection definition type. Can be specified as a string, a transition state, or an expression.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum ProjectionType {
    Expr(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
    Literal(std::string::String),
}

impl Default for ProjectionType {
    fn default() -> Self {
        Self::Literal("mercator".to_string())
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct PromoteId {
    /// A name of a feature property to use as ID for feature state.
    #[serde(flatten)]
    pub star: Option<std::collections::BTreeMap<std::string::String, PromoteIdStar>>,
}

/// A name of a feature property to use as ID for feature state.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct PromoteIdStar(std::string::String);

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct PropertyType;

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct Sky {
    /// How to blend the atmosphere. Where 1 is visible atmosphere and 0 is hidden. It is best to interpolate this expression when using globe projection.
    ///
    /// Range: 0..=1
    #[serde(rename = "atmosphere-blend")]
    pub atmosphere_blend: Option<SkyAtmosphereBlend>,
    /// The base color for the fog. Requires 3D terrain.
    #[serde(rename = "fog-color")]
    pub fog_color: Option<SkyFogColor>,
    /// How to blend the fog over the 3D terrain. Where 0 is the map center and 1 is the horizon.
    ///
    /// Range: 0..=1
    #[serde(rename = "fog-ground-blend")]
    pub fog_ground_blend: Option<SkyFogGroundBlend>,
    /// The base color at the horizon.
    #[serde(rename = "horizon-color")]
    pub horizon_color: Option<SkyHorizonColor>,
    /// How to blend the fog color and the horizon color. Where 0 is using the horizon color only and 1 is using the fog color only.
    ///
    /// Range: 0..=1
    #[serde(rename = "horizon-fog-blend")]
    pub horizon_fog_blend: Option<SkyHorizonFogBlend>,
    /// The base color for the sky.
    #[serde(rename = "sky-color")]
    pub sky_color: Option<SkySkyColor>,
    /// How to blend the sky color and the horizon color. Where 1 is blending the color at the middle of the sky and 0 is not blending at all and using the sky color only.
    ///
    /// Range: 0..=1
    #[serde(rename = "sky-horizon-blend")]
    pub sky_horizon_blend: Option<SkySkyHorizonBlend>,
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SkyAtmosphereBlendExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// How to blend the atmosphere. Where 1 is visible atmosphere and 0 is hidden. It is best to interpolate this expression when using globe projection.
///
/// Range: 0..=1
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SkyAtmosphereBlend {
    Expr(SkyAtmosphereBlendExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for SkyAtmosphereBlend {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_f64(0.8)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SkyFogColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

/// The base color for the fog. Requires 3D terrain.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SkyFogColor {
    Expr(SkyFogColorExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl Default for SkyFogColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("#ffffff"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SkyFogGroundBlendExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// How to blend the fog over the 3D terrain. Where 0 is the map center and 1 is the horizon.
///
/// Range: 0..=1
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SkyFogGroundBlend {
    Expr(SkyFogGroundBlendExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for SkyFogGroundBlend {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_f64(0.5)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SkyHorizonColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

/// The base color at the horizon.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SkyHorizonColor {
    Expr(SkyHorizonColorExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl Default for SkyHorizonColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("#ffffff"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SkyHorizonFogBlendExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// How to blend the fog color and the horizon color. Where 0 is using the horizon color only and 1 is using the fog color only.
///
/// Range: 0..=1
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SkyHorizonFogBlend {
    Expr(SkyHorizonFogBlendExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for SkyHorizonFogBlend {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_f64(0.8)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SkySkyColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

/// The base color for the sky.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SkySkyColor {
    Expr(SkySkyColorExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl Default for SkySkyColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("#88C6FC"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SkySkyHorizonBlendExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// How to blend the sky color and the horizon color. Where 1 is blending the color at the middle of the sky and 0 is not blending at all and using the sky color only.
///
/// Range: 0..=1
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SkySkyHorizonBlend {
    Expr(SkySkyHorizonBlendExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for SkySkyHorizonBlend {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_f64(0.8)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct Terrain {
    /// The exaggeration of the terrain - how high it will look.
    ///
    /// Range: 0..
    pub exaggeration: Option<TerrainExaggeration>,
    /// The source for the terrain data.
    pub source: TerrainSource,
}

/// The exaggeration of the terrain - how high it will look.
///
/// Range: 0..
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct TerrainExaggeration(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for TerrainExaggeration {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(1.0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The source for the terrain data.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct TerrainSource(std::string::String);

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct Transition {
    /// Length of time before a transition begins.
    ///
    /// Range: 0..
    pub delay: Option<TransitionDelay>,
    /// Time allotted for transitions to complete.
    ///
    /// Range: 0..
    pub duration: Option<TransitionDuration>,
}

/// Length of time before a transition begins.
///
/// Range: 0..
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct TransitionDelay(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

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
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct TransitionDuration(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for TransitionDuration {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(300)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Either of the below variants
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum StringLiteralOrNumberLiteralOrArrayOfStringLiteralOrArrayOfNumberLiteralAsUnion {
    StringLiteral(StringLiteral),
    NumberLiteral(NumberLiteral),
    ArrayOfStringLiteral(ArrayOfStringLiteral),
    ArrayOfNumberLiteral(ArrayOfNumberLiteral),
}

/// Either of the below variants
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum NumberLiteralOrNumberAsUnion {
    NumberLiteral(NumberLiteral),
    Number(Box<Number>),
}

/// "Any"
#[derive(serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Any {
    /// Gets the value of a cluster property accumulated so far. Can only be used in the `clusterProperties` option of a clustered GeoJSON source.
    Accumulated,
    /// Retrieves an item from an array.
    At(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Selects the first output whose corresponding test condition evaluates to true, or the fallback value otherwise.
    ///
    ///  - [Create a hover effect](https://maplibre.org/maplibre-gl-js/docs/examples/create-a-hover-effect/)
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    Case((Vec<(Boolean, serde_json::Value)>, serde_json::Value)),
    /// Evaluates each expression in turn until the first non-null value is obtained, and returns that value.
    ///
    ///  - [Use a fallback image](https://maplibre.org/maplibre-gl-js/docs/examples/use-a-fallback-image/)
    Coalesce(Vec<serde_json::Value>),
    /// Retrieves a property value from the current feature's state. Returns null if the requested property is not present on the feature's state. A feature's state is not part of the GeoJSON or vector tile data, and must be set programmatically on each feature. When `source.promoteId` is not provided, features are identified by their `id` attribute, which must be an integer or a string that can be cast to an integer. When `source.promoteId` is provided, features are identified by their `promoteId` property, which may be a number, string, or any primitive data type. Note that ["feature-state"] can only be used with paint properties that support data-driven styling.
    ///
    ///  - [Create a hover effect](https://maplibre.org/maplibre-gl-js/docs/examples/create-a-hover-effect/)
    FeatureState(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Retrieves a property value from the current feature's properties, or from another object if a second argument is provided. Returns null if the requested property is missing.
    ///
    ///  - [Change the case of labels](https://maplibre.org/maplibre-gl-js/docs/examples/change-case-of-labels/)
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    ///
    ///  - [Extrude polygons for 3D indoor mapping](https://maplibre.org/maplibre-gl-js/docs/examples/extrude-polygons-for-3d-indoor-mapping/)
    Get(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_option_json_value))]
         Option<serde_json::Value>,
    ),
    /// Retrieves a property value from global state that can be set with platform-specific APIs. Defaults can be provided using the [`state`](https://maplibre.org/maplibre-style-spec/root/#state) root property. Returns `null` if no value nor default value is set for the retrieved property.
    GlobalState(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Gets the feature's id, if it has one.
    Id,
    /// Binds expressions to named variables, which can then be referenced in the result expression using `["var", "variable_name"]`.
    ///
    ///  - [Visualize population density](https://maplibre.org/maplibre-gl-js/docs/examples/visualize-population-density/)
    Let((Vec<(StringLiteral, serde_json::Value)>, serde_json::Value)),
    /// Selects the output whose label value matches the input value, or the fallback value if no match is found. The input can be any expression (e.g. `["get", "building_type"]`). Each label must be either:
    ///
    ///  - a single literal value; or
    ///
    ///  - an array of literal values, whose values must be all strings or all numbers (e.g. `[100, 101]` or `["c", "b"]`). The input matches if any of the values in the array matches, similar to the `"in"` operator.
    ///
    /// Each label must be unique. If the input type does not match the type of the labels, the result will be the fallback value.
    Match(
        (
            serde_json::Value,
            Vec<(
                StringLiteralOrNumberLiteralOrArrayOfStringLiteralOrArrayOfNumberLiteralAsUnion,
                serde_json::Value,
            )>,
            serde_json::Value,
        ),
    ),
    /// Produces discrete, stepped results by evaluating a piecewise-constant function defined by pairs of input and output values ("stops"). The `input` may be any numeric expression (e.g., `["get", "population"]`). Stop inputs must be numeric literals in strictly ascending order.
    ///
    /// Returns the output value of the stop just less than the input, or the first output if the input is less than the first stop.
    ///
    ///  - [Create and style clusters](https://maplibre.org/maplibre-gl-js/docs/examples/create-and-style-clusters/)
    Step(
        (
            NumberLiteralOrNumberAsUnion,
            serde_json::Value,
            Vec<(NumberLiteral, serde_json::Value)>,
        ),
    ),
    /// References variable bound using `let`.
    ///
    ///  - [Visualize population density](https://maplibre.org/maplibre-gl-js/docs/examples/visualize-population-density/)
    Var(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl<'de> serde::Deserialize<'de> for Any {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(AnyVisitor)
    }
}

/// Visitor for deserializing the syntax enum [`Any`]
struct AnyVisitor;

impl<'de> serde::de::Visitor<'de> for AnyVisitor {
    type Value = Any;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an Any expression (example: [\"accumulated\"])")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        /// Reads the next element from the sequence or reports a missing field error.
        #[allow(dead_code)]
        fn visit_seq_field<'de, A, T>(seq: &mut A, name: &'static str) -> Result<T, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
            T: serde::Deserialize<'de>,
        {
            seq.next_element()?
                .ok_or_else(|| serde::de::Error::missing_field(name))
        }

        // First element: operator string
        let op: std::string::String = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::custom("missing operator"))?;
        match op.as_str() {
            "accumulated" => Ok(Any::Accumulated),
            "at" => {
                let index = visit_seq_field(&mut seq, "index")?;
                let array = visit_seq_field(&mut seq, "array")?;
                Ok(Any::At(index, array))
            }
            "case" => {
                let mut inputs = Vec::new();
                while let Some(condition_i) = seq.next_element()? {
                    let output_i = seq.next_element()?.ok_or_else(|| {
                        serde::de::Error::custom("expected output_i in Any::Case")
                    })?;
                    let element = (condition_i, output_i);
                    inputs.push(element);
                }
                if inputs.is_empty() {
                    return Err(serde::de::Error::custom(
                        "Any::Case requires at least one argument",
                    ));
                }
                let fallback = visit_seq_field(&mut seq, "fallback")?;
                Ok(Any::Case((inputs, fallback)))
            }
            "coalesce" => {
                let mut inputs = Vec::new();
                while let Some(element) = seq.next_element()? {
                    inputs.push(element);
                }
                if inputs.is_empty() {
                    return Err(serde::de::Error::custom(
                        "Any::Coalesce requires at least one argument",
                    ));
                }
                Ok(Any::Coalesce(inputs))
            }
            "feature-state" => {
                let property_name = visit_seq_field(&mut seq, "property_name")?;
                Ok(Any::FeatureState(property_name))
            }
            "get" => {
                let property_name = visit_seq_field(&mut seq, "property_name")?;
                let object = seq.next_element()?;
                Ok(Any::Get(property_name, object))
            }
            "global-state" => {
                let property_name = visit_seq_field(&mut seq, "property_name")?;
                Ok(Any::GlobalState(property_name))
            }
            "id" => Ok(Any::Id),
            "let" => {
                let mut inputs = Vec::new();
                while let Some(var_name_i) = seq.next_element()? {
                    let var_value_i = seq.next_element()?.ok_or_else(|| {
                        serde::de::Error::custom("expected var_value_i in Any::Let")
                    })?;
                    let element = (var_name_i, var_value_i);
                    inputs.push(element);
                }
                if inputs.is_empty() {
                    return Err(serde::de::Error::custom(
                        "Any::Let requires at least one argument",
                    ));
                }
                let expression = visit_seq_field(&mut seq, "expression")?;
                Ok(Any::Let((inputs, expression)))
            }
            "match" => {
                let mut rest: Vec<serde_json::Value> = Vec::new();
                while let Some(v) = seq.next_element()? {
                    rest.push(v);
                }
                if rest.len() < 2 {
                    return Err(serde::de::Error::custom("Any::Match: too few arguments"));
                }
                if rest.len() % 2 != 0 {
                    return Err(serde::de::Error::custom(
                        "Any::Match: expected an even number of arguments after operator (input + label/output pairs + fallback)",
                    ));
                }
                let fallback_v = rest.pop().unwrap();
                let input = rest.remove(0);
                let mut pairs = Vec::new();
                for chunk in rest.chunks_exact(2) {
                    let label_i: StringLiteralOrNumberLiteralOrArrayOfStringLiteralOrArrayOfNumberLiteralAsUnion = serde_json::from_value(chunk[0].clone()).map_err(serde::de::Error::custom)?;
                    let output_i: serde_json::Value = serde_json::from_value(chunk[1].clone())
                        .map_err(serde::de::Error::custom)?;
                    pairs.push((label_i, output_i));
                }
                if pairs.is_empty() {
                    return Err(serde::de::Error::custom(
                        "Any::Match: missing label/output pairs",
                    ));
                }
                let fallback: serde_json::Value =
                    serde_json::from_value(fallback_v).map_err(serde::de::Error::custom)?;
                Ok(Any::Match((input, pairs, fallback)))
            }
            "step" => {
                let input: NumberLiteralOrNumberAsUnion = visit_seq_field(&mut seq, "input")?;
                let output_0: serde_json::Value = visit_seq_field(&mut seq, "output_0")?;
                let mut stops = Vec::new();
                while let Some(stop_input_i) = seq.next_element::<NumberLiteral>()? {
                    let stop_output_i: serde_json::Value =
                        seq.next_element()?.ok_or_else(|| {
                            serde::de::Error::custom("expected stop_output_i in Any::Step")
                        })?;
                    stops.push((stop_input_i, stop_output_i));
                }
                if stops.is_empty() {
                    return Err(serde::de::Error::custom(
                        "Any::Step requires at least one stop pair",
                    ));
                }
                Ok(Any::Step((input, output_0, stops)))
            }
            "var" => {
                let var_name = visit_seq_field(&mut seq, "var_name")?;
                Ok(Any::Var(var_name))
            }
            _ => Err(serde::de::Error::unknown_variant(
                &op,
                &[
                    "accumulated",
                    "at",
                    "case",
                    "coalesce",
                    "feature-state",
                    "get",
                    "global-state",
                    "id",
                    "let",
                    "match",
                    "step",
                    "var",
                ],
            )),
        }
    }
}

/// "Array"
#[derive(serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Array {
    /// Asserts that the input is an array (optionally with a specific item type and length). If, when the input expression is evaluated, it is not of the asserted type or length, then this assertion will cause the whole expression to be aborted.
    Array(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Provides a literal array or object value.
    ///
    ///  - [Display and style rich text labels](https://maplibre.org/maplibre-gl-js/docs/examples/display-and-style-rich-text-labels/)
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns a subarray from an array or a substring from a string from a specified start index, or between a start index and an end index if set. The return value is inclusive of the start index but not of the end index. In a string, a UTF-16 surrogate pair counts as a single position.
    Slice(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_option_json_value))]
         Option<serde_json::Value>,
    ),
    /// Returns a four-element array containing the input color's red, green, blue, and alpha components, in that order.
    ToRgba(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl<'de> serde::Deserialize<'de> for Array {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ArrayVisitor)
    }
}

/// Visitor for deserializing the syntax enum [`Array`]
struct ArrayVisitor;

impl<'de> serde::de::Visitor<'de> for ArrayVisitor {
    type Value = Array;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an Array expression (example: [\"literal\",[\"DIN Offc Pro Italic\",\"Arial Unicode MS Regular\"]])")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        /// Reads the next element from the sequence or reports a missing field error.
        #[allow(dead_code)]
        fn visit_seq_field<'de, A, T>(seq: &mut A, name: &'static str) -> Result<T, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
            T: serde::Deserialize<'de>,
        {
            seq.next_element()?
                .ok_or_else(|| serde::de::Error::missing_field(name))
        }

        // First element: operator string
        let op: std::string::String = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::custom("missing operator"))?;
        match op.as_str() {
            "array" => {
                let value = visit_seq_field(&mut seq, "value")?;
                Ok(Array::Array(value))
            }
            "literal" => {
                let json_array = visit_seq_field(&mut seq, "json_array")?;
                Ok(Array::Literal(json_array))
            }
            "slice" => {
                let array = visit_seq_field(&mut seq, "array")?;
                let start_index = visit_seq_field(&mut seq, "start_index")?;
                let end_index = seq.next_element()?;
                Ok(Array::Slice(array, start_index, end_index))
            }
            "to-rgba" => {
                let color = visit_seq_field(&mut seq, "color")?;
                Ok(Array::ToRgba(color))
            }
            _ => Err(serde::de::Error::unknown_variant(
                &op,
                &["array", "literal", "slice", "to-rgba"],
            )),
        }
    }
}

/// Either of the below variants
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum StringOrNumberOrBooleanAsUnion {
    String(String),
    Number(Number),
    Boolean(Boolean),
}

/// "ArrayLessTypeLengthGreater"
#[derive(serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum ArrayLessTypeLengthGreater {
    /// Asserts that the input is an array (optionally with a specific item type and length). If, when the input expression is evaluated, it is not of the asserted type or length, then this assertion will cause the whole expression to be aborted.
    Array(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl<'de> serde::Deserialize<'de> for ArrayLessTypeLengthGreater {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ArrayLessTypeLengthGreaterVisitor)
    }
}

/// Visitor for deserializing the syntax enum [`ArrayLessTypeLengthGreater`]
struct ArrayLessTypeLengthGreaterVisitor;

impl<'de> serde::de::Visitor<'de> for ArrayLessTypeLengthGreaterVisitor {
    type Value = ArrayLessTypeLengthGreater;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an ArrayLessTypeLengthGreater expression (example: [\"array\",\"string\",3,[\"literal\",[\"a\",\"b\",\"c\"]]])")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        /// Reads the next element from the sequence or reports a missing field error.
        #[allow(dead_code)]
        fn visit_seq_field<'de, A, T>(seq: &mut A, name: &'static str) -> Result<T, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
            T: serde::Deserialize<'de>,
        {
            seq.next_element()?
                .ok_or_else(|| serde::de::Error::missing_field(name))
        }

        // First element: operator string
        let op: std::string::String = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::custom("missing operator"))?;
        match op.as_str() {
            "array" => {
                let r#type = visit_seq_field(&mut seq, "type")?;
                let length = visit_seq_field(&mut seq, "length")?;
                let value = visit_seq_field(&mut seq, "value")?;
                Ok(ArrayLessTypeLengthGreater::Array(r#type, length, value))
            }
            _ => Err(serde::de::Error::unknown_variant(&op, &["array"])),
        }
    }
}

/// "ArrayOfType"
#[derive(serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum ArrayOfType {
    /// Asserts that the input is an array (optionally with a specific item type and length). If, when the input expression is evaluated, it is not of the asserted type or length, then this assertion will cause the whole expression to be aborted.
    Array(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl<'de> serde::Deserialize<'de> for ArrayOfType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ArrayOfTypeVisitor)
    }
}

/// Visitor for deserializing the syntax enum [`ArrayOfType`]
struct ArrayOfTypeVisitor;

impl<'de> serde::de::Visitor<'de> for ArrayOfTypeVisitor {
    type Value = ArrayOfType;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an ArrayOfType expression (example: [\"array\",\"string\",3,[\"literal\",[\"a\",\"b\",\"c\"]]])")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        /// Reads the next element from the sequence or reports a missing field error.
        #[allow(dead_code)]
        fn visit_seq_field<'de, A, T>(seq: &mut A, name: &'static str) -> Result<T, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
            T: serde::Deserialize<'de>,
        {
            seq.next_element()?
                .ok_or_else(|| serde::de::Error::missing_field(name))
        }

        // First element: operator string
        let op: std::string::String = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::custom("missing operator"))?;
        match op.as_str() {
            "array" => {
                let r#type = visit_seq_field(&mut seq, "type")?;
                let value = visit_seq_field(&mut seq, "value")?;
                Ok(ArrayOfType::Array(r#type, value))
            }
            _ => Err(serde::de::Error::unknown_variant(&op, &["array"])),
        }
    }
}

/// "Boolean"
#[derive(serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Boolean {
    /// Logical negation. Returns `true` if the input is `false`, and `false` if the input is `true`.
    ///
    ///  - [Create and style clusters](https://maplibre.org/maplibre-gl-js/docs/examples/create-and-style-clusters/)
    Not(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns `true` if the input values are not equal, `false` otherwise. The comparison is strictly typed: values of different runtime types are always considered unequal. Cases where the types are known to be different at parse time are considered invalid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    NotEqual(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_option_json_value))]
         Option<serde_json::Value>,
    ),
    /// Returns `true` if the first input is strictly less than the second, `false` otherwise. The arguments are required to be either both strings or both numbers; if during evaluation they are not, expression evaluation produces an error. Cases where this constraint is known not to hold at parse time are considered in valid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    Less(LessOptions),
    /// Returns `true` if the first input is less than or equal to the second, `false` otherwise. The arguments are required to be either both strings or both numbers; if during evaluation they are not, expression evaluation produces an error. Cases where this constraint is known not to hold at parse time are considered in valid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    LessEqual(LessEqualOptions),
    /// Returns `true` if the input values are equal, `false` otherwise. The comparison is strictly typed: values of different runtime types are always considered unequal. Cases where the types are known to be different at parse time are considered invalid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    ///
    ///  - [Add multiple geometries from one GeoJSON source](https://maplibre.org/maplibre-gl-js/docs/examples/multiple-geometries/)
    ///
    ///  - [Create a time slider](https://maplibre.org/maplibre-gl-js/docs/examples/timeline-animation/)
    ///
    ///  - [Display buildings in 3D](https://maplibre.org/maplibre-gl-js/docs/examples/display-buildings-in-3d/)
    ///
    ///  - [Filter symbols by toggling a list](https://maplibre.org/maplibre-gl-js/docs/examples/filter-symbols-by-toggling-a-list/)
    EqualEqual(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_option_json_value))]
         Option<serde_json::Value>,
    ),
    /// Returns `true` if the first input is strictly greater than the second, `false` otherwise. The arguments are required to be either both strings or both numbers; if during evaluation they are not, expression evaluation produces an error. Cases where this constraint is known not to hold at parse time are considered in valid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    Greater(GreaterOptions),
    /// Returns `true` if the first input is greater than or equal to the second, `false` otherwise. The arguments are required to be either both strings or both numbers; if during evaluation they are not, expression evaluation produces an error. Cases where this constraint is known not to hold at parse time are considered in valid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    GreaterEqual(GreaterEqualOptions),
    /// Returns `true` if all the inputs are `true`, `false` otherwise. The inputs are evaluated in order, and evaluation is short-circuiting: once an input expression evaluates to `false`, the result is `false` and no further input expressions are evaluated.
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    All(Vec<Box<Boolean>>),
    /// Returns `true` if any of the inputs are `true`, `false` otherwise. The inputs are evaluated in order, and evaluation is short-circuiting: once an input expression evaluates to `true`, the result is `true` and no further input expressions are evaluated.
    Any(Vec<Box<Boolean>>),
    /// Asserts that the input value is a boolean. If multiple values are provided, each one is evaluated in order until a boolean is obtained. If none of the inputs are booleans, the expression is an error.
    ///
    ///  - [Create a hover effect](https://maplibre.org/maplibre-gl-js/docs/examples/create-a-hover-effect/)
    Boolean(Vec<serde_json::Value>),
    /// Tests for the presence of a property value in the current feature's properties, or from another object if a second argument is provided.
    ///
    ///  - [Create and style clusters](https://maplibre.org/maplibre-gl-js/docs/examples/create-and-style-clusters/)
    Has(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_option_json_value))]
         Option<serde_json::Value>,
    ),
    /// Determines whether an item exists in an array or a substring exists in a string.
    ///
    ///  - [Measure distances](https://maplibre.org/maplibre-gl-js/docs/examples/measure-distances/)
    In(InOptions),
    /// Returns `true` if the input string is expected to render legibly. Returns `false` if the input string contains sections that cannot be rendered without potential loss of meaning (e.g. Indic scripts that require complex text shaping, or right-to-left scripts if the `mapbox-gl-rtl-text` plugin is not in use in MapLibre GL JS).
    IsSupportedScript(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Converts the input value to a boolean. The result is `false` when the input is an empty string, 0, `false`, `null`, or `NaN`; otherwise it is `true`.
    ToBoolean(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns `true` if the evaluated feature is fully contained inside a boundary of the input geometry, `false` otherwise. The input value can be a valid GeoJSON of type `Polygon`, `MultiPolygon`, `Feature`, or `FeatureCollection`. Supported features for evaluation:
    ///
    /// - `Point`: Returns `false` if a point is on the boundary or falls outside the boundary.
    ///
    /// - `LineString`: Returns `false` if any part of a line falls outside the boundary, the line intersects the boundary, or a line's endpoint is on the boundary.
    Within(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

/// Options for deserializing the syntax enum variant [`Boolean::Less`]
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LessOptions {
    Args(
        (
            serde_json::Value,
            serde_json::Value,
            Option<serde_json::Value>,
        ),
    ),
}

/// Options for deserializing the syntax enum variant [`Boolean::LessEqual`]
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LessEqualOptions {
    Args(
        (
            serde_json::Value,
            serde_json::Value,
            Option<serde_json::Value>,
        ),
    ),
}

/// Options for deserializing the syntax enum variant [`Boolean::Greater`]
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum GreaterOptions {
    Args(
        (
            serde_json::Value,
            serde_json::Value,
            Option<serde_json::Value>,
        ),
    ),
}

/// Options for deserializing the syntax enum variant [`Boolean::GreaterEqual`]
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum GreaterEqualOptions {
    Args(
        (
            serde_json::Value,
            serde_json::Value,
            Option<serde_json::Value>,
        ),
    ),
}

/// Options for deserializing the syntax enum variant [`Boolean::In`]
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum InOptions {
    Item(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    Substring(String, String),
}

impl<'de> serde::Deserialize<'de> for Boolean {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(BooleanVisitor)
    }
}

/// Visitor for deserializing the syntax enum [`Boolean`]
struct BooleanVisitor;

impl<'de> serde::de::Visitor<'de> for BooleanVisitor {
    type Value = Boolean;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an Boolean expression (example: [\"!\",[\"has\",\"point_count\"]])")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        /// Reads the next element from the sequence or reports a missing field error.
        #[allow(dead_code)]
        fn visit_seq_field<'de, A, T>(seq: &mut A, name: &'static str) -> Result<T, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
            T: serde::Deserialize<'de>,
        {
            seq.next_element()?
                .ok_or_else(|| serde::de::Error::missing_field(name))
        }

        // First element: operator string
        let op: std::string::String = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::custom("missing operator"))?;
        match op.as_str() {
            "!" => {
                let input = visit_seq_field(&mut seq, "input")?;
                Ok(Boolean::Not(input))
            }
            "!=" => {
                let input_1 = visit_seq_field(&mut seq, "input_1")?;
                let input_2 = visit_seq_field(&mut seq, "input_2")?;
                let collator = seq.next_element()?;
                Ok(Boolean::NotEqual(input_1, input_2, collator))
            }
            "<" => {
                // Delegate the remainder of the sequence to LessOptions deserialization
                let remainder_of_sequence = serde::de::value::SeqAccessDeserializer::new(seq);
                let options =
                    <LessOptions as serde::Deserialize>::deserialize(remainder_of_sequence)?;
                Ok(Boolean::Less(options))
            }
            "<=" => {
                // Delegate the remainder of the sequence to LessEqualOptions deserialization
                let remainder_of_sequence = serde::de::value::SeqAccessDeserializer::new(seq);
                let options =
                    <LessEqualOptions as serde::Deserialize>::deserialize(remainder_of_sequence)?;
                Ok(Boolean::LessEqual(options))
            }
            "==" => {
                let input_1 = visit_seq_field(&mut seq, "input_1")?;
                let input_2 = visit_seq_field(&mut seq, "input_2")?;
                let collator = seq.next_element()?;
                Ok(Boolean::EqualEqual(input_1, input_2, collator))
            }
            ">" => {
                // Delegate the remainder of the sequence to GreaterOptions deserialization
                let remainder_of_sequence = serde::de::value::SeqAccessDeserializer::new(seq);
                let options =
                    <GreaterOptions as serde::Deserialize>::deserialize(remainder_of_sequence)?;
                Ok(Boolean::Greater(options))
            }
            ">=" => {
                // Delegate the remainder of the sequence to GreaterEqualOptions deserialization
                let remainder_of_sequence = serde::de::value::SeqAccessDeserializer::new(seq);
                let options = <GreaterEqualOptions as serde::Deserialize>::deserialize(
                    remainder_of_sequence,
                )?;
                Ok(Boolean::GreaterEqual(options))
            }
            "all" => {
                let mut inputs = Vec::new();
                while let Some(element) = seq.next_element()? {
                    inputs.push(element);
                }
                if inputs.is_empty() {
                    return Err(serde::de::Error::custom(
                        "Boolean::All requires at least one argument",
                    ));
                }
                Ok(Boolean::All(inputs))
            }
            "any" => {
                let mut inputs = Vec::new();
                while let Some(element) = seq.next_element()? {
                    inputs.push(element);
                }
                if inputs.is_empty() {
                    return Err(serde::de::Error::custom(
                        "Boolean::Any requires at least one argument",
                    ));
                }
                Ok(Boolean::Any(inputs))
            }
            "boolean" => {
                let mut inputs = Vec::new();
                while let Some(element) = seq.next_element()? {
                    inputs.push(element);
                }
                if inputs.is_empty() {
                    return Err(serde::de::Error::custom(
                        "Boolean::Boolean requires at least one argument",
                    ));
                }
                Ok(Boolean::Boolean(inputs))
            }
            "has" => {
                let property_name = visit_seq_field(&mut seq, "property_name")?;
                let object = seq.next_element()?;
                Ok(Boolean::Has(property_name, object))
            }
            "in" => {
                // Delegate the remainder of the sequence to InOptions deserialization
                let remainder_of_sequence = serde::de::value::SeqAccessDeserializer::new(seq);
                let options =
                    <InOptions as serde::Deserialize>::deserialize(remainder_of_sequence)?;
                Ok(Boolean::In(options))
            }
            "is-supported-script" => {
                let input = visit_seq_field(&mut seq, "input")?;
                Ok(Boolean::IsSupportedScript(input))
            }
            "to-boolean" => {
                let value = visit_seq_field(&mut seq, "value")?;
                Ok(Boolean::ToBoolean(value))
            }
            "within" => {
                let geojson = visit_seq_field(&mut seq, "geojson")?;
                Ok(Boolean::Within(geojson))
            }
            _ => Err(serde::de::Error::unknown_variant(
                &op,
                &[
                    "!",
                    "!=",
                    "<",
                    "<=",
                    "==",
                    ">",
                    ">=",
                    "all",
                    "any",
                    "boolean",
                    "has",
                    "in",
                    "is-supported-script",
                    "to-boolean",
                    "within",
                ],
            )),
        }
    }
}

/// "Collator"
#[derive(serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Collator {
    /// Returns a `collator` for use in locale-dependent comparison operations. Use `resolved-locale` to test the results of locale fallback behavior.
    Collator(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl<'de> serde::Deserialize<'de> for Collator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(CollatorVisitor)
    }
}

/// Visitor for deserializing the syntax enum [`Collator`]
struct CollatorVisitor;

impl<'de> serde::de::Visitor<'de> for CollatorVisitor {
    type Value = Collator;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an Collator expression (example: [\"collator\",{\"case-sensitive\":true,\"diacritic-sensitive\":true,\"locale\":\"fr\"}])")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        /// Reads the next element from the sequence or reports a missing field error.
        #[allow(dead_code)]
        fn visit_seq_field<'de, A, T>(seq: &mut A, name: &'static str) -> Result<T, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
            T: serde::Deserialize<'de>,
        {
            seq.next_element()?
                .ok_or_else(|| serde::de::Error::missing_field(name))
        }

        // First element: operator string
        let op: std::string::String = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::custom("missing operator"))?;
        match op.as_str() {
            "collator" => {
                let options = visit_seq_field(&mut seq, "options")?;
                Ok(Collator::Collator(options))
            }
            _ => Err(serde::de::Error::unknown_variant(&op, &["collator"])),
        }
    }
}

/// "Color"
#[derive(serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Color {
    /// Creates a color value from red, green, and blue components, which must range between 0 and 255, and an alpha component of 1. If any component is out of range, the expression is an error.
    Rgb(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Creates a color value from red, green, blue components, which must range between 0 and 255, and an alpha component which must range between zero and one. If any component is out of range, the expression is an error.
    Rgba(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Converts the input value to a color. If multiple values are provided, each one is evaluated in order until the first successful conversion is obtained. If none of the inputs can be converted, the expression is an error.
    ///
    ///  - [Visualize population density](https://maplibre.org/maplibre-gl-js/docs/examples/visualize-population-density/)
    ToColor(Vec<serde_json::Value>),
}

impl<'de> serde::Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ColorVisitor)
    }
}

/// Visitor for deserializing the syntax enum [`Color`]
struct ColorVisitor;

impl<'de> serde::de::Visitor<'de> for ColorVisitor {
    type Value = Color;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an Color expression (example: [\"rgb\",255,0,0])")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        /// Reads the next element from the sequence or reports a missing field error.
        #[allow(dead_code)]
        fn visit_seq_field<'de, A, T>(seq: &mut A, name: &'static str) -> Result<T, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
            T: serde::Deserialize<'de>,
        {
            seq.next_element()?
                .ok_or_else(|| serde::de::Error::missing_field(name))
        }

        // First element: operator string
        let op: std::string::String = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::custom("missing operator"))?;
        match op.as_str() {
            "rgb" => {
                let red = visit_seq_field(&mut seq, "red")?;
                let green = visit_seq_field(&mut seq, "green")?;
                let blue = visit_seq_field(&mut seq, "blue")?;
                Ok(Color::Rgb(red, green, blue))
            }
            "rgba" => {
                let red = visit_seq_field(&mut seq, "red")?;
                let green = visit_seq_field(&mut seq, "green")?;
                let blue = visit_seq_field(&mut seq, "blue")?;
                let alpha = visit_seq_field(&mut seq, "alpha")?;
                Ok(Color::Rgba(red, green, blue, alpha))
            }
            "to-color" => {
                let mut inputs = Vec::new();
                while let Some(element) = seq.next_element()? {
                    inputs.push(element);
                }
                if inputs.is_empty() {
                    return Err(serde::de::Error::custom(
                        "Color::ToColor requires at least one argument",
                    ));
                }
                Ok(Color::ToColor(inputs))
            }
            _ => Err(serde::de::Error::unknown_variant(
                &op,
                &["rgb", "rgba", "to-color"],
            )),
        }
    }
}

/// Either of the below variants
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum StringLiteralOrColorOrArrayOfColorAsUnion {
    StringLiteral(StringLiteral),
    Color(Color),
    ArrayOfColor(ColorOrArrayOfColor),
}

/// "ColorOrArrayOfColor"
#[derive(serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum ColorOrArrayOfColor {
    /// Produces continuous, smooth results by interpolating between pairs of input and output values ("stops"). Works like `interpolate`, but the output type must be `color` or `array<color>`, and the interpolation is performed in the Hue-Chroma-Luminance color space.
    InterpolateHcl(
        (
            Interpolation,
            NumberLiteralOrNumberAsUnion,
            Vec<(NumberLiteral, StringLiteralOrColorOrArrayOfColorAsUnion)>,
        ),
    ),
    /// Produces continuous, smooth results by interpolating between pairs of input and output values ("stops"). Works like `interpolate`, but the output type must be `color` or `array<color>`, and the interpolation is performed in the CIELAB color space.
    InterpolateLab(
        (
            Interpolation,
            NumberLiteralOrNumberAsUnion,
            Vec<(NumberLiteral, StringLiteralOrColorOrArrayOfColorAsUnion)>,
        ),
    ),
}

impl<'de> serde::Deserialize<'de> for ColorOrArrayOfColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ColorOrArrayOfColorVisitor)
    }
}

/// Visitor for deserializing the syntax enum [`ColorOrArrayOfColor`]
struct ColorOrArrayOfColorVisitor;

impl<'de> serde::de::Visitor<'de> for ColorOrArrayOfColorVisitor {
    type Value = ColorOrArrayOfColor;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an ColorOrArrayOfColor expression (example: [\"interpolate-hcl\",[\"linear\"],[\"zoom\"],15,\"#f00\",15.05,\"#00f\"])")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        /// Reads the next element from the sequence or reports a missing field error.
        #[allow(dead_code)]
        fn visit_seq_field<'de, A, T>(seq: &mut A, name: &'static str) -> Result<T, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
            T: serde::Deserialize<'de>,
        {
            seq.next_element()?
                .ok_or_else(|| serde::de::Error::missing_field(name))
        }

        // First element: operator string
        let op: std::string::String = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::custom("missing operator"))?;
        match op.as_str() {
            "interpolate-hcl" => {
                let interpolation_type: Interpolation =
                    visit_seq_field(&mut seq, "interpolation_type")?;
                let input: NumberLiteralOrNumberAsUnion = visit_seq_field(&mut seq, "input")?;
                let mut stops = Vec::new();
                while let Some(stop_input_i) = seq.next_element::<NumberLiteral>()? {
                    let stop_output_i: StringLiteralOrColorOrArrayOfColorAsUnion =
                        seq.next_element()?.ok_or_else(|| {
                            serde::de::Error::custom(
                                "expected stop_output_i in ColorOrArrayOfColor::InterpolateHcl",
                            )
                        })?;
                    stops.push((stop_input_i, stop_output_i));
                }
                if stops.is_empty() {
                    return Err(serde::de::Error::custom(
                        "ColorOrArrayOfColor::InterpolateHcl requires at least one stop pair",
                    ));
                }
                Ok(ColorOrArrayOfColor::InterpolateHcl((
                    interpolation_type,
                    input,
                    stops,
                )))
            }
            "interpolate-lab" => {
                let interpolation_type: Interpolation =
                    visit_seq_field(&mut seq, "interpolation_type")?;
                let input: NumberLiteralOrNumberAsUnion = visit_seq_field(&mut seq, "input")?;
                let mut stops = Vec::new();
                while let Some(stop_input_i) = seq.next_element::<NumberLiteral>()? {
                    let stop_output_i: StringLiteralOrColorOrArrayOfColorAsUnion =
                        seq.next_element()?.ok_or_else(|| {
                            serde::de::Error::custom(
                                "expected stop_output_i in ColorOrArrayOfColor::InterpolateLab",
                            )
                        })?;
                    stops.push((stop_input_i, stop_output_i));
                }
                if stops.is_empty() {
                    return Err(serde::de::Error::custom(
                        "ColorOrArrayOfColor::InterpolateLab requires at least one stop pair",
                    ));
                }
                Ok(ColorOrArrayOfColor::InterpolateLab((
                    interpolation_type,
                    input,
                    stops,
                )))
            }
            _ => Err(serde::de::Error::unknown_variant(
                &op,
                &["interpolate-hcl", "interpolate-lab"],
            )),
        }
    }
}

/// Either of the below variants
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum StringLiteralOrStringOrImageAsUnion {
    StringLiteral(StringLiteral),
    String(String),
    Image(Image),
}

/// Tuple row for variadic (content, optional style object) pairs.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FormattedFormatVariadicRow(
    StringLiteralOrStringOrImageAsUnion,
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_option_json_map))]
    Option<serde_json::Map<std::string::String, serde_json::Value>>,
);

/// "Formatted"
#[derive(serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Formatted {
    /// Returns a `formatted` string for displaying mixed-format text in the `text-field` property. The input may contain a string literal or expression, including an [`'image'`](#image) expression. Strings may be followed by a style override object.
    ///
    ///  - [Change the case of labels](https://maplibre.org/maplibre-gl-js/docs/examples/change-case-of-labels/)
    ///
    ///  - [Display and style rich text labels](https://maplibre.org/maplibre-gl-js/docs/examples/display-and-style-rich-text-labels/)
    Format(Vec<FormattedFormatVariadicRow>),
}

impl<'de> serde::Deserialize<'de> for Formatted {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(FormattedVisitor)
    }
}

/// Visitor for deserializing the syntax enum [`Formatted`]
struct FormattedVisitor;

impl<'de> serde::de::Visitor<'de> for FormattedVisitor {
    type Value = Formatted;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an Formatted expression (example: [\"format\",[\"upcase\",[\"get\",\"FacilityName\"]],{\"font-scale\":0.8},\"\\n\\n\",{},[\"downcase\",[\"get\",\"Comments\"]],{\"font-scale\":0.6,\"vertical-align\":\"center\"}])")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        /// Reads the next element from the sequence or reports a missing field error.
        #[allow(dead_code)]
        fn visit_seq_field<'de, A, T>(seq: &mut A, name: &'static str) -> Result<T, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
            T: serde::Deserialize<'de>,
        {
            seq.next_element()?
                .ok_or_else(|| serde::de::Error::missing_field(name))
        }

        // First element: operator string
        let op: std::string::String = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::custom("missing operator"))?;
        match op.as_str() {
            "format" => {
                let mut inputs = Vec::new();
                while let Some(input_i) = seq.next_element()? {
                    let style_overrides_i = seq.next_element()?; // optional param
                    let element = FormattedFormatVariadicRow(input_i, style_overrides_i);
                    inputs.push(element);
                }
                if inputs.is_empty() {
                    return Err(serde::de::Error::custom(
                        "Formatted::Format requires at least one argument",
                    ));
                }
                Ok(Formatted::Format(inputs))
            }
            _ => Err(serde::de::Error::unknown_variant(&op, &["format"])),
        }
    }
}

/// "Image"
#[derive(serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Image {
    /// Returns an `image` type for use in `icon-image`, `*-pattern` entries and as a section in the `format` expression. If set, the `image` argument will check that the requested image exists in the style and will return either the resolved image name or `null`, depending on whether or not the image is currently in the style. This validation process is synchronous and requires the image to have been added to the style before requesting it in the `image` argument.
    ///
    ///  - [Use a fallback image](https://maplibre.org/maplibre-gl-js/docs/examples/use-a-fallback-image/)
    Image(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl<'de> serde::Deserialize<'de> for Image {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ImageVisitor)
    }
}

/// Visitor for deserializing the syntax enum [`Image`]
struct ImageVisitor;

impl<'de> serde::de::Visitor<'de> for ImageVisitor {
    type Value = Image;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an Image expression (example: [\"image\",\"marker_15\"])")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        /// Reads the next element from the sequence or reports a missing field error.
        #[allow(dead_code)]
        fn visit_seq_field<'de, A, T>(seq: &mut A, name: &'static str) -> Result<T, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
            T: serde::Deserialize<'de>,
        {
            seq.next_element()?
                .ok_or_else(|| serde::de::Error::missing_field(name))
        }

        // First element: operator string
        let op: std::string::String = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::custom("missing operator"))?;
        match op.as_str() {
            "image" => {
                let image_name = visit_seq_field(&mut seq, "image_name")?;
                Ok(Image::Image(image_name))
            }
            _ => Err(serde::de::Error::unknown_variant(&op, &["image"])),
        }
    }
}

/// Either of the below variants
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum ArrayOrStringAsUnion {
    Array(Array),
    String(String),
}

/// "Number"
#[derive(serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Number {
    /// Returns the remainder after integer division of the first input by the second.
    Percentage(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns the product of the inputs.
    Star(Vec<NumberLiteralOrNumberAsUnion>),
    /// Returns the sum of the inputs.
    Plus(Vec<NumberLiteralOrNumberAsUnion>),
    /// For two inputs, returns the result of subtracting the second input from the first. For a single input, returns the result of subtracting it from 0.
    Minus(MinusOptions),
    /// Returns the result of floating point division of the first input by the second.
    ///
    ///  - [Visualize population density](https://maplibre.org/maplibre-gl-js/docs/examples/visualize-population-density/)
    Slash(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns the result of raising the first input to the power specified by the second.
    Power(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns the absolute value of the input.
    Absolute(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns the arccosine of the input.
    Arccosine(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns the arcsine of the input.
    Asin(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns the arctangent of the input.
    Atan(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns the smallest integer that is greater than or equal to the input.
    Ceil(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns the cosine of the input.
    Cos(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns the shortest distance in meters between the evaluated feature and the input geometry. The input value can be a valid GeoJSON of type `Point`, `MultiPoint`, `LineString`, `MultiLineString`, `Polygon`, `MultiPolygon`, `Feature`, or `FeatureCollection`. Distance values returned may vary in precision due to loss in precision from encoding geometries, particularly below zoom level 13.
    Distance(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns the mathematical constant e.
    E,
    /// Gets the elevation of a pixel (in meters above the vertical datum reference of the `raster-dem` tiles) from a `raster-dem` source. Can only be used in the `color-relief-color` property of a `color-relief` layer.
    Elevation,
    /// Returns the largest integer that is less than or equal to the input.
    Floor(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Gets the kernel density estimation of a pixel in a heatmap layer, which is a relative measure of how many data points are crowded around a particular pixel. Can only be used in the `heatmap-color` property.
    HeatmapDensity,
    /// Returns the first position at which an item can be found in an array or a substring can be found in a string, or `-1` if the input cannot be found. Accepts an optional index from where to begin the search. In a string, a UTF-16 surrogate pair counts as a single position.
    IndexOf(IndexOfOptions),
    /// Gets the length of an array or string. In a string, a UTF-16 surrogate pair counts as a single position.
    Length(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Gets the progress along a gradient line. Can only be used in the `line-gradient` property.
    LineProgress,
    /// Returns the natural logarithm of the input.
    Ln(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns the mathematical constant ln(2).
    Ln2,
    /// Returns the base-ten logarithm of the input.
    Log10(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns the base-two logarithm of the input.
    Log2(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns the maximum value of the inputs.
    Max(Vec<NumberLiteralOrNumberAsUnion>),
    /// Returns the minimum value of the inputs.
    Min(Vec<NumberLiteralOrNumberAsUnion>),
    /// Asserts that the input value is a number. If multiple values are provided, each one is evaluated in order until a number is obtained. If none of the inputs are numbers, the expression is an error.
    Number(Vec<serde_json::Value>),
    /// Returns the mathematical constant pi.
    Pi,
    /// Rounds the input to the nearest integer. Halfway values are rounded away from zero. For example, `["round", -1.5]` evaluates to -2.
    Round(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns the sine of the input.
    Sin(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns the square root of the input.
    Sqrt(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns the tangent of the input.
    Tan(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Converts the input value to a number, if possible. If the input is `null` or `false`, the result is 0. If the input is `true`, the result is 1. If the input is a string, it is converted to a number as specified by the ["ToNumber Applied to the String Type" algorithm](https://tc39.github.io/ecma262/#sec-tonumber-applied-to-the-string-type) of the ECMAScript Language Specification. If multiple values are provided, each one is evaluated in order until the first successful conversion is obtained. If none of the inputs can be converted, the expression is an error.
    ToNumber(Vec<serde_json::Value>),
    /// Gets the current zoom level.  Note that in style layout and paint properties, ["zoom"] may only appear as the input to a top-level "step" or "interpolate" expression.
    Zoom,
}

/// Options for deserializing the syntax enum variant [`Number::Minus`]
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum MinusOptions {
    TwoParams(NumberLiteralOrNumberAsUnion, NumberLiteralOrNumberAsUnion),
    OneParams(NumberLiteralOrNumberAsUnion),
}

/// Options for deserializing the syntax enum variant [`Number::IndexOf`]
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum IndexOfOptions {
    Item(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[serde(default)] Option<serde_json::Value>,
    ),
    Substring(String, String, #[serde(default)] Option<serde_json::Value>),
}

impl<'de> serde::Deserialize<'de> for Number {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(NumberVisitor)
    }
}

/// Visitor for deserializing the syntax enum [`Number`]
struct NumberVisitor;

impl<'de> serde::de::Visitor<'de> for NumberVisitor {
    type Value = Number;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an Number expression (example: [\"%\",10,3])")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        /// Reads the next element from the sequence or reports a missing field error.
        #[allow(dead_code)]
        fn visit_seq_field<'de, A, T>(seq: &mut A, name: &'static str) -> Result<T, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
            T: serde::Deserialize<'de>,
        {
            seq.next_element()?
                .ok_or_else(|| serde::de::Error::missing_field(name))
        }

        // First element: operator string
        let op: std::string::String = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::custom("missing operator"))?;
        match op.as_str() {
            "%" => {
                let input_1 = visit_seq_field(&mut seq, "input_1")?;
                let input_2 = visit_seq_field(&mut seq, "input_2")?;
                Ok(Number::Percentage(input_1, input_2))
            }
            "*" => {
                let mut inputs = Vec::new();
                while let Some(element) = seq.next_element()? {
                    inputs.push(element);
                }
                if inputs.is_empty() {
                    return Err(serde::de::Error::custom(
                        "Number::Star requires at least one argument",
                    ));
                }
                Ok(Number::Star(inputs))
            }
            "+" => {
                let mut inputs = Vec::new();
                while let Some(element) = seq.next_element()? {
                    inputs.push(element);
                }
                if inputs.is_empty() {
                    return Err(serde::de::Error::custom(
                        "Number::Plus requires at least one argument",
                    ));
                }
                Ok(Number::Plus(inputs))
            }
            "-" => {
                let mut rest: Vec<serde_json::Value> = Vec::new();
                while let Some(v) = seq.next_element()? {
                    rest.push(v);
                }
                match rest.len() {
                    2 => Ok(Number::Minus(MinusOptions::TwoParams(
                        serde_json::from_value::<NumberLiteralOrNumberAsUnion>(rest[0].clone())
                            .map_err(serde::de::Error::custom)?,
                        serde_json::from_value::<NumberLiteralOrNumberAsUnion>(rest[1].clone())
                            .map_err(serde::de::Error::custom)?,
                    ))),
                    1 => Ok(Number::Minus(MinusOptions::OneParams(
                        serde_json::from_value::<NumberLiteralOrNumberAsUnion>(rest[0].clone())
                            .map_err(serde::de::Error::custom)?,
                    ))),
                    len => Err(serde::de::Error::custom(format!(
                        "'-': expected 1 or 2 arguments, got {len}"
                    ))),
                }
            }
            "/" => {
                let input_1 = visit_seq_field(&mut seq, "input_1")?;
                let input_2 = visit_seq_field(&mut seq, "input_2")?;
                Ok(Number::Slash(input_1, input_2))
            }
            "^" => {
                let input_1 = visit_seq_field(&mut seq, "input_1")?;
                let input_2 = visit_seq_field(&mut seq, "input_2")?;
                Ok(Number::Power(input_1, input_2))
            }
            "abs" => {
                let input = visit_seq_field(&mut seq, "input")?;
                Ok(Number::Absolute(input))
            }
            "acos" => {
                let input = visit_seq_field(&mut seq, "input")?;
                Ok(Number::Arccosine(input))
            }
            "asin" => {
                let input = visit_seq_field(&mut seq, "input")?;
                Ok(Number::Asin(input))
            }
            "atan" => {
                let input = visit_seq_field(&mut seq, "input")?;
                Ok(Number::Atan(input))
            }
            "ceil" => {
                let input = visit_seq_field(&mut seq, "input")?;
                Ok(Number::Ceil(input))
            }
            "cos" => {
                let input = visit_seq_field(&mut seq, "input")?;
                Ok(Number::Cos(input))
            }
            "distance" => {
                let geojson = visit_seq_field(&mut seq, "geojson")?;
                Ok(Number::Distance(geojson))
            }
            "e" => Ok(Number::E),
            "elevation" => Ok(Number::Elevation),
            "floor" => {
                let input = visit_seq_field(&mut seq, "input")?;
                Ok(Number::Floor(input))
            }
            "heatmap-density" => Ok(Number::HeatmapDensity),
            "index-of" => {
                // Delegate the remainder of the sequence to IndexOfOptions deserialization
                let remainder_of_sequence = serde::de::value::SeqAccessDeserializer::new(seq);
                let options =
                    <IndexOfOptions as serde::Deserialize>::deserialize(remainder_of_sequence)?;
                Ok(Number::IndexOf(options))
            }
            "length" => {
                let array_or_string = visit_seq_field(&mut seq, "array_or_string")?;
                Ok(Number::Length(array_or_string))
            }
            "line-progress" => Ok(Number::LineProgress),
            "ln" => {
                let input = visit_seq_field(&mut seq, "input")?;
                Ok(Number::Ln(input))
            }
            "ln2" => Ok(Number::Ln2),
            "log10" => {
                let input = visit_seq_field(&mut seq, "input")?;
                Ok(Number::Log10(input))
            }
            "log2" => {
                let input = visit_seq_field(&mut seq, "input")?;
                Ok(Number::Log2(input))
            }
            "max" => {
                let mut inputs = Vec::new();
                while let Some(element) = seq.next_element()? {
                    inputs.push(element);
                }
                if inputs.is_empty() {
                    return Err(serde::de::Error::custom(
                        "Number::Max requires at least one argument",
                    ));
                }
                Ok(Number::Max(inputs))
            }
            "min" => {
                let mut inputs = Vec::new();
                while let Some(element) = seq.next_element()? {
                    inputs.push(element);
                }
                if inputs.is_empty() {
                    return Err(serde::de::Error::custom(
                        "Number::Min requires at least one argument",
                    ));
                }
                Ok(Number::Min(inputs))
            }
            "number" => {
                let mut inputs = Vec::new();
                while let Some(element) = seq.next_element()? {
                    inputs.push(element);
                }
                if inputs.is_empty() {
                    return Err(serde::de::Error::custom(
                        "Number::Number requires at least one argument",
                    ));
                }
                Ok(Number::Number(inputs))
            }
            "pi" => Ok(Number::Pi),
            "round" => {
                let input = visit_seq_field(&mut seq, "input")?;
                Ok(Number::Round(input))
            }
            "sin" => {
                let input = visit_seq_field(&mut seq, "input")?;
                Ok(Number::Sin(input))
            }
            "sqrt" => {
                let input = visit_seq_field(&mut seq, "input")?;
                Ok(Number::Sqrt(input))
            }
            "tan" => {
                let input = visit_seq_field(&mut seq, "input")?;
                Ok(Number::Tan(input))
            }
            "to-number" => {
                let mut inputs = Vec::new();
                while let Some(element) = seq.next_element()? {
                    inputs.push(element);
                }
                if inputs.is_empty() {
                    return Err(serde::de::Error::custom(
                        "Number::ToNumber requires at least one argument",
                    ));
                }
                Ok(Number::ToNumber(inputs))
            }
            "zoom" => Ok(Number::Zoom),
            _ => Err(serde::de::Error::unknown_variant(
                &op,
                &[
                    "%",
                    "*",
                    "+",
                    "-",
                    "/",
                    "^",
                    "abs",
                    "acos",
                    "asin",
                    "atan",
                    "ceil",
                    "cos",
                    "distance",
                    "e",
                    "elevation",
                    "floor",
                    "heatmap-density",
                    "index-of",
                    "length",
                    "line-progress",
                    "ln",
                    "ln2",
                    "log10",
                    "log2",
                    "max",
                    "min",
                    "number",
                    "pi",
                    "round",
                    "sin",
                    "sqrt",
                    "tan",
                    "to-number",
                    "zoom",
                ],
            )),
        }
    }
}

/// Either of the below variants
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjectionAsUnion {
    Number(NumberLiteral),
    ArrayOfNumber(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    Color(Color),
    ArrayOfColor(ColorOrArrayOfColor),
    Projection(Box<ProjectionType>),
}

/// "NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection"
#[derive(serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection {
    /// Produces continuous, smooth results by interpolating between pairs of input and output values ("stops"). The `input` may be any numeric expression (e.g., `["get", "population"]`). Stop inputs must be numeric literals in strictly ascending order. The output type must be `number`, `array<number>`, `color`, `array<color>`, or `projection`.
    ///
    ///  - [Animate map camera around a point](https://maplibre.org/maplibre-gl-js/docs/examples/animate-camera-around-point/)
    ///
    ///  - [Change building color based on zoom level](https://maplibre.org/maplibre-gl-js/docs/examples/change-building-color-based-on-zoom-level/)
    ///
    ///  - [Create a heatmap layer](https://maplibre.org/maplibre-gl-js/docs/examples/heatmap-layer/)
    ///
    ///  - [Visualize population density](https://maplibre.org/maplibre-gl-js/docs/examples/visualize-population-density/)
    Interpolate(
        (
            Interpolation,
            NumberLiteralOrNumberAsUnion,
            Vec<(
                NumberLiteral,
                NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjectionAsUnion,
            )>,
        ),
    ),
}

impl<'de> serde::Deserialize<'de> for NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjectionVisitor)
    }
}

/// Visitor for deserializing the syntax enum [`NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection`]
struct NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjectionVisitor;

impl<'de> serde::de::Visitor<'de>
    for NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjectionVisitor
{
    type Value = NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection expression (example: [\"interpolate\",[\"linear\"],[\"zoom\"],15,0,15.05,[\"get\",\"height\"]])")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        /// Reads the next element from the sequence or reports a missing field error.
        #[allow(dead_code)]
        fn visit_seq_field<'de, A, T>(seq: &mut A, name: &'static str) -> Result<T, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
            T: serde::Deserialize<'de>,
        {
            seq.next_element()?
                .ok_or_else(|| serde::de::Error::missing_field(name))
        }

        // First element: operator string
        let op: std::string::String = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::custom("missing operator"))?;
        match op.as_str() {
            "interpolate" => {
                let interpolation_type: Interpolation =
                    visit_seq_field(&mut seq, "interpolation_type")?;
                let input: NumberLiteralOrNumberAsUnion = visit_seq_field(&mut seq, "input")?;
                let mut stops = Vec::new();
                while let Some(stop_input_i) = seq.next_element::<NumberLiteral>()? {
                    let stop_output_i: NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjectionAsUnion = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("expected stop_output_i in NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection::Interpolate"))?;
                    stops.push((stop_input_i, stop_output_i));
                }
                if stops.is_empty() {
                    return Err(serde::de::Error::custom(
                        "NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection::Interpolate requires at least one stop pair",
                    ));
                }
                Ok(
                    NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection::Interpolate((
                        interpolation_type,
                        input,
                        stops,
                    )),
                )
            }
            _ => Err(serde::de::Error::unknown_variant(&op, &["interpolate"])),
        }
    }
}

/// "Object"
#[derive(serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Object {
    /// Provides a literal array or object value.
    ///
    ///  - [Display and style rich text labels](https://maplibre.org/maplibre-gl-js/docs/examples/display-and-style-rich-text-labels/)
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Asserts that the input value is an object. If multiple values are provided, each one is evaluated in order until an object is obtained. If none of the inputs are objects, the expression is an error.
    Object(Vec<serde_json::Value>),
    /// Gets the feature properties object.  Note that in some cases, it may be more efficient to use ["get", "property_name"] directly.
    Properties,
}

impl<'de> serde::Deserialize<'de> for Object {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ObjectVisitor)
    }
}

/// Visitor for deserializing the syntax enum [`Object`]
struct ObjectVisitor;

impl<'de> serde::de::Visitor<'de> for ObjectVisitor {
    type Value = Object;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an Object expression (example: [\"literal\",[\"DIN Offc Pro Italic\",\"Arial Unicode MS Regular\"]])")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        /// Reads the next element from the sequence or reports a missing field error.
        #[allow(dead_code)]
        fn visit_seq_field<'de, A, T>(seq: &mut A, name: &'static str) -> Result<T, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
            T: serde::Deserialize<'de>,
        {
            seq.next_element()?
                .ok_or_else(|| serde::de::Error::missing_field(name))
        }

        // First element: operator string
        let op: std::string::String = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::custom("missing operator"))?;
        match op.as_str() {
            "literal" => {
                let json_object = visit_seq_field(&mut seq, "json_object")?;
                Ok(Object::Literal(json_object))
            }
            "object" => {
                let mut inputs = Vec::new();
                while let Some(element) = seq.next_element()? {
                    inputs.push(element);
                }
                if inputs.is_empty() {
                    return Err(serde::de::Error::custom(
                        "Object::Object requires at least one argument",
                    ));
                }
                Ok(Object::Object(inputs))
            }
            "properties" => Ok(Object::Properties),
            _ => Err(serde::de::Error::unknown_variant(
                &op,
                &["literal", "object", "properties"],
            )),
        }
    }
}

/// "String"
#[derive(serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum String {
    /// Returns a `string` consisting of the concatenation of the inputs. Each input is converted to a string as if by `to-string`.
    ///
    ///  - [Add a generated icon to the map](https://maplibre.org/maplibre-gl-js/docs/examples/add-a-generated-icon-to-the-map/)
    ///
    ///  - [Create a time slider](https://maplibre.org/maplibre-gl-js/docs/examples/create-a-time-slider/)
    ///
    ///  - [Use a fallback image](https://maplibre.org/maplibre-gl-js/docs/examples/fallback-image/)
    ///
    ///  - [Variable label placement](https://maplibre.org/maplibre-gl-js/docs/examples/variable-label-placement/)
    Concat(Vec<serde_json::Value>),
    /// Returns the input string converted to lowercase. Follows the Unicode Default Case Conversion algorithm and the locale-insensitive case mappings in the Unicode Character Database.
    ///
    ///  - [Change the case of labels](https://maplibre.org/maplibre-gl-js/docs/examples/change-case-of-labels/)
    Downcase(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns the feature's simple geometry type: `Point`, `LineString`, or `Polygon`. `MultiPoint`, `MultiLineString`, and `MultiPolygon` are returned as `Point`, `LineString`, and `Polygon`, respectively.
    GeometryType,
    /// Converts the input number into a string representation using the provided format_options.
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    NumberFormat(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns the IETF language tag of the locale being used by the provided `collator`. This can be used to determine the default system locale, or to determine if a requested locale was successfully loaded.
    ResolvedLocale(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns a subarray from an array or a substring from a string from a specified start index, or between a start index and an end index if set. The return value is inclusive of the start index but not of the end index. In a string, a UTF-16 surrogate pair counts as a single position.
    Slice(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_option_json_value))]
         Option<serde_json::Value>,
    ),
    /// Asserts that the input value is a string. If multiple values are provided, each one is evaluated in order until a string is obtained. If none of the inputs are strings, the expression is an error.
    String(Vec<serde_json::Value>),
    /// Converts the input value to a string. If the input is `null`, the result is `""`. If the input is a boolean, the result is `"true"` or `"false"`. If the input is a number, it is converted to a string as specified by the ["NumberToString" algorithm](https://tc39.github.io/ecma262/#sec-tostring-applied-to-the-number-type) of the ECMAScript Language Specification. If the input is a color, it is converted to a string of the form `"rgba(r,g,b,a)"`, where `r`, `g`, and `b` are numerals ranging from 0 to 255, and `a` ranges from 0 to 1. Otherwise, the input is converted to a string in the format specified by the [`JSON.stringify`](https://tc39.github.io/ecma262/#sec-json.stringify) function of the ECMAScript Language Specification.
    ///
    ///  - [Create a time slider](https://maplibre.org/maplibre-gl-js/docs/examples/create-a-time-slider/)
    ToString(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns a string describing the type of the given value.
    Typeof(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
    /// Returns the input string converted to uppercase. Follows the Unicode Default Case Conversion algorithm and the locale-insensitive case mappings in the Unicode Character Database.
    ///
    ///  - [Change the case of labels](https://maplibre.org/maplibre-gl-js/docs/examples/change-case-of-labels/)
    Upcase(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl<'de> serde::Deserialize<'de> for String {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(StringVisitor)
    }
}

/// Visitor for deserializing the syntax enum [`String`]
struct StringVisitor;

impl<'de> serde::de::Visitor<'de> for StringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(
            "an String expression (example: [\"concat\",\"square-rgb-\",[\"get\",\"color\"]])",
        )
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        /// Reads the next element from the sequence or reports a missing field error.
        #[allow(dead_code)]
        fn visit_seq_field<'de, A, T>(seq: &mut A, name: &'static str) -> Result<T, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
            T: serde::Deserialize<'de>,
        {
            seq.next_element()?
                .ok_or_else(|| serde::de::Error::missing_field(name))
        }

        // First element: operator string
        let op: std::string::String = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::custom("missing operator"))?;
        match op.as_str() {
            "concat" => {
                let mut inputs = Vec::new();
                while let Some(element) = seq.next_element()? {
                    inputs.push(element);
                }
                if inputs.is_empty() {
                    return Err(serde::de::Error::custom(
                        "String::Concat requires at least one argument",
                    ));
                }
                Ok(String::Concat(inputs))
            }
            "downcase" => {
                let input = visit_seq_field(&mut seq, "input")?;
                Ok(String::Downcase(input))
            }
            "geometry-type" => Ok(String::GeometryType),
            "number-format" => {
                let input = visit_seq_field(&mut seq, "input")?;
                let format_options = visit_seq_field(&mut seq, "format_options")?;
                Ok(String::NumberFormat(input, format_options))
            }
            "resolved-locale" => {
                let collator = visit_seq_field(&mut seq, "collator")?;
                Ok(String::ResolvedLocale(collator))
            }
            "slice" => {
                let string = visit_seq_field(&mut seq, "string")?;
                let start_index = visit_seq_field(&mut seq, "start_index")?;
                let end_index = seq.next_element()?;
                Ok(String::Slice(string, start_index, end_index))
            }
            "string" => {
                let mut inputs = Vec::new();
                while let Some(element) = seq.next_element()? {
                    inputs.push(element);
                }
                if inputs.is_empty() {
                    return Err(serde::de::Error::custom(
                        "String::String requires at least one argument",
                    ));
                }
                Ok(String::String(inputs))
            }
            "to-string" => {
                let value = visit_seq_field(&mut seq, "value")?;
                Ok(String::ToString(value))
            }
            "typeof" => {
                let value = visit_seq_field(&mut seq, "value")?;
                Ok(String::Typeof(value))
            }
            "upcase" => {
                let input = visit_seq_field(&mut seq, "input")?;
                Ok(String::Upcase(input))
            }
            _ => Err(serde::de::Error::unknown_variant(
                &op,
                &[
                    "concat",
                    "downcase",
                    "geometry-type",
                    "number-format",
                    "resolved-locale",
                    "slice",
                    "string",
                    "to-string",
                    "typeof",
                    "upcase",
                ],
            )),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSource {
    /// Contains an attribution to be displayed when the map is shown to a user.
    pub attribution: Option<GeojsonSourceAttribution>,
    /// Size of the tile buffer on each side. A value of 0 produces no buffer. A value of 512 produces a buffer as wide as the tile itself. Larger values produce fewer rendering artifacts near tile edges and slower performance.
    ///
    /// Range: 0..=512
    pub buffer: Option<GeojsonSourceBuffer>,
    /// If the data is a collection of point features, setting this to true clusters the points by radius into groups. Cluster groups become new `Point` features in the source with additional properties:
    ///
    ///  * `cluster` Is `true` if the point is a cluster
    ///
    ///  * `cluster_id` A unique id for the cluster to be used in conjunction with the [cluster inspection methods](https://maplibre.org/maplibre-gl-js/docs/API/classes/GeoJSONSource/#getclusterexpansionzoom)
    ///
    ///  * `point_count` Number of original points grouped into this cluster
    ///
    ///  * `point_count_abbreviated` An abbreviated point count
    pub cluster: Option<GeojsonSourceCluster>,
    /// Max zoom on which to cluster points if clustering is enabled. Defaults to one zoom less than maxzoom (so that last zoom features are not clustered). Clusters are re-evaluated at integer zoom levels so setting clusterMaxZoom to 14 means the clusters will be displayed until z15.
    #[serde(rename = "clusterMaxZoom")]
    pub cluster_max_zoom: Option<GeojsonSourceClusterMaxZoom>,
    /// Minimum number of points necessary to form a cluster if clustering is enabled. Defaults to `2`.
    #[serde(rename = "clusterMinPoints")]
    pub cluster_min_points: Option<GeojsonSourceClusterMinPoints>,
    /// An object defining custom properties on the generated clusters if clustering is enabled, aggregating values from clustered points. Has the form `{"property_name": [operator, map_expression]}`. `operator` is any expression function that accepts at least 2 operands (e.g. `"+"` or `"max"`) — it accumulates the property value from clusters/points the cluster contains; `map_expression` produces the value of a single point.
    ///
    /// Example: `{"sum": ["+", ["get", "scalerank"]]}`.
    ///
    /// For more advanced use cases, in place of `operator`, you can use a custom reduce expression that references a special `["accumulated"]` value, e.g.:
    ///
    /// `{"sum": [["+", ["accumulated"], ["get", "sum"]], ["get", "scalerank"]]}`
    #[serde(rename = "clusterProperties")]
    pub cluster_properties: Option<GeojsonSourceClusterProperties>,
    /// Radius of each cluster if clustering is enabled. A value of 512 indicates a radius equal to the width of a tile.
    ///
    /// Range: 0..
    #[serde(rename = "clusterRadius")]
    pub cluster_radius: Option<GeojsonSourceClusterRadius>,
    /// A URL to a GeoJSON file, or inline GeoJSON.
    pub data: GeojsonSourceData,
    /// An expression for filtering features prior to processing them for rendering.
    pub filter: Option<GeojsonSourceFilter>,
    /// Whether to generate ids for the geojson features. When enabled, the `feature.id` property will be auto assigned based on its index in the `features` array, over-writing any previous values.
    #[serde(rename = "generateId")]
    pub generate_id: Option<GeojsonSourceGenerateId>,
    /// Whether to calculate line distance metrics. This is required for line layers that specify `line-gradient` values.
    #[serde(rename = "lineMetrics")]
    pub line_metrics: Option<GeojsonSourceLineMetrics>,
    /// Maximum zoom level at which to create vector tiles (higher means greater detail at high zoom levels).
    pub maxzoom: Option<GeojsonSourceMaxzoom>,
    /// A property to use as a feature id (for feature state). Either a property name, or an object of the form `{<sourceLayer>: <propertyName>}`.
    #[serde(rename = "promoteId")]
    pub promote_id: Option<GeojsonSourcePromoteId>,
    /// Douglas-Peucker simplification tolerance (higher means simpler geometries and faster performance).
    pub tolerance: Option<GeojsonSourceTolerance>,
}

/// Contains an attribution to be displayed when the map is shown to a user.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceAttribution(std::string::String);

/// Size of the tile buffer on each side. A value of 0 produces no buffer. A value of 512 produces a buffer as wide as the tile itself. Larger values produce fewer rendering artifacts near tile edges and slower performance.
///
/// Range: 0..=512
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceBuffer(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for GeojsonSourceBuffer {
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
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceCluster(bool);

impl Default for GeojsonSourceCluster {
    fn default() -> Self {
        Self(false)
    }
}

/// Max zoom on which to cluster points if clustering is enabled. Defaults to one zoom less than maxzoom (so that last zoom features are not clustered). Clusters are re-evaluated at integer zoom levels so setting clusterMaxZoom to 14 means the clusters will be displayed until z15.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceClusterMaxZoom(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

/// Minimum number of points necessary to form a cluster if clustering is enabled. Defaults to `2`.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceClusterMinPoints(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

/// An object defining custom properties on the generated clusters if clustering is enabled, aggregating values from clustered points. Has the form `{"property_name": [operator, map_expression]}`. `operator` is any expression function that accepts at least 2 operands (e.g. `"+"` or `"max"`) — it accumulates the property value from clusters/points the cluster contains; `map_expression` produces the value of a single point.
///
/// Example: `{"sum": ["+", ["get", "scalerank"]]}`.
///
/// For more advanced use cases, in place of `operator`, you can use a custom reduce expression that references a special `["accumulated"]` value, e.g.:
///
/// `{"sum": [["+", ["accumulated"], ["get", "sum"]], ["get", "scalerank"]]}`
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceClusterProperties(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
    serde_json::Value,
);

/// Radius of each cluster if clustering is enabled. A value of 512 indicates a radius equal to the width of a tile.
///
/// Range: 0..
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceClusterRadius(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for GeojsonSourceClusterRadius {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(50)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// A URL to a GeoJSON file, or inline GeoJSON.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceData(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
    serde_json::Value,
);

/// An expression for filtering features prior to processing them for rendering.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceFilter(Filter);

/// Whether to generate ids for the geojson features. When enabled, the `feature.id` property will be auto assigned based on its index in the `features` array, over-writing any previous values.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceGenerateId(bool);

impl Default for GeojsonSourceGenerateId {
    fn default() -> Self {
        Self(false)
    }
}

/// Whether to calculate line distance metrics. This is required for line layers that specify `line-gradient` values.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceLineMetrics(bool);

impl Default for GeojsonSourceLineMetrics {
    fn default() -> Self {
        Self(false)
    }
}

/// Maximum zoom level at which to create vector tiles (higher means greater detail at high zoom levels).
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceMaxzoom(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for GeojsonSourceMaxzoom {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(18)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// A property to use as a feature id (for feature state). Either a property name, or an object of the form `{<sourceLayer>: <propertyName>}`.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourcePromoteId(std::string::String);

/// Douglas-Peucker simplification tolerance (higher means simpler geometries and faster performance).
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceTolerance(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for GeojsonSourceTolerance {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(0.375)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct ImageSource {
    /// Corners of image specified in longitude, latitude pairs.
    pub coordinates: ImageSourceCoordinates,
    /// URL that points to an image.
    pub url: ImageSourceUrl,
}

/// A single longitude, latitude pair.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct ImageSourceCoordinatesValue(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_2_json_number))]
     Box<[serde_json::Number; 2]>,
);

/// Corners of image specified in longitude, latitude pairs.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct ImageSourceCoordinates(Box<[ImageSourceCoordinatesValue; 4]>);

/// URL that points to an image.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct ImageSourceUrl(std::string::String);

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterSource {
    /// Contains an attribution to be displayed when the map is shown to a user.
    pub attribution: Option<RasterSourceAttribution>,
    /// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
    pub bounds: Option<RasterSourceBounds>,
    /// Maximum zoom level for which tiles are available, as in the TileJSON spec. Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
    pub maxzoom: Option<RasterSourceMaxzoom>,
    /// Minimum zoom level for which tiles are available, as in the TileJSON spec.
    pub minzoom: Option<RasterSourceMinzoom>,
    /// Influences the y direction of the tile coordinates. The global-mercator (aka Spherical Mercator) profile is assumed.
    pub scheme: Option<RasterSourceScheme>,
    /// The minimum visual size to display tiles for this layer. Only configurable for raster layers.
    #[serde(rename = "tileSize")]
    pub tile_size: Option<RasterSourceTileSize>,
    /// An array of one or more tile source URLs, as in the TileJSON spec.
    pub tiles: Option<RasterSourceTiles>,
    /// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
    pub url: Option<RasterSourceUrl>,
    /// A setting to determine whether a source's tiles are cached locally.
    pub volatile: Option<RasterSourceVolatile>,
}

/// Contains an attribution to be displayed when the map is shown to a user.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterSourceAttribution(std::string::String);

/// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterSourceBounds(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_4_json_number))]
     Box<[serde_json::Number; 4]>,
);

impl Default for RasterSourceBounds {
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
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterSourceMaxzoom(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for RasterSourceMaxzoom {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(22)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Minimum zoom level for which tiles are available, as in the TileJSON spec.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterSourceMinzoom(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for RasterSourceMinzoom {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Influences the y direction of the tile coordinates. The global-mercator (aka Spherical Mercator) profile is assumed.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum RasterSourceScheme {
    /// OSGeo spec scheme.
    #[serde(rename = "tms")]
    Tms,
    /// Slippy map tilenames scheme.
    #[serde(rename = "xyz")]
    #[default]
    Xyz,
}

/// The minimum visual size to display tiles for this layer. Only configurable for raster layers.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterSourceTileSize(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for RasterSourceTileSize {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(512)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// An array of one or more tile source URLs, as in the TileJSON spec.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterSourceTiles(Vec<std::string::String>);

/// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterSourceUrl(std::string::String);

/// A setting to determine whether a source's tiles are cached locally.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterSourceVolatile(bool);

impl Default for RasterSourceVolatile {
    fn default() -> Self {
        Self(false)
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSource {
    /// Contains an attribution to be displayed when the map is shown to a user.
    pub attribution: Option<RasterDemSourceAttribution>,
    /// Value that will be added to the encoding mix when decoding. Only used on custom encodings.
    #[serde(rename = "baseShift")]
    pub base_shift: Option<RasterDemSourceBaseShift>,
    /// Value that will be multiplied by the blue channel value when decoding. Only used on custom encodings.
    #[serde(rename = "blueFactor")]
    pub blue_factor: Option<RasterDemSourceBlueFactor>,
    /// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
    pub bounds: Option<RasterDemSourceBounds>,
    /// The encoding used by this source. Mapbox Terrain RGB is used by default.
    pub encoding: Option<RasterDemSourceEncoding>,
    /// Value that will be multiplied by the green channel value when decoding. Only used on custom encodings.
    #[serde(rename = "greenFactor")]
    pub green_factor: Option<RasterDemSourceGreenFactor>,
    /// Maximum zoom level for which tiles are available, as in the TileJSON spec. Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
    pub maxzoom: Option<RasterDemSourceMaxzoom>,
    /// Minimum zoom level for which tiles are available, as in the TileJSON spec.
    pub minzoom: Option<RasterDemSourceMinzoom>,
    /// Value that will be multiplied by the red channel value when decoding. Only used on custom encodings.
    #[serde(rename = "redFactor")]
    pub red_factor: Option<RasterDemSourceRedFactor>,
    /// The minimum visual size to display tiles for this layer. Only configurable for raster layers.
    #[serde(rename = "tileSize")]
    pub tile_size: Option<RasterDemSourceTileSize>,
    /// An array of one or more tile source URLs, as in the TileJSON spec.
    pub tiles: Option<RasterDemSourceTiles>,
    /// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
    pub url: Option<RasterDemSourceUrl>,
    /// A setting to determine whether a source's tiles are cached locally.
    pub volatile: Option<RasterDemSourceVolatile>,
}

/// Contains an attribution to be displayed when the map is shown to a user.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceAttribution(std::string::String);

/// Value that will be added to the encoding mix when decoding. Only used on custom encodings.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceBaseShift(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for RasterDemSourceBaseShift {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(0.0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Value that will be multiplied by the blue channel value when decoding. Only used on custom encodings.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceBlueFactor(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for RasterDemSourceBlueFactor {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(1.0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceBounds(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_4_json_number))]
     Box<[serde_json::Number; 4]>,
);

impl Default for RasterDemSourceBounds {
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
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum RasterDemSourceEncoding {
    /// Decodes tiles using the redFactor, blueFactor, greenFactor, baseShift parameters.
    #[serde(rename = "custom")]
    Custom,
    /// Mapbox Terrain RGB tiles. See https://www.mapbox.com/help/access-elevation-data/#mapbox-terrain-rgb for more info.
    #[serde(rename = "mapbox")]
    #[default]
    Mapbox,
    /// Terrarium format PNG tiles. See https://aws.amazon.com/es/public-datasets/terrain/ for more info.
    #[serde(rename = "terrarium")]
    Terrarium,
}

/// Value that will be multiplied by the green channel value when decoding. Only used on custom encodings.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceGreenFactor(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for RasterDemSourceGreenFactor {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(1.0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Maximum zoom level for which tiles are available, as in the TileJSON spec. Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceMaxzoom(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for RasterDemSourceMaxzoom {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(22)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Minimum zoom level for which tiles are available, as in the TileJSON spec.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceMinzoom(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for RasterDemSourceMinzoom {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Value that will be multiplied by the red channel value when decoding. Only used on custom encodings.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceRedFactor(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for RasterDemSourceRedFactor {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(1.0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The minimum visual size to display tiles for this layer. Only configurable for raster layers.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceTileSize(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for RasterDemSourceTileSize {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(512)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// An array of one or more tile source URLs, as in the TileJSON spec.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceTiles(Vec<std::string::String>);

/// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceUrl(std::string::String);

/// A setting to determine whether a source's tiles are cached locally.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceVolatile(bool);

impl Default for RasterDemSourceVolatile {
    fn default() -> Self {
        Self(false)
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VectorSource {
    /// Contains an attribution to be displayed when the map is shown to a user.
    pub attribution: Option<VectorSourceAttribution>,
    /// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
    pub bounds: Option<VectorSourceBounds>,
    /// The encoding used by this source. Mapbox Vector Tiles encoding is used by default.
    pub encoding: Option<VectorSourceEncoding>,
    /// Maximum zoom level for which tiles are available, as in the TileJSON spec. Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
    pub maxzoom: Option<VectorSourceMaxzoom>,
    /// Minimum zoom level for which tiles are available, as in the TileJSON spec.
    pub minzoom: Option<VectorSourceMinzoom>,
    /// A property to use as a feature id (for feature state). Either a property name, or an object of the form `{<sourceLayer>: <propertyName>}`. If specified as a string for a vector tile source, the same property is used across all its source layers.
    #[serde(rename = "promoteId")]
    pub promote_id: Option<VectorSourcePromoteId>,
    /// Influences the y direction of the tile coordinates. The global-mercator (aka Spherical Mercator) profile is assumed.
    pub scheme: Option<VectorSourceScheme>,
    /// An array of one or more tile source URLs, as in the TileJSON spec.
    pub tiles: Option<VectorSourceTiles>,
    /// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
    pub url: Option<VectorSourceUrl>,
    /// A setting to determine whether a source's tiles are cached locally.
    pub volatile: Option<VectorSourceVolatile>,
}

/// Contains an attribution to be displayed when the map is shown to a user.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VectorSourceAttribution(std::string::String);

/// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VectorSourceBounds(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_4_json_number))]
     Box<[serde_json::Number; 4]>,
);

impl Default for VectorSourceBounds {
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
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum VectorSourceEncoding {
    /// MapLibre Vector Tiles. See https://github.com/maplibre/maplibre-tile-spec for more info.
    #[serde(rename = "mlt")]
    Mlt,
    /// Mapbox Vector Tiles. See http://github.com/mapbox/vector-tile-spec for more info.
    #[serde(rename = "mvt")]
    #[default]
    Mvt,
}

/// Maximum zoom level for which tiles are available, as in the TileJSON spec. Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VectorSourceMaxzoom(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for VectorSourceMaxzoom {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(22)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Minimum zoom level for which tiles are available, as in the TileJSON spec.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VectorSourceMinzoom(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for VectorSourceMinzoom {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// A property to use as a feature id (for feature state). Either a property name, or an object of the form `{<sourceLayer>: <propertyName>}`. If specified as a string for a vector tile source, the same property is used across all its source layers.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VectorSourcePromoteId(std::string::String);

/// Influences the y direction of the tile coordinates. The global-mercator (aka Spherical Mercator) profile is assumed.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum VectorSourceScheme {
    /// OSGeo spec scheme.
    #[serde(rename = "tms")]
    Tms,
    /// Slippy map tilenames scheme.
    #[serde(rename = "xyz")]
    #[default]
    Xyz,
}

/// An array of one or more tile source URLs, as in the TileJSON spec.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VectorSourceTiles(Vec<std::string::String>);

/// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VectorSourceUrl(std::string::String);

/// A setting to determine whether a source's tiles are cached locally.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VectorSourceVolatile(bool);

impl Default for VectorSourceVolatile {
    fn default() -> Self {
        Self(false)
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VideoSource {
    /// Corners of video specified in longitude, latitude pairs.
    pub coordinates: VideoSourceCoordinates,
    /// URLs to video content in order of preferred format.
    pub urls: VideoSourceUrls,
}

/// A single longitude, latitude pair.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VideoSourceCoordinatesValue(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_2_json_number))]
     Box<[serde_json::Number; 2]>,
);

/// Corners of video specified in longitude, latitude pairs.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VideoSourceCoordinates(Box<[VideoSourceCoordinatesValue; 4]>);

/// URLs to video content in order of preferred format.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VideoSourceUrls(Vec<std::string::String>);

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(tag = "type")]
pub enum Source {
    #[serde(rename = "geojson")]
    GeojsonSource(GeojsonSource),
    #[serde(rename = "image")]
    ImageSource(ImageSource),
    #[serde(rename = "raster")]
    RasterSource(RasterSource),
    #[serde(rename = "raster-dem")]
    RasterDemSource(RasterDemSource),
    #[serde(rename = "vector")]
    VectorSource(VectorSource),
    #[serde(rename = "video")]
    VideoSource(VideoSource),
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct Layer {
    /// A expression specifying conditions on source features. Only features that match the filter are displayed. Zoom expressions in filters are only evaluated at integer zoom levels. The `feature-state` expression is not supported in filter expressions.
    pub filter: Option<LayerFilter>,
    /// Unique layer name.
    pub id: LayerId,
    /// The maximum zoom level for the layer. At zoom levels equal to or greater than the maxzoom, the layer will be hidden.
    pub maxzoom: Option<LayerMaxzoom>,
    /// Arbitrary properties useful to track with the layer, but do not influence rendering. Properties should be prefixed to avoid collisions, like 'maplibre:'.
    pub metadata: Option<LayerMetadata>,
    /// The minimum zoom level for the layer. At zoom levels less than the minzoom, the layer will be hidden.
    pub minzoom: Option<LayerMinzoom>,
    /// Name of a source description to be used for this layer. Required for all layer types except `background`.
    pub source: Option<LayerSource>,
    /// Layer to use from a vector tile source. Required for vector tile sources; prohibited for all other source types, including GeoJSON sources.
    #[serde(rename = "source-layer")]
    pub source_layer: Option<LayerSourceLayer>,
}

/// A expression specifying conditions on source features. Only features that match the filter are displayed. Zoom expressions in filters are only evaluated at integer zoom levels. The `feature-state` expression is not supported in filter expressions.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct LayerFilter(std::string::String);

/// Unique layer name.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct LayerId(std::string::String);

/// The maximum zoom level for the layer. At zoom levels equal to or greater than the maxzoom, the layer will be hidden.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct LayerMaxzoom(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

/// Arbitrary properties useful to track with the layer, but do not influence rendering. Properties should be prefixed to avoid collisions, like 'maplibre:'.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct LayerMetadata(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
    serde_json::Value,
);

/// The minimum zoom level for the layer. At zoom levels less than the minzoom, the layer will be hidden.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct LayerMinzoom(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

/// Name of a source description to be used for this layer. Required for all layer types except `background`.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct LayerSource(std::string::String);

/// Layer to use from a vector tile source. Required for vector tile sources; prohibited for all other source types, including GeoJSON sources.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct LayerSourceLayer(std::string::String);

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct BackgroundLayoutLayer {
    /// Whether this layer is displayed.
    pub visibility: Option<BackgroundLayoutLayerVisibility>,
}

/// Whether this layer is displayed.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum BackgroundLayoutLayerVisibility {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "visible")]
    #[default]
    Visible,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct BackgroundPaintLayer {
    /// The color with which the background will be drawn.
    #[serde(rename = "background-color")]
    pub background_color: Option<BackgroundPaintLayerBackgroundColor>,
    /// The opacity at which the background will be drawn.
    #[serde(rename = "background-opacity")]
    pub background_opacity: Option<BackgroundPaintLayerBackgroundOpacity>,
    /// Name of image in sprite to use for drawing an image background. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
    #[serde(rename = "background-pattern")]
    pub background_pattern: Option<BackgroundPaintLayerBackgroundPattern>,
}

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum BackgroundPaintLayerBackgroundColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

/// The color with which the background will be drawn.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum BackgroundPaintLayerBackgroundColor {
    Expr(BackgroundPaintLayerBackgroundColorExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl Default for BackgroundPaintLayerBackgroundColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("#000000"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum BackgroundPaintLayerBackgroundOpacityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// The opacity at which the background will be drawn.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum BackgroundPaintLayerBackgroundOpacity {
    Expr(BackgroundPaintLayerBackgroundOpacityExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for BackgroundPaintLayerBackgroundOpacity {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Name of image in sprite to use for drawing an image background. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct BackgroundPaintLayerBackgroundPattern(std::string::String);

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct CircleLayoutLayer {
    /// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
    #[serde(rename = "circle-sort-key")]
    pub circle_sort_key: Option<CircleLayoutLayerCircleSortKey>,
    /// Whether this layer is displayed.
    pub visibility: Option<CircleLayoutLayerVisibility>,
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum CircleLayoutLayerCircleSortKeyExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum CircleLayoutLayerCircleSortKey {
    Expr(CircleLayoutLayerCircleSortKeyExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

/// Whether this layer is displayed.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum CircleLayoutLayerVisibility {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "visible")]
    #[default]
    Visible,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct CirclePaintLayer {
    /// Amount to blur the circle. 1 blurs the circle such that only the centerpoint is full opacity.
    #[serde(rename = "circle-blur")]
    pub circle_blur: Option<CirclePaintLayerCircleBlur>,
    /// The fill color of the circle.
    #[serde(rename = "circle-color")]
    pub circle_color: Option<CirclePaintLayerCircleColor>,
    /// The opacity at which the circle will be drawn.
    #[serde(rename = "circle-opacity")]
    pub circle_opacity: Option<CirclePaintLayerCircleOpacity>,
    /// Orientation of circle when map is pitched.
    #[serde(rename = "circle-pitch-alignment")]
    pub circle_pitch_alignment: Option<CirclePaintLayerCirclePitchAlignment>,
    /// Controls the scaling behavior of the circle when the map is pitched.
    #[serde(rename = "circle-pitch-scale")]
    pub circle_pitch_scale: Option<CirclePaintLayerCirclePitchScale>,
    /// Circle radius.
    #[serde(rename = "circle-radius")]
    pub circle_radius: Option<CirclePaintLayerCircleRadius>,
    /// The stroke color of the circle.
    #[serde(rename = "circle-stroke-color")]
    pub circle_stroke_color: Option<CirclePaintLayerCircleStrokeColor>,
    /// The opacity of the circle's stroke.
    #[serde(rename = "circle-stroke-opacity")]
    pub circle_stroke_opacity: Option<CirclePaintLayerCircleStrokeOpacity>,
    /// The width of the circle's stroke. Strokes are placed outside of the `circle-radius`.
    #[serde(rename = "circle-stroke-width")]
    pub circle_stroke_width: Option<CirclePaintLayerCircleStrokeWidth>,
    /// The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.
    #[serde(rename = "circle-translate")]
    pub circle_translate: Option<CirclePaintLayerCircleTranslate>,
    /// Controls the frame of reference for `circle-translate`.
    #[serde(rename = "circle-translate-anchor")]
    pub circle_translate_anchor: Option<CirclePaintLayerCircleTranslateAnchor>,
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum CirclePaintLayerCircleBlurExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Amount to blur the circle. 1 blurs the circle such that only the centerpoint is full opacity.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum CirclePaintLayerCircleBlur {
    Expr(CirclePaintLayerCircleBlurExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for CirclePaintLayerCircleBlur {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum CirclePaintLayerCircleColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

/// The fill color of the circle.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum CirclePaintLayerCircleColor {
    Expr(CirclePaintLayerCircleColorExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl Default for CirclePaintLayerCircleColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("#000000"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum CirclePaintLayerCircleOpacityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// The opacity at which the circle will be drawn.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum CirclePaintLayerCircleOpacity {
    Expr(CirclePaintLayerCircleOpacityExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for CirclePaintLayerCircleOpacity {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Orientation of circle when map is pitched.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum CirclePaintLayerCirclePitchAlignment {
    #[serde(rename = "map")]
    Map,
    #[serde(rename = "viewport")]
    #[default]
    Viewport,
}

/// Controls the scaling behavior of the circle when the map is pitched.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum CirclePaintLayerCirclePitchScale {
    #[serde(rename = "map")]
    #[default]
    Map,
    #[serde(rename = "viewport")]
    Viewport,
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum CirclePaintLayerCircleRadiusExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Circle radius.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum CirclePaintLayerCircleRadius {
    Expr(CirclePaintLayerCircleRadiusExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for CirclePaintLayerCircleRadius {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(5)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum CirclePaintLayerCircleStrokeColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

/// The stroke color of the circle.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum CirclePaintLayerCircleStrokeColor {
    Expr(CirclePaintLayerCircleStrokeColorExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl Default for CirclePaintLayerCircleStrokeColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("#000000"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum CirclePaintLayerCircleStrokeOpacityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// The opacity of the circle's stroke.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum CirclePaintLayerCircleStrokeOpacity {
    Expr(CirclePaintLayerCircleStrokeOpacityExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for CirclePaintLayerCircleStrokeOpacity {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum CirclePaintLayerCircleStrokeWidthExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// The width of the circle's stroke. Strokes are placed outside of the `circle-radius`.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum CirclePaintLayerCircleStrokeWidth {
    Expr(CirclePaintLayerCircleStrokeWidthExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for CirclePaintLayerCircleStrokeWidth {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct CirclePaintLayerCircleTranslate(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_2_json_number))]
     Box<[serde_json::Number; 2]>,
);

impl Default for CirclePaintLayerCircleTranslate {
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
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum CirclePaintLayerCircleTranslateAnchor {
    #[serde(rename = "map")]
    #[default]
    Map,
    #[serde(rename = "viewport")]
    Viewport,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct ColorReliefLayoutLayer {
    /// Whether this layer is displayed.
    pub visibility: Option<ColorReliefLayoutLayerVisibility>,
}

/// Whether this layer is displayed.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum ColorReliefLayoutLayerVisibility {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "visible")]
    #[default]
    Visible,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct ColorReliefPaintLayer {
    /// Defines the color of each pixel based on its elevation. Should be an expression that uses `["elevation"]` as input.
    #[serde(rename = "color-relief-color")]
    pub color_relief_color: Option<ColorReliefPaintLayerColorReliefColor>,
    /// The opacity at which the color-relief will be drawn.
    #[serde(rename = "color-relief-opacity")]
    pub color_relief_opacity: Option<ColorReliefPaintLayerColorReliefOpacity>,
}

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum ColorReliefPaintLayerColorReliefColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

/// Defines the color of each pixel based on its elevation. Should be an expression that uses `["elevation"]` as input.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum ColorReliefPaintLayerColorReliefColor {
    Expr(ColorReliefPaintLayerColorReliefColorExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum ColorReliefPaintLayerColorReliefOpacityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// The opacity at which the color-relief will be drawn.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum ColorReliefPaintLayerColorReliefOpacity {
    Expr(ColorReliefPaintLayerColorReliefOpacityExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for ColorReliefPaintLayerColorReliefOpacity {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FillLayoutLayer {
    /// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
    #[serde(rename = "fill-sort-key")]
    pub fill_sort_key: Option<FillLayoutLayerFillSortKey>,
    /// Whether this layer is displayed.
    pub visibility: Option<FillLayoutLayerVisibility>,
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum FillLayoutLayerFillSortKeyExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum FillLayoutLayerFillSortKey {
    Expr(FillLayoutLayerFillSortKeyExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

/// Whether this layer is displayed.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FillLayoutLayerVisibility {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "visible")]
    #[default]
    Visible,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FillPaintLayer {
    /// Whether or not the fill should be antialiased.
    #[serde(rename = "fill-antialias")]
    pub fill_antialias: Option<FillPaintLayerFillAntialias>,
    /// The color of the filled part of this layer. This color can be specified as `rgba` with an alpha component and the color's opacity will not affect the opacity of the 1px stroke, if it is used.
    #[serde(rename = "fill-color")]
    pub fill_color: Option<FillPaintLayerFillColor>,
    /// The opacity of the entire fill layer. In contrast to the `fill-color`, this value will also affect the 1px stroke around the fill, if the stroke is used.
    #[serde(rename = "fill-opacity")]
    pub fill_opacity: Option<FillPaintLayerFillOpacity>,
    /// The outline color of the fill. Matches the value of `fill-color` if unspecified.
    #[serde(rename = "fill-outline-color")]
    pub fill_outline_color: Option<FillPaintLayerFillOutlineColor>,
    /// Name of image in sprite to use for drawing image fills. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
    #[serde(rename = "fill-pattern")]
    pub fill_pattern: Option<FillPaintLayerFillPattern>,
    /// The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.
    #[serde(rename = "fill-translate")]
    pub fill_translate: Option<FillPaintLayerFillTranslate>,
    /// Controls the frame of reference for `fill-translate`.
    #[serde(rename = "fill-translate-anchor")]
    pub fill_translate_anchor: Option<FillPaintLayerFillTranslateAnchor>,
}

/// Whether or not the fill should be antialiased.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FillPaintLayerFillAntialias(bool);

impl Default for FillPaintLayerFillAntialias {
    fn default() -> Self {
        Self(true)
    }
}

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum FillPaintLayerFillColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

/// The color of the filled part of this layer. This color can be specified as `rgba` with an alpha component and the color's opacity will not affect the opacity of the 1px stroke, if it is used.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum FillPaintLayerFillColor {
    Expr(FillPaintLayerFillColorExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl Default for FillPaintLayerFillColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("#000000"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum FillPaintLayerFillOpacityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// The opacity of the entire fill layer. In contrast to the `fill-color`, this value will also affect the 1px stroke around the fill, if the stroke is used.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum FillPaintLayerFillOpacity {
    Expr(FillPaintLayerFillOpacityExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for FillPaintLayerFillOpacity {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum FillPaintLayerFillOutlineColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

/// The outline color of the fill. Matches the value of `fill-color` if unspecified.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum FillPaintLayerFillOutlineColor {
    Expr(FillPaintLayerFillOutlineColorExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

/// Name of image in sprite to use for drawing image fills. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FillPaintLayerFillPattern(std::string::String);

/// The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FillPaintLayerFillTranslate(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_2_json_number))]
     Box<[serde_json::Number; 2]>,
);

impl Default for FillPaintLayerFillTranslate {
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
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FillPaintLayerFillTranslateAnchor {
    #[serde(rename = "map")]
    #[default]
    Map,
    #[serde(rename = "viewport")]
    Viewport,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FillExtrusionLayoutLayer {
    /// Whether this layer is displayed.
    pub visibility: Option<FillExtrusionLayoutLayerVisibility>,
}

/// Whether this layer is displayed.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FillExtrusionLayoutLayerVisibility {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "visible")]
    #[default]
    Visible,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FillExtrusionPaintLayer {
    /// The height with which to extrude the base of this layer. Must be less than or equal to `fill-extrusion-height`.
    #[serde(rename = "fill-extrusion-base")]
    pub fill_extrusion_base: Option<FillExtrusionPaintLayerFillExtrusionBase>,
    /// The base color of the extruded fill. The extrusion's surfaces will be shaded differently based on this color in combination with the root `light` settings. If this color is specified as `rgba` with an alpha component, the alpha component will be ignored; use `fill-extrusion-opacity` to set layer opacity.
    #[serde(rename = "fill-extrusion-color")]
    pub fill_extrusion_color: Option<FillExtrusionPaintLayerFillExtrusionColor>,
    /// The height with which to extrude this layer.
    #[serde(rename = "fill-extrusion-height")]
    pub fill_extrusion_height: Option<FillExtrusionPaintLayerFillExtrusionHeight>,
    /// The opacity of the entire fill extrusion layer. This is rendered on a per-layer, not per-feature, basis, and data-driven styling is not available.
    #[serde(rename = "fill-extrusion-opacity")]
    pub fill_extrusion_opacity: Option<FillExtrusionPaintLayerFillExtrusionOpacity>,
    /// Name of image in sprite to use for drawing images on extruded fills. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
    #[serde(rename = "fill-extrusion-pattern")]
    pub fill_extrusion_pattern: Option<FillExtrusionPaintLayerFillExtrusionPattern>,
    /// The geometry's offset. Values are [x, y] where negatives indicate left and up (on the flat plane), respectively.
    #[serde(rename = "fill-extrusion-translate")]
    pub fill_extrusion_translate: Option<FillExtrusionPaintLayerFillExtrusionTranslate>,
    /// Controls the frame of reference for `fill-extrusion-translate`.
    #[serde(rename = "fill-extrusion-translate-anchor")]
    pub fill_extrusion_translate_anchor:
        Option<FillExtrusionPaintLayerFillExtrusionTranslateAnchor>,
    /// Whether to apply a vertical gradient to the sides of a fill-extrusion layer. If true, sides will be shaded slightly darker farther down.
    #[serde(rename = "fill-extrusion-vertical-gradient")]
    pub fill_extrusion_vertical_gradient:
        Option<FillExtrusionPaintLayerFillExtrusionVerticalGradient>,
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum FillExtrusionPaintLayerFillExtrusionBaseExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// The height with which to extrude the base of this layer. Must be less than or equal to `fill-extrusion-height`.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum FillExtrusionPaintLayerFillExtrusionBase {
    Expr(FillExtrusionPaintLayerFillExtrusionBaseExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for FillExtrusionPaintLayerFillExtrusionBase {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum FillExtrusionPaintLayerFillExtrusionColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

/// The base color of the extruded fill. The extrusion's surfaces will be shaded differently based on this color in combination with the root `light` settings. If this color is specified as `rgba` with an alpha component, the alpha component will be ignored; use `fill-extrusion-opacity` to set layer opacity.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum FillExtrusionPaintLayerFillExtrusionColor {
    Expr(FillExtrusionPaintLayerFillExtrusionColorExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl Default for FillExtrusionPaintLayerFillExtrusionColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("#000000"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum FillExtrusionPaintLayerFillExtrusionHeightExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// The height with which to extrude this layer.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum FillExtrusionPaintLayerFillExtrusionHeight {
    Expr(FillExtrusionPaintLayerFillExtrusionHeightExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for FillExtrusionPaintLayerFillExtrusionHeight {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum FillExtrusionPaintLayerFillExtrusionOpacityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// The opacity of the entire fill extrusion layer. This is rendered on a per-layer, not per-feature, basis, and data-driven styling is not available.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum FillExtrusionPaintLayerFillExtrusionOpacity {
    Expr(FillExtrusionPaintLayerFillExtrusionOpacityExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for FillExtrusionPaintLayerFillExtrusionOpacity {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Name of image in sprite to use for drawing images on extruded fills. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FillExtrusionPaintLayerFillExtrusionPattern(std::string::String);

/// The geometry's offset. Values are [x, y] where negatives indicate left and up (on the flat plane), respectively.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FillExtrusionPaintLayerFillExtrusionTranslate(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_2_json_number))]
     Box<[serde_json::Number; 2]>,
);

impl Default for FillExtrusionPaintLayerFillExtrusionTranslate {
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
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FillExtrusionPaintLayerFillExtrusionTranslateAnchor {
    #[serde(rename = "map")]
    #[default]
    Map,
    #[serde(rename = "viewport")]
    Viewport,
}

/// Whether to apply a vertical gradient to the sides of a fill-extrusion layer. If true, sides will be shaded slightly darker farther down.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FillExtrusionPaintLayerFillExtrusionVerticalGradient(bool);

impl Default for FillExtrusionPaintLayerFillExtrusionVerticalGradient {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct HeatmapLayoutLayer {
    /// Whether this layer is displayed.
    pub visibility: Option<HeatmapLayoutLayerVisibility>,
}

/// Whether this layer is displayed.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum HeatmapLayoutLayerVisibility {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "visible")]
    #[default]
    Visible,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct HeatmapPaintLayer {
    /// Defines the color of each pixel based on its density value in a heatmap.  Should be an expression that uses `["heatmap-density"]` as input.
    #[serde(rename = "heatmap-color")]
    pub heatmap_color: Option<HeatmapPaintLayerHeatmapColor>,
    /// Similar to `heatmap-weight` but controls the intensity of the heatmap globally. Primarily used for adjusting the heatmap based on zoom level.
    #[serde(rename = "heatmap-intensity")]
    pub heatmap_intensity: Option<HeatmapPaintLayerHeatmapIntensity>,
    /// The global opacity at which the heatmap layer will be drawn.
    #[serde(rename = "heatmap-opacity")]
    pub heatmap_opacity: Option<HeatmapPaintLayerHeatmapOpacity>,
    /// Radius of influence of one heatmap point in pixels. Increasing the value makes the heatmap smoother, but less detailed.
    #[serde(rename = "heatmap-radius")]
    pub heatmap_radius: Option<HeatmapPaintLayerHeatmapRadius>,
    /// A measure of how much an individual point contributes to the heatmap. A value of 10 would be equivalent to having 10 points of weight 1 in the same spot. Especially useful when combined with clustering.
    #[serde(rename = "heatmap-weight")]
    pub heatmap_weight: Option<HeatmapPaintLayerHeatmapWeight>,
}

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum HeatmapPaintLayerHeatmapColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

/// Defines the color of each pixel based on its density value in a heatmap.  Should be an expression that uses `["heatmap-density"]` as input.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum HeatmapPaintLayerHeatmapColor {
    Expr(HeatmapPaintLayerHeatmapColorExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl Default for HeatmapPaintLayerHeatmapColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!([
            "interpolate",
            ["linear"],
            ["heatmap-density"],
            0,
            "rgba(0, 0, 255, 0)",
            0.1,
            "royalblue",
            0.3,
            "cyan",
            0.5,
            "lime",
            0.7,
            "yellow",
            1,
            "red"
        ]))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum HeatmapPaintLayerHeatmapIntensityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Similar to `heatmap-weight` but controls the intensity of the heatmap globally. Primarily used for adjusting the heatmap based on zoom level.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum HeatmapPaintLayerHeatmapIntensity {
    Expr(HeatmapPaintLayerHeatmapIntensityExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for HeatmapPaintLayerHeatmapIntensity {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum HeatmapPaintLayerHeatmapOpacityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// The global opacity at which the heatmap layer will be drawn.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum HeatmapPaintLayerHeatmapOpacity {
    Expr(HeatmapPaintLayerHeatmapOpacityExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for HeatmapPaintLayerHeatmapOpacity {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum HeatmapPaintLayerHeatmapRadiusExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Radius of influence of one heatmap point in pixels. Increasing the value makes the heatmap smoother, but less detailed.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum HeatmapPaintLayerHeatmapRadius {
    Expr(HeatmapPaintLayerHeatmapRadiusExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for HeatmapPaintLayerHeatmapRadius {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(30)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum HeatmapPaintLayerHeatmapWeightExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// A measure of how much an individual point contributes to the heatmap. A value of 10 would be equivalent to having 10 points of weight 1 in the same spot. Especially useful when combined with clustering.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum HeatmapPaintLayerHeatmapWeight {
    Expr(HeatmapPaintLayerHeatmapWeightExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for HeatmapPaintLayerHeatmapWeight {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct HillshadeLayoutLayer {
    /// Whether this layer is displayed.
    pub visibility: Option<HillshadeLayoutLayerVisibility>,
}

/// Whether this layer is displayed.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum HillshadeLayoutLayerVisibility {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "visible")]
    #[default]
    Visible,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct HillshadePaintLayer {
    /// The shading color used to accentuate rugged terrain like sharp cliffs and gorges.
    #[serde(rename = "hillshade-accent-color")]
    pub hillshade_accent_color: Option<HillshadePaintLayerHillshadeAccentColor>,
    /// Intensity of the hillshade
    #[serde(rename = "hillshade-exaggeration")]
    pub hillshade_exaggeration: Option<HillshadePaintLayerHillshadeExaggeration>,
    /// The shading color of areas that faces towards the light source(s). Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
    #[serde(rename = "hillshade-highlight-color")]
    pub hillshade_highlight_color: Option<HillshadePaintLayerHillshadeHighlightColor>,
    /// The altitude of the light source(s) used to generate the hillshading with 0 as sunset and 90 as noon. Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
    #[serde(rename = "hillshade-illumination-altitude")]
    pub hillshade_illumination_altitude: Option<HillshadePaintLayerHillshadeIlluminationAltitude>,
    /// Direction of light source when map is rotated.
    #[serde(rename = "hillshade-illumination-anchor")]
    pub hillshade_illumination_anchor: Option<HillshadePaintLayerHillshadeIlluminationAnchor>,
    /// The direction of the light source(s) used to generate the hillshading with 0 as the top of the viewport if `hillshade-illumination-anchor` is set to `viewport` and due north if `hillshade-illumination-anchor` is set to `map`. Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
    #[serde(rename = "hillshade-illumination-direction")]
    pub hillshade_illumination_direction: Option<HillshadePaintLayerHillshadeIlluminationDirection>,
    /// The hillshade algorithm to use, one of `standard`, `basic`, `combined`, `igor`, or `multidirectional`. ![image](assets/hillshade_methods.png)
    #[serde(rename = "hillshade-method")]
    pub hillshade_method: Option<HillshadePaintLayerHillshadeMethod>,
    /// The shading color of areas that face away from the light source(s). Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
    #[serde(rename = "hillshade-shadow-color")]
    pub hillshade_shadow_color: Option<HillshadePaintLayerHillshadeShadowColor>,
}

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum HillshadePaintLayerHillshadeAccentColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

/// The shading color used to accentuate rugged terrain like sharp cliffs and gorges.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum HillshadePaintLayerHillshadeAccentColor {
    Expr(HillshadePaintLayerHillshadeAccentColorExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl Default for HillshadePaintLayerHillshadeAccentColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("#000000"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum HillshadePaintLayerHillshadeExaggerationExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Intensity of the hillshade
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum HillshadePaintLayerHillshadeExaggeration {
    Expr(HillshadePaintLayerHillshadeExaggerationExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for HillshadePaintLayerHillshadeExaggeration {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_f64(0.5)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The shading color of areas that faces towards the light source(s). Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum HillshadePaintLayerHillshadeHighlightColor {
    /// A color
    One(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_dynamic_color))]
         color::DynamicColor,
    ),
    /// A set of colors
    Multiple(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_vec_dynamic_color))]
         Vec<color::DynamicColor>,
    ),
}

impl Default for HillshadePaintLayerHillshadeHighlightColor {
    fn default() -> Self {
        Self::One(
            color::parse_color("#FFFFFF").expect("Invalid color specified as the default value"),
        )
    }
}

/// The altitude of the light source(s) used to generate the hillshading with 0 as sunset and 90 as noon. Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum HillshadePaintLayerHillshadeIlluminationAltitude {
    One(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
    Many(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_vec_json_number))]
         Vec<serde_json::Number>,
    ),
}

impl Default for HillshadePaintLayerHillshadeIlluminationAltitude {
    fn default() -> Self {
        Self::One(
            serde_json::Number::from_i128(45)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Direction of light source when map is rotated.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum HillshadePaintLayerHillshadeIlluminationAnchor {
    #[serde(rename = "map")]
    Map,
    #[serde(rename = "viewport")]
    #[default]
    Viewport,
}

/// The direction of the light source(s) used to generate the hillshading with 0 as the top of the viewport if `hillshade-illumination-anchor` is set to `viewport` and due north if `hillshade-illumination-anchor` is set to `map`. Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum HillshadePaintLayerHillshadeIlluminationDirection {
    One(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
    Many(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_vec_json_number))]
         Vec<serde_json::Number>,
    ),
}

impl Default for HillshadePaintLayerHillshadeIlluminationDirection {
    fn default() -> Self {
        Self::One(
            serde_json::Number::from_i128(335)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The hillshade algorithm to use, one of `standard`, `basic`, `combined`, `igor`, or `multidirectional`. ![image](assets/hillshade_methods.png)
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum HillshadePaintLayerHillshadeMethod {
    #[serde(rename = "basic")]
    Basic,
    #[serde(rename = "combined")]
    Combined,
    #[serde(rename = "igor")]
    Igor,
    #[serde(rename = "multidirectional")]
    Multidirectional,
    #[serde(rename = "standard")]
    #[default]
    Standard,
}

/// The shading color of areas that face away from the light source(s). Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum HillshadePaintLayerHillshadeShadowColor {
    /// A color
    One(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_dynamic_color))]
         color::DynamicColor,
    ),
    /// A set of colors
    Multiple(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_vec_dynamic_color))]
         Vec<color::DynamicColor>,
    ),
}

impl Default for HillshadePaintLayerHillshadeShadowColor {
    fn default() -> Self {
        Self::One(
            color::parse_color("#000000").expect("Invalid color specified as the default value"),
        )
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct LineLayoutLayer {
    /// The display of line endings.
    #[serde(rename = "line-cap")]
    pub line_cap: Option<LineLayoutLayerLineCap>,
    /// The display of lines when joining.
    #[serde(rename = "line-join")]
    pub line_join: Option<LineLayoutLayerLineJoin>,
    /// Used to automatically convert miter joins to bevel joins for sharp angles.
    #[serde(rename = "line-miter-limit")]
    pub line_miter_limit: Option<LineLayoutLayerLineMiterLimit>,
    /// Used to automatically convert round joins to miter joins for shallow angles.
    #[serde(rename = "line-round-limit")]
    pub line_round_limit: Option<LineLayoutLayerLineRoundLimit>,
    /// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
    #[serde(rename = "line-sort-key")]
    pub line_sort_key: Option<LineLayoutLayerLineSortKey>,
    /// Whether this layer is displayed.
    pub visibility: Option<LineLayoutLayerVisibility>,
}

/// The display of line endings.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LineLayoutLayerLineCap {
    #[serde(rename = "butt")]
    #[default]
    Butt,
    #[serde(rename = "round")]
    Round,
    #[serde(rename = "square")]
    Square,
}

/// The display of lines when joining.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LineLayoutLayerLineJoin {
    #[serde(rename = "bevel")]
    Bevel,
    #[serde(rename = "miter")]
    #[default]
    Miter,
    #[serde(rename = "round")]
    Round,
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LineLayoutLayerLineMiterLimitExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Used to automatically convert miter joins to bevel joins for sharp angles.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LineLayoutLayerLineMiterLimit {
    Expr(LineLayoutLayerLineMiterLimitExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for LineLayoutLayerLineMiterLimit {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(2)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LineLayoutLayerLineRoundLimitExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Used to automatically convert round joins to miter joins for shallow angles.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LineLayoutLayerLineRoundLimit {
    Expr(LineLayoutLayerLineRoundLimitExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for LineLayoutLayerLineRoundLimit {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_f64(1.05)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LineLayoutLayerLineSortKeyExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LineLayoutLayerLineSortKey {
    Expr(LineLayoutLayerLineSortKeyExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

/// Whether this layer is displayed.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LineLayoutLayerVisibility {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "visible")]
    #[default]
    Visible,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct LinePaintLayer {
    /// Blur applied to the line, in pixels.
    #[serde(rename = "line-blur")]
    pub line_blur: Option<LinePaintLayerLineBlur>,
    /// The color with which the line will be drawn.
    #[serde(rename = "line-color")]
    pub line_color: Option<LinePaintLayerLineColor>,
    /// Specifies the lengths of the alternating dashes and gaps that form the dash pattern. The lengths are later scaled by the line width. To convert a dash length to pixels, multiply the length by the current line width. GeoJSON sources with `lineMetrics: true` specified won't render dashed lines to the expected scale. Zoom-dependent expressions will be evaluated only at integer zoom levels. The only way to create an array value is using `["literal", [...]]`; arrays cannot be read from or derived from feature properties.
    #[serde(rename = "line-dasharray")]
    pub line_dasharray: Option<LinePaintLayerLineDasharray>,
    /// Draws a line casing outside of a line's actual path. Value indicates the width of the inner gap.
    #[serde(rename = "line-gap-width")]
    pub line_gap_width: Option<LinePaintLayerLineGapWidth>,
    /// Defines a gradient with which to color a line feature. Can only be used with GeoJSON sources that specify `"lineMetrics": true`.
    #[serde(rename = "line-gradient")]
    pub line_gradient: Option<LinePaintLayerLineGradient>,
    /// The line's offset. For linear features, a positive value offsets the line to the right, relative to the direction of the line, and a negative value to the left. For polygon features, a positive value results in an inset, and a negative value results in an outset.
    #[serde(rename = "line-offset")]
    pub line_offset: Option<LinePaintLayerLineOffset>,
    /// The opacity at which the line will be drawn.
    #[serde(rename = "line-opacity")]
    pub line_opacity: Option<LinePaintLayerLineOpacity>,
    /// Name of image in sprite to use for drawing image lines. For seamless patterns, image width must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
    #[serde(rename = "line-pattern")]
    pub line_pattern: Option<LinePaintLayerLinePattern>,
    /// The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.
    #[serde(rename = "line-translate")]
    pub line_translate: Option<LinePaintLayerLineTranslate>,
    /// Controls the frame of reference for `line-translate`.
    #[serde(rename = "line-translate-anchor")]
    pub line_translate_anchor: Option<LinePaintLayerLineTranslateAnchor>,
    /// Stroke thickness.
    #[serde(rename = "line-width")]
    pub line_width: Option<LinePaintLayerLineWidth>,
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LinePaintLayerLineBlurExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Blur applied to the line, in pixels.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LinePaintLayerLineBlur {
    Expr(LinePaintLayerLineBlurExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for LinePaintLayerLineBlur {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LinePaintLayerLineColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

/// The color with which the line will be drawn.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LinePaintLayerLineColor {
    Expr(LinePaintLayerLineColorExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl Default for LinePaintLayerLineColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("#000000"))
    }
}

/// Specifies the lengths of the alternating dashes and gaps that form the dash pattern. The lengths are later scaled by the line width. To convert a dash length to pixels, multiply the length by the current line width. GeoJSON sources with `lineMetrics: true` specified won't render dashed lines to the expected scale. Zoom-dependent expressions will be evaluated only at integer zoom levels. The only way to create an array value is using `["literal", [...]]`; arrays cannot be read from or derived from feature properties.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct LinePaintLayerLineDasharray(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_vec_json_number))]
    Vec<serde_json::Number>,
);

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LinePaintLayerLineGapWidthExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Draws a line casing outside of a line's actual path. Value indicates the width of the inner gap.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LinePaintLayerLineGapWidth {
    Expr(LinePaintLayerLineGapWidthExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for LinePaintLayerLineGapWidth {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LinePaintLayerLineGradientExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

/// Defines a gradient with which to color a line feature. Can only be used with GeoJSON sources that specify `"lineMetrics": true`.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LinePaintLayerLineGradient {
    Expr(LinePaintLayerLineGradientExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LinePaintLayerLineOffsetExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// The line's offset. For linear features, a positive value offsets the line to the right, relative to the direction of the line, and a negative value to the left. For polygon features, a positive value results in an inset, and a negative value results in an outset.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LinePaintLayerLineOffset {
    Expr(LinePaintLayerLineOffsetExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for LinePaintLayerLineOffset {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LinePaintLayerLineOpacityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// The opacity at which the line will be drawn.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LinePaintLayerLineOpacity {
    Expr(LinePaintLayerLineOpacityExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for LinePaintLayerLineOpacity {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Name of image in sprite to use for drawing image lines. For seamless patterns, image width must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct LinePaintLayerLinePattern(std::string::String);

/// The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct LinePaintLayerLineTranslate(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_2_json_number))]
     Box<[serde_json::Number; 2]>,
);

impl Default for LinePaintLayerLineTranslate {
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
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LinePaintLayerLineTranslateAnchor {
    #[serde(rename = "map")]
    #[default]
    Map,
    #[serde(rename = "viewport")]
    Viewport,
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LinePaintLayerLineWidthExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Stroke thickness.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum LinePaintLayerLineWidth {
    Expr(LinePaintLayerLineWidthExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for LinePaintLayerLineWidth {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterLayoutLayer {
    /// Whether this layer is displayed.
    pub visibility: Option<RasterLayoutLayerVisibility>,
}

/// Whether this layer is displayed.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum RasterLayoutLayerVisibility {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "visible")]
    #[default]
    Visible,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterPaintLayer {
    /// Increase or reduce the brightness of the image. The value is the maximum brightness.
    #[serde(rename = "raster-brightness-max")]
    pub raster_brightness_max: Option<RasterPaintLayerRasterBrightnessMax>,
    /// Increase or reduce the brightness of the image. The value is the minimum brightness.
    #[serde(rename = "raster-brightness-min")]
    pub raster_brightness_min: Option<RasterPaintLayerRasterBrightnessMin>,
    /// Increase or reduce the contrast of the image.
    #[serde(rename = "raster-contrast")]
    pub raster_contrast: Option<RasterPaintLayerRasterContrast>,
    /// Fade duration when a new tile is added, or when a video is started or its coordinates are updated.
    #[serde(rename = "raster-fade-duration")]
    pub raster_fade_duration: Option<RasterPaintLayerRasterFadeDuration>,
    /// Rotates hues around the color wheel.
    #[serde(rename = "raster-hue-rotate")]
    pub raster_hue_rotate: Option<RasterPaintLayerRasterHueRotate>,
    /// The opacity at which the image will be drawn.
    #[serde(rename = "raster-opacity")]
    pub raster_opacity: Option<RasterPaintLayerRasterOpacity>,
    /// The resampling/interpolation method to use for overscaling, also known as texture magnification filter
    #[serde(rename = "raster-resampling")]
    pub raster_resampling: Option<RasterPaintLayerRasterResampling>,
    /// Increase or reduce the saturation of the image.
    #[serde(rename = "raster-saturation")]
    pub raster_saturation: Option<RasterPaintLayerRasterSaturation>,
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum RasterPaintLayerRasterBrightnessMaxExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Increase or reduce the brightness of the image. The value is the maximum brightness.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum RasterPaintLayerRasterBrightnessMax {
    Expr(RasterPaintLayerRasterBrightnessMaxExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for RasterPaintLayerRasterBrightnessMax {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum RasterPaintLayerRasterBrightnessMinExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Increase or reduce the brightness of the image. The value is the minimum brightness.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum RasterPaintLayerRasterBrightnessMin {
    Expr(RasterPaintLayerRasterBrightnessMinExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for RasterPaintLayerRasterBrightnessMin {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum RasterPaintLayerRasterContrastExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Increase or reduce the contrast of the image.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum RasterPaintLayerRasterContrast {
    Expr(RasterPaintLayerRasterContrastExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for RasterPaintLayerRasterContrast {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum RasterPaintLayerRasterFadeDurationExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Fade duration when a new tile is added, or when a video is started or its coordinates are updated.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum RasterPaintLayerRasterFadeDuration {
    Expr(RasterPaintLayerRasterFadeDurationExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for RasterPaintLayerRasterFadeDuration {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(300)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum RasterPaintLayerRasterHueRotateExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Rotates hues around the color wheel.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum RasterPaintLayerRasterHueRotate {
    Expr(RasterPaintLayerRasterHueRotateExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for RasterPaintLayerRasterHueRotate {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum RasterPaintLayerRasterOpacityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// The opacity at which the image will be drawn.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum RasterPaintLayerRasterOpacity {
    Expr(RasterPaintLayerRasterOpacityExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for RasterPaintLayerRasterOpacity {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The resampling/interpolation method to use for overscaling, also known as texture magnification filter
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum RasterPaintLayerRasterResampling {
    #[serde(rename = "linear")]
    #[default]
    Linear,
    #[serde(rename = "nearest")]
    Nearest,
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum RasterPaintLayerRasterSaturationExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Increase or reduce the saturation of the image.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum RasterPaintLayerRasterSaturation {
    Expr(RasterPaintLayerRasterSaturationExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for RasterPaintLayerRasterSaturation {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayer {
    /// If true, the icon will be visible even if it collides with other previously drawn symbols.
    #[serde(rename = "icon-allow-overlap")]
    pub icon_allow_overlap: Option<SymbolLayoutLayerIconAllowOverlap>,
    /// Part of the icon placed closest to the anchor.
    #[serde(rename = "icon-anchor")]
    pub icon_anchor: Option<SymbolLayoutLayerIconAnchor>,
    /// If true, other symbols can be visible even if they collide with the icon.
    #[serde(rename = "icon-ignore-placement")]
    pub icon_ignore_placement: Option<SymbolLayoutLayerIconIgnorePlacement>,
    /// Name of image in sprite to use for drawing an image background.
    #[serde(rename = "icon-image")]
    pub icon_image: Option<SymbolLayoutLayerIconImage>,
    /// If true, the icon may be flipped to prevent it from being rendered upside-down.
    #[serde(rename = "icon-keep-upright")]
    pub icon_keep_upright: Option<SymbolLayoutLayerIconKeepUpright>,
    /// Offset distance of icon from its anchor. Positive values indicate right and down, while negative values indicate left and up. Each component is multiplied by the value of `icon-size` to obtain the final offset in pixels. When combined with `icon-rotate` the offset will be as if the rotated direction was up.
    #[serde(rename = "icon-offset")]
    pub icon_offset: Option<SymbolLayoutLayerIconOffset>,
    /// If true, text will display without their corresponding icons when the icon collides with other symbols and the text does not.
    #[serde(rename = "icon-optional")]
    pub icon_optional: Option<SymbolLayoutLayerIconOptional>,
    /// Allows for control over whether to show an icon when it overlaps other symbols on the map. If `icon-overlap` is not set, `icon-allow-overlap` is used instead.
    #[serde(rename = "icon-overlap")]
    pub icon_overlap: Option<SymbolLayoutLayerIconOverlap>,
    /// Size of additional area round the icon bounding box used for detecting symbol collisions.
    #[serde(rename = "icon-padding")]
    pub icon_padding: Option<SymbolLayoutLayerIconPadding>,
    /// Orientation of icon when map is pitched.
    #[serde(rename = "icon-pitch-alignment")]
    pub icon_pitch_alignment: Option<SymbolLayoutLayerIconPitchAlignment>,
    /// Rotates the icon clockwise.
    #[serde(rename = "icon-rotate")]
    pub icon_rotate: Option<SymbolLayoutLayerIconRotate>,
    /// In combination with `symbol-placement`, determines the rotation behavior of icons.
    #[serde(rename = "icon-rotation-alignment")]
    pub icon_rotation_alignment: Option<SymbolLayoutLayerIconRotationAlignment>,
    /// Scales the original size of the icon by the provided factor. The new pixel size of the image will be the original pixel size multiplied by `icon-size`. 1 is the original size; 3 triples the size of the image.
    #[serde(rename = "icon-size")]
    pub icon_size: Option<SymbolLayoutLayerIconSize>,
    /// Scales the icon to fit around the associated text.
    #[serde(rename = "icon-text-fit")]
    pub icon_text_fit: Option<SymbolLayoutLayerIconTextFit>,
    /// Size of the additional area added to dimensions determined by `icon-text-fit`, in clockwise order: top, right, bottom, left.
    #[serde(rename = "icon-text-fit-padding")]
    pub icon_text_fit_padding: Option<SymbolLayoutLayerIconTextFitPadding>,
    /// If true, the symbols will not cross tile edges to avoid mutual collisions. Recommended in layers that don't have enough padding in the vector tile to prevent collisions, or if it is a point symbol layer placed after a line symbol layer. When using a client that supports global collision detection, like MapLibre GL JS version 0.42.0 or greater, enabling this property is not needed to prevent clipped labels at tile boundaries.
    #[serde(rename = "symbol-avoid-edges")]
    pub symbol_avoid_edges: Option<SymbolLayoutLayerSymbolAvoidEdges>,
    /// Label placement relative to its geometry.
    #[serde(rename = "symbol-placement")]
    pub symbol_placement: Option<SymbolLayoutLayerSymbolPlacement>,
    /// Sorts features in ascending order based on this value. Features with lower sort keys are drawn and placed first.  When `icon-allow-overlap` or `text-allow-overlap` is `false`, features with a lower sort key will have priority during placement. When `icon-allow-overlap` or `text-allow-overlap` is set to `true`, features with a higher sort key will overlap over features with a lower sort key.
    #[serde(rename = "symbol-sort-key")]
    pub symbol_sort_key: Option<SymbolLayoutLayerSymbolSortKey>,
    /// Distance between two symbol anchors.
    #[serde(rename = "symbol-spacing")]
    pub symbol_spacing: Option<SymbolLayoutLayerSymbolSpacing>,
    /// Determines whether overlapping symbols in the same layer are rendered in the order that they appear in the data source or by their y-position relative to the viewport. To control the order and prioritization of symbols otherwise, use `symbol-sort-key`.
    #[serde(rename = "symbol-z-order")]
    pub symbol_z_order: Option<SymbolLayoutLayerSymbolZOrder>,
    /// If true, the text will be visible even if it collides with other previously drawn symbols.
    #[serde(rename = "text-allow-overlap")]
    pub text_allow_overlap: Option<SymbolLayoutLayerTextAllowOverlap>,
    /// Part of the text placed closest to the anchor.
    #[serde(rename = "text-anchor")]
    pub text_anchor: Option<SymbolLayoutLayerTextAnchor>,
    /// Value to use for a text label. If a plain `string` is provided, it will be treated as a `formatted` with default/inherited formatting options.
    #[serde(rename = "text-field")]
    pub text_field: Option<SymbolLayoutLayerTextField>,
    /// Fonts to use for displaying text. If the `glyphs` root property is specified, this array is joined together and interpreted as a font stack name. Otherwise, it is interpreted as a cascading fallback list of local font names.
    #[serde(rename = "text-font")]
    pub text_font: Option<SymbolLayoutLayerTextFont>,
    /// If true, other symbols can be visible even if they collide with the text.
    #[serde(rename = "text-ignore-placement")]
    pub text_ignore_placement: Option<SymbolLayoutLayerTextIgnorePlacement>,
    /// Text justification options.
    #[serde(rename = "text-justify")]
    pub text_justify: Option<SymbolLayoutLayerTextJustify>,
    /// If true, the text may be flipped vertically to prevent it from being rendered upside-down.
    #[serde(rename = "text-keep-upright")]
    pub text_keep_upright: Option<SymbolLayoutLayerTextKeepUpright>,
    /// Text tracking amount.
    #[serde(rename = "text-letter-spacing")]
    pub text_letter_spacing: Option<SymbolLayoutLayerTextLetterSpacing>,
    /// Text leading value for multi-line text.
    #[serde(rename = "text-line-height")]
    pub text_line_height: Option<SymbolLayoutLayerTextLineHeight>,
    /// Maximum angle change between adjacent characters.
    #[serde(rename = "text-max-angle")]
    pub text_max_angle: Option<SymbolLayoutLayerTextMaxAngle>,
    /// The maximum line width for text wrapping.
    #[serde(rename = "text-max-width")]
    pub text_max_width: Option<SymbolLayoutLayerTextMaxWidth>,
    /// Offset distance of text from its anchor. Positive values indicate right and down, while negative values indicate left and up. If used with text-variable-anchor, input values will be taken as absolute values. Offsets along the x- and y-axis will be applied automatically based on the anchor position.
    #[serde(rename = "text-offset")]
    pub text_offset: Option<SymbolLayoutLayerTextOffset>,
    /// If true, icons will display without their corresponding text when the text collides with other symbols and the icon does not.
    #[serde(rename = "text-optional")]
    pub text_optional: Option<SymbolLayoutLayerTextOptional>,
    /// Allows for control over whether to show symbol text when it overlaps other symbols on the map. If `text-overlap` is not set, `text-allow-overlap` is used instead
    #[serde(rename = "text-overlap")]
    pub text_overlap: Option<SymbolLayoutLayerTextOverlap>,
    /// Size of the additional area around the text bounding box used for detecting symbol collisions.
    #[serde(rename = "text-padding")]
    pub text_padding: Option<SymbolLayoutLayerTextPadding>,
    /// Orientation of text when map is pitched.
    #[serde(rename = "text-pitch-alignment")]
    pub text_pitch_alignment: Option<SymbolLayoutLayerTextPitchAlignment>,
    /// Radial offset of text, in the direction of the symbol's anchor. Useful in combination with `text-variable-anchor`, which defaults to using the two-dimensional `text-offset` if present.
    #[serde(rename = "text-radial-offset")]
    pub text_radial_offset: Option<SymbolLayoutLayerTextRadialOffset>,
    /// Rotates the text clockwise.
    #[serde(rename = "text-rotate")]
    pub text_rotate: Option<SymbolLayoutLayerTextRotate>,
    /// In combination with `symbol-placement`, determines the rotation behavior of the individual glyphs forming the text.
    #[serde(rename = "text-rotation-alignment")]
    pub text_rotation_alignment: Option<SymbolLayoutLayerTextRotationAlignment>,
    /// Font size.
    #[serde(rename = "text-size")]
    pub text_size: Option<SymbolLayoutLayerTextSize>,
    /// Specifies how to capitalize text, similar to the CSS `text-transform` property.
    #[serde(rename = "text-transform")]
    pub text_transform: Option<SymbolLayoutLayerTextTransform>,
    /// To increase the chance of placing high-priority labels on the map, you can provide an array of `text-anchor` locations: the renderer will attempt to place the label at each location, in order, before moving onto the next label. Use `text-justify: auto` to choose justification based on anchor position. To apply an offset, use the `text-radial-offset` or the two-dimensional `text-offset`.
    #[serde(rename = "text-variable-anchor")]
    pub text_variable_anchor: Option<SymbolLayoutLayerTextVariableAnchor>,
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
    pub text_variable_anchor_offset: Option<SymbolLayoutLayerTextVariableAnchorOffset>,
    /// The property allows control over a symbol's orientation. Note that the property values act as a hint, so that a symbol whose language doesn’t support the provided orientation will be laid out in its natural orientation. Example: English point symbol will be rendered horizontally even if array value contains single 'vertical' enum value. The order of elements in an array define priority order for the placement of an orientation variant.
    #[serde(rename = "text-writing-mode")]
    pub text_writing_mode: Option<SymbolLayoutLayerTextWritingMode>,
    /// Whether this layer is displayed.
    pub visibility: Option<SymbolLayoutLayerVisibility>,
}

/// If true, the icon will be visible even if it collides with other previously drawn symbols.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerIconAllowOverlap(bool);

impl Default for SymbolLayoutLayerIconAllowOverlap {
    fn default() -> Self {
        Self(false)
    }
}

/// Part of the icon placed closest to the anchor.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerIconAnchor {
    #[serde(rename = "bottom")]
    Bottom,
    #[serde(rename = "bottom-left")]
    BottomLeft,
    #[serde(rename = "bottom-right")]
    BottomRight,
    #[serde(rename = "center")]
    #[default]
    Center,
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "right")]
    Right,
    #[serde(rename = "top")]
    Top,
    #[serde(rename = "top-left")]
    TopLeft,
    #[serde(rename = "top-right")]
    TopRight,
}

/// If true, other symbols can be visible even if they collide with the icon.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerIconIgnorePlacement(bool);

impl Default for SymbolLayoutLayerIconIgnorePlacement {
    fn default() -> Self {
        Self(false)
    }
}

/// Name of image in sprite to use for drawing an image background.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerIconImage(std::string::String);

/// If true, the icon may be flipped to prevent it from being rendered upside-down.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerIconKeepUpright(bool);

impl Default for SymbolLayoutLayerIconKeepUpright {
    fn default() -> Self {
        Self(false)
    }
}

/// Offset distance of icon from its anchor. Positive values indicate right and down, while negative values indicate left and up. Each component is multiplied by the value of `icon-size` to obtain the final offset in pixels. When combined with `icon-rotate` the offset will be as if the rotated direction was up.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerIconOffset(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_2_json_number))]
     Box<[serde_json::Number; 2]>,
);

impl Default for SymbolLayoutLayerIconOffset {
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
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerIconOptional(bool);

impl Default for SymbolLayoutLayerIconOptional {
    fn default() -> Self {
        Self(false)
    }
}

/// Allows for control over whether to show an icon when it overlaps other symbols on the map. If `icon-overlap` is not set, `icon-allow-overlap` is used instead.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerIconOverlap {
    #[serde(rename = "always")]
    Always,
    #[serde(rename = "cooperative")]
    Cooperative,
    #[serde(rename = "never")]
    Never,
}

/// Size of additional area round the icon bounding box used for detecting symbol collisions.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerIconPadding {
    /// A single value applies to all four sides
    One(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_1_json_number))]
         Box<[serde_json::Number; 1]>,
    ),
    /// two values apply to `[top/bottom, left/right]`
    Two(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_2_json_number))]
         Box<[serde_json::Number; 2]>,
    ),
    /// three values apply to `[top, left/right, bottom]`
    Three(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_3_json_number))]
         Box<[serde_json::Number; 3]>,
    ),
    /// four values apply to `[top, right, bottom, left]`
    Four(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_4_json_number))]
         Box<[serde_json::Number; 4]>,
    ),
}

impl Default for SymbolLayoutLayerIconPadding {
    fn default() -> Self {
        Self::One(Box::new([2.into()]))
    }
}

/// Orientation of icon when map is pitched.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerIconPitchAlignment {
    #[serde(rename = "auto")]
    #[default]
    Auto,
    #[serde(rename = "map")]
    Map,
    #[serde(rename = "viewport")]
    Viewport,
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerIconRotateExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Rotates the icon clockwise.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerIconRotate {
    Expr(SymbolLayoutLayerIconRotateExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for SymbolLayoutLayerIconRotate {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// In combination with `symbol-placement`, determines the rotation behavior of icons.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerIconRotationAlignment {
    #[serde(rename = "auto")]
    #[default]
    Auto,
    #[serde(rename = "map")]
    Map,
    #[serde(rename = "viewport")]
    Viewport,
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerIconSizeExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Scales the original size of the icon by the provided factor. The new pixel size of the image will be the original pixel size multiplied by `icon-size`. 1 is the original size; 3 triples the size of the image.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerIconSize {
    Expr(SymbolLayoutLayerIconSizeExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for SymbolLayoutLayerIconSize {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Scales the icon to fit around the associated text.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerIconTextFit {
    #[serde(rename = "both")]
    Both,
    #[serde(rename = "height")]
    Height,
    #[serde(rename = "none")]
    #[default]
    None,
    #[serde(rename = "width")]
    Width,
}

/// Size of the additional area added to dimensions determined by `icon-text-fit`, in clockwise order: top, right, bottom, left.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerIconTextFitPadding(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_4_json_number))]
     Box<[serde_json::Number; 4]>,
);

impl Default for SymbolLayoutLayerIconTextFitPadding {
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
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerSymbolAvoidEdges(bool);

impl Default for SymbolLayoutLayerSymbolAvoidEdges {
    fn default() -> Self {
        Self(false)
    }
}

/// Label placement relative to its geometry.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerSymbolPlacement {
    #[serde(rename = "line")]
    Line,
    #[serde(rename = "line-center")]
    LineCenter,
    #[serde(rename = "point")]
    #[default]
    Point,
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerSymbolSortKeyExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Sorts features in ascending order based on this value. Features with lower sort keys are drawn and placed first.  When `icon-allow-overlap` or `text-allow-overlap` is `false`, features with a lower sort key will have priority during placement. When `icon-allow-overlap` or `text-allow-overlap` is set to `true`, features with a higher sort key will overlap over features with a lower sort key.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerSymbolSortKey {
    Expr(SymbolLayoutLayerSymbolSortKeyExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerSymbolSpacingExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Distance between two symbol anchors.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerSymbolSpacing {
    Expr(SymbolLayoutLayerSymbolSpacingExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for SymbolLayoutLayerSymbolSpacing {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(250)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Determines whether overlapping symbols in the same layer are rendered in the order that they appear in the data source or by their y-position relative to the viewport. To control the order and prioritization of symbols otherwise, use `symbol-sort-key`.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerSymbolZOrder {
    #[serde(rename = "auto")]
    #[default]
    Auto,
    #[serde(rename = "source")]
    Source,
    #[serde(rename = "viewport-y")]
    ViewportY,
}

/// If true, the text will be visible even if it collides with other previously drawn symbols.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerTextAllowOverlap(bool);

impl Default for SymbolLayoutLayerTextAllowOverlap {
    fn default() -> Self {
        Self(false)
    }
}

/// Part of the text placed closest to the anchor.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextAnchor {
    #[serde(rename = "bottom")]
    Bottom,
    #[serde(rename = "bottom-left")]
    BottomLeft,
    #[serde(rename = "bottom-right")]
    BottomRight,
    #[serde(rename = "center")]
    #[default]
    Center,
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "right")]
    Right,
    #[serde(rename = "top")]
    Top,
    #[serde(rename = "top-left")]
    TopLeft,
    #[serde(rename = "top-right")]
    TopRight,
}

/// Value to use for a text label. If a plain `string` is provided, it will be treated as a `formatted` with default/inherited formatting options.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerTextField(std::string::String);

impl Default for SymbolLayoutLayerTextField {
    fn default() -> Self {
        Self("".to_string())
    }
}

/// Fonts to use for displaying text. If the `glyphs` root property is specified, this array is joined together and interpreted as a font stack name. Otherwise, it is interpreted as a cascading fallback list of local font names.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerTextFont(Vec<std::string::String>);

impl Default for SymbolLayoutLayerTextFont {
    fn default() -> Self {
        Self(Vec::from([
            "Open Sans Regular".to_string(),
            "Arial Unicode MS Regular".to_string(),
        ]))
    }
}

/// If true, other symbols can be visible even if they collide with the text.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerTextIgnorePlacement(bool);

impl Default for SymbolLayoutLayerTextIgnorePlacement {
    fn default() -> Self {
        Self(false)
    }
}

/// Text justification options.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextJustify {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "center")]
    #[default]
    Center,
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "right")]
    Right,
}

/// If true, the text may be flipped vertically to prevent it from being rendered upside-down.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerTextKeepUpright(bool);

impl Default for SymbolLayoutLayerTextKeepUpright {
    fn default() -> Self {
        Self(true)
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerTextLetterSpacingExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Text tracking amount.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerTextLetterSpacing {
    Expr(SymbolLayoutLayerTextLetterSpacingExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for SymbolLayoutLayerTextLetterSpacing {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerTextLineHeightExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Text leading value for multi-line text.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerTextLineHeight {
    Expr(SymbolLayoutLayerTextLineHeightExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for SymbolLayoutLayerTextLineHeight {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_f64(1.2)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerTextMaxAngleExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Maximum angle change between adjacent characters.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerTextMaxAngle {
    Expr(SymbolLayoutLayerTextMaxAngleExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for SymbolLayoutLayerTextMaxAngle {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(45)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerTextMaxWidthExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// The maximum line width for text wrapping.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerTextMaxWidth {
    Expr(SymbolLayoutLayerTextMaxWidthExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for SymbolLayoutLayerTextMaxWidth {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(10)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Offset distance of text from its anchor. Positive values indicate right and down, while negative values indicate left and up. If used with text-variable-anchor, input values will be taken as absolute values. Offsets along the x- and y-axis will be applied automatically based on the anchor position.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerTextOffset(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_2_json_number))]
     Box<[serde_json::Number; 2]>,
);

impl Default for SymbolLayoutLayerTextOffset {
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
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerTextOptional(bool);

impl Default for SymbolLayoutLayerTextOptional {
    fn default() -> Self {
        Self(false)
    }
}

/// Allows for control over whether to show symbol text when it overlaps other symbols on the map. If `text-overlap` is not set, `text-allow-overlap` is used instead
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextOverlap {
    #[serde(rename = "always")]
    Always,
    #[serde(rename = "cooperative")]
    Cooperative,
    #[serde(rename = "never")]
    Never,
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerTextPaddingExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Size of the additional area around the text bounding box used for detecting symbol collisions.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerTextPadding {
    Expr(SymbolLayoutLayerTextPaddingExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for SymbolLayoutLayerTextPadding {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(2)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Orientation of text when map is pitched.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextPitchAlignment {
    #[serde(rename = "auto")]
    #[default]
    Auto,
    #[serde(rename = "map")]
    Map,
    #[serde(rename = "viewport")]
    Viewport,
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerTextRadialOffsetExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Radial offset of text, in the direction of the symbol's anchor. Useful in combination with `text-variable-anchor`, which defaults to using the two-dimensional `text-offset` if present.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerTextRadialOffset {
    Expr(SymbolLayoutLayerTextRadialOffsetExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for SymbolLayoutLayerTextRadialOffset {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerTextRotateExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Rotates the text clockwise.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerTextRotate {
    Expr(SymbolLayoutLayerTextRotateExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for SymbolLayoutLayerTextRotate {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// In combination with `symbol-placement`, determines the rotation behavior of the individual glyphs forming the text.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextRotationAlignment {
    #[serde(rename = "auto")]
    #[default]
    Auto,
    #[serde(rename = "map")]
    Map,
    #[serde(rename = "viewport")]
    Viewport,
    #[serde(rename = "viewport-glyph")]
    ViewportGlyph,
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerTextSizeExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Font size.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolLayoutLayerTextSize {
    Expr(SymbolLayoutLayerTextSizeExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for SymbolLayoutLayerTextSize {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(16)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Specifies how to capitalize text, similar to the CSS `text-transform` property.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextTransform {
    #[serde(rename = "lowercase")]
    Lowercase,
    #[serde(rename = "none")]
    #[default]
    None,
    #[serde(rename = "uppercase")]
    Uppercase,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextVariableAnchorValue {
    #[serde(rename = "bottom")]
    Bottom,
    #[serde(rename = "bottom-left")]
    BottomLeft,
    #[serde(rename = "bottom-right")]
    BottomRight,
    #[serde(rename = "center")]
    Center,
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "right")]
    Right,
    #[serde(rename = "top")]
    Top,
    #[serde(rename = "top-left")]
    TopLeft,
    #[serde(rename = "top-right")]
    TopRight,
}

/// To increase the chance of placing high-priority labels on the map, you can provide an array of `text-anchor` locations: the renderer will attempt to place the label at each location, in order, before moving onto the next label. Use `text-justify: auto` to choose justification based on anchor position. To apply an offset, use the `text-radial-offset` or the two-dimensional `text-offset`.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerTextVariableAnchor(Vec<SymbolLayoutLayerTextVariableAnchorValue>);

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
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerTextVariableAnchorOffset(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
    serde_json::Value,
);

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextWritingModeValue {
    #[serde(rename = "horizontal")]
    Horizontal,
    #[serde(rename = "vertical")]
    Vertical,
}

/// The property allows control over a symbol's orientation. Note that the property values act as a hint, so that a symbol whose language doesn’t support the provided orientation will be laid out in its natural orientation. Example: English point symbol will be rendered horizontally even if array value contains single 'vertical' enum value. The order of elements in an array define priority order for the placement of an orientation variant.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerTextWritingMode(Vec<SymbolLayoutLayerTextWritingModeValue>);

/// Whether this layer is displayed.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerVisibility {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "visible")]
    #[default]
    Visible,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolPaintLayer {
    /// The color of the icon. This can only be used with SDF icons.
    #[serde(rename = "icon-color")]
    pub icon_color: Option<SymbolPaintLayerIconColor>,
    /// Fade out the halo towards the outside.
    #[serde(rename = "icon-halo-blur")]
    pub icon_halo_blur: Option<SymbolPaintLayerIconHaloBlur>,
    /// The color of the icon's halo. Icon halos can only be used with SDF icons.
    #[serde(rename = "icon-halo-color")]
    pub icon_halo_color: Option<SymbolPaintLayerIconHaloColor>,
    /// Distance of halo to the icon outline.
    ///
    /// The unit is in pixels only for SDF sprites that were created with a blur radius of 8, multiplied by the display density. I.e., the radius needs to be 16 for `@2x` sprites, etc.
    #[serde(rename = "icon-halo-width")]
    pub icon_halo_width: Option<SymbolPaintLayerIconHaloWidth>,
    /// The opacity at which the icon will be drawn.
    #[serde(rename = "icon-opacity")]
    pub icon_opacity: Option<SymbolPaintLayerIconOpacity>,
    /// Distance that the icon's anchor is moved from its original placement. Positive values indicate right and down, while negative values indicate left and up.
    #[serde(rename = "icon-translate")]
    pub icon_translate: Option<SymbolPaintLayerIconTranslate>,
    /// Controls the frame of reference for `icon-translate`.
    #[serde(rename = "icon-translate-anchor")]
    pub icon_translate_anchor: Option<SymbolPaintLayerIconTranslateAnchor>,
    /// The color with which the text will be drawn.
    #[serde(rename = "text-color")]
    pub text_color: Option<SymbolPaintLayerTextColor>,
    /// The halo's fadeout distance towards the outside.
    #[serde(rename = "text-halo-blur")]
    pub text_halo_blur: Option<SymbolPaintLayerTextHaloBlur>,
    /// The color of the text's halo, which helps it stand out from backgrounds.
    #[serde(rename = "text-halo-color")]
    pub text_halo_color: Option<SymbolPaintLayerTextHaloColor>,
    /// Distance of halo to the font outline. Max text halo width is 1/4 of the font-size.
    #[serde(rename = "text-halo-width")]
    pub text_halo_width: Option<SymbolPaintLayerTextHaloWidth>,
    /// The opacity at which the text will be drawn.
    #[serde(rename = "text-opacity")]
    pub text_opacity: Option<SymbolPaintLayerTextOpacity>,
    /// Distance that the text's anchor is moved from its original placement. Positive values indicate right and down, while negative values indicate left and up.
    #[serde(rename = "text-translate")]
    pub text_translate: Option<SymbolPaintLayerTextTranslate>,
    /// Controls the frame of reference for `text-translate`.
    #[serde(rename = "text-translate-anchor")]
    pub text_translate_anchor: Option<SymbolPaintLayerTextTranslateAnchor>,
}

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolPaintLayerIconColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

/// The color of the icon. This can only be used with SDF icons.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolPaintLayerIconColor {
    Expr(SymbolPaintLayerIconColorExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl Default for SymbolPaintLayerIconColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("#000000"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolPaintLayerIconHaloBlurExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Fade out the halo towards the outside.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolPaintLayerIconHaloBlur {
    Expr(SymbolPaintLayerIconHaloBlurExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for SymbolPaintLayerIconHaloBlur {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolPaintLayerIconHaloColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

/// The color of the icon's halo. Icon halos can only be used with SDF icons.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolPaintLayerIconHaloColor {
    Expr(SymbolPaintLayerIconHaloColorExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl Default for SymbolPaintLayerIconHaloColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("rgba(0, 0, 0, 0)"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolPaintLayerIconHaloWidthExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Distance of halo to the icon outline.
///
/// The unit is in pixels only for SDF sprites that were created with a blur radius of 8, multiplied by the display density. I.e., the radius needs to be 16 for `@2x` sprites, etc.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolPaintLayerIconHaloWidth {
    Expr(SymbolPaintLayerIconHaloWidthExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for SymbolPaintLayerIconHaloWidth {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolPaintLayerIconOpacityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// The opacity at which the icon will be drawn.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolPaintLayerIconOpacity {
    Expr(SymbolPaintLayerIconOpacityExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for SymbolPaintLayerIconOpacity {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Distance that the icon's anchor is moved from its original placement. Positive values indicate right and down, while negative values indicate left and up.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolPaintLayerIconTranslate(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_2_json_number))]
     Box<[serde_json::Number; 2]>,
);

impl Default for SymbolPaintLayerIconTranslate {
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
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolPaintLayerIconTranslateAnchor {
    #[serde(rename = "map")]
    #[default]
    Map,
    #[serde(rename = "viewport")]
    Viewport,
}

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolPaintLayerTextColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

/// The color with which the text will be drawn.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolPaintLayerTextColor {
    Expr(SymbolPaintLayerTextColorExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl Default for SymbolPaintLayerTextColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("#000000"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolPaintLayerTextHaloBlurExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// The halo's fadeout distance towards the outside.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolPaintLayerTextHaloBlur {
    Expr(SymbolPaintLayerTextHaloBlurExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for SymbolPaintLayerTextHaloBlur {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolPaintLayerTextHaloColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

/// The color of the text's halo, which helps it stand out from backgrounds.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolPaintLayerTextHaloColor {
    Expr(SymbolPaintLayerTextHaloColorExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl Default for SymbolPaintLayerTextHaloColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("rgba(0, 0, 0, 0)"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolPaintLayerTextHaloWidthExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// Distance of halo to the font outline. Max text halo width is 1/4 of the font-size.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolPaintLayerTextHaloWidth {
    Expr(SymbolPaintLayerTextHaloWidthExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for SymbolPaintLayerTextHaloWidth {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolPaintLayerTextOpacityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

/// The opacity at which the text will be drawn.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum SymbolPaintLayerTextOpacity {
    Expr(SymbolPaintLayerTextOpacityExpression),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl Default for SymbolPaintLayerTextOpacity {
    fn default() -> Self {
        Self::Literal(
            serde_json::Number::from_i128(1)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Distance that the text's anchor is moved from its original placement. Positive values indicate right and down, while negative values indicate left and up.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolPaintLayerTextTranslate(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_2_json_number))]
     Box<[serde_json::Number; 2]>,
);

impl Default for SymbolPaintLayerTextTranslate {
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
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolPaintLayerTextTranslateAnchor {
    #[serde(rename = "map")]
    #[default]
    Map,
    #[serde(rename = "viewport")]
    Viewport,
}

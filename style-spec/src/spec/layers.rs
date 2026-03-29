#![allow(clippy::large_enum_variant, clippy::type_complexity)]
#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use crate::{array_prop, boolean_prop, color_prop, formatted_prop, numeric_prop, string_prop};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct Layer {
    /// A expression specifying conditions on source features. Only features that match the filter are displayed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<Boolean>,
    /// Unique layer name.
    pub id: LayerId,
    /// The maximum zoom level for the layer. At zoom levels equal to or greater than the maxzoom, the layer will be hidden.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maxzoom: Option<LayerMaxzoom>,
    /// Arbitrary properties useful to track with the layer, but do not influence rendering. Properties should be prefixed to avoid collisions, like 'maplibre:'.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<LayerMetadata>,
    /// The minimum zoom level for the layer. At zoom levels less than the minzoom, the layer will be hidden.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minzoom: Option<LayerMinzoom>,
    /// Name of a source description to be used for this layer. Required for all layer types except `background`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<LayerSource>,
    /// Layer to use from a vector tile source. Required for vector tile sources; prohibited for all other source types, including GeoJSON sources.
    #[serde(rename = "source-layer")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_layer: Option<LayerSourceLayer>,
}

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<BackgroundLayoutLayerVisibility>,
}

pub type BackgroundLayoutLayerVisibility = Visibility;

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct BackgroundPaintLayer {
    /// The color with which the background will be drawn.
    #[serde(rename = "background-color")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_color: Option<BackgroundPaintLayerBackgroundColor>,
    /// The opacity at which the background will be drawn.
    #[serde(rename = "background-opacity")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_opacity: Option<BackgroundPaintLayerBackgroundOpacity>,
    /// Name of image in sprite to use for drawing an image background. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
    #[serde(rename = "background-pattern")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_pattern: Option<BackgroundPaintLayerBackgroundPattern>,
}

color_prop!(
    BackgroundPaintLayerBackgroundColor,
    doc = "The color with which the background will be drawn.",
    default = serde_json::json!("#000000")
);

numeric_prop!(
    BackgroundPaintLayerBackgroundOpacity,
    doc = "The opacity at which the background will be drawn.",
    min = 0_f64,
    max = 1_f64,
    default = serde_json::Number::from_i128(1)
        .expect("the number is serialised from a number and is thus always valid")
);

string_prop!(
    BackgroundPaintLayerBackgroundPattern,
    doc = "Name of image in sprite to use for drawing an image background. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels."
);

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct CircleLayoutLayer {
    /// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
    #[serde(rename = "circle-sort-key")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circle_sort_key: Option<CircleLayoutLayerCircleSortKey>,
    /// Whether this layer is displayed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<CircleLayoutLayerVisibility>,
}

numeric_prop!(
    CircleLayoutLayerCircleSortKey,
    doc = "Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key."
);

pub type CircleLayoutLayerVisibility = Visibility;

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct CirclePaintLayer {
    /// Amount to blur the circle. 1 blurs the circle such that only the centerpoint is full opacity.
    #[serde(rename = "circle-blur")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circle_blur: Option<CirclePaintLayerCircleBlur>,
    /// The fill color of the circle.
    #[serde(rename = "circle-color")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circle_color: Option<CirclePaintLayerCircleColor>,
    /// The opacity at which the circle will be drawn.
    #[serde(rename = "circle-opacity")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circle_opacity: Option<CirclePaintLayerCircleOpacity>,
    /// Orientation of circle when map is pitched.
    #[serde(rename = "circle-pitch-alignment")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circle_pitch_alignment: Option<CirclePaintLayerCirclePitchAlignment>,
    /// Controls the scaling behavior of the circle when the map is pitched.
    #[serde(rename = "circle-pitch-scale")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circle_pitch_scale: Option<CirclePaintLayerCirclePitchScale>,
    /// Circle radius.
    #[serde(rename = "circle-radius")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circle_radius: Option<CirclePaintLayerCircleRadius>,
    /// The stroke color of the circle.
    #[serde(rename = "circle-stroke-color")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circle_stroke_color: Option<CirclePaintLayerCircleStrokeColor>,
    /// The opacity of the circle's stroke.
    #[serde(rename = "circle-stroke-opacity")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circle_stroke_opacity: Option<CirclePaintLayerCircleStrokeOpacity>,
    /// The width of the circle's stroke. Strokes are placed outside of the `circle-radius`.
    #[serde(rename = "circle-stroke-width")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circle_stroke_width: Option<CirclePaintLayerCircleStrokeWidth>,
    /// The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.
    #[serde(rename = "circle-translate")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circle_translate: Option<CirclePaintLayerCircleTranslate>,
    /// Controls the frame of reference for `circle-translate`.
    #[serde(rename = "circle-translate-anchor")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circle_translate_anchor: Option<CirclePaintLayerCircleTranslateAnchor>,
}

numeric_prop!(
    CirclePaintLayerCircleBlur,
    doc = "Amount to blur the circle. 1 blurs the circle such that only the centerpoint is full opacity.",
    default = serde_json::Number::from_i128(0)
        .expect("the number is serialised from a number and is thus always valid")
);

color_prop!(
    CirclePaintLayerCircleColor,
    doc = "The fill color of the circle.",
    default = serde_json::json!("#000000")
);

numeric_prop!(
    CirclePaintLayerCircleOpacity,
    doc = "The opacity at which the circle will be drawn.",
    min = 0_f64,
    max = 1_f64,
    default = serde_json::Number::from_i128(1)
        .expect("the number is serialised from a number and is thus always valid")
);

string_prop!(
    CirclePaintLayerCirclePitchAlignment,
    doc = "Orientation of circle when map is pitched.",
    default = "viewport".to_string()
);

string_prop!(
    CirclePaintLayerCirclePitchScale,
    doc = "Controls the scaling behavior of the circle when the map is pitched.",
    default = "map".to_string()
);

numeric_prop!(
    CirclePaintLayerCircleRadius,
    doc = "Circle radius.",
    min = 0_f64,
    default = serde_json::Number::from_i128(5)
        .expect("the number is serialised from a number and is thus always valid")
);

color_prop!(
    CirclePaintLayerCircleStrokeColor,
    doc = "The stroke color of the circle.",
    default = serde_json::json!("#000000")
);

numeric_prop!(
    CirclePaintLayerCircleStrokeOpacity,
    doc = "The opacity of the circle's stroke.",
    min = 0_f64,
    max = 1_f64,
    default = serde_json::Number::from_i128(1)
        .expect("the number is serialised from a number and is thus always valid")
);

numeric_prop!(
    CirclePaintLayerCircleStrokeWidth,
    doc = "The width of the circle's stroke. Strokes are placed outside of the `circle-radius`.",
    min = 0_f64,
    default = serde_json::Number::from_i128(0)
        .expect("the number is serialised from a number and is thus always valid")
);

array_prop!(
    CirclePaintLayerCircleTranslate,
    doc = "The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.",
    default = serde_json::json!([0, 0])
);

string_prop!(
    CirclePaintLayerCircleTranslateAnchor,
    doc = "Controls the frame of reference for `circle-translate`.",
    default = "map".to_string()
);

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct ColorReliefLayoutLayer {
    /// Whether this layer is displayed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<ColorReliefLayoutLayerVisibility>,
}

pub type ColorReliefLayoutLayerVisibility = Visibility;

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct ColorReliefPaintLayer {
    /// Defines the color of each pixel based on its elevation. Should be an expression that uses `["elevation"]` as input.
    #[serde(rename = "color-relief-color")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color_relief_color: Option<ColorReliefPaintLayerColorReliefColor>,
    /// The opacity at which the color-relief will be drawn.
    #[serde(rename = "color-relief-opacity")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color_relief_opacity: Option<ColorReliefPaintLayerColorReliefOpacity>,
    /// The resampling/interpolation method to use for overscaling, also known as texture magnification filter.
    /// ![Visual comparison of linear resampling versus nearest resampling](assets/resampling.png)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resampling: Option<ColorReliefPaintLayerResampling>,
}

color_prop!(
    ColorReliefPaintLayerColorReliefColor,
    doc = "Defines the color of each pixel based on its elevation. Should be an expression that uses `[\"elevation\"]` as input."
);

numeric_prop!(
    ColorReliefPaintLayerColorReliefOpacity,
    doc = "The opacity at which the color-relief will be drawn.",
    min = 0_f64,
    max = 1_f64,
    default = serde_json::Number::from_i128(1)
        .expect("the number is serialised from a number and is thus always valid")
);

string_prop!(ColorReliefPaintLayerResampling, doc = "The resampling/interpolation method to use for overscaling, also known as texture magnification filter.
![Visual comparison of linear resampling versus nearest resampling](assets/resampling.png)", default = "linear".to_string());

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FillLayoutLayer {
    /// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
    #[serde(rename = "fill-sort-key")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_sort_key: Option<FillLayoutLayerFillSortKey>,
    /// Whether this layer is displayed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<FillLayoutLayerVisibility>,
}

numeric_prop!(
    FillLayoutLayerFillSortKey,
    doc = "Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key."
);

pub type FillLayoutLayerVisibility = Visibility;

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FillPaintLayer {
    /// Whether or not the fill should be antialiased.
    #[serde(rename = "fill-antialias")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_antialias: Option<FillPaintLayerFillAntialias>,
    /// The color of the filled part of this layer. This color can be specified as `rgba` with an alpha component and the color's opacity will not affect the opacity of the 1px stroke, if it is used.
    #[serde(rename = "fill-color")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<FillPaintLayerFillColor>,
    /// The opacity of the entire fill layer. In contrast to the `fill-color`, this value will also affect the 1px stroke around the fill, if the stroke is used.
    #[serde(rename = "fill-opacity")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_opacity: Option<FillPaintLayerFillOpacity>,
    /// The outline color of the fill. Matches the value of `fill-color` if unspecified.
    #[serde(rename = "fill-outline-color")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_outline_color: Option<FillPaintLayerFillOutlineColor>,
    /// Name of image in sprite to use for drawing image fills. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
    #[serde(rename = "fill-pattern")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_pattern: Option<FillPaintLayerFillPattern>,
    /// The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.
    #[serde(rename = "fill-translate")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_translate: Option<FillPaintLayerFillTranslate>,
    /// Controls the frame of reference for `fill-translate`.
    #[serde(rename = "fill-translate-anchor")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_translate_anchor: Option<FillPaintLayerFillTranslateAnchor>,
}

boolean_prop!(
    FillPaintLayerFillAntialias,
    doc = "Whether or not the fill should be antialiased.",
    default = true
);

color_prop!(
    FillPaintLayerFillColor,
    doc = "The color of the filled part of this layer. This color can be specified as `rgba` with an alpha component and the color's opacity will not affect the opacity of the 1px stroke, if it is used.",
    default = serde_json::json!("#000000")
);

numeric_prop!(
    FillPaintLayerFillOpacity,
    doc = "The opacity of the entire fill layer. In contrast to the `fill-color`, this value will also affect the 1px stroke around the fill, if the stroke is used.",
    min = 0_f64,
    max = 1_f64,
    default = serde_json::Number::from_i128(1)
        .expect("the number is serialised from a number and is thus always valid")
);

color_prop!(
    FillPaintLayerFillOutlineColor,
    doc = "The outline color of the fill. Matches the value of `fill-color` if unspecified."
);

string_prop!(
    FillPaintLayerFillPattern,
    doc = "Name of image in sprite to use for drawing image fills. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels."
);

array_prop!(
    FillPaintLayerFillTranslate,
    doc = "The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.",
    default = serde_json::json!([0, 0])
);

string_prop!(
    FillPaintLayerFillTranslateAnchor,
    doc = "Controls the frame of reference for `fill-translate`.",
    default = "map".to_string()
);

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FillExtrusionLayoutLayer {
    /// Whether this layer is displayed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<FillExtrusionLayoutLayerVisibility>,
}

pub type FillExtrusionLayoutLayerVisibility = Visibility;

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FillExtrusionPaintLayer {
    /// The height with which to extrude the base of this layer. Must be less than or equal to `fill-extrusion-height`.
    #[serde(rename = "fill-extrusion-base")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_extrusion_base: Option<FillExtrusionPaintLayerFillExtrusionBase>,
    /// The base color of the extruded fill. The extrusion's surfaces will be shaded differently based on this color in combination with the root `light` settings. If this color is specified as `rgba` with an alpha component, the alpha component will be ignored; use `fill-extrusion-opacity` to set layer opacity.
    #[serde(rename = "fill-extrusion-color")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_extrusion_color: Option<FillExtrusionPaintLayerFillExtrusionColor>,
    /// The height with which to extrude this layer.
    #[serde(rename = "fill-extrusion-height")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_extrusion_height: Option<FillExtrusionPaintLayerFillExtrusionHeight>,
    /// The opacity of the entire fill extrusion layer. This is rendered on a per-layer, not per-feature, basis, and data-driven styling is not available.
    #[serde(rename = "fill-extrusion-opacity")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_extrusion_opacity: Option<FillExtrusionPaintLayerFillExtrusionOpacity>,
    /// Name of image in sprite to use for drawing images on extruded fills. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
    #[serde(rename = "fill-extrusion-pattern")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_extrusion_pattern: Option<FillExtrusionPaintLayerFillExtrusionPattern>,
    /// The geometry's offset. Values are [x, y] where negatives indicate left and up (on the flat plane), respectively.
    #[serde(rename = "fill-extrusion-translate")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_extrusion_translate: Option<FillExtrusionPaintLayerFillExtrusionTranslate>,
    /// Controls the frame of reference for `fill-extrusion-translate`.
    #[serde(rename = "fill-extrusion-translate-anchor")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_extrusion_translate_anchor:
        Option<FillExtrusionPaintLayerFillExtrusionTranslateAnchor>,
    /// Whether to apply a vertical gradient to the sides of a fill-extrusion layer. If true, sides will be shaded slightly darker farther down.
    #[serde(rename = "fill-extrusion-vertical-gradient")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_extrusion_vertical_gradient:
        Option<FillExtrusionPaintLayerFillExtrusionVerticalGradient>,
}

numeric_prop!(
    FillExtrusionPaintLayerFillExtrusionBase,
    doc = "The height with which to extrude the base of this layer. Must be less than or equal to `fill-extrusion-height`.",
    min = 0_f64,
    default = serde_json::Number::from_i128(0)
        .expect("the number is serialised from a number and is thus always valid")
);

color_prop!(
    FillExtrusionPaintLayerFillExtrusionColor,
    doc = "The base color of the extruded fill. The extrusion's surfaces will be shaded differently based on this color in combination with the root `light` settings. If this color is specified as `rgba` with an alpha component, the alpha component will be ignored; use `fill-extrusion-opacity` to set layer opacity.",
    default = serde_json::json!("#000000")
);

numeric_prop!(
    FillExtrusionPaintLayerFillExtrusionHeight,
    doc = "The height with which to extrude this layer.",
    min = 0_f64,
    default = serde_json::Number::from_i128(0)
        .expect("the number is serialised from a number and is thus always valid")
);

numeric_prop!(
    FillExtrusionPaintLayerFillExtrusionOpacity,
    doc = "The opacity of the entire fill extrusion layer. This is rendered on a per-layer, not per-feature, basis, and data-driven styling is not available.",
    min = 0_f64,
    max = 1_f64,
    default = serde_json::Number::from_i128(1)
        .expect("the number is serialised from a number and is thus always valid")
);

string_prop!(
    FillExtrusionPaintLayerFillExtrusionPattern,
    doc = "Name of image in sprite to use for drawing images on extruded fills. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels."
);

array_prop!(
    FillExtrusionPaintLayerFillExtrusionTranslate,
    doc = "The geometry's offset. Values are [x, y] where negatives indicate left and up (on the flat plane), respectively.",
    default = serde_json::json!([0, 0])
);

string_prop!(
    FillExtrusionPaintLayerFillExtrusionTranslateAnchor,
    doc = "Controls the frame of reference for `fill-extrusion-translate`.",
    default = "map".to_string()
);

boolean_prop!(
    FillExtrusionPaintLayerFillExtrusionVerticalGradient,
    doc = "Whether to apply a vertical gradient to the sides of a fill-extrusion layer. If true, sides will be shaded slightly darker farther down.",
    default = true
);

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct HeatmapLayoutLayer {
    /// Whether this layer is displayed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<HeatmapLayoutLayerVisibility>,
}

pub type HeatmapLayoutLayerVisibility = Visibility;

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct HeatmapPaintLayer {
    /// Defines the color of each pixel based on its density value in a heatmap.  Should be an expression that uses `["heatmap-density"]` as input.
    #[serde(rename = "heatmap-color")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heatmap_color: Option<HeatmapPaintLayerHeatmapColor>,
    /// Similar to `heatmap-weight` but controls the intensity of the heatmap globally. Primarily used for adjusting the heatmap based on zoom level.
    #[serde(rename = "heatmap-intensity")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heatmap_intensity: Option<HeatmapPaintLayerHeatmapIntensity>,
    /// The global opacity at which the heatmap layer will be drawn.
    #[serde(rename = "heatmap-opacity")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heatmap_opacity: Option<HeatmapPaintLayerHeatmapOpacity>,
    /// Radius of influence of one heatmap point in pixels. Increasing the value makes the heatmap smoother, but less detailed.
    #[serde(rename = "heatmap-radius")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heatmap_radius: Option<HeatmapPaintLayerHeatmapRadius>,
    /// A measure of how much an individual point contributes to the heatmap. A value of 10 would be equivalent to having 10 points of weight 1 in the same spot. Especially useful when combined with clustering.
    #[serde(rename = "heatmap-weight")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heatmap_weight: Option<HeatmapPaintLayerHeatmapWeight>,
}

color_prop!(
    HeatmapPaintLayerHeatmapColor,
    doc = "Defines the color of each pixel based on its density value in a heatmap.  Should be an expression that uses `[\"heatmap-density\"]` as input.",
    default = serde_json::json!([
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
    ])
);

numeric_prop!(
    HeatmapPaintLayerHeatmapIntensity,
    doc = "Similar to `heatmap-weight` but controls the intensity of the heatmap globally. Primarily used for adjusting the heatmap based on zoom level.",
    min = 0_f64,
    default = serde_json::Number::from_i128(1)
        .expect("the number is serialised from a number and is thus always valid")
);

numeric_prop!(
    HeatmapPaintLayerHeatmapOpacity,
    doc = "The global opacity at which the heatmap layer will be drawn.",
    min = 0_f64,
    max = 1_f64,
    default = serde_json::Number::from_i128(1)
        .expect("the number is serialised from a number and is thus always valid")
);

numeric_prop!(
    HeatmapPaintLayerHeatmapRadius,
    doc = "Radius of influence of one heatmap point in pixels. Increasing the value makes the heatmap smoother, but less detailed.",
    min = 1_f64,
    default = serde_json::Number::from_i128(30)
        .expect("the number is serialised from a number and is thus always valid")
);

numeric_prop!(
    HeatmapPaintLayerHeatmapWeight,
    doc = "A measure of how much an individual point contributes to the heatmap. A value of 10 would be equivalent to having 10 points of weight 1 in the same spot. Especially useful when combined with clustering.",
    min = 0_f64,
    default = serde_json::Number::from_i128(1)
        .expect("the number is serialised from a number and is thus always valid")
);

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct HillshadeLayoutLayer {
    /// Whether this layer is displayed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<HillshadeLayoutLayerVisibility>,
}

pub type HillshadeLayoutLayerVisibility = Visibility;

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct HillshadePaintLayer {
    /// The shading color used to accentuate rugged terrain like sharp cliffs and gorges.
    #[serde(rename = "hillshade-accent-color")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hillshade_accent_color: Option<HillshadePaintLayerHillshadeAccentColor>,
    /// Intensity of the hillshade
    #[serde(rename = "hillshade-exaggeration")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hillshade_exaggeration: Option<HillshadePaintLayerHillshadeExaggeration>,
    /// The shading color of areas that faces towards the light source(s). Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
    #[serde(rename = "hillshade-highlight-color")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hillshade_highlight_color: Option<HillshadePaintLayerHillshadeHighlightColor>,
    /// The altitude of the light source(s) used to generate the hillshading with 0 as sunset and 90 as noon. Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
    #[serde(rename = "hillshade-illumination-altitude")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hillshade_illumination_altitude: Option<HillshadePaintLayerHillshadeIlluminationAltitude>,
    /// Direction of light source when map is rotated.
    #[serde(rename = "hillshade-illumination-anchor")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hillshade_illumination_anchor: Option<HillshadePaintLayerHillshadeIlluminationAnchor>,
    /// The direction of the light source(s) used to generate the hillshading with 0 as the top of the viewport if `hillshade-illumination-anchor` is set to `viewport` and due north if `hillshade-illumination-anchor` is set to `map`. Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
    #[serde(rename = "hillshade-illumination-direction")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hillshade_illumination_direction: Option<HillshadePaintLayerHillshadeIlluminationDirection>,
    /// The hillshade algorithm to use, one of `standard`, `basic`, `combined`, `igor`, or `multidirectional`.
    /// ![Visual comparison of standard, basic, igor, combined, and multidirectional hillshade-method](assets/hillshade_methods.png)
    #[serde(rename = "hillshade-method")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hillshade_method: Option<HillshadePaintLayerHillshadeMethod>,
    /// The shading color of areas that face away from the light source(s). Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
    #[serde(rename = "hillshade-shadow-color")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hillshade_shadow_color: Option<HillshadePaintLayerHillshadeShadowColor>,
    /// The resampling/interpolation method to use for overscaling, also known as texture magnification filter.
    /// ![Visual comparison of linear resampling versus nearest resampling](assets/resampling.png)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resampling: Option<HillshadePaintLayerResampling>,
}

color_prop!(
    HillshadePaintLayerHillshadeAccentColor,
    doc = "The shading color used to accentuate rugged terrain like sharp cliffs and gorges.",
    default = serde_json::json!("#000000")
);

numeric_prop!(
    HillshadePaintLayerHillshadeExaggeration,
    doc = "Intensity of the hillshade",
    min = 0_f64,
    max = 1_f64,
    default = serde_json::Number::from_f64(0.5)
        .expect("the number is serialised from a number and is thus always valid")
);

color_prop!(
    HillshadePaintLayerHillshadeHighlightColor,
    doc = "The shading color of areas that faces towards the light source(s). Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.",
    default = serde_json::json!("#FFFFFF")
);

array_prop!(
    HillshadePaintLayerHillshadeIlluminationAltitude,
    doc = "The altitude of the light source(s) used to generate the hillshading with 0 as sunset and 90 as noon. Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.",
    default = serde_json::json!(45)
);

string_prop!(
    HillshadePaintLayerHillshadeIlluminationAnchor,
    doc = "Direction of light source when map is rotated.",
    default = "viewport".to_string()
);

array_prop!(
    HillshadePaintLayerHillshadeIlluminationDirection,
    doc = "The direction of the light source(s) used to generate the hillshading with 0 as the top of the viewport if `hillshade-illumination-anchor` is set to `viewport` and due north if `hillshade-illumination-anchor` is set to `map`. Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.",
    default = serde_json::json!(335)
);

string_prop!(HillshadePaintLayerHillshadeMethod, doc = "The hillshade algorithm to use, one of `standard`, `basic`, `combined`, `igor`, or `multidirectional`.
![Visual comparison of standard, basic, igor, combined, and multidirectional hillshade-method](assets/hillshade_methods.png)", default = "standard".to_string());

color_prop!(
    HillshadePaintLayerHillshadeShadowColor,
    doc = "The shading color of areas that face away from the light source(s). Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.",
    default = serde_json::json!("#000000")
);

string_prop!(HillshadePaintLayerResampling, doc = "The resampling/interpolation method to use for overscaling, also known as texture magnification filter.
![Visual comparison of linear resampling versus nearest resampling](assets/resampling.png)", default = "linear".to_string());

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct LineLayoutLayer {
    /// The display of line endings.
    #[serde(rename = "line-cap")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_cap: Option<LineLayoutLayerLineCap>,
    /// The display of lines when joining.
    #[serde(rename = "line-join")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_join: Option<LineLayoutLayerLineJoin>,
    /// Used to automatically convert miter joins to bevel joins for sharp angles.
    #[serde(rename = "line-miter-limit")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_miter_limit: Option<LineLayoutLayerLineMiterLimit>,
    /// Used to automatically convert round joins to miter joins for shallow angles.
    #[serde(rename = "line-round-limit")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_round_limit: Option<LineLayoutLayerLineRoundLimit>,
    /// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
    #[serde(rename = "line-sort-key")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_sort_key: Option<LineLayoutLayerLineSortKey>,
    /// Whether this layer is displayed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<LineLayoutLayerVisibility>,
}

string_prop!(
    LineLayoutLayerLineCap,
    doc = "The display of line endings.",
    default = "butt".to_string()
);

string_prop!(
    LineLayoutLayerLineJoin,
    doc = "The display of lines when joining.",
    default = "miter".to_string()
);

numeric_prop!(
    LineLayoutLayerLineMiterLimit,
    doc = "Used to automatically convert miter joins to bevel joins for sharp angles.",
    default = serde_json::Number::from_i128(2)
        .expect("the number is serialised from a number and is thus always valid")
);

numeric_prop!(
    LineLayoutLayerLineRoundLimit,
    doc = "Used to automatically convert round joins to miter joins for shallow angles.",
    default = serde_json::Number::from_f64(1.05)
        .expect("the number is serialised from a number and is thus always valid")
);

numeric_prop!(
    LineLayoutLayerLineSortKey,
    doc = "Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key."
);

pub type LineLayoutLayerVisibility = Visibility;

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct LinePaintLayer {
    /// Blur applied to the line, in pixels.
    #[serde(rename = "line-blur")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_blur: Option<LinePaintLayerLineBlur>,
    /// The color with which the line will be drawn.
    #[serde(rename = "line-color")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_color: Option<LinePaintLayerLineColor>,
    /// Specifies the lengths of the alternating dashes and gaps that form the dash pattern. The lengths are later scaled by the line width. To convert a dash length to pixels, multiply the length by the current line width. GeoJSON sources with `lineMetrics: true` specified won't render dashed lines to the expected scale. Zoom-dependent expressions will be evaluated only at integer zoom levels. The only way to create an array value is using `["literal", [...]]`; arrays cannot be read from or derived from feature properties.
    #[serde(rename = "line-dasharray")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_dasharray: Option<LinePaintLayerLineDasharray>,
    /// Draws a line casing outside of a line's actual path. Value indicates the width of the inner gap.
    #[serde(rename = "line-gap-width")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_gap_width: Option<LinePaintLayerLineGapWidth>,
    /// Defines a gradient with which to color a line feature. Can only be used with GeoJSON sources that specify `"lineMetrics": true`.
    #[serde(rename = "line-gradient")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_gradient: Option<LinePaintLayerLineGradient>,
    /// The line's offset. For linear features, a positive value offsets the line to the right, relative to the direction of the line, and a negative value to the left. For polygon features, a positive value results in an inset, and a negative value results in an outset.
    #[serde(rename = "line-offset")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_offset: Option<LinePaintLayerLineOffset>,
    /// The opacity at which the line will be drawn.
    #[serde(rename = "line-opacity")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_opacity: Option<LinePaintLayerLineOpacity>,
    /// Name of image in sprite to use for drawing image lines. For seamless patterns, image width must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
    #[serde(rename = "line-pattern")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_pattern: Option<LinePaintLayerLinePattern>,
    /// The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.
    #[serde(rename = "line-translate")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_translate: Option<LinePaintLayerLineTranslate>,
    /// Controls the frame of reference for `line-translate`.
    #[serde(rename = "line-translate-anchor")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_translate_anchor: Option<LinePaintLayerLineTranslateAnchor>,
    /// Stroke thickness.
    #[serde(rename = "line-width")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_width: Option<LinePaintLayerLineWidth>,
}

numeric_prop!(
    LinePaintLayerLineBlur,
    doc = "Blur applied to the line, in pixels.",
    min = 0_f64,
    default = serde_json::Number::from_i128(0)
        .expect("the number is serialised from a number and is thus always valid")
);

color_prop!(
    LinePaintLayerLineColor,
    doc = "The color with which the line will be drawn.",
    default = serde_json::json!("#000000")
);

array_prop!(
    LinePaintLayerLineDasharray,
    doc = "Specifies the lengths of the alternating dashes and gaps that form the dash pattern. The lengths are later scaled by the line width. To convert a dash length to pixels, multiply the length by the current line width. GeoJSON sources with `lineMetrics: true` specified won't render dashed lines to the expected scale. Zoom-dependent expressions will be evaluated only at integer zoom levels. The only way to create an array value is using `[\"literal\", [...]]`; arrays cannot be read from or derived from feature properties."
);

numeric_prop!(
    LinePaintLayerLineGapWidth,
    doc = "Draws a line casing outside of a line's actual path. Value indicates the width of the inner gap.",
    min = 0_f64,
    default = serde_json::Number::from_i128(0)
        .expect("the number is serialised from a number and is thus always valid")
);

color_prop!(
    LinePaintLayerLineGradient,
    doc = "Defines a gradient with which to color a line feature. Can only be used with GeoJSON sources that specify `\"lineMetrics\": true`."
);

numeric_prop!(
    LinePaintLayerLineOffset,
    doc = "The line's offset. For linear features, a positive value offsets the line to the right, relative to the direction of the line, and a negative value to the left. For polygon features, a positive value results in an inset, and a negative value results in an outset.",
    default = serde_json::Number::from_i128(0)
        .expect("the number is serialised from a number and is thus always valid")
);

numeric_prop!(
    LinePaintLayerLineOpacity,
    doc = "The opacity at which the line will be drawn.",
    min = 0_f64,
    max = 1_f64,
    default = serde_json::Number::from_i128(1)
        .expect("the number is serialised from a number and is thus always valid")
);

string_prop!(
    LinePaintLayerLinePattern,
    doc = "Name of image in sprite to use for drawing image lines. For seamless patterns, image width must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels."
);

array_prop!(
    LinePaintLayerLineTranslate,
    doc = "The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.",
    default = serde_json::json!([0, 0])
);

string_prop!(
    LinePaintLayerLineTranslateAnchor,
    doc = "Controls the frame of reference for `line-translate`.",
    default = "map".to_string()
);

numeric_prop!(
    LinePaintLayerLineWidth,
    doc = "Stroke thickness.",
    min = 0_f64,
    default = serde_json::Number::from_i128(1)
        .expect("the number is serialised from a number and is thus always valid")
);

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterLayoutLayer {
    /// Whether this layer is displayed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<RasterLayoutLayerVisibility>,
}

pub type RasterLayoutLayerVisibility = Visibility;

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterPaintLayer {
    /// Increase or reduce the brightness of the image. The value is the maximum brightness.
    #[serde(rename = "raster-brightness-max")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raster_brightness_max: Option<RasterPaintLayerRasterBrightnessMax>,
    /// Increase or reduce the brightness of the image. The value is the minimum brightness.
    #[serde(rename = "raster-brightness-min")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raster_brightness_min: Option<RasterPaintLayerRasterBrightnessMin>,
    /// Increase or reduce the contrast of the image.
    #[serde(rename = "raster-contrast")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raster_contrast: Option<RasterPaintLayerRasterContrast>,
    /// Fade duration when a new tile is added, or when a video is started or its coordinates are updated.
    #[serde(rename = "raster-fade-duration")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raster_fade_duration: Option<RasterPaintLayerRasterFadeDuration>,
    /// Rotates hues around the color wheel.
    #[serde(rename = "raster-hue-rotate")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raster_hue_rotate: Option<RasterPaintLayerRasterHueRotate>,
    /// The opacity at which the image will be drawn.
    #[serde(rename = "raster-opacity")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raster_opacity: Option<RasterPaintLayerRasterOpacity>,
    /// The resampling/interpolation method to use for overscaling, also known as texture magnification filter. It is advised to use the generic `resampling` paint property instead.
    #[serde(rename = "raster-resampling")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raster_resampling: Option<RasterPaintLayerRasterResampling>,
    /// Increase or reduce the saturation of the image.
    #[serde(rename = "raster-saturation")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raster_saturation: Option<RasterPaintLayerRasterSaturation>,
    /// The resampling/interpolation method to use for overscaling, also known as texture magnification filter.
    /// ![Visual comparison of linear resampling versus nearest resampling](assets/resampling.png)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resampling: Option<RasterPaintLayerResampling>,
}

numeric_prop!(
    RasterPaintLayerRasterBrightnessMax,
    doc = "Increase or reduce the brightness of the image. The value is the maximum brightness.",
    min = 0_f64,
    max = 1_f64,
    default = serde_json::Number::from_i128(1)
        .expect("the number is serialised from a number and is thus always valid")
);

numeric_prop!(
    RasterPaintLayerRasterBrightnessMin,
    doc = "Increase or reduce the brightness of the image. The value is the minimum brightness.",
    min = 0_f64,
    max = 1_f64,
    default = serde_json::Number::from_i128(0)
        .expect("the number is serialised from a number and is thus always valid")
);

numeric_prop!(
    RasterPaintLayerRasterContrast,
    doc = "Increase or reduce the contrast of the image.",
    min = -1_f64,
    max = 1_f64,
    default = serde_json::Number::from_i128(0)
        .expect("the number is serialised from a number and is thus always valid")
);

numeric_prop!(
    RasterPaintLayerRasterFadeDuration,
    doc = "Fade duration when a new tile is added, or when a video is started or its coordinates are updated.",
    min = 0_f64,
    default = serde_json::Number::from_i128(300)
        .expect("the number is serialised from a number and is thus always valid")
);

numeric_prop!(
    RasterPaintLayerRasterHueRotate,
    doc = "Rotates hues around the color wheel.",
    default = serde_json::Number::from_i128(0)
        .expect("the number is serialised from a number and is thus always valid")
);

numeric_prop!(
    RasterPaintLayerRasterOpacity,
    doc = "The opacity at which the image will be drawn.",
    min = 0_f64,
    max = 1_f64,
    default = serde_json::Number::from_i128(1)
        .expect("the number is serialised from a number and is thus always valid")
);

string_prop!(
    RasterPaintLayerRasterResampling,
    doc = "The resampling/interpolation method to use for overscaling, also known as texture magnification filter. It is advised to use the generic `resampling` paint property instead.",
    default = "linear".to_string()
);

numeric_prop!(
    RasterPaintLayerRasterSaturation,
    doc = "Increase or reduce the saturation of the image.",
    min = -1_f64,
    max = 1_f64,
    default = serde_json::Number::from_i128(0)
        .expect("the number is serialised from a number and is thus always valid")
);

string_prop!(RasterPaintLayerResampling, doc = "The resampling/interpolation method to use for overscaling, also known as texture magnification filter.
![Visual comparison of linear resampling versus nearest resampling](assets/resampling.png)", default = "linear".to_string());

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayer {
    /// If true, the icon will be visible even if it collides with other previously drawn symbols.
    #[serde(rename = "icon-allow-overlap")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_allow_overlap: Option<SymbolLayoutLayerIconAllowOverlap>,
    /// Part of the icon placed closest to the anchor.
    #[serde(rename = "icon-anchor")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_anchor: Option<SymbolLayoutLayerIconAnchor>,
    /// If true, other symbols can be visible even if they collide with the icon.
    #[serde(rename = "icon-ignore-placement")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_ignore_placement: Option<SymbolLayoutLayerIconIgnorePlacement>,
    /// Name of image in sprite to use for drawing an image background.
    #[serde(rename = "icon-image")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_image: Option<SymbolLayoutLayerIconImage>,
    /// If true, the icon may be flipped to prevent it from being rendered upside-down.
    #[serde(rename = "icon-keep-upright")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_keep_upright: Option<SymbolLayoutLayerIconKeepUpright>,
    /// Offset distance of icon from its anchor. Positive values indicate right and down, while negative values indicate left and up. Each component is multiplied by the value of `icon-size` to obtain the final offset in pixels. When combined with `icon-rotate` the offset will be as if the rotated direction was up.
    #[serde(rename = "icon-offset")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_offset: Option<SymbolLayoutLayerIconOffset>,
    /// If true, text will display without their corresponding icons when the icon collides with other symbols and the text does not.
    #[serde(rename = "icon-optional")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_optional: Option<SymbolLayoutLayerIconOptional>,
    /// Allows for control over whether to show an icon when it overlaps other symbols on the map. If `icon-overlap` is not set, `icon-allow-overlap` is used instead.
    #[serde(rename = "icon-overlap")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_overlap: Option<SymbolLayoutLayerIconOverlap>,
    /// Size of additional area round the icon bounding box used for detecting symbol collisions.
    #[serde(rename = "icon-padding")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_padding: Option<SymbolLayoutLayerIconPadding>,
    /// Orientation of icon when map is pitched.
    #[serde(rename = "icon-pitch-alignment")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_pitch_alignment: Option<SymbolLayoutLayerIconPitchAlignment>,
    /// Rotates the icon clockwise.
    #[serde(rename = "icon-rotate")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_rotate: Option<SymbolLayoutLayerIconRotate>,
    /// In combination with `symbol-placement`, determines the rotation behavior of icons.
    #[serde(rename = "icon-rotation-alignment")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_rotation_alignment: Option<SymbolLayoutLayerIconRotationAlignment>,
    /// Scales the original size of the icon by the provided factor. The new pixel size of the image will be the original pixel size multiplied by `icon-size`. 1 is the original size; 3 triples the size of the image.
    #[serde(rename = "icon-size")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_size: Option<SymbolLayoutLayerIconSize>,
    /// Scales the icon to fit around the associated text.
    #[serde(rename = "icon-text-fit")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_text_fit: Option<SymbolLayoutLayerIconTextFit>,
    /// Size of the additional area added to dimensions determined by `icon-text-fit`, in clockwise order: top, right, bottom, left.
    #[serde(rename = "icon-text-fit-padding")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_text_fit_padding: Option<SymbolLayoutLayerIconTextFitPadding>,
    /// If true, the symbols will not cross tile edges to avoid mutual collisions. Recommended in layers that don't have enough padding in the vector tile to prevent collisions, or if it is a point symbol layer placed after a line symbol layer. When using a client that supports global collision detection, like MapLibre GL JS version 0.42.0 or greater, enabling this property is not needed to prevent clipped labels at tile boundaries.
    #[serde(rename = "symbol-avoid-edges")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol_avoid_edges: Option<SymbolLayoutLayerSymbolAvoidEdges>,
    /// Label placement relative to its geometry.
    #[serde(rename = "symbol-placement")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol_placement: Option<SymbolLayoutLayerSymbolPlacement>,
    /// Sorts features in ascending order based on this value. Features with lower sort keys are drawn and placed first.  When `icon-allow-overlap` or `text-allow-overlap` is `false`, features with a lower sort key will have priority during placement. When `icon-allow-overlap` or `text-allow-overlap` is set to `true`, features with a higher sort key will overlap over features with a lower sort key.
    #[serde(rename = "symbol-sort-key")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol_sort_key: Option<SymbolLayoutLayerSymbolSortKey>,
    /// Distance between two symbol anchors.
    #[serde(rename = "symbol-spacing")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol_spacing: Option<SymbolLayoutLayerSymbolSpacing>,
    /// Determines whether overlapping symbols in the same layer are rendered in the order that they appear in the data source or by their y-position relative to the viewport. To control the order and prioritization of symbols otherwise, use `symbol-sort-key`.
    #[serde(rename = "symbol-z-order")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol_z_order: Option<SymbolLayoutLayerSymbolZOrder>,
    /// If true, the text will be visible even if it collides with other previously drawn symbols.
    #[serde(rename = "text-allow-overlap")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_allow_overlap: Option<SymbolLayoutLayerTextAllowOverlap>,
    /// Part of the text placed closest to the anchor.
    #[serde(rename = "text-anchor")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_anchor: Option<SymbolLayoutLayerTextAnchor>,
    /// Value to use for a text label. If a plain `string` is provided, it will be treated as a `formatted` with default/inherited formatting options.
    #[serde(rename = "text-field")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_field: Option<SymbolLayoutLayerTextField>,
    /// Fonts to use for displaying text. If the `glyphs` root property is specified, this array is joined together and interpreted as a font stack name. Otherwise, it is interpreted as a cascading fallback list of local font names.
    #[serde(rename = "text-font")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_font: Option<SymbolLayoutLayerTextFont>,
    /// If true, other symbols can be visible even if they collide with the text.
    #[serde(rename = "text-ignore-placement")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_ignore_placement: Option<SymbolLayoutLayerTextIgnorePlacement>,
    /// Text justification options.
    #[serde(rename = "text-justify")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_justify: Option<SymbolLayoutLayerTextJustify>,
    /// If true, the text may be flipped vertically to prevent it from being rendered upside-down.
    #[serde(rename = "text-keep-upright")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_keep_upright: Option<SymbolLayoutLayerTextKeepUpright>,
    /// Text tracking amount.
    #[serde(rename = "text-letter-spacing")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_letter_spacing: Option<SymbolLayoutLayerTextLetterSpacing>,
    /// Text leading value for multi-line text.
    #[serde(rename = "text-line-height")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_line_height: Option<SymbolLayoutLayerTextLineHeight>,
    /// Maximum angle change between adjacent characters.
    #[serde(rename = "text-max-angle")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_max_angle: Option<SymbolLayoutLayerTextMaxAngle>,
    /// The maximum line width for text wrapping.
    #[serde(rename = "text-max-width")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_max_width: Option<SymbolLayoutLayerTextMaxWidth>,
    /// Offset distance of text from its anchor. Positive values indicate right and down, while negative values indicate left and up. If used with text-variable-anchor, input values will be taken as absolute values. Offsets along the x- and y-axis will be applied automatically based on the anchor position.
    #[serde(rename = "text-offset")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_offset: Option<SymbolLayoutLayerTextOffset>,
    /// If true, icons will display without their corresponding text when the text collides with other symbols and the icon does not.
    #[serde(rename = "text-optional")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_optional: Option<SymbolLayoutLayerTextOptional>,
    /// Allows for control over whether to show symbol text when it overlaps other symbols on the map. If `text-overlap` is not set, `text-allow-overlap` is used instead
    #[serde(rename = "text-overlap")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_overlap: Option<SymbolLayoutLayerTextOverlap>,
    /// Size of the additional area around the text bounding box used for detecting symbol collisions.
    #[serde(rename = "text-padding")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_padding: Option<SymbolLayoutLayerTextPadding>,
    /// Orientation of text when map is pitched.
    #[serde(rename = "text-pitch-alignment")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_pitch_alignment: Option<SymbolLayoutLayerTextPitchAlignment>,
    /// Radial offset of text, in the direction of the symbol's anchor. Useful in combination with `text-variable-anchor`, which defaults to using the two-dimensional `text-offset` if present.
    #[serde(rename = "text-radial-offset")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_radial_offset: Option<SymbolLayoutLayerTextRadialOffset>,
    /// Rotates the text clockwise.
    #[serde(rename = "text-rotate")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_rotate: Option<SymbolLayoutLayerTextRotate>,
    /// In combination with `symbol-placement`, determines the rotation behavior of the individual glyphs forming the text.
    #[serde(rename = "text-rotation-alignment")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_rotation_alignment: Option<SymbolLayoutLayerTextRotationAlignment>,
    /// Font size.
    #[serde(rename = "text-size")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_size: Option<SymbolLayoutLayerTextSize>,
    /// Specifies how to capitalize text, similar to the CSS `text-transform` property.
    #[serde(rename = "text-transform")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_transform: Option<SymbolLayoutLayerTextTransform>,
    /// To increase the chance of placing high-priority labels on the map, you can provide an array of `text-anchor` locations: the renderer will attempt to place the label at each location, in order, before moving onto the next label. Use `text-justify: auto` to choose justification based on anchor position. To apply an offset, use the `text-radial-offset` or the two-dimensional `text-offset`.
    #[serde(rename = "text-variable-anchor")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_variable_anchor_offset: Option<SymbolLayoutLayerTextVariableAnchorOffset>,
    /// The property allows control over a symbol's orientation. Note that the property values act as a hint, so that a symbol whose language doesn’t support the provided orientation will be laid out in its natural orientation. Example: English point symbol will be rendered horizontally even if array value contains single 'vertical' enum value. The order of elements in an array define priority order for the placement of an orientation variant.
    #[serde(rename = "text-writing-mode")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_writing_mode: Option<SymbolLayoutLayerTextWritingMode>,
    /// Whether this layer is displayed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<SymbolLayoutLayerVisibility>,
}

boolean_prop!(
    SymbolLayoutLayerIconAllowOverlap,
    doc = "If true, the icon will be visible even if it collides with other previously drawn symbols.",
    default = false
);

string_prop!(
    SymbolLayoutLayerIconAnchor,
    doc = "Part of the icon placed closest to the anchor.",
    default = "center".to_string()
);

boolean_prop!(
    SymbolLayoutLayerIconIgnorePlacement,
    doc = "If true, other symbols can be visible even if they collide with the icon.",
    default = false
);

string_prop!(
    SymbolLayoutLayerIconImage,
    doc = "Name of image in sprite to use for drawing an image background."
);

boolean_prop!(
    SymbolLayoutLayerIconKeepUpright,
    doc = "If true, the icon may be flipped to prevent it from being rendered upside-down.",
    default = false
);

array_prop!(
    SymbolLayoutLayerIconOffset,
    doc = "Offset distance of icon from its anchor. Positive values indicate right and down, while negative values indicate left and up. Each component is multiplied by the value of `icon-size` to obtain the final offset in pixels. When combined with `icon-rotate` the offset will be as if the rotated direction was up.",
    default = serde_json::json!([0, 0])
);

boolean_prop!(
    SymbolLayoutLayerIconOptional,
    doc = "If true, text will display without their corresponding icons when the icon collides with other symbols and the text does not.",
    default = false
);

string_prop!(
    SymbolLayoutLayerIconOverlap,
    doc = "Allows for control over whether to show an icon when it overlaps other symbols on the map. If `icon-overlap` is not set, `icon-allow-overlap` is used instead."
);

array_prop!(
    SymbolLayoutLayerIconPadding,
    doc =
        "Size of additional area round the icon bounding box used for detecting symbol collisions.",
    default = serde_json::json!([2])
);

string_prop!(
    SymbolLayoutLayerIconPitchAlignment,
    doc = "Orientation of icon when map is pitched.",
    default = "auto".to_string()
);

numeric_prop!(
    SymbolLayoutLayerIconRotate,
    doc = "Rotates the icon clockwise.",
    default = serde_json::Number::from_i128(0)
        .expect("the number is serialised from a number and is thus always valid")
);

string_prop!(
    SymbolLayoutLayerIconRotationAlignment,
    doc = "In combination with `symbol-placement`, determines the rotation behavior of icons.",
    default = "auto".to_string()
);

numeric_prop!(
    SymbolLayoutLayerIconSize,
    doc = "Scales the original size of the icon by the provided factor. The new pixel size of the image will be the original pixel size multiplied by `icon-size`. 1 is the original size; 3 triples the size of the image.",
    min = 0_f64,
    default = serde_json::Number::from_i128(1)
        .expect("the number is serialised from a number and is thus always valid")
);

string_prop!(
    SymbolLayoutLayerIconTextFit,
    doc = "Scales the icon to fit around the associated text.",
    default = "none".to_string()
);

array_prop!(
    SymbolLayoutLayerIconTextFitPadding,
    doc = "Size of the additional area added to dimensions determined by `icon-text-fit`, in clockwise order: top, right, bottom, left.",
    default = serde_json::json!([0, 0, 0, 0])
);

boolean_prop!(
    SymbolLayoutLayerSymbolAvoidEdges,
    doc = "If true, the symbols will not cross tile edges to avoid mutual collisions. Recommended in layers that don't have enough padding in the vector tile to prevent collisions, or if it is a point symbol layer placed after a line symbol layer. When using a client that supports global collision detection, like MapLibre GL JS version 0.42.0 or greater, enabling this property is not needed to prevent clipped labels at tile boundaries.",
    default = false
);

string_prop!(
    SymbolLayoutLayerSymbolPlacement,
    doc = "Label placement relative to its geometry.",
    default = "point".to_string()
);

numeric_prop!(
    SymbolLayoutLayerSymbolSortKey,
    doc = "Sorts features in ascending order based on this value. Features with lower sort keys are drawn and placed first.  When `icon-allow-overlap` or `text-allow-overlap` is `false`, features with a lower sort key will have priority during placement. When `icon-allow-overlap` or `text-allow-overlap` is set to `true`, features with a higher sort key will overlap over features with a lower sort key."
);

numeric_prop!(
    SymbolLayoutLayerSymbolSpacing,
    doc = "Distance between two symbol anchors.",
    min = 1_f64,
    default = serde_json::Number::from_i128(250)
        .expect("the number is serialised from a number and is thus always valid")
);

string_prop!(
    SymbolLayoutLayerSymbolZOrder,
    doc = "Determines whether overlapping symbols in the same layer are rendered in the order that they appear in the data source or by their y-position relative to the viewport. To control the order and prioritization of symbols otherwise, use `symbol-sort-key`.",
    default = "auto".to_string()
);

boolean_prop!(
    SymbolLayoutLayerTextAllowOverlap,
    doc = "If true, the text will be visible even if it collides with other previously drawn symbols.",
    default = false
);

string_prop!(
    SymbolLayoutLayerTextAnchor,
    doc = "Part of the text placed closest to the anchor.",
    default = "center".to_string()
);

formatted_prop!(
    SymbolLayoutLayerTextField,
    doc = "Value to use for a text label. If a plain `string` is provided, it will be treated as a `formatted` with default/inherited formatting options.",
    default = "".to_string()
);

array_prop!(
    SymbolLayoutLayerTextFont,
    doc = "Fonts to use for displaying text. If the `glyphs` root property is specified, this array is joined together and interpreted as a font stack name. Otherwise, it is interpreted as a cascading fallback list of local font names.",
    default = serde_json::json!(["Open Sans Regular", "Arial Unicode MS Regular"])
);

boolean_prop!(
    SymbolLayoutLayerTextIgnorePlacement,
    doc = "If true, other symbols can be visible even if they collide with the text.",
    default = false
);

string_prop!(
    SymbolLayoutLayerTextJustify,
    doc = "Text justification options.",
    default = "center".to_string()
);

boolean_prop!(
    SymbolLayoutLayerTextKeepUpright,
    doc = "If true, the text may be flipped vertically to prevent it from being rendered upside-down.",
    default = true
);

numeric_prop!(
    SymbolLayoutLayerTextLetterSpacing,
    doc = "Text tracking amount.",
    default = serde_json::Number::from_i128(0)
        .expect("the number is serialised from a number and is thus always valid")
);

numeric_prop!(
    SymbolLayoutLayerTextLineHeight,
    doc = "Text leading value for multi-line text.",
    default = serde_json::Number::from_f64(1.2)
        .expect("the number is serialised from a number and is thus always valid")
);

numeric_prop!(
    SymbolLayoutLayerTextMaxAngle,
    doc = "Maximum angle change between adjacent characters.",
    default = serde_json::Number::from_i128(45)
        .expect("the number is serialised from a number and is thus always valid")
);

numeric_prop!(
    SymbolLayoutLayerTextMaxWidth,
    doc = "The maximum line width for text wrapping.",
    min = 0_f64,
    default = serde_json::Number::from_i128(10)
        .expect("the number is serialised from a number and is thus always valid")
);

array_prop!(
    SymbolLayoutLayerTextOffset,
    doc = "Offset distance of text from its anchor. Positive values indicate right and down, while negative values indicate left and up. If used with text-variable-anchor, input values will be taken as absolute values. Offsets along the x- and y-axis will be applied automatically based on the anchor position.",
    default = serde_json::json!([0, 0])
);

boolean_prop!(
    SymbolLayoutLayerTextOptional,
    doc = "If true, icons will display without their corresponding text when the text collides with other symbols and the icon does not.",
    default = false
);

string_prop!(
    SymbolLayoutLayerTextOverlap,
    doc = "Allows for control over whether to show symbol text when it overlaps other symbols on the map. If `text-overlap` is not set, `text-allow-overlap` is used instead"
);

numeric_prop!(
    SymbolLayoutLayerTextPadding,
    doc = "Size of the additional area around the text bounding box used for detecting symbol collisions.",
    min = 0_f64,
    default = serde_json::Number::from_i128(2)
        .expect("the number is serialised from a number and is thus always valid")
);

string_prop!(
    SymbolLayoutLayerTextPitchAlignment,
    doc = "Orientation of text when map is pitched.",
    default = "auto".to_string()
);

numeric_prop!(
    SymbolLayoutLayerTextRadialOffset,
    doc = "Radial offset of text, in the direction of the symbol's anchor. Useful in combination with `text-variable-anchor`, which defaults to using the two-dimensional `text-offset` if present.",
    default = serde_json::Number::from_i128(0)
        .expect("the number is serialised from a number and is thus always valid")
);

numeric_prop!(
    SymbolLayoutLayerTextRotate,
    doc = "Rotates the text clockwise.",
    default = serde_json::Number::from_i128(0)
        .expect("the number is serialised from a number and is thus always valid")
);

string_prop!(
    SymbolLayoutLayerTextRotationAlignment,
    doc = "In combination with `symbol-placement`, determines the rotation behavior of the individual glyphs forming the text.",
    default = "auto".to_string()
);

numeric_prop!(
    SymbolLayoutLayerTextSize,
    doc = "Font size.",
    min = 0_f64,
    default = serde_json::Number::from_i128(16)
        .expect("the number is serialised from a number and is thus always valid")
);

string_prop!(
    SymbolLayoutLayerTextTransform,
    doc = "Specifies how to capitalize text, similar to the CSS `text-transform` property.",
    default = "none".to_string()
);

array_prop!(
    SymbolLayoutLayerTextVariableAnchor,
    doc = "To increase the chance of placing high-priority labels on the map, you can provide an array of `text-anchor` locations: the renderer will attempt to place the label at each location, in order, before moving onto the next label. Use `text-justify: auto` to choose justification based on anchor position. To apply an offset, use the `text-radial-offset` or the two-dimensional `text-offset`."
);

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

array_prop!(
    SymbolLayoutLayerTextWritingMode,
    doc = "The property allows control over a symbol's orientation. Note that the property values act as a hint, so that a symbol whose language doesn’t support the provided orientation will be laid out in its natural orientation. Example: English point symbol will be rendered horizontally even if array value contains single 'vertical' enum value. The order of elements in an array define priority order for the placement of an orientation variant."
);

pub type SymbolLayoutLayerVisibility = Visibility;

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolPaintLayer {
    /// The color of the icon. This can only be used with SDF icons.
    #[serde(rename = "icon-color")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_color: Option<SymbolPaintLayerIconColor>,
    /// Fade out the halo towards the outside.
    #[serde(rename = "icon-halo-blur")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_halo_blur: Option<SymbolPaintLayerIconHaloBlur>,
    /// The color of the icon's halo. Icon halos can only be used with SDF icons.
    #[serde(rename = "icon-halo-color")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_halo_color: Option<SymbolPaintLayerIconHaloColor>,
    /// Distance of halo to the icon outline.
    ///
    /// The unit is in pixels only for SDF sprites that were created with a blur radius of 8, multiplied by the display density. I.e., the radius needs to be 16 for `@2x` sprites, etc.
    #[serde(rename = "icon-halo-width")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_halo_width: Option<SymbolPaintLayerIconHaloWidth>,
    /// The opacity at which the icon will be drawn.
    #[serde(rename = "icon-opacity")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_opacity: Option<SymbolPaintLayerIconOpacity>,
    /// Distance that the icon's anchor is moved from its original placement. Positive values indicate right and down, while negative values indicate left and up.
    #[serde(rename = "icon-translate")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_translate: Option<SymbolPaintLayerIconTranslate>,
    /// Controls the frame of reference for `icon-translate`.
    #[serde(rename = "icon-translate-anchor")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_translate_anchor: Option<SymbolPaintLayerIconTranslateAnchor>,
    /// The color with which the text will be drawn.
    #[serde(rename = "text-color")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_color: Option<SymbolPaintLayerTextColor>,
    /// The halo's fadeout distance towards the outside.
    #[serde(rename = "text-halo-blur")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_halo_blur: Option<SymbolPaintLayerTextHaloBlur>,
    /// The color of the text's halo, which helps it stand out from backgrounds.
    #[serde(rename = "text-halo-color")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_halo_color: Option<SymbolPaintLayerTextHaloColor>,
    /// Distance of halo to the font outline. Max text halo width is 1/4 of the font-size.
    #[serde(rename = "text-halo-width")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_halo_width: Option<SymbolPaintLayerTextHaloWidth>,
    /// The opacity at which the text will be drawn.
    #[serde(rename = "text-opacity")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_opacity: Option<SymbolPaintLayerTextOpacity>,
    /// Distance that the text's anchor is moved from its original placement. Positive values indicate right and down, while negative values indicate left and up.
    #[serde(rename = "text-translate")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_translate: Option<SymbolPaintLayerTextTranslate>,
    /// Controls the frame of reference for `text-translate`.
    #[serde(rename = "text-translate-anchor")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_translate_anchor: Option<SymbolPaintLayerTextTranslateAnchor>,
}

color_prop!(
    SymbolPaintLayerIconColor,
    doc = "The color of the icon. This can only be used with SDF icons.",
    default = serde_json::json!("#000000")
);

numeric_prop!(
    SymbolPaintLayerIconHaloBlur,
    doc = "Fade out the halo towards the outside.",
    min = 0_f64,
    default = serde_json::Number::from_i128(0)
        .expect("the number is serialised from a number and is thus always valid")
);

color_prop!(
    SymbolPaintLayerIconHaloColor,
    doc = "The color of the icon's halo. Icon halos can only be used with SDF icons.",
    default = serde_json::json!("rgba(0, 0, 0, 0)")
);

numeric_prop!(SymbolPaintLayerIconHaloWidth, doc = "Distance of halo to the icon outline. 

The unit is in pixels only for SDF sprites that were created with a blur radius of 8, multiplied by the display density. I.e., the radius needs to be 16 for `@2x` sprites, etc.", min = 0_f64, default = serde_json::Number::from_i128(0).expect("the number is serialised from a number and is thus always valid"));

numeric_prop!(
    SymbolPaintLayerIconOpacity,
    doc = "The opacity at which the icon will be drawn.",
    min = 0_f64,
    max = 1_f64,
    default = serde_json::Number::from_i128(1)
        .expect("the number is serialised from a number and is thus always valid")
);

array_prop!(
    SymbolPaintLayerIconTranslate,
    doc = "Distance that the icon's anchor is moved from its original placement. Positive values indicate right and down, while negative values indicate left and up.",
    default = serde_json::json!([0, 0])
);

string_prop!(
    SymbolPaintLayerIconTranslateAnchor,
    doc = "Controls the frame of reference for `icon-translate`.",
    default = "map".to_string()
);

color_prop!(
    SymbolPaintLayerTextColor,
    doc = "The color with which the text will be drawn.",
    default = serde_json::json!("#000000")
);

numeric_prop!(
    SymbolPaintLayerTextHaloBlur,
    doc = "The halo's fadeout distance towards the outside.",
    min = 0_f64,
    default = serde_json::Number::from_i128(0)
        .expect("the number is serialised from a number and is thus always valid")
);

color_prop!(
    SymbolPaintLayerTextHaloColor,
    doc = "The color of the text's halo, which helps it stand out from backgrounds.",
    default = serde_json::json!("rgba(0, 0, 0, 0)")
);

numeric_prop!(
    SymbolPaintLayerTextHaloWidth,
    doc = "Distance of halo to the font outline. Max text halo width is 1/4 of the font-size.",
    min = 0_f64,
    default = serde_json::Number::from_i128(0)
        .expect("the number is serialised from a number and is thus always valid")
);

numeric_prop!(
    SymbolPaintLayerTextOpacity,
    doc = "The opacity at which the text will be drawn.",
    min = 0_f64,
    max = 1_f64,
    default = serde_json::Number::from_i128(1)
        .expect("the number is serialised from a number and is thus always valid")
);

array_prop!(
    SymbolPaintLayerTextTranslate,
    doc = "Distance that the text's anchor is moved from its original placement. Positive values indicate right and down, while negative values indicate left and up.",
    default = serde_json::json!([0, 0])
);

string_prop!(
    SymbolPaintLayerTextTranslateAnchor,
    doc = "Controls the frame of reference for `text-translate`.",
    default = "map".to_string()
);

/// A style layer with its type-specific paint and layout properties.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(tag = "type")]
pub enum TypedLayer {
    #[serde(rename = "background")]
    Background {
        #[serde(flatten)]
        common: Layer,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        paint: Option<BackgroundPaintLayer>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        layout: Option<BackgroundLayoutLayer>,
    },
    #[serde(rename = "circle")]
    Circle {
        #[serde(flatten)]
        common: Layer,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        paint: Option<CirclePaintLayer>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        layout: Option<CircleLayoutLayer>,
    },
    #[serde(rename = "color-relief")]
    ColorRelief {
        #[serde(flatten)]
        common: Layer,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        paint: Option<ColorReliefPaintLayer>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        layout: Option<ColorReliefLayoutLayer>,
    },
    #[serde(rename = "fill")]
    Fill {
        #[serde(flatten)]
        common: Layer,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        paint: Option<FillPaintLayer>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        layout: Option<FillLayoutLayer>,
    },
    #[serde(rename = "fill-extrusion")]
    FillExtrusion {
        #[serde(flatten)]
        common: Layer,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        paint: Option<FillExtrusionPaintLayer>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        layout: Option<FillExtrusionLayoutLayer>,
    },
    #[serde(rename = "heatmap")]
    Heatmap {
        #[serde(flatten)]
        common: Layer,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        paint: Option<HeatmapPaintLayer>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        layout: Option<HeatmapLayoutLayer>,
    },
    #[serde(rename = "hillshade")]
    Hillshade {
        #[serde(flatten)]
        common: Layer,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        paint: Option<HillshadePaintLayer>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        layout: Option<HillshadeLayoutLayer>,
    },
    #[serde(rename = "line")]
    Line {
        #[serde(flatten)]
        common: Layer,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        paint: Option<LinePaintLayer>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        layout: Option<LineLayoutLayer>,
    },
    #[serde(rename = "raster")]
    Raster {
        #[serde(flatten)]
        common: Layer,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        paint: Option<RasterPaintLayer>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        layout: Option<RasterLayoutLayer>,
    },
    #[serde(rename = "symbol")]
    Symbol {
        #[serde(flatten)]
        common: Layer,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        paint: Option<SymbolPaintLayer>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        layout: Option<SymbolLayoutLayer>,
    },
}

/// A layer that inherits its type and properties from a referenced layer via `ref`.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RefLayer {
    pub id: LayerId,
    #[serde(rename = "ref")]
    pub r#ref: std::string::String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<Boolean>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minzoom: Option<LayerMinzoom>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maxzoom: Option<LayerMaxzoom>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<LayerMetadata>,
}

/// A layer in the style: either a fully typed layer or a `ref` layer.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum AnyLayer {
    Typed(TypedLayer),
    Ref(RefLayer),
}

impl serde::Serialize for AnyLayer {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Typed(v) => v.serialize(serializer),
            Self::Ref(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for AnyLayer {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <TypedLayer as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Typed(v)),
            Err(e) => errors.push(("Typed", e.to_string())),
        }
        match <RefLayer as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ref(v)),
            Err(e) => errors.push(("Ref", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "AnyLayer: no variant matched. Expected Typed(TypedLayer) | Ref(RefLayer). Errors: [{}]",
            details.join("; ")
        )))
    }
}

impl TypedLayer {
    /// Access the common `Layer` fields shared by all typed layers.
    pub fn common(&self) -> &Layer {
        match self {
            TypedLayer::Background { common, .. }
            | TypedLayer::Circle { common, .. }
            | TypedLayer::ColorRelief { common, .. }
            | TypedLayer::Fill { common, .. }
            | TypedLayer::FillExtrusion { common, .. }
            | TypedLayer::Heatmap { common, .. }
            | TypedLayer::Hillshade { common, .. }
            | TypedLayer::Line { common, .. }
            | TypedLayer::Raster { common, .. }
            | TypedLayer::Symbol { common, .. } => common,
        }
    }

    /// Mutably access the common `Layer` fields.
    pub fn common_mut(&mut self) -> &mut Layer {
        match self {
            TypedLayer::Background { common, .. }
            | TypedLayer::Circle { common, .. }
            | TypedLayer::ColorRelief { common, .. }
            | TypedLayer::Fill { common, .. }
            | TypedLayer::FillExtrusion { common, .. }
            | TypedLayer::Heatmap { common, .. }
            | TypedLayer::Hillshade { common, .. }
            | TypedLayer::Line { common, .. }
            | TypedLayer::Raster { common, .. }
            | TypedLayer::Symbol { common, .. } => common,
        }
    }

    /// The layer type string as it appears in JSON (e.g. `"fill"`, `"line"`).
    pub fn layer_type(&self) -> &'static str {
        match self {
            TypedLayer::Background { .. } => "background",
            TypedLayer::Circle { .. } => "circle",
            TypedLayer::ColorRelief { .. } => "color-relief",
            TypedLayer::Fill { .. } => "fill",
            TypedLayer::FillExtrusion { .. } => "fill-extrusion",
            TypedLayer::Heatmap { .. } => "heatmap",
            TypedLayer::Hillshade { .. } => "hillshade",
            TypedLayer::Line { .. } => "line",
            TypedLayer::Raster { .. } => "raster",
            TypedLayer::Symbol { .. } => "symbol",
        }
    }
}

impl AnyLayer {
    /// Get the layer ID regardless of layer kind.
    pub fn id(&self) -> &LayerId {
        match self {
            AnyLayer::Typed(t) => &t.common().id,
            AnyLayer::Ref(r) => &r.id,
        }
    }

    /// Access the common `Layer` if this is a typed layer.
    pub fn common(&self) -> Option<&Layer> {
        match self {
            AnyLayer::Typed(t) => Some(t.common()),
            AnyLayer::Ref(_) => None,
        }
    }

    /// Access the common `Layer` mutably if this is a typed layer.
    pub fn common_mut(&mut self) -> Option<&mut Layer> {
        match self {
            AnyLayer::Typed(t) => Some(t.common_mut()),
            AnyLayer::Ref(_) => None,
        }
    }

    /// The effective layer type string, or `None` for ref layers.
    pub fn layer_type(&self) -> Option<&'static str> {
        match self {
            AnyLayer::Typed(t) => Some(t.layer_type()),
            AnyLayer::Ref(_) => None,
        }
    }

    /// Get the source name if this is a typed layer with a source.
    pub fn source(&self) -> Option<&str> {
        self.common()?.source.as_ref().map(|s| s.as_str())
    }

    /// Get the source-layer name if this is a typed layer with one.
    pub fn source_layer(&self) -> Option<&str> {
        self.common()?.source_layer.as_ref().map(|s| s.as_str())
    }
}

impl LayerId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl LayerSource {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl LayerSourceLayer {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Boolean {
    /// Returns `true` if this filter is the literal `false` (layer never renders).
    pub fn is_always_false(&self) -> bool {
        matches!(self, Boolean::Literal(false))
    }

    /// Returns `true` if this filter is the literal `true` (layer always renders).
    pub fn is_always_true(&self) -> bool {
        matches!(self, Boolean::Literal(true))
    }

    /// Serialize to `serde_json::Value` for passes that still operate on JSON.
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::to_value(self).expect("Boolean serialization is infallible")
    }

    /// Deserialize from `serde_json::Value`.  Returns `None` if the value is not
    /// a valid filter (e.g. a string or object).
    ///
    /// A double round-trip normalises `ExprOrLiteral` values: optimizer-produced
    /// `["literal", true]` deserialises as `BooleanExpr(Literal(true))` on the
    /// first pass, then serialises to bare `true`, which becomes `Bool(true)` on
    /// the second pass.
    pub fn from_value(v: serde_json::Value) -> Option<Self> {
        let filter: Self = serde_json::from_value(v).ok()?;
        let normalised = serde_json::to_value(&filter).ok()?;
        serde_json::from_value(normalised).ok()
    }
}

impl LayerMinzoom {
    pub fn as_f64(&self) -> Option<f64> {
        self.0.as_f64()
    }

    pub fn from_f64(n: f64) -> Option<Self> {
        serde_json::Number::from_f64(n).map(Self)
    }
}

impl LayerMaxzoom {
    pub fn as_f64(&self) -> Option<f64> {
        self.0.as_f64()
    }

    pub fn from_f64(n: f64) -> Option<Self> {
        serde_json::Number::from_f64(n).map(Self)
    }
}

/// Walk all expression-backed paint/layout properties on typed layer structs.
///
/// For each layer, visits the filter (if present), then each expression-backed
/// paint and layout property, then the layer itself via `visit_layer`.
#[expect(clippy::collapsible_if)]
pub fn walk_typed_style_mut(
    style: &mut MaplibreStyleSpecification,
    visitor: &mut impl crate::typed_visitor::TypedStyleVisitor,
) {
    for (i, layer) in style.layers.iter_mut().enumerate() {
        let AnyLayer::Typed(typed) = layer else {
            continue;
        };
        let layer_type = typed.layer_type();

        if let Some(filter) = &mut typed.common_mut().filter {
            visitor.visit_filter(i, layer_type, filter);
        }

        match typed {
            TypedLayer::Background { paint, .. } => {
                if let Some(paint) = paint {
                    if let Some(v) = &mut paint.background_color {
                        visitor.visit_color(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "background",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "background-color",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.background_opacity {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "background",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "background-opacity",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.background_pattern {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "background",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "background-pattern",
                            },
                            &mut v.0,
                        );
                    }
                }
            }
            TypedLayer::Circle { paint, layout, .. } => {
                if let Some(paint) = paint {
                    if let Some(v) = &mut paint.circle_blur {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "circle",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "circle-blur",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.circle_color {
                        visitor.visit_color(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "circle",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "circle-color",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.circle_opacity {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "circle",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "circle-opacity",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.circle_pitch_alignment {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "circle",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "circle-pitch-alignment",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.circle_pitch_scale {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "circle",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "circle-pitch-scale",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.circle_radius {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "circle",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "circle-radius",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.circle_stroke_color {
                        visitor.visit_color(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "circle",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "circle-stroke-color",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.circle_stroke_opacity {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "circle",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "circle-stroke-opacity",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.circle_stroke_width {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "circle",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "circle-stroke-width",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.circle_translate {
                        visitor.visit_array(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "circle",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "circle-translate",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.circle_translate_anchor {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "circle",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "circle-translate-anchor",
                            },
                            &mut v.0,
                        );
                    }
                }
                if let Some(layout) = layout {
                    if let Some(v) = &mut layout.circle_sort_key {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "circle",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "circle-sort-key",
                            },
                            &mut v.0,
                        );
                    }
                }
            }
            TypedLayer::ColorRelief { paint, .. } => {
                if let Some(paint) = paint {
                    if let Some(v) = &mut paint.color_relief_color {
                        visitor.visit_color(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "color-relief",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "color-relief-color",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.color_relief_opacity {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "color-relief",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "color-relief-opacity",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.resampling {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "color-relief",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "resampling",
                            },
                            &mut v.0,
                        );
                    }
                }
            }
            TypedLayer::Fill { paint, layout, .. } => {
                if let Some(paint) = paint {
                    if let Some(v) = &mut paint.fill_antialias {
                        visitor.visit_boolean_prop(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "fill",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "fill-antialias",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.fill_color {
                        visitor.visit_color(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "fill",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "fill-color",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.fill_opacity {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "fill",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "fill-opacity",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.fill_outline_color {
                        visitor.visit_color(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "fill",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "fill-outline-color",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.fill_pattern {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "fill",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "fill-pattern",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.fill_translate {
                        visitor.visit_array(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "fill",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "fill-translate",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.fill_translate_anchor {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "fill",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "fill-translate-anchor",
                            },
                            &mut v.0,
                        );
                    }
                }
                if let Some(layout) = layout {
                    if let Some(v) = &mut layout.fill_sort_key {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "fill",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "fill-sort-key",
                            },
                            &mut v.0,
                        );
                    }
                }
            }
            TypedLayer::FillExtrusion { paint, .. } => {
                if let Some(paint) = paint {
                    if let Some(v) = &mut paint.fill_extrusion_base {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "fill-extrusion",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "fill-extrusion-base",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.fill_extrusion_color {
                        visitor.visit_color(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "fill-extrusion",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "fill-extrusion-color",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.fill_extrusion_height {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "fill-extrusion",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "fill-extrusion-height",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.fill_extrusion_opacity {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "fill-extrusion",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "fill-extrusion-opacity",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.fill_extrusion_pattern {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "fill-extrusion",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "fill-extrusion-pattern",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.fill_extrusion_translate {
                        visitor.visit_array(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "fill-extrusion",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "fill-extrusion-translate",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.fill_extrusion_translate_anchor {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "fill-extrusion",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "fill-extrusion-translate-anchor",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.fill_extrusion_vertical_gradient {
                        visitor.visit_boolean_prop(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "fill-extrusion",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "fill-extrusion-vertical-gradient",
                            },
                            &mut v.0,
                        );
                    }
                }
            }
            TypedLayer::Heatmap { paint, .. } => {
                if let Some(paint) = paint {
                    if let Some(v) = &mut paint.heatmap_color {
                        visitor.visit_color(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "heatmap",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "heatmap-color",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.heatmap_intensity {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "heatmap",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "heatmap-intensity",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.heatmap_opacity {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "heatmap",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "heatmap-opacity",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.heatmap_radius {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "heatmap",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "heatmap-radius",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.heatmap_weight {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "heatmap",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "heatmap-weight",
                            },
                            &mut v.0,
                        );
                    }
                }
            }
            TypedLayer::Hillshade { paint, .. } => {
                if let Some(paint) = paint {
                    if let Some(v) = &mut paint.hillshade_accent_color {
                        visitor.visit_color(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "hillshade",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "hillshade-accent-color",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.hillshade_exaggeration {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "hillshade",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "hillshade-exaggeration",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.hillshade_highlight_color {
                        visitor.visit_color(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "hillshade",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "hillshade-highlight-color",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.hillshade_illumination_altitude {
                        visitor.visit_array(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "hillshade",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "hillshade-illumination-altitude",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.hillshade_illumination_anchor {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "hillshade",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "hillshade-illumination-anchor",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.hillshade_illumination_direction {
                        visitor.visit_array(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "hillshade",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "hillshade-illumination-direction",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.hillshade_method {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "hillshade",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "hillshade-method",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.hillshade_shadow_color {
                        visitor.visit_color(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "hillshade",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "hillshade-shadow-color",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.resampling {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "hillshade",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "resampling",
                            },
                            &mut v.0,
                        );
                    }
                }
            }
            TypedLayer::Line { paint, layout, .. } => {
                if let Some(paint) = paint {
                    if let Some(v) = &mut paint.line_blur {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "line",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "line-blur",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.line_color {
                        visitor.visit_color(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "line",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "line-color",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.line_dasharray {
                        visitor.visit_array(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "line",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "line-dasharray",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.line_gap_width {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "line",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "line-gap-width",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.line_gradient {
                        visitor.visit_color(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "line",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "line-gradient",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.line_offset {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "line",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "line-offset",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.line_opacity {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "line",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "line-opacity",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.line_pattern {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "line",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "line-pattern",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.line_translate {
                        visitor.visit_array(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "line",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "line-translate",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.line_translate_anchor {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "line",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "line-translate-anchor",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.line_width {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "line",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "line-width",
                            },
                            &mut v.0,
                        );
                    }
                }
                if let Some(layout) = layout {
                    if let Some(v) = &mut layout.line_cap {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "line",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "line-cap",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.line_join {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "line",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "line-join",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.line_miter_limit {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "line",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "line-miter-limit",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.line_round_limit {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "line",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "line-round-limit",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.line_sort_key {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "line",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "line-sort-key",
                            },
                            &mut v.0,
                        );
                    }
                }
            }
            TypedLayer::Raster { paint, .. } => {
                if let Some(paint) = paint {
                    if let Some(v) = &mut paint.raster_brightness_max {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "raster",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "raster-brightness-max",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.raster_brightness_min {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "raster",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "raster-brightness-min",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.raster_contrast {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "raster",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "raster-contrast",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.raster_fade_duration {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "raster",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "raster-fade-duration",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.raster_hue_rotate {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "raster",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "raster-hue-rotate",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.raster_opacity {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "raster",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "raster-opacity",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.raster_resampling {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "raster",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "raster-resampling",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.raster_saturation {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "raster",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "raster-saturation",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.resampling {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "raster",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "resampling",
                            },
                            &mut v.0,
                        );
                    }
                }
            }
            TypedLayer::Symbol { paint, layout, .. } => {
                if let Some(paint) = paint {
                    if let Some(v) = &mut paint.icon_color {
                        visitor.visit_color(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "icon-color",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.icon_halo_blur {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "icon-halo-blur",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.icon_halo_color {
                        visitor.visit_color(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "icon-halo-color",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.icon_halo_width {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "icon-halo-width",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.icon_opacity {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "icon-opacity",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.icon_translate {
                        visitor.visit_array(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "icon-translate",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.icon_translate_anchor {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "icon-translate-anchor",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.text_color {
                        visitor.visit_color(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "text-color",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.text_halo_blur {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "text-halo-blur",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.text_halo_color {
                        visitor.visit_color(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "text-halo-color",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.text_halo_width {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "text-halo-width",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.text_opacity {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "text-opacity",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.text_translate {
                        visitor.visit_array(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "text-translate",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut paint.text_translate_anchor {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Paint,
                                property_name: "text-translate-anchor",
                            },
                            &mut v.0,
                        );
                    }
                }
                if let Some(layout) = layout {
                    if let Some(v) = &mut layout.icon_allow_overlap {
                        visitor.visit_boolean_prop(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "icon-allow-overlap",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.icon_anchor {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "icon-anchor",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.icon_ignore_placement {
                        visitor.visit_boolean_prop(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "icon-ignore-placement",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.icon_image {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "icon-image",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.icon_keep_upright {
                        visitor.visit_boolean_prop(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "icon-keep-upright",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.icon_offset {
                        visitor.visit_array(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "icon-offset",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.icon_optional {
                        visitor.visit_boolean_prop(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "icon-optional",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.icon_overlap {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "icon-overlap",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.icon_padding {
                        visitor.visit_array(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "icon-padding",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.icon_pitch_alignment {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "icon-pitch-alignment",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.icon_rotate {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "icon-rotate",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.icon_rotation_alignment {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "icon-rotation-alignment",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.icon_size {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "icon-size",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.icon_text_fit {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "icon-text-fit",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.icon_text_fit_padding {
                        visitor.visit_array(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "icon-text-fit-padding",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.symbol_avoid_edges {
                        visitor.visit_boolean_prop(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "symbol-avoid-edges",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.symbol_placement {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "symbol-placement",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.symbol_sort_key {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "symbol-sort-key",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.symbol_spacing {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "symbol-spacing",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.symbol_z_order {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "symbol-z-order",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.text_allow_overlap {
                        visitor.visit_boolean_prop(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "text-allow-overlap",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.text_anchor {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "text-anchor",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.text_field {
                        visitor.visit_formatted(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "text-field",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.text_font {
                        visitor.visit_array(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "text-font",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.text_ignore_placement {
                        visitor.visit_boolean_prop(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "text-ignore-placement",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.text_justify {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "text-justify",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.text_keep_upright {
                        visitor.visit_boolean_prop(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "text-keep-upright",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.text_letter_spacing {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "text-letter-spacing",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.text_line_height {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "text-line-height",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.text_max_angle {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "text-max-angle",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.text_max_width {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "text-max-width",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.text_offset {
                        visitor.visit_array(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "text-offset",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.text_optional {
                        visitor.visit_boolean_prop(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "text-optional",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.text_overlap {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "text-overlap",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.text_padding {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "text-padding",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.text_pitch_alignment {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "text-pitch-alignment",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.text_radial_offset {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "text-radial-offset",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.text_rotate {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "text-rotate",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.text_rotation_alignment {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "text-rotation-alignment",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.text_size {
                        visitor.visit_numeric(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "text-size",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.text_transform {
                        visitor.visit_string(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "text-transform",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.text_variable_anchor {
                        visitor.visit_array(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "text-variable-anchor",
                            },
                            &mut v.0,
                        );
                    }
                    if let Some(v) = &mut layout.text_writing_mode {
                        visitor.visit_array(
                            &crate::typed_visitor::TypedPropertyContext {
                                layer_index: i,
                                layer_type: "symbol",
                                section: crate::mir::MirPropertySection::Layout,
                                property_name: "text-writing-mode",
                            },
                            &mut v.0,
                        );
                    }
                }
            }
        }

        visitor.visit_layer(i, layer_type, typed);
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod test {
    use super::*;
}

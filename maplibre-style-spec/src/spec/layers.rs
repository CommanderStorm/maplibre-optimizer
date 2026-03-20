#[allow(unused_imports)]
use super::*;

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
    Expr(Box<BackgroundPaintLayerBackgroundColorExpression>),
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
    Expr(Box<BackgroundPaintLayerBackgroundOpacityExpression>),
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
    Expr(Box<CircleLayoutLayerCircleSortKeyExpression>),
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
    Expr(Box<CirclePaintLayerCircleBlurExpression>),
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
    Expr(Box<CirclePaintLayerCircleColorExpression>),
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
    Expr(Box<CirclePaintLayerCircleOpacityExpression>),
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
    Expr(Box<CirclePaintLayerCircleRadiusExpression>),
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
    Expr(Box<CirclePaintLayerCircleStrokeColorExpression>),
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
    Expr(Box<CirclePaintLayerCircleStrokeOpacityExpression>),
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
    Expr(Box<CirclePaintLayerCircleStrokeWidthExpression>),
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
    Expr(Box<ColorReliefPaintLayerColorReliefColorExpression>),
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
    Expr(Box<ColorReliefPaintLayerColorReliefOpacityExpression>),
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
    Expr(Box<FillLayoutLayerFillSortKeyExpression>),
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
    Expr(Box<FillPaintLayerFillColorExpression>),
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
    Expr(Box<FillPaintLayerFillOpacityExpression>),
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
    Expr(Box<FillPaintLayerFillOutlineColorExpression>),
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
    Expr(Box<FillExtrusionPaintLayerFillExtrusionBaseExpression>),
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
    Expr(Box<FillExtrusionPaintLayerFillExtrusionColorExpression>),
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
    Expr(Box<FillExtrusionPaintLayerFillExtrusionHeightExpression>),
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
    Expr(Box<FillExtrusionPaintLayerFillExtrusionOpacityExpression>),
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
    Expr(Box<HeatmapPaintLayerHeatmapColorExpression>),
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
    Expr(Box<HeatmapPaintLayerHeatmapIntensityExpression>),
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
    Expr(Box<HeatmapPaintLayerHeatmapOpacityExpression>),
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
    Expr(Box<HeatmapPaintLayerHeatmapRadiusExpression>),
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
    Expr(Box<HeatmapPaintLayerHeatmapWeightExpression>),
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
    Expr(Box<HillshadePaintLayerHillshadeAccentColorExpression>),
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
    Expr(Box<HillshadePaintLayerHillshadeExaggerationExpression>),
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
    Expr(Box<LineLayoutLayerLineMiterLimitExpression>),
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
    Expr(Box<LineLayoutLayerLineRoundLimitExpression>),
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
    Expr(Box<LineLayoutLayerLineSortKeyExpression>),
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
    Expr(Box<LinePaintLayerLineBlurExpression>),
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
    Expr(Box<LinePaintLayerLineColorExpression>),
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
    Expr(Box<LinePaintLayerLineGapWidthExpression>),
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
    Expr(Box<LinePaintLayerLineGradientExpression>),
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
    Expr(Box<LinePaintLayerLineOffsetExpression>),
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
    Expr(Box<LinePaintLayerLineOpacityExpression>),
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
    Expr(Box<LinePaintLayerLineWidthExpression>),
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
    Expr(Box<RasterPaintLayerRasterBrightnessMaxExpression>),
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
    Expr(Box<RasterPaintLayerRasterBrightnessMinExpression>),
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
    Expr(Box<RasterPaintLayerRasterContrastExpression>),
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
    Expr(Box<RasterPaintLayerRasterFadeDurationExpression>),
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
    Expr(Box<RasterPaintLayerRasterHueRotateExpression>),
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
    Expr(Box<RasterPaintLayerRasterOpacityExpression>),
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
    Expr(Box<RasterPaintLayerRasterSaturationExpression>),
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
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerIconAllowOverlap(bool);

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
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerIconIgnorePlacement(bool);

/// Name of image in sprite to use for drawing an image background.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerIconImage(std::string::String);

/// If true, the icon may be flipped to prevent it from being rendered upside-down.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerIconKeepUpright(bool);

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
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerIconOptional(bool);

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
    Expr(Box<SymbolLayoutLayerIconRotateExpression>),
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
    Expr(Box<SymbolLayoutLayerIconSizeExpression>),
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
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerSymbolAvoidEdges(bool);

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
    Expr(Box<SymbolLayoutLayerSymbolSortKeyExpression>),
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
    Expr(Box<SymbolLayoutLayerSymbolSpacingExpression>),
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
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerTextAllowOverlap(bool);

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
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerTextIgnorePlacement(bool);

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
    Expr(Box<SymbolLayoutLayerTextLetterSpacingExpression>),
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
    Expr(Box<SymbolLayoutLayerTextLineHeightExpression>),
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
    Expr(Box<SymbolLayoutLayerTextMaxAngleExpression>),
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
    Expr(Box<SymbolLayoutLayerTextMaxWidthExpression>),
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
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct SymbolLayoutLayerTextOptional(bool);

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
    Expr(Box<SymbolLayoutLayerTextPaddingExpression>),
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
    Expr(Box<SymbolLayoutLayerTextRadialOffsetExpression>),
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
    Expr(Box<SymbolLayoutLayerTextRotateExpression>),
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
    Expr(Box<SymbolLayoutLayerTextSizeExpression>),
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
    Expr(Box<SymbolPaintLayerIconColorExpression>),
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
    Expr(Box<SymbolPaintLayerIconHaloBlurExpression>),
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
    Expr(Box<SymbolPaintLayerIconHaloColorExpression>),
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
    Expr(Box<SymbolPaintLayerIconHaloWidthExpression>),
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
    Expr(Box<SymbolPaintLayerIconOpacityExpression>),
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
    Expr(Box<SymbolPaintLayerTextColorExpression>),
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
    Expr(Box<SymbolPaintLayerTextHaloBlurExpression>),
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
    Expr(Box<SymbolPaintLayerTextHaloColorExpression>),
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
    Expr(Box<SymbolPaintLayerTextHaloWidthExpression>),
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
    Expr(Box<SymbolPaintLayerTextOpacityExpression>),
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

#[cfg(test)]
#[allow(unused_imports)]
mod test {
    use super::*;
}

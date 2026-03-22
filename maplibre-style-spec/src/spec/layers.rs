#![allow(clippy::large_enum_variant)]
#[allow(unused_imports)]
use super::*;

/// A filter expression — semantically a boolean expression or literal,
/// stored as `serde_json::Value` for lossless JSON round-tripping.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct LayerFilter(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
    serde_json::Value,
);

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct Layer {
    /// A expression specifying conditions on source features. Only features that match the filter are displayed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<LayerFilter>,
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

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum BackgroundPaintLayerBackgroundColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

impl serde::Serialize for BackgroundPaintLayerBackgroundColorExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Color(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for BackgroundPaintLayerBackgroundColorExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Color as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Color(v)),
            Err(e) => errors.push(("Color", e.to_string())),
        }
        match <ColorOrArrayOfColor as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "BackgroundPaintLayerBackgroundColorExpression: no variant matched. Expected Color(Color) | Ramp(ColorOrArrayOfColor). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The color with which the background will be drawn.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum BackgroundPaintLayerBackgroundColor {
    Expr(Box<BackgroundPaintLayerBackgroundColorExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl serde::Serialize for BackgroundPaintLayerBackgroundColor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for BackgroundPaintLayerBackgroundColor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <BackgroundPaintLayerBackgroundColorExpression as serde::Deserialize>::deserialize(
            &value,
        ) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Value as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "BackgroundPaintLayerBackgroundColor: no variant matched. Expected Expr(BackgroundPaintLayerBackgroundColorExpression) | Literal(serde_json::Value). Errors: [{}]",
            details.join("; ")
        )))
    }
}

impl Default for BackgroundPaintLayerBackgroundColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("#000000"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum BackgroundPaintLayerBackgroundOpacityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for BackgroundPaintLayerBackgroundOpacityExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for BackgroundPaintLayerBackgroundOpacityExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "BackgroundPaintLayerBackgroundOpacityExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The opacity at which the background will be drawn.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum BackgroundPaintLayerBackgroundOpacity {
    Expr(Box<BackgroundPaintLayerBackgroundOpacityExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for BackgroundPaintLayerBackgroundOpacity {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for BackgroundPaintLayerBackgroundOpacity {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <BackgroundPaintLayerBackgroundOpacityExpression as serde::Deserialize>::deserialize(
            &value,
        ) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "BackgroundPaintLayerBackgroundOpacity: no variant matched. Expected Expr(BackgroundPaintLayerBackgroundOpacityExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circle_sort_key: Option<CircleLayoutLayerCircleSortKey>,
    /// Whether this layer is displayed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<CircleLayoutLayerVisibility>,
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum CircleLayoutLayerCircleSortKeyExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for CircleLayoutLayerCircleSortKeyExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for CircleLayoutLayerCircleSortKeyExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "CircleLayoutLayerCircleSortKeyExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum CircleLayoutLayerCircleSortKey {
    Expr(Box<CircleLayoutLayerCircleSortKeyExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for CircleLayoutLayerCircleSortKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for CircleLayoutLayerCircleSortKey {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <CircleLayoutLayerCircleSortKeyExpression as serde::Deserialize>::deserialize(&value)
        {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "CircleLayoutLayerCircleSortKey: no variant matched. Expected Expr(CircleLayoutLayerCircleSortKeyExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum CirclePaintLayerCircleBlurExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for CirclePaintLayerCircleBlurExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for CirclePaintLayerCircleBlurExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "CirclePaintLayerCircleBlurExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Amount to blur the circle. 1 blurs the circle such that only the centerpoint is full opacity.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum CirclePaintLayerCircleBlur {
    Expr(Box<CirclePaintLayerCircleBlurExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for CirclePaintLayerCircleBlur {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for CirclePaintLayerCircleBlur {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <CirclePaintLayerCircleBlurExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "CirclePaintLayerCircleBlur: no variant matched. Expected Expr(CirclePaintLayerCircleBlurExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum CirclePaintLayerCircleColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

impl serde::Serialize for CirclePaintLayerCircleColorExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Color(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for CirclePaintLayerCircleColorExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Color as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Color(v)),
            Err(e) => errors.push(("Color", e.to_string())),
        }
        match <ColorOrArrayOfColor as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "CirclePaintLayerCircleColorExpression: no variant matched. Expected Color(Color) | Ramp(ColorOrArrayOfColor). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The fill color of the circle.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum CirclePaintLayerCircleColor {
    Expr(Box<CirclePaintLayerCircleColorExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl serde::Serialize for CirclePaintLayerCircleColor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for CirclePaintLayerCircleColor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <CirclePaintLayerCircleColorExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Value as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "CirclePaintLayerCircleColor: no variant matched. Expected Expr(CirclePaintLayerCircleColorExpression) | Literal(serde_json::Value). Errors: [{}]",
            details.join("; ")
        )))
    }
}

impl Default for CirclePaintLayerCircleColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("#000000"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum CirclePaintLayerCircleOpacityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for CirclePaintLayerCircleOpacityExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for CirclePaintLayerCircleOpacityExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "CirclePaintLayerCircleOpacityExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The opacity at which the circle will be drawn.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum CirclePaintLayerCircleOpacity {
    Expr(Box<CirclePaintLayerCircleOpacityExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for CirclePaintLayerCircleOpacity {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for CirclePaintLayerCircleOpacity {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <CirclePaintLayerCircleOpacityExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "CirclePaintLayerCircleOpacity: no variant matched. Expected Expr(CirclePaintLayerCircleOpacityExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum CirclePaintLayerCircleRadiusExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for CirclePaintLayerCircleRadiusExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for CirclePaintLayerCircleRadiusExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "CirclePaintLayerCircleRadiusExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Circle radius.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum CirclePaintLayerCircleRadius {
    Expr(Box<CirclePaintLayerCircleRadiusExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for CirclePaintLayerCircleRadius {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for CirclePaintLayerCircleRadius {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <CirclePaintLayerCircleRadiusExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "CirclePaintLayerCircleRadius: no variant matched. Expected Expr(CirclePaintLayerCircleRadiusExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum CirclePaintLayerCircleStrokeColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

impl serde::Serialize for CirclePaintLayerCircleStrokeColorExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Color(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for CirclePaintLayerCircleStrokeColorExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Color as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Color(v)),
            Err(e) => errors.push(("Color", e.to_string())),
        }
        match <ColorOrArrayOfColor as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "CirclePaintLayerCircleStrokeColorExpression: no variant matched. Expected Color(Color) | Ramp(ColorOrArrayOfColor). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The stroke color of the circle.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum CirclePaintLayerCircleStrokeColor {
    Expr(Box<CirclePaintLayerCircleStrokeColorExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl serde::Serialize for CirclePaintLayerCircleStrokeColor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for CirclePaintLayerCircleStrokeColor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <CirclePaintLayerCircleStrokeColorExpression as serde::Deserialize>::deserialize(
            &value,
        ) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Value as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "CirclePaintLayerCircleStrokeColor: no variant matched. Expected Expr(CirclePaintLayerCircleStrokeColorExpression) | Literal(serde_json::Value). Errors: [{}]",
            details.join("; ")
        )))
    }
}

impl Default for CirclePaintLayerCircleStrokeColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("#000000"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum CirclePaintLayerCircleStrokeOpacityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for CirclePaintLayerCircleStrokeOpacityExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for CirclePaintLayerCircleStrokeOpacityExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "CirclePaintLayerCircleStrokeOpacityExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The opacity of the circle's stroke.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum CirclePaintLayerCircleStrokeOpacity {
    Expr(Box<CirclePaintLayerCircleStrokeOpacityExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for CirclePaintLayerCircleStrokeOpacity {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for CirclePaintLayerCircleStrokeOpacity {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <CirclePaintLayerCircleStrokeOpacityExpression as serde::Deserialize>::deserialize(
            &value,
        ) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "CirclePaintLayerCircleStrokeOpacity: no variant matched. Expected Expr(CirclePaintLayerCircleStrokeOpacityExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum CirclePaintLayerCircleStrokeWidthExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for CirclePaintLayerCircleStrokeWidthExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for CirclePaintLayerCircleStrokeWidthExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "CirclePaintLayerCircleStrokeWidthExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The width of the circle's stroke. Strokes are placed outside of the `circle-radius`.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum CirclePaintLayerCircleStrokeWidth {
    Expr(Box<CirclePaintLayerCircleStrokeWidthExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for CirclePaintLayerCircleStrokeWidth {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for CirclePaintLayerCircleStrokeWidth {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <CirclePaintLayerCircleStrokeWidthExpression as serde::Deserialize>::deserialize(
            &value,
        ) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "CirclePaintLayerCircleStrokeWidth: no variant matched. Expected Expr(CirclePaintLayerCircleStrokeWidthExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color_relief_color: Option<ColorReliefPaintLayerColorReliefColor>,
    /// The opacity at which the color-relief will be drawn.
    #[serde(rename = "color-relief-opacity")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color_relief_opacity: Option<ColorReliefPaintLayerColorReliefOpacity>,
}

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum ColorReliefPaintLayerColorReliefColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

impl serde::Serialize for ColorReliefPaintLayerColorReliefColorExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Color(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for ColorReliefPaintLayerColorReliefColorExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Color as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Color(v)),
            Err(e) => errors.push(("Color", e.to_string())),
        }
        match <ColorOrArrayOfColor as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "ColorReliefPaintLayerColorReliefColorExpression: no variant matched. Expected Color(Color) | Ramp(ColorOrArrayOfColor). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Defines the color of each pixel based on its elevation. Should be an expression that uses `["elevation"]` as input.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum ColorReliefPaintLayerColorReliefColor {
    Expr(Box<ColorReliefPaintLayerColorReliefColorExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl serde::Serialize for ColorReliefPaintLayerColorReliefColor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for ColorReliefPaintLayerColorReliefColor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <ColorReliefPaintLayerColorReliefColorExpression as serde::Deserialize>::deserialize(
            &value,
        ) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Value as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "ColorReliefPaintLayerColorReliefColor: no variant matched. Expected Expr(ColorReliefPaintLayerColorReliefColorExpression) | Literal(serde_json::Value). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum ColorReliefPaintLayerColorReliefOpacityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for ColorReliefPaintLayerColorReliefOpacityExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for ColorReliefPaintLayerColorReliefOpacityExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "ColorReliefPaintLayerColorReliefOpacityExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The opacity at which the color-relief will be drawn.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum ColorReliefPaintLayerColorReliefOpacity {
    Expr(Box<ColorReliefPaintLayerColorReliefOpacityExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for ColorReliefPaintLayerColorReliefOpacity {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for ColorReliefPaintLayerColorReliefOpacity {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <ColorReliefPaintLayerColorReliefOpacityExpression as serde::Deserialize>::deserialize(
            &value,
        ) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "ColorReliefPaintLayerColorReliefOpacity: no variant matched. Expected Expr(ColorReliefPaintLayerColorReliefOpacityExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_sort_key: Option<FillLayoutLayerFillSortKey>,
    /// Whether this layer is displayed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<FillLayoutLayerVisibility>,
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FillLayoutLayerFillSortKeyExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for FillLayoutLayerFillSortKeyExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for FillLayoutLayerFillSortKeyExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "FillLayoutLayerFillSortKeyExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FillLayoutLayerFillSortKey {
    Expr(Box<FillLayoutLayerFillSortKeyExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for FillLayoutLayerFillSortKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for FillLayoutLayerFillSortKey {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <FillLayoutLayerFillSortKeyExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "FillLayoutLayerFillSortKey: no variant matched. Expected Expr(FillLayoutLayerFillSortKeyExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FillPaintLayerFillColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

impl serde::Serialize for FillPaintLayerFillColorExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Color(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for FillPaintLayerFillColorExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Color as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Color(v)),
            Err(e) => errors.push(("Color", e.to_string())),
        }
        match <ColorOrArrayOfColor as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "FillPaintLayerFillColorExpression: no variant matched. Expected Color(Color) | Ramp(ColorOrArrayOfColor). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The color of the filled part of this layer. This color can be specified as `rgba` with an alpha component and the color's opacity will not affect the opacity of the 1px stroke, if it is used.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FillPaintLayerFillColor {
    Expr(Box<FillPaintLayerFillColorExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl serde::Serialize for FillPaintLayerFillColor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for FillPaintLayerFillColor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <FillPaintLayerFillColorExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Value as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "FillPaintLayerFillColor: no variant matched. Expected Expr(FillPaintLayerFillColorExpression) | Literal(serde_json::Value). Errors: [{}]",
            details.join("; ")
        )))
    }
}

impl Default for FillPaintLayerFillColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("#000000"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FillPaintLayerFillOpacityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for FillPaintLayerFillOpacityExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for FillPaintLayerFillOpacityExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "FillPaintLayerFillOpacityExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The opacity of the entire fill layer. In contrast to the `fill-color`, this value will also affect the 1px stroke around the fill, if the stroke is used.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FillPaintLayerFillOpacity {
    Expr(Box<FillPaintLayerFillOpacityExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for FillPaintLayerFillOpacity {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for FillPaintLayerFillOpacity {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <FillPaintLayerFillOpacityExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "FillPaintLayerFillOpacity: no variant matched. Expected Expr(FillPaintLayerFillOpacityExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FillPaintLayerFillOutlineColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

impl serde::Serialize for FillPaintLayerFillOutlineColorExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Color(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for FillPaintLayerFillOutlineColorExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Color as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Color(v)),
            Err(e) => errors.push(("Color", e.to_string())),
        }
        match <ColorOrArrayOfColor as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "FillPaintLayerFillOutlineColorExpression: no variant matched. Expected Color(Color) | Ramp(ColorOrArrayOfColor). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The outline color of the fill. Matches the value of `fill-color` if unspecified.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FillPaintLayerFillOutlineColor {
    Expr(Box<FillPaintLayerFillOutlineColorExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl serde::Serialize for FillPaintLayerFillOutlineColor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for FillPaintLayerFillOutlineColor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <FillPaintLayerFillOutlineColorExpression as serde::Deserialize>::deserialize(&value)
        {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Value as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "FillPaintLayerFillOutlineColor: no variant matched. Expected Expr(FillPaintLayerFillOutlineColorExpression) | Literal(serde_json::Value). Errors: [{}]",
            details.join("; ")
        )))
    }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
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

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FillExtrusionPaintLayerFillExtrusionBaseExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for FillExtrusionPaintLayerFillExtrusionBaseExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for FillExtrusionPaintLayerFillExtrusionBaseExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "FillExtrusionPaintLayerFillExtrusionBaseExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The height with which to extrude the base of this layer. Must be less than or equal to `fill-extrusion-height`.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FillExtrusionPaintLayerFillExtrusionBase {
    Expr(Box<FillExtrusionPaintLayerFillExtrusionBaseExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for FillExtrusionPaintLayerFillExtrusionBase {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for FillExtrusionPaintLayerFillExtrusionBase {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <FillExtrusionPaintLayerFillExtrusionBaseExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "FillExtrusionPaintLayerFillExtrusionBase: no variant matched. Expected Expr(FillExtrusionPaintLayerFillExtrusionBaseExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FillExtrusionPaintLayerFillExtrusionColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

impl serde::Serialize for FillExtrusionPaintLayerFillExtrusionColorExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Color(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for FillExtrusionPaintLayerFillExtrusionColorExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Color as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Color(v)),
            Err(e) => errors.push(("Color", e.to_string())),
        }
        match <ColorOrArrayOfColor as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "FillExtrusionPaintLayerFillExtrusionColorExpression: no variant matched. Expected Color(Color) | Ramp(ColorOrArrayOfColor). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The base color of the extruded fill. The extrusion's surfaces will be shaded differently based on this color in combination with the root `light` settings. If this color is specified as `rgba` with an alpha component, the alpha component will be ignored; use `fill-extrusion-opacity` to set layer opacity.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FillExtrusionPaintLayerFillExtrusionColor {
    Expr(Box<FillExtrusionPaintLayerFillExtrusionColorExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl serde::Serialize for FillExtrusionPaintLayerFillExtrusionColor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for FillExtrusionPaintLayerFillExtrusionColor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <FillExtrusionPaintLayerFillExtrusionColorExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Value as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "FillExtrusionPaintLayerFillExtrusionColor: no variant matched. Expected Expr(FillExtrusionPaintLayerFillExtrusionColorExpression) | Literal(serde_json::Value). Errors: [{}]",
            details.join("; ")
        )))
    }
}

impl Default for FillExtrusionPaintLayerFillExtrusionColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("#000000"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FillExtrusionPaintLayerFillExtrusionHeightExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for FillExtrusionPaintLayerFillExtrusionHeightExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for FillExtrusionPaintLayerFillExtrusionHeightExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "FillExtrusionPaintLayerFillExtrusionHeightExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The height with which to extrude this layer.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FillExtrusionPaintLayerFillExtrusionHeight {
    Expr(Box<FillExtrusionPaintLayerFillExtrusionHeightExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for FillExtrusionPaintLayerFillExtrusionHeight {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for FillExtrusionPaintLayerFillExtrusionHeight {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <FillExtrusionPaintLayerFillExtrusionHeightExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "FillExtrusionPaintLayerFillExtrusionHeight: no variant matched. Expected Expr(FillExtrusionPaintLayerFillExtrusionHeightExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FillExtrusionPaintLayerFillExtrusionOpacityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for FillExtrusionPaintLayerFillExtrusionOpacityExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for FillExtrusionPaintLayerFillExtrusionOpacityExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "FillExtrusionPaintLayerFillExtrusionOpacityExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The opacity of the entire fill extrusion layer. This is rendered on a per-layer, not per-feature, basis, and data-driven styling is not available.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FillExtrusionPaintLayerFillExtrusionOpacity {
    Expr(Box<FillExtrusionPaintLayerFillExtrusionOpacityExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for FillExtrusionPaintLayerFillExtrusionOpacity {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for FillExtrusionPaintLayerFillExtrusionOpacity {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <FillExtrusionPaintLayerFillExtrusionOpacityExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "FillExtrusionPaintLayerFillExtrusionOpacity: no variant matched. Expected Expr(FillExtrusionPaintLayerFillExtrusionOpacityExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
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

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum HeatmapPaintLayerHeatmapColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

impl serde::Serialize for HeatmapPaintLayerHeatmapColorExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Color(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for HeatmapPaintLayerHeatmapColorExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Color as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Color(v)),
            Err(e) => errors.push(("Color", e.to_string())),
        }
        match <ColorOrArrayOfColor as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "HeatmapPaintLayerHeatmapColorExpression: no variant matched. Expected Color(Color) | Ramp(ColorOrArrayOfColor). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Defines the color of each pixel based on its density value in a heatmap.  Should be an expression that uses `["heatmap-density"]` as input.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum HeatmapPaintLayerHeatmapColor {
    Expr(Box<HeatmapPaintLayerHeatmapColorExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl serde::Serialize for HeatmapPaintLayerHeatmapColor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for HeatmapPaintLayerHeatmapColor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <HeatmapPaintLayerHeatmapColorExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Value as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "HeatmapPaintLayerHeatmapColor: no variant matched. Expected Expr(HeatmapPaintLayerHeatmapColorExpression) | Literal(serde_json::Value). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum HeatmapPaintLayerHeatmapIntensityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for HeatmapPaintLayerHeatmapIntensityExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for HeatmapPaintLayerHeatmapIntensityExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "HeatmapPaintLayerHeatmapIntensityExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Similar to `heatmap-weight` but controls the intensity of the heatmap globally. Primarily used for adjusting the heatmap based on zoom level.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum HeatmapPaintLayerHeatmapIntensity {
    Expr(Box<HeatmapPaintLayerHeatmapIntensityExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for HeatmapPaintLayerHeatmapIntensity {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for HeatmapPaintLayerHeatmapIntensity {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <HeatmapPaintLayerHeatmapIntensityExpression as serde::Deserialize>::deserialize(
            &value,
        ) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "HeatmapPaintLayerHeatmapIntensity: no variant matched. Expected Expr(HeatmapPaintLayerHeatmapIntensityExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum HeatmapPaintLayerHeatmapOpacityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for HeatmapPaintLayerHeatmapOpacityExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for HeatmapPaintLayerHeatmapOpacityExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "HeatmapPaintLayerHeatmapOpacityExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The global opacity at which the heatmap layer will be drawn.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum HeatmapPaintLayerHeatmapOpacity {
    Expr(Box<HeatmapPaintLayerHeatmapOpacityExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for HeatmapPaintLayerHeatmapOpacity {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for HeatmapPaintLayerHeatmapOpacity {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <HeatmapPaintLayerHeatmapOpacityExpression as serde::Deserialize>::deserialize(&value)
        {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "HeatmapPaintLayerHeatmapOpacity: no variant matched. Expected Expr(HeatmapPaintLayerHeatmapOpacityExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum HeatmapPaintLayerHeatmapRadiusExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for HeatmapPaintLayerHeatmapRadiusExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for HeatmapPaintLayerHeatmapRadiusExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "HeatmapPaintLayerHeatmapRadiusExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Radius of influence of one heatmap point in pixels. Increasing the value makes the heatmap smoother, but less detailed.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum HeatmapPaintLayerHeatmapRadius {
    Expr(Box<HeatmapPaintLayerHeatmapRadiusExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for HeatmapPaintLayerHeatmapRadius {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for HeatmapPaintLayerHeatmapRadius {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <HeatmapPaintLayerHeatmapRadiusExpression as serde::Deserialize>::deserialize(&value)
        {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "HeatmapPaintLayerHeatmapRadius: no variant matched. Expected Expr(HeatmapPaintLayerHeatmapRadiusExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum HeatmapPaintLayerHeatmapWeightExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for HeatmapPaintLayerHeatmapWeightExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for HeatmapPaintLayerHeatmapWeightExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "HeatmapPaintLayerHeatmapWeightExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// A measure of how much an individual point contributes to the heatmap. A value of 10 would be equivalent to having 10 points of weight 1 in the same spot. Especially useful when combined with clustering.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum HeatmapPaintLayerHeatmapWeight {
    Expr(Box<HeatmapPaintLayerHeatmapWeightExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for HeatmapPaintLayerHeatmapWeight {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for HeatmapPaintLayerHeatmapWeight {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <HeatmapPaintLayerHeatmapWeightExpression as serde::Deserialize>::deserialize(&value)
        {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "HeatmapPaintLayerHeatmapWeight: no variant matched. Expected Expr(HeatmapPaintLayerHeatmapWeightExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    /// The hillshade algorithm to use, one of `standard`, `basic`, `combined`, `igor`, or `multidirectional`. ![image](assets/hillshade_methods.png)
    #[serde(rename = "hillshade-method")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hillshade_method: Option<HillshadePaintLayerHillshadeMethod>,
    /// The shading color of areas that face away from the light source(s). Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
    #[serde(rename = "hillshade-shadow-color")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hillshade_shadow_color: Option<HillshadePaintLayerHillshadeShadowColor>,
}

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum HillshadePaintLayerHillshadeAccentColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

impl serde::Serialize for HillshadePaintLayerHillshadeAccentColorExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Color(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for HillshadePaintLayerHillshadeAccentColorExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Color as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Color(v)),
            Err(e) => errors.push(("Color", e.to_string())),
        }
        match <ColorOrArrayOfColor as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "HillshadePaintLayerHillshadeAccentColorExpression: no variant matched. Expected Color(Color) | Ramp(ColorOrArrayOfColor). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The shading color used to accentuate rugged terrain like sharp cliffs and gorges.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum HillshadePaintLayerHillshadeAccentColor {
    Expr(Box<HillshadePaintLayerHillshadeAccentColorExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl serde::Serialize for HillshadePaintLayerHillshadeAccentColor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for HillshadePaintLayerHillshadeAccentColor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <HillshadePaintLayerHillshadeAccentColorExpression as serde::Deserialize>::deserialize(
            &value,
        ) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Value as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "HillshadePaintLayerHillshadeAccentColor: no variant matched. Expected Expr(HillshadePaintLayerHillshadeAccentColorExpression) | Literal(serde_json::Value). Errors: [{}]",
            details.join("; ")
        )))
    }
}

impl Default for HillshadePaintLayerHillshadeAccentColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("#000000"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum HillshadePaintLayerHillshadeExaggerationExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for HillshadePaintLayerHillshadeExaggerationExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for HillshadePaintLayerHillshadeExaggerationExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "HillshadePaintLayerHillshadeExaggerationExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Intensity of the hillshade
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum HillshadePaintLayerHillshadeExaggeration {
    Expr(Box<HillshadePaintLayerHillshadeExaggerationExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for HillshadePaintLayerHillshadeExaggeration {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for HillshadePaintLayerHillshadeExaggeration {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <HillshadePaintLayerHillshadeExaggerationExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "HillshadePaintLayerHillshadeExaggeration: no variant matched. Expected Expr(HillshadePaintLayerHillshadeExaggerationExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
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

impl serde::Serialize for HillshadePaintLayerHillshadeHighlightColor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::One(v) => v.serialize(serializer),
            Self::Multiple(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for HillshadePaintLayerHillshadeHighlightColor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <color::DynamicColor as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::One(v)),
            Err(e) => errors.push(("One", e.to_string())),
        }
        match <Vec<color::DynamicColor> as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Multiple(v)),
            Err(e) => errors.push(("Multiple", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "HillshadePaintLayerHillshadeHighlightColor: no variant matched. Expected One(color::DynamicColor) | Multiple(Vec<color::DynamicColor>). Errors: [{}]",
            details.join("; ")
        )))
    }
}

impl Default for HillshadePaintLayerHillshadeHighlightColor {
    fn default() -> Self {
        Self::One(
            color::parse_color("#FFFFFF").expect("Invalid color specified as the default value"),
        )
    }
}

/// The altitude of the light source(s) used to generate the hillshading with 0 as sunset and 90 as noon. Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
#[derive(PartialEq, Debug, Clone)]
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

impl serde::Serialize for HillshadePaintLayerHillshadeIlluminationAltitude {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::One(v) => v.serialize(serializer),
            Self::Many(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for HillshadePaintLayerHillshadeIlluminationAltitude {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::One(v)),
            Err(e) => errors.push(("One", e.to_string())),
        }
        match <Vec<serde_json::Number> as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Many(v)),
            Err(e) => errors.push(("Many", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "HillshadePaintLayerHillshadeIlluminationAltitude: no variant matched. Expected One(serde_json::Number) | Many(Vec<serde_json::Number>). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
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

impl serde::Serialize for HillshadePaintLayerHillshadeIlluminationDirection {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::One(v) => v.serialize(serializer),
            Self::Many(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for HillshadePaintLayerHillshadeIlluminationDirection {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::One(v)),
            Err(e) => errors.push(("One", e.to_string())),
        }
        match <Vec<serde_json::Number> as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Many(v)),
            Err(e) => errors.push(("Many", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "HillshadePaintLayerHillshadeIlluminationDirection: no variant matched. Expected One(serde_json::Number) | Many(Vec<serde_json::Number>). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
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

impl serde::Serialize for HillshadePaintLayerHillshadeShadowColor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::One(v) => v.serialize(serializer),
            Self::Multiple(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for HillshadePaintLayerHillshadeShadowColor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <color::DynamicColor as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::One(v)),
            Err(e) => errors.push(("One", e.to_string())),
        }
        match <Vec<color::DynamicColor> as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Multiple(v)),
            Err(e) => errors.push(("Multiple", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "HillshadePaintLayerHillshadeShadowColor: no variant matched. Expected One(color::DynamicColor) | Multiple(Vec<color::DynamicColor>). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LineLayoutLayerLineMiterLimitExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for LineLayoutLayerLineMiterLimitExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for LineLayoutLayerLineMiterLimitExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "LineLayoutLayerLineMiterLimitExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Used to automatically convert miter joins to bevel joins for sharp angles.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LineLayoutLayerLineMiterLimit {
    Expr(Box<LineLayoutLayerLineMiterLimitExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for LineLayoutLayerLineMiterLimit {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for LineLayoutLayerLineMiterLimit {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <LineLayoutLayerLineMiterLimitExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "LineLayoutLayerLineMiterLimit: no variant matched. Expected Expr(LineLayoutLayerLineMiterLimitExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LineLayoutLayerLineRoundLimitExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for LineLayoutLayerLineRoundLimitExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for LineLayoutLayerLineRoundLimitExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "LineLayoutLayerLineRoundLimitExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Used to automatically convert round joins to miter joins for shallow angles.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LineLayoutLayerLineRoundLimit {
    Expr(Box<LineLayoutLayerLineRoundLimitExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for LineLayoutLayerLineRoundLimit {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for LineLayoutLayerLineRoundLimit {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <LineLayoutLayerLineRoundLimitExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "LineLayoutLayerLineRoundLimit: no variant matched. Expected Expr(LineLayoutLayerLineRoundLimitExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LineLayoutLayerLineSortKeyExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for LineLayoutLayerLineSortKeyExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for LineLayoutLayerLineSortKeyExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "LineLayoutLayerLineSortKeyExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LineLayoutLayerLineSortKey {
    Expr(Box<LineLayoutLayerLineSortKeyExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for LineLayoutLayerLineSortKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for LineLayoutLayerLineSortKey {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <LineLayoutLayerLineSortKeyExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "LineLayoutLayerLineSortKey: no variant matched. Expected Expr(LineLayoutLayerLineSortKeyExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LinePaintLayerLineBlurExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for LinePaintLayerLineBlurExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for LinePaintLayerLineBlurExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "LinePaintLayerLineBlurExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Blur applied to the line, in pixels.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LinePaintLayerLineBlur {
    Expr(Box<LinePaintLayerLineBlurExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for LinePaintLayerLineBlur {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for LinePaintLayerLineBlur {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <LinePaintLayerLineBlurExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "LinePaintLayerLineBlur: no variant matched. Expected Expr(LinePaintLayerLineBlurExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LinePaintLayerLineColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

impl serde::Serialize for LinePaintLayerLineColorExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Color(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for LinePaintLayerLineColorExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Color as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Color(v)),
            Err(e) => errors.push(("Color", e.to_string())),
        }
        match <ColorOrArrayOfColor as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "LinePaintLayerLineColorExpression: no variant matched. Expected Color(Color) | Ramp(ColorOrArrayOfColor). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The color with which the line will be drawn.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LinePaintLayerLineColor {
    Expr(Box<LinePaintLayerLineColorExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl serde::Serialize for LinePaintLayerLineColor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for LinePaintLayerLineColor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <LinePaintLayerLineColorExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Value as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "LinePaintLayerLineColor: no variant matched. Expected Expr(LinePaintLayerLineColorExpression) | Literal(serde_json::Value). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LinePaintLayerLineGapWidthExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for LinePaintLayerLineGapWidthExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for LinePaintLayerLineGapWidthExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "LinePaintLayerLineGapWidthExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Draws a line casing outside of a line's actual path. Value indicates the width of the inner gap.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LinePaintLayerLineGapWidth {
    Expr(Box<LinePaintLayerLineGapWidthExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for LinePaintLayerLineGapWidth {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for LinePaintLayerLineGapWidth {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <LinePaintLayerLineGapWidthExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "LinePaintLayerLineGapWidth: no variant matched. Expected Expr(LinePaintLayerLineGapWidthExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LinePaintLayerLineGradientExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

impl serde::Serialize for LinePaintLayerLineGradientExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Color(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for LinePaintLayerLineGradientExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Color as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Color(v)),
            Err(e) => errors.push(("Color", e.to_string())),
        }
        match <ColorOrArrayOfColor as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "LinePaintLayerLineGradientExpression: no variant matched. Expected Color(Color) | Ramp(ColorOrArrayOfColor). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Defines a gradient with which to color a line feature. Can only be used with GeoJSON sources that specify `"lineMetrics": true`.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LinePaintLayerLineGradient {
    Expr(Box<LinePaintLayerLineGradientExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl serde::Serialize for LinePaintLayerLineGradient {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for LinePaintLayerLineGradient {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <LinePaintLayerLineGradientExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Value as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "LinePaintLayerLineGradient: no variant matched. Expected Expr(LinePaintLayerLineGradientExpression) | Literal(serde_json::Value). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LinePaintLayerLineOffsetExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for LinePaintLayerLineOffsetExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for LinePaintLayerLineOffsetExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "LinePaintLayerLineOffsetExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The line's offset. For linear features, a positive value offsets the line to the right, relative to the direction of the line, and a negative value to the left. For polygon features, a positive value results in an inset, and a negative value results in an outset.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LinePaintLayerLineOffset {
    Expr(Box<LinePaintLayerLineOffsetExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for LinePaintLayerLineOffset {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for LinePaintLayerLineOffset {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <LinePaintLayerLineOffsetExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "LinePaintLayerLineOffset: no variant matched. Expected Expr(LinePaintLayerLineOffsetExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LinePaintLayerLineOpacityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for LinePaintLayerLineOpacityExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for LinePaintLayerLineOpacityExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "LinePaintLayerLineOpacityExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The opacity at which the line will be drawn.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LinePaintLayerLineOpacity {
    Expr(Box<LinePaintLayerLineOpacityExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for LinePaintLayerLineOpacity {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for LinePaintLayerLineOpacity {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <LinePaintLayerLineOpacityExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "LinePaintLayerLineOpacity: no variant matched. Expected Expr(LinePaintLayerLineOpacityExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LinePaintLayerLineWidthExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for LinePaintLayerLineWidthExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for LinePaintLayerLineWidthExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "LinePaintLayerLineWidthExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Stroke thickness.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LinePaintLayerLineWidth {
    Expr(Box<LinePaintLayerLineWidthExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for LinePaintLayerLineWidth {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for LinePaintLayerLineWidth {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <LinePaintLayerLineWidthExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "LinePaintLayerLineWidth: no variant matched. Expected Expr(LinePaintLayerLineWidthExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    /// The resampling/interpolation method to use for overscaling, also known as texture magnification filter
    #[serde(rename = "raster-resampling")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raster_resampling: Option<RasterPaintLayerRasterResampling>,
    /// Increase or reduce the saturation of the image.
    #[serde(rename = "raster-saturation")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raster_saturation: Option<RasterPaintLayerRasterSaturation>,
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum RasterPaintLayerRasterBrightnessMaxExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for RasterPaintLayerRasterBrightnessMaxExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for RasterPaintLayerRasterBrightnessMaxExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "RasterPaintLayerRasterBrightnessMaxExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Increase or reduce the brightness of the image. The value is the maximum brightness.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum RasterPaintLayerRasterBrightnessMax {
    Expr(Box<RasterPaintLayerRasterBrightnessMaxExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for RasterPaintLayerRasterBrightnessMax {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for RasterPaintLayerRasterBrightnessMax {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <RasterPaintLayerRasterBrightnessMaxExpression as serde::Deserialize>::deserialize(
            &value,
        ) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "RasterPaintLayerRasterBrightnessMax: no variant matched. Expected Expr(RasterPaintLayerRasterBrightnessMaxExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum RasterPaintLayerRasterBrightnessMinExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for RasterPaintLayerRasterBrightnessMinExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for RasterPaintLayerRasterBrightnessMinExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "RasterPaintLayerRasterBrightnessMinExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Increase or reduce the brightness of the image. The value is the minimum brightness.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum RasterPaintLayerRasterBrightnessMin {
    Expr(Box<RasterPaintLayerRasterBrightnessMinExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for RasterPaintLayerRasterBrightnessMin {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for RasterPaintLayerRasterBrightnessMin {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <RasterPaintLayerRasterBrightnessMinExpression as serde::Deserialize>::deserialize(
            &value,
        ) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "RasterPaintLayerRasterBrightnessMin: no variant matched. Expected Expr(RasterPaintLayerRasterBrightnessMinExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum RasterPaintLayerRasterContrastExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for RasterPaintLayerRasterContrastExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for RasterPaintLayerRasterContrastExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "RasterPaintLayerRasterContrastExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Increase or reduce the contrast of the image.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum RasterPaintLayerRasterContrast {
    Expr(Box<RasterPaintLayerRasterContrastExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for RasterPaintLayerRasterContrast {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for RasterPaintLayerRasterContrast {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <RasterPaintLayerRasterContrastExpression as serde::Deserialize>::deserialize(&value)
        {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "RasterPaintLayerRasterContrast: no variant matched. Expected Expr(RasterPaintLayerRasterContrastExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum RasterPaintLayerRasterFadeDurationExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for RasterPaintLayerRasterFadeDurationExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for RasterPaintLayerRasterFadeDurationExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "RasterPaintLayerRasterFadeDurationExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Fade duration when a new tile is added, or when a video is started or its coordinates are updated.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum RasterPaintLayerRasterFadeDuration {
    Expr(Box<RasterPaintLayerRasterFadeDurationExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for RasterPaintLayerRasterFadeDuration {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for RasterPaintLayerRasterFadeDuration {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <RasterPaintLayerRasterFadeDurationExpression as serde::Deserialize>::deserialize(
            &value,
        ) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "RasterPaintLayerRasterFadeDuration: no variant matched. Expected Expr(RasterPaintLayerRasterFadeDurationExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum RasterPaintLayerRasterHueRotateExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for RasterPaintLayerRasterHueRotateExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for RasterPaintLayerRasterHueRotateExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "RasterPaintLayerRasterHueRotateExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Rotates hues around the color wheel.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum RasterPaintLayerRasterHueRotate {
    Expr(Box<RasterPaintLayerRasterHueRotateExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for RasterPaintLayerRasterHueRotate {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for RasterPaintLayerRasterHueRotate {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <RasterPaintLayerRasterHueRotateExpression as serde::Deserialize>::deserialize(&value)
        {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "RasterPaintLayerRasterHueRotate: no variant matched. Expected Expr(RasterPaintLayerRasterHueRotateExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum RasterPaintLayerRasterOpacityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for RasterPaintLayerRasterOpacityExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for RasterPaintLayerRasterOpacityExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "RasterPaintLayerRasterOpacityExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The opacity at which the image will be drawn.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum RasterPaintLayerRasterOpacity {
    Expr(Box<RasterPaintLayerRasterOpacityExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for RasterPaintLayerRasterOpacity {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for RasterPaintLayerRasterOpacity {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <RasterPaintLayerRasterOpacityExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "RasterPaintLayerRasterOpacity: no variant matched. Expected Expr(RasterPaintLayerRasterOpacityExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum RasterPaintLayerRasterSaturationExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for RasterPaintLayerRasterSaturationExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for RasterPaintLayerRasterSaturationExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "RasterPaintLayerRasterSaturationExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Increase or reduce the saturation of the image.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum RasterPaintLayerRasterSaturation {
    Expr(Box<RasterPaintLayerRasterSaturationExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for RasterPaintLayerRasterSaturation {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for RasterPaintLayerRasterSaturation {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <RasterPaintLayerRasterSaturationExpression as serde::Deserialize>::deserialize(
            &value,
        ) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "RasterPaintLayerRasterSaturation: no variant matched. Expected Expr(RasterPaintLayerRasterSaturationExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
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

impl serde::Serialize for SymbolLayoutLayerIconPadding {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::One(v) => v.as_ref().serialize(serializer),
            Self::Two(v) => v.as_ref().serialize(serializer),
            Self::Three(v) => v.as_ref().serialize(serializer),
            Self::Four(v) => v.as_ref().serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerIconPadding {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <[serde_json::Number; 1] as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::One(Box::new(v))),
            Err(e) => errors.push(("One", e.to_string())),
        }
        match <[serde_json::Number; 2] as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Two(Box::new(v))),
            Err(e) => errors.push(("Two", e.to_string())),
        }
        match <[serde_json::Number; 3] as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Three(Box::new(v))),
            Err(e) => errors.push(("Three", e.to_string())),
        }
        match <[serde_json::Number; 4] as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Four(Box::new(v))),
            Err(e) => errors.push(("Four", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerIconPadding: no variant matched. Expected One([serde_json::Number; 1]) | Two([serde_json::Number; 2]) | Three([serde_json::Number; 3]) | Four([serde_json::Number; 4]). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerIconRotateExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for SymbolLayoutLayerIconRotateExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerIconRotateExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerIconRotateExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Rotates the icon clockwise.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerIconRotate {
    Expr(Box<SymbolLayoutLayerIconRotateExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for SymbolLayoutLayerIconRotate {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerIconRotate {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <SymbolLayoutLayerIconRotateExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerIconRotate: no variant matched. Expected Expr(SymbolLayoutLayerIconRotateExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerIconSizeExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for SymbolLayoutLayerIconSizeExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerIconSizeExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerIconSizeExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Scales the original size of the icon by the provided factor. The new pixel size of the image will be the original pixel size multiplied by `icon-size`. 1 is the original size; 3 triples the size of the image.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerIconSize {
    Expr(Box<SymbolLayoutLayerIconSizeExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for SymbolLayoutLayerIconSize {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerIconSize {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <SymbolLayoutLayerIconSizeExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerIconSize: no variant matched. Expected Expr(SymbolLayoutLayerIconSizeExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerSymbolSortKeyExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for SymbolLayoutLayerSymbolSortKeyExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerSymbolSortKeyExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerSymbolSortKeyExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Sorts features in ascending order based on this value. Features with lower sort keys are drawn and placed first.  When `icon-allow-overlap` or `text-allow-overlap` is `false`, features with a lower sort key will have priority during placement. When `icon-allow-overlap` or `text-allow-overlap` is set to `true`, features with a higher sort key will overlap over features with a lower sort key.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerSymbolSortKey {
    Expr(Box<SymbolLayoutLayerSymbolSortKeyExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for SymbolLayoutLayerSymbolSortKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerSymbolSortKey {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <SymbolLayoutLayerSymbolSortKeyExpression as serde::Deserialize>::deserialize(&value)
        {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerSymbolSortKey: no variant matched. Expected Expr(SymbolLayoutLayerSymbolSortKeyExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerSymbolSpacingExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for SymbolLayoutLayerSymbolSpacingExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerSymbolSpacingExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerSymbolSpacingExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Distance between two symbol anchors.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerSymbolSpacing {
    Expr(Box<SymbolLayoutLayerSymbolSpacingExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for SymbolLayoutLayerSymbolSpacing {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerSymbolSpacing {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <SymbolLayoutLayerSymbolSpacingExpression as serde::Deserialize>::deserialize(&value)
        {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerSymbolSpacing: no variant matched. Expected Expr(SymbolLayoutLayerSymbolSpacingExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextLetterSpacingExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for SymbolLayoutLayerTextLetterSpacingExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerTextLetterSpacingExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerTextLetterSpacingExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Text tracking amount.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextLetterSpacing {
    Expr(Box<SymbolLayoutLayerTextLetterSpacingExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for SymbolLayoutLayerTextLetterSpacing {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerTextLetterSpacing {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <SymbolLayoutLayerTextLetterSpacingExpression as serde::Deserialize>::deserialize(
            &value,
        ) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerTextLetterSpacing: no variant matched. Expected Expr(SymbolLayoutLayerTextLetterSpacingExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextLineHeightExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for SymbolLayoutLayerTextLineHeightExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerTextLineHeightExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerTextLineHeightExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Text leading value for multi-line text.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextLineHeight {
    Expr(Box<SymbolLayoutLayerTextLineHeightExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for SymbolLayoutLayerTextLineHeight {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerTextLineHeight {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <SymbolLayoutLayerTextLineHeightExpression as serde::Deserialize>::deserialize(&value)
        {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerTextLineHeight: no variant matched. Expected Expr(SymbolLayoutLayerTextLineHeightExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextMaxAngleExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for SymbolLayoutLayerTextMaxAngleExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerTextMaxAngleExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerTextMaxAngleExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Maximum angle change between adjacent characters.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextMaxAngle {
    Expr(Box<SymbolLayoutLayerTextMaxAngleExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for SymbolLayoutLayerTextMaxAngle {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerTextMaxAngle {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <SymbolLayoutLayerTextMaxAngleExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerTextMaxAngle: no variant matched. Expected Expr(SymbolLayoutLayerTextMaxAngleExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextMaxWidthExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for SymbolLayoutLayerTextMaxWidthExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerTextMaxWidthExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerTextMaxWidthExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The maximum line width for text wrapping.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextMaxWidth {
    Expr(Box<SymbolLayoutLayerTextMaxWidthExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for SymbolLayoutLayerTextMaxWidth {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerTextMaxWidth {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <SymbolLayoutLayerTextMaxWidthExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerTextMaxWidth: no variant matched. Expected Expr(SymbolLayoutLayerTextMaxWidthExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextPaddingExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for SymbolLayoutLayerTextPaddingExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerTextPaddingExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerTextPaddingExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Size of the additional area around the text bounding box used for detecting symbol collisions.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextPadding {
    Expr(Box<SymbolLayoutLayerTextPaddingExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for SymbolLayoutLayerTextPadding {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerTextPadding {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <SymbolLayoutLayerTextPaddingExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerTextPadding: no variant matched. Expected Expr(SymbolLayoutLayerTextPaddingExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextRadialOffsetExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for SymbolLayoutLayerTextRadialOffsetExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerTextRadialOffsetExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerTextRadialOffsetExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Radial offset of text, in the direction of the symbol's anchor. Useful in combination with `text-variable-anchor`, which defaults to using the two-dimensional `text-offset` if present.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextRadialOffset {
    Expr(Box<SymbolLayoutLayerTextRadialOffsetExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for SymbolLayoutLayerTextRadialOffset {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerTextRadialOffset {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <SymbolLayoutLayerTextRadialOffsetExpression as serde::Deserialize>::deserialize(
            &value,
        ) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerTextRadialOffset: no variant matched. Expected Expr(SymbolLayoutLayerTextRadialOffsetExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextRotateExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for SymbolLayoutLayerTextRotateExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerTextRotateExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerTextRotateExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Rotates the text clockwise.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextRotate {
    Expr(Box<SymbolLayoutLayerTextRotateExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for SymbolLayoutLayerTextRotate {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerTextRotate {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <SymbolLayoutLayerTextRotateExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerTextRotate: no variant matched. Expected Expr(SymbolLayoutLayerTextRotateExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextSizeExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for SymbolLayoutLayerTextSizeExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerTextSizeExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerTextSizeExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Font size.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolLayoutLayerTextSize {
    Expr(Box<SymbolLayoutLayerTextSizeExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for SymbolLayoutLayerTextSize {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolLayoutLayerTextSize {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <SymbolLayoutLayerTextSizeExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolLayoutLayerTextSize: no variant matched. Expected Expr(SymbolLayoutLayerTextSizeExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolPaintLayerIconColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

impl serde::Serialize for SymbolPaintLayerIconColorExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Color(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolPaintLayerIconColorExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Color as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Color(v)),
            Err(e) => errors.push(("Color", e.to_string())),
        }
        match <ColorOrArrayOfColor as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolPaintLayerIconColorExpression: no variant matched. Expected Color(Color) | Ramp(ColorOrArrayOfColor). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The color of the icon. This can only be used with SDF icons.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolPaintLayerIconColor {
    Expr(Box<SymbolPaintLayerIconColorExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl serde::Serialize for SymbolPaintLayerIconColor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolPaintLayerIconColor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <SymbolPaintLayerIconColorExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Value as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolPaintLayerIconColor: no variant matched. Expected Expr(SymbolPaintLayerIconColorExpression) | Literal(serde_json::Value). Errors: [{}]",
            details.join("; ")
        )))
    }
}

impl Default for SymbolPaintLayerIconColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("#000000"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolPaintLayerIconHaloBlurExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for SymbolPaintLayerIconHaloBlurExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolPaintLayerIconHaloBlurExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolPaintLayerIconHaloBlurExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Fade out the halo towards the outside.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolPaintLayerIconHaloBlur {
    Expr(Box<SymbolPaintLayerIconHaloBlurExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for SymbolPaintLayerIconHaloBlur {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolPaintLayerIconHaloBlur {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <SymbolPaintLayerIconHaloBlurExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolPaintLayerIconHaloBlur: no variant matched. Expected Expr(SymbolPaintLayerIconHaloBlurExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolPaintLayerIconHaloColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

impl serde::Serialize for SymbolPaintLayerIconHaloColorExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Color(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolPaintLayerIconHaloColorExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Color as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Color(v)),
            Err(e) => errors.push(("Color", e.to_string())),
        }
        match <ColorOrArrayOfColor as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolPaintLayerIconHaloColorExpression: no variant matched. Expected Color(Color) | Ramp(ColorOrArrayOfColor). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The color of the icon's halo. Icon halos can only be used with SDF icons.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolPaintLayerIconHaloColor {
    Expr(Box<SymbolPaintLayerIconHaloColorExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl serde::Serialize for SymbolPaintLayerIconHaloColor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolPaintLayerIconHaloColor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <SymbolPaintLayerIconHaloColorExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Value as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolPaintLayerIconHaloColor: no variant matched. Expected Expr(SymbolPaintLayerIconHaloColorExpression) | Literal(serde_json::Value). Errors: [{}]",
            details.join("; ")
        )))
    }
}

impl Default for SymbolPaintLayerIconHaloColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("rgba(0, 0, 0, 0)"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolPaintLayerIconHaloWidthExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for SymbolPaintLayerIconHaloWidthExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolPaintLayerIconHaloWidthExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolPaintLayerIconHaloWidthExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Distance of halo to the icon outline.
///
/// The unit is in pixels only for SDF sprites that were created with a blur radius of 8, multiplied by the display density. I.e., the radius needs to be 16 for `@2x` sprites, etc.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolPaintLayerIconHaloWidth {
    Expr(Box<SymbolPaintLayerIconHaloWidthExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for SymbolPaintLayerIconHaloWidth {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolPaintLayerIconHaloWidth {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <SymbolPaintLayerIconHaloWidthExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolPaintLayerIconHaloWidth: no variant matched. Expected Expr(SymbolPaintLayerIconHaloWidthExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolPaintLayerIconOpacityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for SymbolPaintLayerIconOpacityExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolPaintLayerIconOpacityExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolPaintLayerIconOpacityExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The opacity at which the icon will be drawn.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolPaintLayerIconOpacity {
    Expr(Box<SymbolPaintLayerIconOpacityExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for SymbolPaintLayerIconOpacity {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolPaintLayerIconOpacity {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <SymbolPaintLayerIconOpacityExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolPaintLayerIconOpacity: no variant matched. Expected Expr(SymbolPaintLayerIconOpacityExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolPaintLayerTextColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

impl serde::Serialize for SymbolPaintLayerTextColorExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Color(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolPaintLayerTextColorExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Color as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Color(v)),
            Err(e) => errors.push(("Color", e.to_string())),
        }
        match <ColorOrArrayOfColor as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolPaintLayerTextColorExpression: no variant matched. Expected Color(Color) | Ramp(ColorOrArrayOfColor). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The color with which the text will be drawn.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolPaintLayerTextColor {
    Expr(Box<SymbolPaintLayerTextColorExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl serde::Serialize for SymbolPaintLayerTextColor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolPaintLayerTextColor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <SymbolPaintLayerTextColorExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Value as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolPaintLayerTextColor: no variant matched. Expected Expr(SymbolPaintLayerTextColorExpression) | Literal(serde_json::Value). Errors: [{}]",
            details.join("; ")
        )))
    }
}

impl Default for SymbolPaintLayerTextColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("#000000"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolPaintLayerTextHaloBlurExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for SymbolPaintLayerTextHaloBlurExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolPaintLayerTextHaloBlurExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolPaintLayerTextHaloBlurExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The halo's fadeout distance towards the outside.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolPaintLayerTextHaloBlur {
    Expr(Box<SymbolPaintLayerTextHaloBlurExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for SymbolPaintLayerTextHaloBlur {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolPaintLayerTextHaloBlur {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <SymbolPaintLayerTextHaloBlurExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolPaintLayerTextHaloBlur: no variant matched. Expected Expr(SymbolPaintLayerTextHaloBlurExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolPaintLayerTextHaloColorExpression {
    Color(Color),
    Ramp(ColorOrArrayOfColor),
}

impl serde::Serialize for SymbolPaintLayerTextHaloColorExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Color(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolPaintLayerTextHaloColorExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Color as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Color(v)),
            Err(e) => errors.push(("Color", e.to_string())),
        }
        match <ColorOrArrayOfColor as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolPaintLayerTextHaloColorExpression: no variant matched. Expected Color(Color) | Ramp(ColorOrArrayOfColor). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The color of the text's halo, which helps it stand out from backgrounds.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolPaintLayerTextHaloColor {
    Expr(Box<SymbolPaintLayerTextHaloColorExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
        serde_json::Value,
    ),
}

impl serde::Serialize for SymbolPaintLayerTextHaloColor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolPaintLayerTextHaloColor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <SymbolPaintLayerTextHaloColorExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Value as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolPaintLayerTextHaloColor: no variant matched. Expected Expr(SymbolPaintLayerTextHaloColorExpression) | Literal(serde_json::Value). Errors: [{}]",
            details.join("; ")
        )))
    }
}

impl Default for SymbolPaintLayerTextHaloColor {
    fn default() -> Self {
        Self::Literal(serde_json::json!("rgba(0, 0, 0, 0)"))
    }
}

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolPaintLayerTextHaloWidthExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for SymbolPaintLayerTextHaloWidthExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolPaintLayerTextHaloWidthExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolPaintLayerTextHaloWidthExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Distance of halo to the font outline. Max text halo width is 1/4 of the font-size.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolPaintLayerTextHaloWidth {
    Expr(Box<SymbolPaintLayerTextHaloWidthExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for SymbolPaintLayerTextHaloWidth {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolPaintLayerTextHaloWidth {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <SymbolPaintLayerTextHaloWidthExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolPaintLayerTextHaloWidth: no variant matched. Expected Expr(SymbolPaintLayerTextHaloWidthExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolPaintLayerTextOpacityExpression {
    Number(Number),
    Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for SymbolPaintLayerTextOpacityExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolPaintLayerTextOpacityExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolPaintLayerTextOpacityExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// The opacity at which the text will be drawn.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum SymbolPaintLayerTextOpacity {
    Expr(Box<SymbolPaintLayerTextOpacityExpression>),
    Literal(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
        serde_json::Number,
    ),
}

impl serde::Serialize for SymbolPaintLayerTextOpacity {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SymbolPaintLayerTextOpacity {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <SymbolPaintLayerTextOpacityExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "SymbolPaintLayerTextOpacity: no variant matched. Expected Expr(SymbolPaintLayerTextOpacityExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
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
    pub filter: Option<LayerFilter>,
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

impl LayerFilter {
    pub fn as_value(&self) -> &serde_json::Value {
        &self.0
    }

    pub fn as_value_mut(&mut self) -> &mut serde_json::Value {
        &mut self.0
    }

    pub fn from_value(v: serde_json::Value) -> Self {
        Self(v)
    }

    /// Returns `Some(true)` or `Some(false)` if this filter is a literal boolean
    /// (`true`, `false`, `["literal", true]`, or `["literal", false]`).
    pub fn as_literal_bool(&self) -> Option<bool> {
        match &self.0 {
            serde_json::Value::Bool(b) => Some(*b),
            serde_json::Value::Array(a) if a.len() == 2 && a[0].as_str() == Some("literal") => {
                a[1].as_bool()
            }
            _ => None,
        }
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

#[cfg(test)]
#[allow(unused_imports)]
mod test {
    use super::*;
}

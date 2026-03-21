#![allow(clippy::large_enum_variant)]
#[allow(unused_imports)]
use super::*;

/// A filter selects specific features from a layer.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct Filter(bool);

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
    CubicBezier(NumberLiteral, NumberLiteral, NumberLiteral, NumberLiteral),
    /// Interpolates exponentially between the stops just less than and just greater than the input.
    Exponential(NumberLiteral),
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

#[cfg(test)]
#[allow(unused_imports)]
mod test {
    use super::*;

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
    Expr(Box<LightColorExpression>),
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
    Expr(Box<LightIntensityExpression>),
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
    Expr(Box<SkyAtmosphereBlendExpression>),
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
    Expr(Box<SkyFogColorExpression>),
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
    Expr(Box<SkyFogGroundBlendExpression>),
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
    Expr(Box<SkyHorizonColorExpression>),
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
    Expr(Box<SkyHorizonFogBlendExpression>),
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
    Expr(Box<SkySkyColorExpression>),
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
    Expr(Box<SkySkyHorizonBlendExpression>),
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

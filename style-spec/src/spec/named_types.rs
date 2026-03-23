#![allow(clippy::large_enum_variant)]
#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use crate::{array_prop, boolean_prop, color_prop, formatted_prop, numeric_prop, string_prop};

boolean_prop!(
    Filter,
    doc = "A filter selects specific features from a layer."
);

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct Function {
    /// The exponential base of the interpolation curve. It controls the rate at which the result increases. Higher values make the result increase more towards the high end of the range. With `1` the stops are interpolated linearly.
    ///
    /// Range: 0..
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<FunctionBase>,
    /// The color space in which colors interpolated. Interpolating colors in perceptual color spaces like LAB and HCL tend to produce color ramps that look more consistent and produce colors that can be differentiated more easily than those interpolated in RGB space.
    #[serde(rename = "colorSpace")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<FunctionDefault>,
    /// An expression.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expression: Option<FunctionExpression>,
    /// The name of a feature property to use as the function input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub property: Option<FunctionProperty>,
    /// An array of stops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stops: Option<FunctionStops>,
    /// The interpolation strategy to use in function evaluation.
    #[serde(rename = "type")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
pub struct FunctionExpression(pub Any);

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
#[derive(PartialEq, Debug, Clone)]
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

impl serde::Serialize for FunctionStopValue {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Zero(v) => v.serialize(serializer),
            Self::One(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for FunctionStopValue {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Zero(v)),
            Err(e) => errors.push(("Zero", e.to_string())),
        }
        match <color::DynamicColor as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::One(v)),
            Err(e) => errors.push(("One", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "FunctionStopValue: no variant matched. Expected Zero(serde_json::Number) | One(color::DynamicColor). Errors: [{}]",
            details.join("; ")
        )))
    }
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
#[derive(PartialEq, Debug, Clone)]
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

impl serde::Serialize for InterpolationName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        match self {
            InterpolationName::CubicBezier(f0, f1, f2, f3) => {
                let mut elems = vec![
                    serde_json::to_value(f0).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f1).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f2).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f3).map_err(serde::ser::Error::custom)?,
                ];
                while elems.len() > 4 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("cubic-bezier")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            InterpolationName::Exponential(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("exponential")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            InterpolationName::Linear => {
                let mut elems = vec![];
                while !elems.is_empty() && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("linear")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<LightAnchor>,
    /// Color tint for lighting extruded geometries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<LightColor>,
    /// Intensity of lighting (on a scale from 0 to 1). Higher numbers will present as more extreme contrast.
    ///
    /// Range: 0..=1
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intensity: Option<LightIntensity>,
    /// Position of the light source relative to lit (extruded) geometries, in [r radial coordinate, a azimuthal angle, p polar angle] where r indicates the distance from the center of the base of an object to its light, a indicates the position of the light relative to 0° (0° when `light.anchor` is set to `viewport` corresponds to the top of the viewport, or 0° when `light.anchor` is set to `map` corresponds to due north, and degrees proceed clockwise), and p indicates the height of the light (from 0°, directly above, to 180°, directly below).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<LightPosition>,
}

string_prop!(
    LightAnchor,
    doc = "Whether extruded geometries are lit relative to the map or viewport.",
    default = "viewport".to_string()
);

color_prop!(
    LightColor,
    doc = "Color tint for lighting extruded geometries.",
    default = serde_json::json!("#ffffff")
);

numeric_prop!(LightIntensity, doc = "Intensity of lighting (on a scale from 0 to 1). Higher numbers will present as more extreme contrast.

Range: 0..=1", min = 0_f64, max = 1_f64, default = serde_json::Number::from_f64(0.5).expect("the number is serialised from a number and is thus always valid"));

array_prop!(
    LightPosition,
    doc = "Position of the light source relative to lit (extruded) geometries, in [r radial coordinate, a azimuthal angle, p polar angle] where r indicates the distance from the center of the base of an object to its light, a indicates the position of the light relative to 0° (0° when `light.anchor` is set to `viewport` corresponds to the top of the viewport, or 0° when `light.anchor` is set to `map` corresponds to due north, and degrees proceed clockwise), and p indicates the height of the light (from 0°, directly above, to 180°, directly below).",
    default = serde_json::json!([1.15, 210, 30])
);

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct Projection {
    /// The projection definition type. Can be specified as a string, a transition state, or an expression.
    #[serde(rename = "type")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<ProjectionType>,
}

/// The projection definition type. Can be specified as a string, a transition state, or an expression.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum ProjectionType {
    Expr(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
    Literal(std::string::String),
}

impl serde::Serialize for ProjectionType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for ProjectionType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(v)),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <std::string::String as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "ProjectionType: no variant matched. Expected Expr(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection) | Literal(std::string::String). Errors: [{}]",
            details.join("; ")
        )))
    }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub atmosphere_blend: Option<SkyAtmosphereBlend>,
    /// The base color for the fog. Requires 3D terrain.
    #[serde(rename = "fog-color")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fog_color: Option<SkyFogColor>,
    /// How to blend the fog over the 3D terrain. Where 0 is the map center and 1 is the horizon.
    ///
    /// Range: 0..=1
    #[serde(rename = "fog-ground-blend")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fog_ground_blend: Option<SkyFogGroundBlend>,
    /// The base color at the horizon.
    #[serde(rename = "horizon-color")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub horizon_color: Option<SkyHorizonColor>,
    /// How to blend the fog color and the horizon color. Where 0 is using the horizon color only and 1 is using the fog color only.
    ///
    /// Range: 0..=1
    #[serde(rename = "horizon-fog-blend")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub horizon_fog_blend: Option<SkyHorizonFogBlend>,
    /// The base color for the sky.
    #[serde(rename = "sky-color")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sky_color: Option<SkySkyColor>,
    /// How to blend the sky color and the horizon color. Where 1 is blending the color at the middle of the sky and 0 is not blending at all and using the sky color only.
    ///
    /// Range: 0..=1
    #[serde(rename = "sky-horizon-blend")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sky_horizon_blend: Option<SkySkyHorizonBlend>,
}

numeric_prop!(SkyAtmosphereBlend, doc = "How to blend the atmosphere. Where 1 is visible atmosphere and 0 is hidden. It is best to interpolate this expression when using globe projection.

Range: 0..=1", min = 0_f64, max = 1_f64, default = serde_json::Number::from_f64(0.8).expect("the number is serialised from a number and is thus always valid"));

color_prop!(
    SkyFogColor,
    doc = "The base color for the fog. Requires 3D terrain.",
    default = serde_json::json!("#ffffff")
);

numeric_prop!(
    SkyFogGroundBlend,
    doc =
        "How to blend the fog over the 3D terrain. Where 0 is the map center and 1 is the horizon.

Range: 0..=1",
    min = 0_f64,
    max = 1_f64,
    default = serde_json::Number::from_f64(0.5)
        .expect("the number is serialised from a number and is thus always valid")
);

color_prop!(
    SkyHorizonColor,
    doc = "The base color at the horizon.",
    default = serde_json::json!("#ffffff")
);

numeric_prop!(SkyHorizonFogBlend, doc = "How to blend the fog color and the horizon color. Where 0 is using the horizon color only and 1 is using the fog color only.

Range: 0..=1", min = 0_f64, max = 1_f64, default = serde_json::Number::from_f64(0.8).expect("the number is serialised from a number and is thus always valid"));

color_prop!(
    SkySkyColor,
    doc = "The base color for the sky.",
    default = serde_json::json!("#88C6FC")
);

numeric_prop!(SkySkyHorizonBlend, doc = "How to blend the sky color and the horizon color. Where 1 is blending the color at the middle of the sky and 0 is not blending at all and using the sky color only.

Range: 0..=1", min = 0_f64, max = 1_f64, default = serde_json::Number::from_f64(0.8).expect("the number is serialised from a number and is thus always valid"));

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct Terrain {
    /// The exaggeration of the terrain - how high it will look.
    ///
    /// Range: 0..
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delay: Option<TransitionDelay>,
    /// Time allotted for transitions to complete.
    ///
    /// Range: 0..
    #[serde(default, skip_serializing_if = "Option::is_none")]
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

#![allow(clippy::large_enum_variant, clippy::type_complexity)]
#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use crate::{array_prop, boolean_prop, color_prop, formatted_prop, numeric_prop, string_prop};

/// An expression node or a literal JSON value in expression positions.
#[derive(PartialEq, Debug, Clone)]
pub enum ExprOrLiteral {
    Null,
    Bool(bool),
    NumberLiteral(NumberLiteral),
    StringLiteral(StringLiteral),
    GeoJSONObjectLiteral(GeoJSONObjectLiteral),
    JSONObjectLiteral(JSONObjectLiteral),
    JSONArrayLiteral(JSONArrayLiteral),
    AnyExpr(Box<Any>),
    ArrayExpr(Box<Array>),
    BooleanExpr(Box<Boolean>),
    CollatorExpr(Box<Collator>),
    ColorExpr(Box<Color>),
    FormattedExpr(Box<Formatted>),
    ImageExpr(Box<Image>),
    NumberExpr(Box<Number>),
    ObjectExpr(Box<Object>),
    StringExpr(Box<String>),
}

impl serde::Serialize for ExprOrLiteral {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Null => serializer.serialize_unit(),
            Self::Bool(v) => v.serialize(serializer),
            Self::NumberLiteral(v) => v.serialize(serializer),
            Self::StringLiteral(v) => v.serialize(serializer),
            Self::GeoJSONObjectLiteral(v) => v.serialize(serializer),
            Self::JSONObjectLiteral(v) => v.serialize(serializer),
            Self::AnyExpr(v) => v.as_ref().serialize(serializer),
            Self::ArrayExpr(v) => v.as_ref().serialize(serializer),
            Self::BooleanExpr(v) => v.as_ref().serialize(serializer),
            Self::CollatorExpr(v) => v.as_ref().serialize(serializer),
            Self::ColorExpr(v) => v.as_ref().serialize(serializer),
            Self::FormattedExpr(v) => v.as_ref().serialize(serializer),
            Self::ImageExpr(v) => v.as_ref().serialize(serializer),
            Self::NumberExpr(v) => v.as_ref().serialize(serializer),
            Self::ObjectExpr(v) => v.as_ref().serialize(serializer),
            Self::StringExpr(v) => v.as_ref().serialize(serializer),
            Self::JSONArrayLiteral(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for ExprOrLiteral {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        if value.is_null() {
            return Ok(Self::Null);
        }
        match <bool as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Bool(v)),
            Err(e) => errors.push(("Bool", e.to_string())),
        }
        match <NumberLiteral as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::NumberLiteral(v)),
            Err(e) => errors.push(("NumberLiteral", e.to_string())),
        }
        match <StringLiteral as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::StringLiteral(v)),
            Err(e) => errors.push(("StringLiteral", e.to_string())),
        }
        match <GeoJSONObjectLiteral as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::GeoJSONObjectLiteral(v)),
            Err(e) => errors.push(("GeoJSONObjectLiteral", e.to_string())),
        }
        if !(value.is_array()) {
            match <JSONObjectLiteral as serde::Deserialize>::deserialize(&value) {
                Ok(v) => return Ok(Self::JSONObjectLiteral(v)),
                Err(e) => errors.push(("JSONObjectLiteral", e.to_string())),
            }
        }
        match <Any as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::AnyExpr(Box::new(v))),
            Err(e) => errors.push(("AnyExpr", e.to_string())),
        }
        match <Array as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::ArrayExpr(Box::new(v))),
            Err(e) => errors.push(("ArrayExpr", e.to_string())),
        }
        match <Boolean as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::BooleanExpr(Box::new(v))),
            Err(e) => errors.push(("BooleanExpr", e.to_string())),
        }
        match <Collator as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::CollatorExpr(Box::new(v))),
            Err(e) => errors.push(("CollatorExpr", e.to_string())),
        }
        match <Color as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::ColorExpr(Box::new(v))),
            Err(e) => errors.push(("ColorExpr", e.to_string())),
        }
        match <Formatted as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::FormattedExpr(Box::new(v))),
            Err(e) => errors.push(("FormattedExpr", e.to_string())),
        }
        match <Image as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::ImageExpr(Box::new(v))),
            Err(e) => errors.push(("ImageExpr", e.to_string())),
        }
        match <Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::NumberExpr(Box::new(v))),
            Err(e) => errors.push(("NumberExpr", e.to_string())),
        }
        match <Object as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::ObjectExpr(Box::new(v))),
            Err(e) => errors.push(("ObjectExpr", e.to_string())),
        }
        match <String as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::StringExpr(Box::new(v))),
            Err(e) => errors.push(("StringExpr", e.to_string())),
        }
        match <JSONArrayLiteral as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::JSONArrayLiteral(v)),
            Err(e) => errors.push(("JSONArrayLiteral", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "ExprOrLiteral: no variant matched. Expected Null | Bool(bool) | NumberLiteral(NumberLiteral) | StringLiteral(StringLiteral) | GeoJSONObjectLiteral(GeoJSONObjectLiteral) | JSONObjectLiteral(JSONObjectLiteral) | AnyExpr(Any) | ArrayExpr(Array) | BooleanExpr(Boolean) | CollatorExpr(Collator) | ColorExpr(Color) | FormattedExpr(Formatted) | ImageExpr(Image) | NumberExpr(Number) | ObjectExpr(Object) | StringExpr(String) | JSONArrayLiteral(JSONArrayLiteral). Errors: [{}]",
            details.join("; ")
        )))
    }
}

impl ExprOrLiteral {
    /// Collapse expression-wrapping-literal into the canonical literal variant.
    ///
    /// Expression enums (`Boolean`, `Number`, `String`, `Color`) have `Literal` and
    /// `AnyExpr` variants for top-level use (e.g. filter position). When boxed inside
    /// `ExprOrLiteral`, those overlap with `Bool`, `NumberLiteral`, `StringLiteral`,
    /// and `AnyExpr`. This method normalises to the canonical form.
    #[must_use]
    pub fn normalize(self) -> Self {
        match self {
            ExprOrLiteral::BooleanExpr(b) => match *b {
                Boolean::Literal(v) => ExprOrLiteral::Bool(v),
                Boolean::AnyExpr(a) => ExprOrLiteral::AnyExpr(a),
                other => ExprOrLiteral::BooleanExpr(Box::new(other)),
            },
            ExprOrLiteral::NumberExpr(n) => match *n {
                Number::Literal(v) => ExprOrLiteral::NumberLiteral(v),
                Number::AnyExpr(a) => ExprOrLiteral::AnyExpr(a),
                other => ExprOrLiteral::NumberExpr(Box::new(other)),
            },
            ExprOrLiteral::StringExpr(s) => match *s {
                String::Literal(v) => ExprOrLiteral::StringLiteral(v),
                String::AnyExpr(a) => ExprOrLiteral::AnyExpr(a),
                other => ExprOrLiteral::StringExpr(Box::new(other)),
            },
            ExprOrLiteral::ColorExpr(c) => match *c {
                Color::Literal(v) => ExprOrLiteral::StringLiteral(v),
                Color::AnyExpr(a) => ExprOrLiteral::AnyExpr(a),
                other => ExprOrLiteral::ColorExpr(Box::new(other)),
            },
            ExprOrLiteral::ArrayExpr(a) => match *a {
                Array::Literal(v) => ExprOrLiteral::JSONArrayLiteral(v),
                other => ExprOrLiteral::ArrayExpr(Box::new(other)),
            },
            ExprOrLiteral::ObjectExpr(o) => match *o {
                Object::Literal(v) => ExprOrLiteral::JSONObjectLiteral(v),
                other => ExprOrLiteral::ObjectExpr(Box::new(other)),
            },
            // JSONObjectLiteral/JSONArrayLiteral can wrap any serde_json::Value;
            // normalise primitive contents to the matching literal variant.
            ExprOrLiteral::JSONObjectLiteral(JSONObjectLiteral(v)) => match v {
                serde_json::Value::Null => ExprOrLiteral::Null,
                serde_json::Value::Bool(b) => ExprOrLiteral::Bool(b),
                serde_json::Value::Number(n) => {
                    ExprOrLiteral::NumberLiteral(NumberLiteral::from(n))
                }
                serde_json::Value::String(s) => {
                    ExprOrLiteral::StringLiteral(StringLiteral::from(s))
                }
                other => ExprOrLiteral::JSONObjectLiteral(JSONObjectLiteral(other)),
            },
            other => other,
        }
    }
}

#[cfg(feature = "fuzz")]
impl<'a> arbitrary::Arbitrary<'a> for ExprOrLiteral {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let tag: u8 = u.arbitrary()?;
        Ok(match tag % 17 {
            0 => ExprOrLiteral::Null,
            1 => ExprOrLiteral::Bool(u.arbitrary()?),
            2 => ExprOrLiteral::NumberLiteral(u.arbitrary()?),
            3 => ExprOrLiteral::StringLiteral(u.arbitrary()?),
            4 => ExprOrLiteral::GeoJSONObjectLiteral(u.arbitrary()?),
            5 => ExprOrLiteral::JSONObjectLiteral(u.arbitrary()?),
            6 => ExprOrLiteral::JSONArrayLiteral(u.arbitrary()?),
            7 => ExprOrLiteral::AnyExpr(u.arbitrary()?),
            8 => ExprOrLiteral::ArrayExpr(u.arbitrary()?),
            9 => ExprOrLiteral::BooleanExpr(u.arbitrary()?),
            10 => ExprOrLiteral::CollatorExpr(u.arbitrary()?),
            11 => ExprOrLiteral::ColorExpr(u.arbitrary()?),
            12 => ExprOrLiteral::FormattedExpr(u.arbitrary()?),
            13 => ExprOrLiteral::ImageExpr(u.arbitrary()?),
            14 => ExprOrLiteral::NumberExpr(u.arbitrary()?),
            15 => ExprOrLiteral::ObjectExpr(u.arbitrary()?),
            _ => ExprOrLiteral::StringExpr(u.arbitrary()?),
        }
        .normalize())
    }
}

/// Either of the below variants
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum StringLiteralOrNumberLiteralOrArrayOfStringLiteralOrArrayOfNumberLiteralOrAnyAsUnion {
    StringLiteral(StringLiteral),
    NumberLiteral(NumberLiteral),
    ArrayOfStringLiteral(ArrayOfStringLiteral),
    ArrayOfNumberLiteral(ArrayOfNumberLiteral),
    Any(Box<Any>),
}

impl serde::Serialize
    for StringLiteralOrNumberLiteralOrArrayOfStringLiteralOrArrayOfNumberLiteralOrAnyAsUnion
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::StringLiteral(v) => v.serialize(serializer),
            Self::NumberLiteral(v) => v.serialize(serializer),
            Self::ArrayOfStringLiteral(v) => v.serialize(serializer),
            Self::ArrayOfNumberLiteral(v) => v.serialize(serializer),
            Self::Any(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de>
    for StringLiteralOrNumberLiteralOrArrayOfStringLiteralOrArrayOfNumberLiteralOrAnyAsUnion
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <StringLiteral as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::StringLiteral(v)),
            Err(e) => errors.push(("StringLiteral", e.to_string())),
        }
        match <NumberLiteral as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::NumberLiteral(v)),
            Err(e) => errors.push(("NumberLiteral", e.to_string())),
        }
        match <ArrayOfStringLiteral as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::ArrayOfStringLiteral(v)),
            Err(e) => errors.push(("ArrayOfStringLiteral", e.to_string())),
        }
        match <ArrayOfNumberLiteral as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::ArrayOfNumberLiteral(v)),
            Err(e) => errors.push(("ArrayOfNumberLiteral", e.to_string())),
        }
        match <Box<Any> as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Any(v)),
            Err(e) => errors.push(("Any", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "StringLiteralOrNumberLiteralOrArrayOfStringLiteralOrArrayOfNumberLiteralOrAnyAsUnion: no variant matched. Expected StringLiteral(StringLiteral) | NumberLiteral(NumberLiteral) | ArrayOfStringLiteral(ArrayOfStringLiteral) | ArrayOfNumberLiteral(ArrayOfNumberLiteral) | Any(Box<Any>). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// "Any"
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Any {
    /// Gets the value of a cluster property accumulated so far. Can only be used in the `clusterProperties` option of a clustered GeoJSON source.
Accumulated,
/// Retrieves an item from an array.
At(Box<Number>, ExprOrLiteral),
/// Selects the first output whose corresponding test condition evaluates to true, or the fallback value otherwise.
/// 
///  - [Create a hover effect](https://maplibre.org/maplibre-gl-js/docs/examples/create-a-hover-effect/)
/// 
///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
Case((Vec<(Box<Boolean>,ExprOrLiteral)>,ExprOrLiteral)),
/// Evaluates each expression in turn until the first non-null value is obtained, and returns that value.
/// 
///  - [Use a fallback image](https://maplibre.org/maplibre-gl-js/docs/examples/use-a-fallback-image/)
Coalesce(Vec<ExprOrLiteral>),
/// Retrieves a property value from the current feature's state. Returns null if the requested property is not present on the feature's state. A feature's state is not part of the GeoJSON or vector tile data, and must be set programmatically on each feature. When `source.promoteId` is not provided, features are identified by their `id` attribute, which must be an integer or a string that can be cast to an integer. When `source.promoteId` is provided, features are identified by their `promoteId` property, which may be a number, string, or any primitive data type. Note that ["feature-state"] can only be used with paint properties that support data-driven styling.
/// 
///  - [Create a hover effect](https://maplibre.org/maplibre-gl-js/docs/examples/create-a-hover-effect/)
FeatureState(Box<String>),
/// Retrieves a property value from the current feature's properties, or from another object if a second argument is provided. Returns null if the requested property is missing.
/// 
///  - [Change the case of labels](https://maplibre.org/maplibre-gl-js/docs/examples/change-case-of-labels/)
/// 
///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
/// 
///  - [Extrude polygons for 3D indoor mapping](https://maplibre.org/maplibre-gl-js/docs/examples/extrude-polygons-for-3d-indoor-mapping/)
Get(Box<String>, Option<Box<Object>>),
/// Retrieves a property value from global state that can be set with platform-specific APIs. Defaults can be provided using the [`state`](https://maplibre.org/maplibre-style-spec/root/#state) root property. Returns `null` if no value nor default value is set for the retrieved property.
GlobalState(StringLiteral),
/// Gets the feature's id, if it has one.
Id,
/// Binds expressions to named variables, which can then be referenced in the result expression using `["var", "variable_name"]`.
/// 
///  - [Visualize population density](https://maplibre.org/maplibre-gl-js/docs/examples/visualize-population-density/)
Let((Vec<(StringLiteral,ExprOrLiteral)>,ExprOrLiteral)),
/// Selects the output whose label value matches the input value, or the fallback value if no match is found. The input can be any expression (e.g. `["get", "building_type"]`). Each label must be either:
/// 
///  - a single literal value; or
/// 
///  - an array of literal values, whose values must be all strings or all numbers (e.g. `[100, 101]` or `["c", "b"]`). The input matches if any of the values in the array matches, similar to the `"in"` operator.
/// 
/// Each label must be unique. If the input type does not match the type of the labels, the result will be the fallback value.
Match((ExprOrLiteral, Vec<(StringLiteralOrNumberLiteralOrArrayOfStringLiteralOrArrayOfNumberLiteralOrAnyAsUnion,ExprOrLiteral)>, ExprOrLiteral)),
/// Produces discrete, stepped results by evaluating a piecewise-constant function defined by pairs of input and output values ("stops"). The `input` may be any numeric expression (e.g., `["get", "population"]`). Stop inputs must be numeric literals in strictly ascending order.
/// 
/// Returns the output value of the stop just less than the input, or the first output if the input is less than the first stop.
/// 
///  - [Create and style clusters](https://maplibre.org/maplibre-gl-js/docs/examples/create-and-style-clusters/)
Step((Box<Number>,ExprOrLiteral,Vec<(NumberLiteral,ExprOrLiteral)>)),
/// References variable bound using `let`.
/// 
///  - [Visualize population density](https://maplibre.org/maplibre-gl-js/docs/examples/visualize-population-density/)
Var(StringLiteral),
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
                let mut rest: Vec<serde_json::Value> = Vec::new();
                while let Some(v) = seq.next_element()? {
                    rest.push(v);
                }
                if rest.is_empty() {
                    return Err(serde::de::Error::custom(
                        "{name}::{variant_name}: too few arguments",
                    ));
                }
                if !(rest.len() - 1).is_multiple_of(2) {
                    return Err(serde::de::Error::custom(
                        "Any::Case: malformed template/suffix layout",
                    ));
                }
                let inputs_len = (rest.len() - 1) / 2;
                for i in 0..inputs_len {
                    let condition_i = serde_json::from_value(rest[i * 2].clone())
                        .map_err(serde::de::Error::custom)?;
                    let output_i = serde_json::from_value(rest[i * 2 + 1].clone())
                        .map_err(serde::de::Error::custom)?;
                    let element = (condition_i, output_i);
                    inputs.push(element);
                }
                let fallback = serde_json::from_value(rest[inputs_len * 2].clone())
                    .map_err(serde::de::Error::custom)?;
                Ok(Any::Case((inputs, fallback)))
            }
            "coalesce" => {
                let mut inputs = Vec::new();
                while let Some(element) = seq.next_element()? {
                    inputs.push(element);
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
                let mut rest: Vec<serde_json::Value> = Vec::new();
                while let Some(v) = seq.next_element()? {
                    rest.push(v);
                }
                if rest.is_empty() {
                    return Err(serde::de::Error::custom(
                        "{name}::{variant_name}: too few arguments",
                    ));
                }
                if !(rest.len() - 1).is_multiple_of(2) {
                    return Err(serde::de::Error::custom(
                        "Any::Let: malformed template/suffix layout",
                    ));
                }
                let inputs_len = (rest.len() - 1) / 2;
                for i in 0..inputs_len {
                    let var_name_i = serde_json::from_value(rest[i * 2].clone())
                        .map_err(serde::de::Error::custom)?;
                    let var_value_i = serde_json::from_value(rest[i * 2 + 1].clone())
                        .map_err(serde::de::Error::custom)?;
                    let element = (var_name_i, var_value_i);
                    inputs.push(element);
                }
                let expression = serde_json::from_value(rest[inputs_len * 2].clone())
                    .map_err(serde::de::Error::custom)?;
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
                if !rest.len().is_multiple_of(2) {
                    return Err(serde::de::Error::custom(
                        "Any::Match: expected an even number of arguments after operator (input + label/output pairs + fallback)",
                    ));
                }
                let fallback_v = rest.pop().unwrap();
                let input: ExprOrLiteral =
                    serde_json::from_value(rest.remove(0)).map_err(serde::de::Error::custom)?;
                let mut pairs = Vec::new();
                for chunk in rest.chunks_exact(2) {
                    let label_i: StringLiteralOrNumberLiteralOrArrayOfStringLiteralOrArrayOfNumberLiteralOrAnyAsUnion = serde_json::from_value(chunk[0].clone()).map_err(serde::de::Error::custom)?;
                    let output_i: ExprOrLiteral = serde_json::from_value(chunk[1].clone())
                        .map_err(serde::de::Error::custom)?;
                    pairs.push((label_i, output_i));
                }
                let fallback: ExprOrLiteral =
                    serde_json::from_value(fallback_v).map_err(serde::de::Error::custom)?;
                Ok(Any::Match((input, pairs, fallback)))
            }
            "step" => {
                let input: Box<Number> = visit_seq_field(&mut seq, "input")?;
                let output_0: ExprOrLiteral = visit_seq_field(&mut seq, "output_0")?;
                let mut stops = Vec::new();
                while let Some(stop_input_i) = seq.next_element::<NumberLiteral>()? {
                    let stop_output_i: ExprOrLiteral = seq.next_element()?.ok_or_else(|| {
                        serde::de::Error::custom("expected stop_output_i in Any::Step")
                    })?;
                    stops.push((stop_input_i, stop_output_i));
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

impl serde::Serialize for Any {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        match self {
            Any::Accumulated => {
                let mut elems = vec![];
                while !elems.is_empty() && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("accumulated")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Any::At(f0, f1) => {
                let mut elems = vec![
                    serde_json::to_value(f0).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f1).map_err(serde::ser::Error::custom)?,
                ];
                while elems.len() > 2 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("at")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Any::Case(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("case")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        if let serde_json::Value::Array(sub) = elem {
                            if sub.is_empty() {
                                // Empty Vec — nothing to flatten.
                            } else if sub[0].is_array() {
                                // An array-of-arrays is the Vec<(A,B)> — flatten it.
                                for pair in sub {
                                    if let serde_json::Value::Array(pair_elems) = pair {
                                        for pe in pair_elems {
                                            seq.serialize_element(pe)?;
                                        }
                                    } else {
                                        seq.serialize_element(pair)?;
                                    }
                                }
                            } else {
                                // Plain array value (e.g. a sub-expression like ["zoom"]).
                                seq.serialize_element(elem)?;
                            }
                        } else {
                            seq.serialize_element(elem)?;
                        }
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
            Any::Coalesce(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("coalesce")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        seq.serialize_element(elem)?;
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
            Any::FeatureState(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("feature-state")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Any::Get(f0, f1) => {
                let mut elems = vec![
                    serde_json::to_value(f0).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f1).map_err(serde::ser::Error::custom)?,
                ];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("get")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Any::GlobalState(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("global-state")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Any::Id => {
                let mut elems = vec![];
                while !elems.is_empty() && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("id")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Any::Let(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("let")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        if let serde_json::Value::Array(sub) = elem {
                            if sub.is_empty() {
                                // Empty Vec — nothing to flatten.
                            } else if sub[0].is_array() {
                                // An array-of-arrays is the Vec<(A,B)> — flatten it.
                                for pair in sub {
                                    if let serde_json::Value::Array(pair_elems) = pair {
                                        for pe in pair_elems {
                                            seq.serialize_element(pe)?;
                                        }
                                    } else {
                                        seq.serialize_element(pair)?;
                                    }
                                }
                            } else {
                                // Plain array value (e.g. a sub-expression like ["zoom"]).
                                seq.serialize_element(elem)?;
                            }
                        } else {
                            seq.serialize_element(elem)?;
                        }
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
            Any::Match(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("match")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        if let serde_json::Value::Array(sub) = elem {
                            if sub.is_empty() {
                                // Empty Vec — nothing to flatten.
                            } else if sub[0].is_array() {
                                // An array-of-arrays is the Vec<(A,B)> — flatten it.
                                for pair in sub {
                                    if let serde_json::Value::Array(pair_elems) = pair {
                                        for pe in pair_elems {
                                            seq.serialize_element(pe)?;
                                        }
                                    } else {
                                        seq.serialize_element(pair)?;
                                    }
                                }
                            } else {
                                // Plain array value (e.g. a sub-expression like ["zoom"]).
                                seq.serialize_element(elem)?;
                            }
                        } else {
                            seq.serialize_element(elem)?;
                        }
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
            Any::Step(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("step")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        if let serde_json::Value::Array(sub) = elem {
                            if sub.is_empty() {
                                // Empty Vec — nothing to flatten.
                            } else if sub[0].is_array() {
                                // An array-of-arrays is the Vec<(A,B)> — flatten it.
                                for pair in sub {
                                    if let serde_json::Value::Array(pair_elems) = pair {
                                        for pe in pair_elems {
                                            seq.serialize_element(pe)?;
                                        }
                                    } else {
                                        seq.serialize_element(pair)?;
                                    }
                                }
                            } else {
                                // Plain array value (e.g. a sub-expression like ["zoom"]).
                                seq.serialize_element(elem)?;
                            }
                        } else {
                            seq.serialize_element(elem)?;
                        }
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
            Any::Var(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("var")?;
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
    #[case::t_split(serde_json::json!(["split",["get","name"],";"]))]
    fn test_example_array_of_string_decodes(#[case] example: serde_json::Value) {
        let _ = serde_json::from_value::<ArrayOfString>(example).expect("example should decode");
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
    #[case::t_join(serde_json::json!(["join",["split",["get","name"],";"],"\n"]))]
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

/// "Array"
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Array {
    /// Asserts that the input is an array (optionally with a specific item type and length). If, when the input expression is evaluated, it is not of the asserted type or length, then this assertion will cause the whole expression to be aborted.
    Op(ExprOrLiteral),
    /// Provides a literal array or object value.
    ///
    ///  - [Display and style rich text labels](https://maplibre.org/maplibre-gl-js/docs/examples/display-and-style-rich-text-labels/)
    Literal(JSONArrayLiteral),
    /// Returns a subarray from an array or a substring from a string from a specified start index, or between a start index and an end index if set. The return value is inclusive of the start index but not of the end index. In a string, a UTF-16 surrogate pair counts as a single position.
    Slice(ExprOrLiteral, Box<Number>, Option<Box<Number>>),
    /// Returns a four-element array containing the input color's red, green, blue, and alpha components, in that order.
    ToRgba(Box<Color>),
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
                Ok(Array::Op(value))
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

impl serde::Serialize for Array {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        match self {
            Array::Op(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("array")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Array::Literal(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("literal")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Array::Slice(f0, f1, f2) => {
                let mut elems = vec![
                    serde_json::to_value(f0).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f1).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f2).map_err(serde::ser::Error::custom)?,
                ];
                while elems.len() > 2 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("slice")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Array::ToRgba(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("to-rgba")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
        }
    }
}

/// One of the valid type-assertion string names.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum StringOrNumberOrBooleanOrColorAsEnum {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "color")]
    Color,
}

/// "ArrayLessTypeLengthGreater"
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum ArrayLessTypeLengthGreater {
    /// Asserts that the input is an array (optionally with a specific item type and length). If, when the input expression is evaluated, it is not of the asserted type or length, then this assertion will cause the whole expression to be aborted.
    Array(
        StringOrNumberOrBooleanOrColorAsEnum,
        NumberLiteral,
        ExprOrLiteral,
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

impl serde::Serialize for ArrayLessTypeLengthGreater {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        match self {
            ArrayLessTypeLengthGreater::Array(f0, f1, f2) => {
                let mut elems = vec![
                    serde_json::to_value(f0).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f1).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f2).map_err(serde::ser::Error::custom)?,
                ];
                while elems.len() > 3 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("array")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
        }
    }
}

/// "ArrayOfString"
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum ArrayOfString {
    /// Returns an array of substrings formed by splitting an input string by a separator string.
    Split(String, String),
}

impl<'de> serde::Deserialize<'de> for ArrayOfString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ArrayOfStringVisitor)
    }
}

/// Visitor for deserializing the syntax enum [`ArrayOfString`]
struct ArrayOfStringVisitor;

impl<'de> serde::de::Visitor<'de> for ArrayOfStringVisitor {
    type Value = ArrayOfString;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(
            "an ArrayOfString expression (example: [\"split\",[\"get\",\"name\"],\";\"])",
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
            "split" => {
                let input = visit_seq_field(&mut seq, "input")?;
                let separator = visit_seq_field(&mut seq, "separator")?;
                Ok(ArrayOfString::Split(input, separator))
            }
            _ => Err(serde::de::Error::unknown_variant(&op, &["split"])),
        }
    }
}

impl serde::Serialize for ArrayOfString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        match self {
            ArrayOfString::Split(f0, f1) => {
                let mut elems = vec![
                    serde_json::to_value(f0).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f1).map_err(serde::ser::Error::custom)?,
                ];
                while elems.len() > 2 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("split")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
        }
    }
}

/// "ArrayOfType"
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum ArrayOfType {
    /// Asserts that the input is an array (optionally with a specific item type and length). If, when the input expression is evaluated, it is not of the asserted type or length, then this assertion will cause the whole expression to be aborted.
    Array(StringOrNumberOrBooleanOrColorAsEnum, ExprOrLiteral),
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

impl serde::Serialize for ArrayOfType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        match self {
            ArrayOfType::Array(f0, f1) => {
                let mut elems = vec![
                    serde_json::to_value(f0).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f1).map_err(serde::ser::Error::custom)?,
                ];
                while elems.len() > 2 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("array")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
        }
    }
}

/// "Boolean"
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Boolean {
    /// Logical negation. Returns `true` if the input is `false`, and `false` if the input is `true`.
    ///
    ///  - [Create and style clusters](https://maplibre.org/maplibre-gl-js/docs/examples/create-and-style-clusters/)
    Not(Box<Boolean>),
    /// Returns `true` if the input values are not equal, `false` otherwise. The comparison is strictly typed: values of different runtime types are always considered unequal. Cases where the types are known to be different at parse time are considered invalid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    NotEqual(ExprOrLiteral, ExprOrLiteral, Option<Collator>),
    /// Returns `true` if the first input is strictly less than the second, `false` otherwise. The arguments are required to be either both strings or both numbers; if during evaluation they are not, expression evaluation produces an error. Cases where this constraint is known not to hold at parse time are considered in valid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    Less(ExprOrLiteral, ExprOrLiteral, Option<Collator>),
    /// Returns `true` if the first input is less than or equal to the second, `false` otherwise. The arguments are required to be either both strings or both numbers; if during evaluation they are not, expression evaluation produces an error. Cases where this constraint is known not to hold at parse time are considered in valid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    LessEqual(ExprOrLiteral, ExprOrLiteral, Option<Collator>),
    /// Returns `true` if the input values are equal, `false` otherwise. The comparison is strictly typed: values of different runtime types are always considered unequal. Cases where the types are known to be different at parse time are considered invalid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    ///
    ///  - [Add multiple geometries from one GeoJSON source](https://maplibre.org/maplibre-gl-js/docs/examples/multiple-geometries/)
    ///
    ///  - [Create a time slider](https://maplibre.org/maplibre-gl-js/docs/examples/timeline-animation/)
    ///
    ///  - [Display buildings in 3D](https://maplibre.org/maplibre-gl-js/docs/examples/display-buildings-in-3d/)
    ///
    ///  - [Filter symbols by toggling a list](https://maplibre.org/maplibre-gl-js/docs/examples/filter-symbols-by-toggling-a-list/)
    EqualEqual(ExprOrLiteral, ExprOrLiteral, Option<Collator>),
    /// Returns `true` if the first input is strictly greater than the second, `false` otherwise. The arguments are required to be either both strings or both numbers; if during evaluation they are not, expression evaluation produces an error. Cases where this constraint is known not to hold at parse time are considered in valid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    Greater(ExprOrLiteral, ExprOrLiteral, Option<Collator>),
    /// Returns `true` if the first input is greater than or equal to the second, `false` otherwise. The arguments are required to be either both strings or both numbers; if during evaluation they are not, expression evaluation produces an error. Cases where this constraint is known not to hold at parse time are considered in valid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    GreaterEqual(ExprOrLiteral, ExprOrLiteral, Option<Collator>),
    /// Returns `true` if all the inputs are `true`, `false` otherwise. The inputs are evaluated in order, and evaluation is short-circuiting: once an input expression evaluates to `false`, the result is `false` and no further input expressions are evaluated.
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    All(Vec<Boolean>),
    /// Returns `true` if any of the inputs are `true`, `false` otherwise. The inputs are evaluated in order, and evaluation is short-circuiting: once an input expression evaluates to `true`, the result is `true` and no further input expressions are evaluated.
    Any(Vec<Boolean>),
    /// Asserts that the input value is a boolean. If multiple values are provided, each one is evaluated in order until a boolean is obtained. If none of the inputs are booleans, the expression is an error.
    ///
    ///  - [Create a hover effect](https://maplibre.org/maplibre-gl-js/docs/examples/create-a-hover-effect/)
    Op(Vec<ExprOrLiteral>),
    /// Tests for the presence of a property value in the current feature's properties, or from another object if a second argument is provided.
    ///
    ///  - [Create and style clusters](https://maplibre.org/maplibre-gl-js/docs/examples/create-and-style-clusters/)
    Has(Box<String>, Option<Box<Object>>),
    /// Determines whether an item exists in an array or a substring exists in a string.
    ///
    ///  - [Measure distances](https://maplibre.org/maplibre-gl-js/docs/examples/measure-distances/)
    In(ExprOrLiteral, ExprOrLiteral),
    /// Returns `true` if the input string is expected to render legibly. Returns `false` if the input string contains sections that cannot be rendered without potential loss of meaning (e.g. Indic scripts that require complex text shaping, or right-to-left scripts if the `mapbox-gl-rtl-text` plugin is not in use in MapLibre GL JS).
    IsSupportedScript(Box<String>),
    /// Converts the input value to a boolean. The result is `false` when the input is an empty string, 0, `false`, `null`, or `NaN`; otherwise it is `true`.
    To(ExprOrLiteral),
    /// Returns `true` if the evaluated feature is fully contained inside a boundary of the input geometry, `false` otherwise. The input value can be a valid GeoJSON of type `Polygon`, `MultiPolygon`, `Feature`, or `FeatureCollection`. Supported features for evaluation:
    ///
    /// - `Point`: Returns `false` if a point is on the boundary or falls outside the boundary.
    ///
    /// - `LineString`: Returns `false` if any part of a line falls outside the boundary, the line intersects the boundary, or a line's endpoint is on the boundary.
    Within(GeoJSONObjectLiteral),
    /// A boolean literal value (`true` or `false`).
    Literal(bool),
    /// A polymorphic expression (`case`, `match`, `get`, …) in a boolean position.
    AnyExpr(Box<Any>),
}

impl<'de> serde::Deserialize<'de> for Boolean {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(BooleanVisitor)
    }
}

/// Visitor for deserializing the syntax enum [`Boolean`]
struct BooleanVisitor;

impl<'de> serde::de::Visitor<'de> for BooleanVisitor {
    type Value = Boolean;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an Boolean expression (example: [\"!\",[\"has\",\"point_count\"]])")
    }

    fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Self::Value, E> {
        Ok(Boolean::Literal(v))
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
                let v: serde_json::Value = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::missing_field("value"))?;
                if let Some(b) = v.as_bool() {
                    return Ok(Boolean::Literal(b));
                }
                // Not a boolean — reconstruct as ["literal", v] and parse as AnyExpr.
                let mut elems = vec![serde_json::Value::String("literal".into()), v];
                while let Some(e) = seq.next_element::<serde_json::Value>()? {
                    elems.push(e);
                }
                let arr = serde_json::Value::Array(elems);
                let any_expr =
                    serde_json::from_value::<Any>(arr).map_err(serde::de::Error::custom)?;
                Ok(Boolean::AnyExpr(Box::new(any_expr)))
            }
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
                let input_1 = visit_seq_field(&mut seq, "input_1")?;
                let input_2 = visit_seq_field(&mut seq, "input_2")?;
                let collator = seq.next_element()?;
                Ok(Boolean::Less(input_1, input_2, collator))
            }
            "<=" => {
                let input_1 = visit_seq_field(&mut seq, "input_1")?;
                let input_2 = visit_seq_field(&mut seq, "input_2")?;
                let collator = seq.next_element()?;
                Ok(Boolean::LessEqual(input_1, input_2, collator))
            }
            "==" => {
                let input_1 = visit_seq_field(&mut seq, "input_1")?;
                let input_2 = visit_seq_field(&mut seq, "input_2")?;
                let collator = seq.next_element()?;
                Ok(Boolean::EqualEqual(input_1, input_2, collator))
            }
            ">" => {
                let input_1 = visit_seq_field(&mut seq, "input_1")?;
                let input_2 = visit_seq_field(&mut seq, "input_2")?;
                let collator = seq.next_element()?;
                Ok(Boolean::Greater(input_1, input_2, collator))
            }
            ">=" => {
                let input_1 = visit_seq_field(&mut seq, "input_1")?;
                let input_2 = visit_seq_field(&mut seq, "input_2")?;
                let collator = seq.next_element()?;
                Ok(Boolean::GreaterEqual(input_1, input_2, collator))
            }
            "all" => {
                let mut inputs = Vec::new();
                while let Some(element) = seq.next_element()? {
                    inputs.push(element);
                }
                Ok(Boolean::All(inputs))
            }
            "any" => {
                let mut inputs = Vec::new();
                while let Some(element) = seq.next_element()? {
                    inputs.push(element);
                }
                Ok(Boolean::Any(inputs))
            }
            "boolean" => {
                let mut inputs = Vec::new();
                while let Some(element) = seq.next_element()? {
                    inputs.push(element);
                }
                Ok(Boolean::Op(inputs))
            }
            "has" => {
                let property_name = visit_seq_field(&mut seq, "property_name")?;
                let object = seq.next_element()?;
                Ok(Boolean::Has(property_name, object))
            }
            "in" => {
                let item = visit_seq_field(&mut seq, "item")?;
                let array = visit_seq_field(&mut seq, "array")?;
                Ok(Boolean::In(item, array))
            }
            "is-supported-script" => {
                let input = visit_seq_field(&mut seq, "input")?;
                Ok(Boolean::IsSupportedScript(input))
            }
            "to-boolean" => {
                let value = visit_seq_field(&mut seq, "value")?;
                Ok(Boolean::To(value))
            }
            "within" => {
                let geojson = visit_seq_field(&mut seq, "geojson")?;
                Ok(Boolean::Within(geojson))
            }
            _ => {
                let mut elems = vec![serde_json::Value::String(op)];
                while let Some(v) = seq.next_element::<serde_json::Value>()? {
                    elems.push(v);
                }
                let arr = serde_json::Value::Array(elems);
                let any_expr =
                    serde_json::from_value::<Any>(arr).map_err(serde::de::Error::custom)?;
                Ok(Boolean::AnyExpr(Box::new(any_expr)))
            }
        }
    }
}

impl serde::Serialize for Boolean {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        match self {
            Boolean::Not(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("!")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Boolean::NotEqual(f0, f1, f2) => {
                let mut elems = vec![
                    serde_json::to_value(f0).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f1).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f2).map_err(serde::ser::Error::custom)?,
                ];
                while elems.len() > 2 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("!=")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Boolean::Less(f0, f1, f2) => {
                let mut elems = vec![
                    serde_json::to_value(f0).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f1).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f2).map_err(serde::ser::Error::custom)?,
                ];
                while elems.len() > 2 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("<")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Boolean::LessEqual(f0, f1, f2) => {
                let mut elems = vec![
                    serde_json::to_value(f0).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f1).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f2).map_err(serde::ser::Error::custom)?,
                ];
                while elems.len() > 2 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("<=")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Boolean::EqualEqual(f0, f1, f2) => {
                let mut elems = vec![
                    serde_json::to_value(f0).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f1).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f2).map_err(serde::ser::Error::custom)?,
                ];
                while elems.len() > 2 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("==")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Boolean::Greater(f0, f1, f2) => {
                let mut elems = vec![
                    serde_json::to_value(f0).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f1).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f2).map_err(serde::ser::Error::custom)?,
                ];
                while elems.len() > 2 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element(">")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Boolean::GreaterEqual(f0, f1, f2) => {
                let mut elems = vec![
                    serde_json::to_value(f0).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f1).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f2).map_err(serde::ser::Error::custom)?,
                ];
                while elems.len() > 2 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element(">=")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Boolean::All(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("all")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        seq.serialize_element(elem)?;
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
            Boolean::Any(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("any")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        seq.serialize_element(elem)?;
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
            Boolean::Op(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("boolean")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        seq.serialize_element(elem)?;
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
            Boolean::Has(f0, f1) => {
                let mut elems = vec![
                    serde_json::to_value(f0).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f1).map_err(serde::ser::Error::custom)?,
                ];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("has")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Boolean::In(f0, f1) => {
                let mut elems = vec![
                    serde_json::to_value(f0).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f1).map_err(serde::ser::Error::custom)?,
                ];
                while elems.len() > 2 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("in")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Boolean::IsSupportedScript(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("is-supported-script")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Boolean::To(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("to-boolean")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Boolean::Within(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("within")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Boolean::Literal(b) => serializer.serialize_bool(*b),
            Boolean::AnyExpr(a) => a.serialize(serializer),
        }
    }
}

/// "Collator"
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Collator {
    /// Returns a `collator` for use in locale-dependent comparison operations. Use `resolved-locale` to test the results of locale fallback behavior.
    Op(
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_map))]
        serde_json::Map<std::string::String, serde_json::Value>,
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
                Ok(Collator::Op(options))
            }
            _ => Err(serde::de::Error::unknown_variant(&op, &["collator"])),
        }
    }
}

impl serde::Serialize for Collator {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        match self {
            Collator::Op(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("collator")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
        }
    }
}

/// "Color"
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Color {
    /// Creates a color value from red, green, and blue components, which must range between 0 and 255, and an alpha component of 1. If any component is out of range, the expression is an error.
    Rgb(Box<Number>, Box<Number>, Box<Number>),
    /// Creates a color value from red, green, blue components, which must range between 0 and 255, and an alpha component which must range between zero and one. If any component is out of range, the expression is an error.
    Rgba(Box<Number>, Box<Number>, Box<Number>, Box<Number>),
    /// Converts the input value to a color. If multiple values are provided, each one is evaluated in order until the first successful conversion is obtained. If none of the inputs can be converted, the expression is an error.
    ///
    ///  - [Visualize population density](https://maplibre.org/maplibre-gl-js/docs/examples/visualize-population-density/)
    To(Vec<ExprOrLiteral>),
    /// A CSS color string literal (e.g. `"#ff0000"`, `"rgba(255,0,0,1)"`).
    Literal(StringLiteral),
    /// A polymorphic expression (`case`, `match`, `get`, …) in a color position.
    AnyExpr(Box<Any>),
}

impl<'de> serde::Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ColorVisitor)
    }
}

/// Visitor for deserializing the syntax enum [`Color`]
struct ColorVisitor;

impl<'de> serde::de::Visitor<'de> for ColorVisitor {
    type Value = Color;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an Color expression (example: [\"rgb\",255,0,0])")
    }

    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(Color::Literal(StringLiteral::from(v.to_string())))
    }

    fn visit_string<E: serde::de::Error>(self, v: std::string::String) -> Result<Self::Value, E> {
        Ok(Color::Literal(StringLiteral::from(v)))
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
                Ok(Color::To(inputs))
            }
            _ => {
                let mut elems = vec![serde_json::Value::String(op)];
                while let Some(v) = seq.next_element::<serde_json::Value>()? {
                    elems.push(v);
                }
                let arr = serde_json::Value::Array(elems);
                let any_expr =
                    serde_json::from_value::<Any>(arr).map_err(serde::de::Error::custom)?;
                Ok(Color::AnyExpr(Box::new(any_expr)))
            }
        }
    }
}

impl serde::Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        match self {
            Color::Rgb(f0, f1, f2) => {
                let mut elems = vec![
                    serde_json::to_value(f0).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f1).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f2).map_err(serde::ser::Error::custom)?,
                ];
                while elems.len() > 3 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("rgb")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Color::Rgba(f0, f1, f2, f3) => {
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
                seq.serialize_element("rgba")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Color::To(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("to-color")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        seq.serialize_element(elem)?;
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
            Color::Literal(s) => s.serialize(serializer),
            Color::AnyExpr(a) => a.serialize(serializer),
        }
    }
}

/// Either of the below variants
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum ColorOrArrayOfColorAsUnion {
    Color(Color),
    ArrayOfColor(ColorOrArrayOfColor),
}

impl serde::Serialize for ColorOrArrayOfColorAsUnion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Color(v) => v.serialize(serializer),
            Self::ArrayOfColor(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for ColorOrArrayOfColorAsUnion {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Color as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Color(v)),
            Err(e) => errors.push(("Color", e.to_string())),
        }
        match <ColorOrArrayOfColor as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::ArrayOfColor(v)),
            Err(e) => errors.push(("ArrayOfColor", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "ColorOrArrayOfColorAsUnion: no variant matched. Expected Color(Color) | ArrayOfColor(ColorOrArrayOfColor). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// "ColorOrArrayOfColor"
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum ColorOrArrayOfColor {
    /// Produces continuous, smooth results by interpolating between pairs of input and output values ("stops"). Works like `interpolate`, but the output type must be `color` or `array<color>`, and the interpolation is performed in the Hue-Chroma-Luminance color space.
    InterpolateHcl(
        (
            Interpolation,
            Number,
            Vec<(NumberLiteral, Box<ColorOrArrayOfColorAsUnion>)>,
        ),
    ),
    /// Produces continuous, smooth results by interpolating between pairs of input and output values ("stops"). Works like `interpolate`, but the output type must be `color` or `array<color>`, and the interpolation is performed in the CIELAB color space.
    InterpolateLab(
        (
            Interpolation,
            Number,
            Vec<(NumberLiteral, Box<ColorOrArrayOfColorAsUnion>)>,
        ),
    ),
    /// Produces discrete, stepped results by evaluating a piecewise-constant function defined by pairs of input and output values ("stops"). The `input` may be any numeric expression (e.g., `["get", "population"]`). Stop inputs must be numeric literals in strictly ascending order.
    ///
    /// Returns the output value of the stop just less than the input, or the first output if the input is less than the first stop.
    ///
    ///  - [Create and style clusters](https://maplibre.org/maplibre-gl-js/docs/examples/create-and-style-clusters/)
    Step(
        (
            Number,
            Box<ColorOrArrayOfColorAsUnion>,
            Vec<(NumberLiteral, Box<ColorOrArrayOfColorAsUnion>)>,
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
                let input: Number = visit_seq_field(&mut seq, "input")?;
                let mut stops = Vec::new();
                while let Some(stop_input_i) = seq.next_element::<NumberLiteral>()? {
                    let stop_output_i: Box<ColorOrArrayOfColorAsUnion> =
                        seq.next_element()?.ok_or_else(|| {
                            serde::de::Error::custom(
                                "expected stop_output_i in ColorOrArrayOfColor::InterpolateHcl",
                            )
                        })?;
                    stops.push((stop_input_i, stop_output_i));
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
                let input: Number = visit_seq_field(&mut seq, "input")?;
                let mut stops = Vec::new();
                while let Some(stop_input_i) = seq.next_element::<NumberLiteral>()? {
                    let stop_output_i: Box<ColorOrArrayOfColorAsUnion> =
                        seq.next_element()?.ok_or_else(|| {
                            serde::de::Error::custom(
                                "expected stop_output_i in ColorOrArrayOfColor::InterpolateLab",
                            )
                        })?;
                    stops.push((stop_input_i, stop_output_i));
                }
                Ok(ColorOrArrayOfColor::InterpolateLab((
                    interpolation_type,
                    input,
                    stops,
                )))
            }
            "step" => {
                let input: Number = visit_seq_field(&mut seq, "input")?;
                let output_0: Box<ColorOrArrayOfColorAsUnion> =
                    visit_seq_field(&mut seq, "output_0")?;
                let mut stops = Vec::new();
                while let Some(stop_input_i) = seq.next_element::<NumberLiteral>()? {
                    let stop_output_i: Box<ColorOrArrayOfColorAsUnion> =
                        seq.next_element()?.ok_or_else(|| {
                            serde::de::Error::custom(
                                "expected stop_output_i in ColorOrArrayOfColor::Step",
                            )
                        })?;
                    stops.push((stop_input_i, stop_output_i));
                }
                Ok(ColorOrArrayOfColor::Step((input, output_0, stops)))
            }
            _ => Err(serde::de::Error::unknown_variant(
                &op,
                &["interpolate-hcl", "interpolate-lab", "step"],
            )),
        }
    }
}

impl serde::Serialize for ColorOrArrayOfColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        match self {
            ColorOrArrayOfColor::InterpolateHcl(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("interpolate-hcl")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        if let serde_json::Value::Array(sub) = elem {
                            if sub.is_empty() {
                                // Empty Vec — nothing to flatten.
                            } else if sub[0].is_array() {
                                // An array-of-arrays is the Vec<(A,B)> — flatten it.
                                for pair in sub {
                                    if let serde_json::Value::Array(pair_elems) = pair {
                                        for pe in pair_elems {
                                            seq.serialize_element(pe)?;
                                        }
                                    } else {
                                        seq.serialize_element(pair)?;
                                    }
                                }
                            } else {
                                // Plain array value (e.g. a sub-expression like ["zoom"]).
                                seq.serialize_element(elem)?;
                            }
                        } else {
                            seq.serialize_element(elem)?;
                        }
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
            ColorOrArrayOfColor::InterpolateLab(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("interpolate-lab")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        if let serde_json::Value::Array(sub) = elem {
                            if sub.is_empty() {
                                // Empty Vec — nothing to flatten.
                            } else if sub[0].is_array() {
                                // An array-of-arrays is the Vec<(A,B)> — flatten it.
                                for pair in sub {
                                    if let serde_json::Value::Array(pair_elems) = pair {
                                        for pe in pair_elems {
                                            seq.serialize_element(pe)?;
                                        }
                                    } else {
                                        seq.serialize_element(pair)?;
                                    }
                                }
                            } else {
                                // Plain array value (e.g. a sub-expression like ["zoom"]).
                                seq.serialize_element(elem)?;
                            }
                        } else {
                            seq.serialize_element(elem)?;
                        }
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
            ColorOrArrayOfColor::Step(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("step")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        if let serde_json::Value::Array(sub) = elem {
                            if sub.is_empty() {
                                // Empty Vec — nothing to flatten.
                            } else if sub[0].is_array() {
                                // An array-of-arrays is the Vec<(A,B)> — flatten it.
                                for pair in sub {
                                    if let serde_json::Value::Array(pair_elems) = pair {
                                        for pe in pair_elems {
                                            seq.serialize_element(pe)?;
                                        }
                                    } else {
                                        seq.serialize_element(pair)?;
                                    }
                                }
                            } else {
                                // Plain array value (e.g. a sub-expression like ["zoom"]).
                                seq.serialize_element(elem)?;
                            }
                        } else {
                            seq.serialize_element(elem)?;
                        }
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
        }
    }
}

/// Either of the below variants
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum StringOrImageAsUnion {
    String(Box<String>),
    Image(Image),
}

impl serde::Serialize for StringOrImageAsUnion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::String(v) => v.serialize(serializer),
            Self::Image(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for StringOrImageAsUnion {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Box<String> as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::String(v)),
            Err(e) => errors.push(("String", e.to_string())),
        }
        match <Image as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Image(v)),
            Err(e) => errors.push(("Image", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "StringOrImageAsUnion: no variant matched. Expected String(Box<String>) | Image(Image). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Tuple row for variadic (content, optional style object) pairs.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct FormattedFormatVariadicRow(
    StringOrImageAsUnion,
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_option_json_map))]
    Option<serde_json::Map<std::string::String, serde_json::Value>>,
);

/// "Formatted"
#[derive(PartialEq, Debug, Clone)]
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
                let mut rest: Vec<serde_json::Value> = Vec::new();
                while let Some(v) = seq.next_element()? {
                    rest.push(v);
                }
                let mut idx = 0;
                while idx < rest.len() {
                    let input_i = serde_json::from_value(rest[idx].clone())
                        .map_err(serde::de::Error::custom)?;
                    idx += 1;
                    let style_overrides_i = if idx < rest.len() && rest[idx].is_object() {
                        let v = serde_json::from_value(rest[idx].clone())
                            .map_err(serde::de::Error::custom)?;
                        idx += 1;
                        Some(v)
                    } else {
                        None
                    };
                    let element = FormattedFormatVariadicRow(input_i, style_overrides_i);
                    inputs.push(element);
                }
                Ok(Formatted::Format(inputs))
            }
            _ => Err(serde::de::Error::unknown_variant(&op, &["format"])),
        }
    }
}

impl serde::Serialize for Formatted {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        match self {
            Formatted::Format(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("format")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        if let serde_json::Value::Array(pair_elems) = elem {
                            let mut trimmed = pair_elems.clone();
                            while trimmed.last().is_some_and(serde_json::Value::is_null) {
                                trimmed.pop();
                            }
                            for pe in &trimmed {
                                seq.serialize_element(pe)?;
                            }
                        } else {
                            seq.serialize_element(elem)?;
                        }
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
        }
    }
}

/// "Image"
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Image {
    /// Returns an `image` type for use in `icon-image`, `*-pattern` entries and as a section in the `format` expression. If set, the `image` argument will check that the requested image exists in the style and will return either the resolved image name or `null`, depending on whether or not the image is currently in the style. This validation process is synchronous and requires the image to have been added to the style before requesting it in the `image` argument.
    ///
    ///  - [Use a fallback image](https://maplibre.org/maplibre-gl-js/docs/examples/use-a-fallback-image/)
    Op(Box<String>),
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
                Ok(Image::Op(image_name))
            }
            _ => Err(serde::de::Error::unknown_variant(&op, &["image"])),
        }
    }
}

impl serde::Serialize for Image {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        match self {
            Image::Op(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("image")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
        }
    }
}

/// Either of the below variants
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum ArrayOrStringAsUnion {
    Array(Array),
    String(Box<String>),
}

impl serde::Serialize for ArrayOrStringAsUnion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Array(v) => v.serialize(serializer),
            Self::String(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for ArrayOrStringAsUnion {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <Array as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Array(v)),
            Err(e) => errors.push(("Array", e.to_string())),
        }
        match <Box<String> as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::String(v)),
            Err(e) => errors.push(("String", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "ArrayOrStringAsUnion: no variant matched. Expected Array(Array) | String(Box<String>). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// "Number"
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Number {
    /// Returns the remainder after integer division of the first input by the second.
    Percentage(Box<Number>, Box<Number>),
    /// Returns the product of the inputs.
    Star(Vec<Number>),
    /// Returns the sum of the inputs.
    Plus(Vec<Number>),
    /// For two inputs, returns the result of subtracting the second input from the first. For a single input, returns the result of subtracting it from 0.
    Minus(MinusOptions),
    /// Returns the result of floating point division of the first input by the second.
    ///
    ///  - [Visualize population density](https://maplibre.org/maplibre-gl-js/docs/examples/visualize-population-density/)
    Slash(Box<Number>, Box<Number>),
    /// Returns the result of raising the first input to the power specified by the second.
    Power(Box<Number>, Box<Number>),
    /// Returns the absolute value of the input.
    Absolute(Box<Number>),
    /// Returns the arccosine of the input.
    Arccosine(Box<Number>),
    /// Returns the arcsine of the input.
    Asin(Box<Number>),
    /// Returns the arctangent of the input.
    Atan(Box<Number>),
    /// Returns the smallest integer that is greater than or equal to the input.
    Ceil(Box<Number>),
    /// Returns the cosine of the input.
    Cos(Box<Number>),
    /// Returns the shortest distance in meters between the evaluated feature and the input geometry. The input value can be a valid GeoJSON of type `Point`, `MultiPoint`, `LineString`, `MultiLineString`, `Polygon`, `MultiPolygon`, `Feature`, or `FeatureCollection`. Distance values returned may vary in precision due to loss in precision from encoding geometries, particularly below zoom level 13.
    Distance(GeoJSONObjectLiteral),
    /// Returns the mathematical constant e.
    E,
    /// Gets the elevation of a pixel (in meters above the vertical datum reference of the `raster-dem` tiles) from a `raster-dem` source. Can only be used in the `color-relief-color` property of a `color-relief` layer.
    Elevation,
    /// Returns the largest integer that is less than or equal to the input.
    Floor(Box<Number>),
    /// Gets the kernel density estimation of a pixel in a heatmap layer, which is a relative measure of how many data points are crowded around a particular pixel. Can only be used in the `heatmap-color` property.
    HeatmapDensity,
    /// Returns the first position at which an item can be found in an array or a substring can be found in a string, or `-1` if the input cannot be found. Accepts an optional index from where to begin the search. In a string, a UTF-16 surrogate pair counts as a single position.
    IndexOf(IndexOfOptions),
    /// Gets the length of an array or string. In a string, a UTF-16 surrogate pair counts as a single position.
    Length(ArrayOrStringAsUnion),
    /// Gets the progress along a gradient line. Can only be used in the `line-gradient` property.
    LineProgress,
    /// Returns the natural logarithm of the input.
    Ln(Box<Number>),
    /// Returns the mathematical constant ln(2).
    Ln2,
    /// Returns the base-ten logarithm of the input.
    Log10(Box<Number>),
    /// Returns the base-two logarithm of the input.
    Log2(Box<Number>),
    /// Returns the maximum value of the inputs.
    Max(Vec<Number>),
    /// Returns the minimum value of the inputs.
    Min(Vec<Number>),
    /// Asserts that the input value is a number. If multiple values are provided, each one is evaluated in order until a number is obtained. If none of the inputs are numbers, the expression is an error.
    Op(Vec<ExprOrLiteral>),
    /// Returns the mathematical constant pi.
    Pi,
    /// Rounds the input to the nearest integer. Halfway values are rounded away from zero. For example, `["round", -1.5]` evaluates to -2.
    Round(Box<Number>),
    /// Returns the sine of the input.
    Sin(Box<Number>),
    /// Returns the square root of the input.
    Sqrt(Box<Number>),
    /// Returns the tangent of the input.
    Tan(Box<Number>),
    /// Converts the input value to a number, if possible. If the input is `null` or `false`, the result is 0. If the input is `true`, the result is 1. If the input is a string, it is converted to a number as specified by the ["ToNumber Applied to the String Type" algorithm](https://tc39.github.io/ecma262/#sec-tonumber-applied-to-the-string-type) of the ECMAScript Language Specification. If multiple values are provided, each one is evaluated in order until the first successful conversion is obtained. If none of the inputs can be converted, the expression is an error.
    To(Vec<ExprOrLiteral>),
    /// Gets the current zoom level.  Note that in style layout and paint properties, ["zoom"] may only appear as the input to a top-level "step" or "interpolate" expression.
    Zoom,
    /// A numeric literal value.
    Literal(NumberLiteral),
    /// A polymorphic expression (`case`, `match`, `get`, …) in a numeric position.
    AnyExpr(Box<Any>),
}

/// Options for deserializing the syntax enum variant [`Number::Minus`]
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum MinusOptions {
    TwoParams(Box<Number>, Box<Number>),
    OneParams(Box<Number>),
}

/// Options for deserializing the syntax enum variant [`Number::IndexOf`]
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum IndexOfOptions {
    Item(
        ExprOrLiteral,
        ExprOrLiteral,
        #[serde(default)]
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_option_json_value))]
        Option<serde_json::Value>,
    ),
    Substring(
        Box<String>,
        Box<String>,
        #[serde(default)]
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_option_json_value))]
        Option<serde_json::Value>,
    ),
}

impl<'de> serde::Deserialize<'de> for Number {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(NumberVisitor)
    }
}

/// Visitor for deserializing the syntax enum [`Number`]
struct NumberVisitor;

impl<'de> serde::de::Visitor<'de> for NumberVisitor {
    type Value = Number;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an Number expression (example: [\"%\",10,3])")
    }

    fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
        Ok(Number::Literal(NumberLiteral::from(
            serde_json::Number::from(v),
        )))
    }

    fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
        Ok(Number::Literal(NumberLiteral::from(
            serde_json::Number::from(v),
        )))
    }

    fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Self::Value, E> {
        serde_json::Number::from_f64(v)
            .map(|n| Number::Literal(NumberLiteral::from(n)))
            .ok_or_else(|| serde::de::Error::custom("non-finite f64"))
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
                Ok(Number::Star(inputs))
            }
            "+" => {
                let mut inputs = Vec::new();
                while let Some(element) = seq.next_element()? {
                    inputs.push(element);
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
                        serde_json::from_value::<Box<Number>>(rest[0].clone())
                            .map_err(serde::de::Error::custom)?,
                        serde_json::from_value::<Box<Number>>(rest[1].clone())
                            .map_err(serde::de::Error::custom)?,
                    ))),
                    1 => Ok(Number::Minus(MinusOptions::OneParams(
                        serde_json::from_value::<Box<Number>>(rest[0].clone())
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
                Ok(Number::Max(inputs))
            }
            "min" => {
                let mut inputs = Vec::new();
                while let Some(element) = seq.next_element()? {
                    inputs.push(element);
                }
                Ok(Number::Min(inputs))
            }
            "number" => {
                let mut inputs = Vec::new();
                while let Some(element) = seq.next_element()? {
                    inputs.push(element);
                }
                Ok(Number::Op(inputs))
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
                Ok(Number::To(inputs))
            }
            "zoom" => Ok(Number::Zoom),
            _ => {
                let mut elems = vec![serde_json::Value::String(op)];
                while let Some(v) = seq.next_element::<serde_json::Value>()? {
                    elems.push(v);
                }
                let arr = serde_json::Value::Array(elems);
                let any_expr =
                    serde_json::from_value::<Any>(arr).map_err(serde::de::Error::custom)?;
                Ok(Number::AnyExpr(Box::new(any_expr)))
            }
        }
    }
}

impl serde::Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        match self {
            Number::Percentage(f0, f1) => {
                let mut elems = vec![
                    serde_json::to_value(f0).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f1).map_err(serde::ser::Error::custom)?,
                ];
                while elems.len() > 2 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("%")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::Star(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("*")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        seq.serialize_element(elem)?;
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
            Number::Plus(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("+")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        seq.serialize_element(elem)?;
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
            Number::Minus(opts) => {
                let opts_val = serde_json::to_value(opts).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("-")?;
                if let serde_json::Value::Array(mut arr) = opts_val {
                    if arr.first().is_some_and(serde_json::Value::is_string) {
                        // Single-field variant unwrapped by serde — emit as one element.
                        seq.serialize_element(&serde_json::Value::Array(arr))?;
                    } else {
                        while arr.len() > 1 && arr.last().is_some_and(serde_json::Value::is_null) {
                            arr.pop();
                        }
                        for elem in &arr {
                            seq.serialize_element(elem)?;
                        }
                    }
                } else {
                    seq.serialize_element(&opts_val)?;
                }
                seq.end()
            }
            Number::Slash(f0, f1) => {
                let mut elems = vec![
                    serde_json::to_value(f0).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f1).map_err(serde::ser::Error::custom)?,
                ];
                while elems.len() > 2 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("/")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::Power(f0, f1) => {
                let mut elems = vec![
                    serde_json::to_value(f0).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f1).map_err(serde::ser::Error::custom)?,
                ];
                while elems.len() > 2 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("^")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::Absolute(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("abs")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::Arccosine(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("acos")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::Asin(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("asin")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::Atan(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("atan")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::Ceil(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("ceil")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::Cos(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("cos")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::Distance(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("distance")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::E => {
                let mut elems = vec![];
                while !elems.is_empty() && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("e")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::Elevation => {
                let mut elems = vec![];
                while !elems.is_empty() && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("elevation")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::Floor(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("floor")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::HeatmapDensity => {
                let mut elems = vec![];
                while !elems.is_empty() && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("heatmap-density")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::IndexOf(opts) => {
                let opts_val = serde_json::to_value(opts).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("index-of")?;
                if let serde_json::Value::Array(mut arr) = opts_val {
                    while arr.len() > 2 && arr.last().is_some_and(serde_json::Value::is_null) {
                        arr.pop();
                    }
                    for elem in &arr {
                        seq.serialize_element(elem)?;
                    }
                } else {
                    seq.serialize_element(&opts_val)?;
                }
                seq.end()
            }
            Number::Length(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("length")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::LineProgress => {
                let mut elems = vec![];
                while !elems.is_empty() && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("line-progress")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::Ln(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("ln")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::Ln2 => {
                let mut elems = vec![];
                while !elems.is_empty() && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("ln2")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::Log10(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("log10")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::Log2(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("log2")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::Max(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("max")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        seq.serialize_element(elem)?;
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
            Number::Min(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("min")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        seq.serialize_element(elem)?;
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
            Number::Op(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("number")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        seq.serialize_element(elem)?;
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
            Number::Pi => {
                let mut elems = vec![];
                while !elems.is_empty() && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("pi")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::Round(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("round")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::Sin(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("sin")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::Sqrt(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("sqrt")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::Tan(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("tan")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::To(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("to-number")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        seq.serialize_element(elem)?;
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
            Number::Zoom => {
                let mut elems = vec![];
                while !elems.is_empty() && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("zoom")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Number::Literal(n) => n.serialize(serializer),
            Number::AnyExpr(a) => a.serialize(serializer),
        }
    }
}

/// Either of the below variants
#[derive(PartialEq, Debug, Clone)]
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

impl serde::Serialize for NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjectionAsUnion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::ArrayOfNumber(v) => v.serialize(serializer),
            Self::Color(v) => v.serialize(serializer),
            Self::ArrayOfColor(v) => v.serialize(serializer),
            Self::Projection(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de>
    for NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjectionAsUnion
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        match <NumberLiteral as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <Color as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Color(v)),
            Err(e) => errors.push(("Color", e.to_string())),
        }
        match <ColorOrArrayOfColor as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::ArrayOfColor(v)),
            Err(e) => errors.push(("ArrayOfColor", e.to_string())),
        }
        match <Box<ProjectionType> as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Projection(v)),
            Err(e) => errors.push(("Projection", e.to_string())),
        }
        match <serde_json::Value as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::ArrayOfNumber(v)),
            Err(e) => errors.push(("ArrayOfNumber", e.to_string())),
        }

        let details: Vec<std::string::String> =
            errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjectionAsUnion: no variant matched. Expected Number(NumberLiteral) | ArrayOfNumber(serde_json::Value) | Color(Color) | ArrayOfColor(ColorOrArrayOfColor) | Projection(Box<ProjectionType>). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// "NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection"
#[derive(PartialEq, Debug, Clone)]
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
            Number,
            Vec<(
                NumberLiteral,
                Box<NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjectionAsUnion>,
            )>,
        ),
    ),
    /// Produces discrete, stepped results by evaluating a piecewise-constant function defined by pairs of input and output values ("stops"). The `input` may be any numeric expression (e.g., `["get", "population"]`). Stop inputs must be numeric literals in strictly ascending order.
    ///
    /// Returns the output value of the stop just less than the input, or the first output if the input is less than the first stop.
    ///
    ///  - [Create and style clusters](https://maplibre.org/maplibre-gl-js/docs/examples/create-and-style-clusters/)
    Step(
        (
            Number,
            Box<NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjectionAsUnion>,
            Vec<(
                NumberLiteral,
                Box<NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjectionAsUnion>,
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
                let input: Number = visit_seq_field(&mut seq, "input")?;
                let mut stops = Vec::new();
                while let Some(stop_input_i) = seq.next_element::<NumberLiteral>()? {
                    let stop_output_i: Box<NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjectionAsUnion> = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("expected stop_output_i in NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection::Interpolate"))?;
                    stops.push((stop_input_i, stop_output_i));
                }
                Ok(
                    NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection::Interpolate((
                        interpolation_type,
                        input,
                        stops,
                    )),
                )
            }
            "step" => {
                let input: Number = visit_seq_field(&mut seq, "input")?;
                let output_0: Box<NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjectionAsUnion> =
                    visit_seq_field(&mut seq, "output_0")?;
                let mut stops = Vec::new();
                while let Some(stop_input_i) = seq.next_element::<NumberLiteral>()? {
                    let stop_output_i: Box<NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjectionAsUnion> = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("expected stop_output_i in NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection::Step"))?;
                    stops.push((stop_input_i, stop_output_i));
                }
                Ok(
                    NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection::Step((
                        input, output_0, stops,
                    )),
                )
            }
            _ => Err(serde::de::Error::unknown_variant(
                &op,
                &["interpolate", "step"],
            )),
        }
    }
}

impl serde::Serialize for NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        match self {
            NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection::Interpolate(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("interpolate")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        if let serde_json::Value::Array(sub) = elem {
                            if sub.is_empty() {
                                // Empty Vec — nothing to flatten.
                            } else if sub[0].is_array() {
                                // An array-of-arrays is the Vec<(A,B)> — flatten it.
                                for pair in sub {
                                    if let serde_json::Value::Array(pair_elems) = pair {
                                        for pe in pair_elems {
                                            seq.serialize_element(pe)?;
                                        }
                                    } else {
                                        seq.serialize_element(pair)?;
                                    }
                                }
                            } else {
                                // Plain array value (e.g. a sub-expression like ["zoom"]).
                                seq.serialize_element(elem)?;
                            }
                        } else {
                            seq.serialize_element(elem)?;
                        }
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
            NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection::Step(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("step")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        if let serde_json::Value::Array(sub) = elem {
                            if sub.is_empty() {
                                // Empty Vec — nothing to flatten.
                            } else if sub[0].is_array() {
                                // An array-of-arrays is the Vec<(A,B)> — flatten it.
                                for pair in sub {
                                    if let serde_json::Value::Array(pair_elems) = pair {
                                        for pe in pair_elems {
                                            seq.serialize_element(pe)?;
                                        }
                                    } else {
                                        seq.serialize_element(pair)?;
                                    }
                                }
                            } else {
                                // Plain array value (e.g. a sub-expression like ["zoom"]).
                                seq.serialize_element(elem)?;
                            }
                        } else {
                            seq.serialize_element(elem)?;
                        }
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
        }
    }
}

/// "Object"
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Object {
    /// Provides a literal array or object value.
    ///
    ///  - [Display and style rich text labels](https://maplibre.org/maplibre-gl-js/docs/examples/display-and-style-rich-text-labels/)
    Literal(JSONObjectLiteral),
    /// Asserts that the input value is an object. If multiple values are provided, each one is evaluated in order until an object is obtained. If none of the inputs are objects, the expression is an error.
    Op(Vec<ExprOrLiteral>),
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
                Ok(Object::Op(inputs))
            }
            "properties" => Ok(Object::Properties),
            _ => Err(serde::de::Error::unknown_variant(
                &op,
                &["literal", "object", "properties"],
            )),
        }
    }
}

impl serde::Serialize for Object {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        match self {
            Object::Literal(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("literal")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Object::Op(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("object")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        seq.serialize_element(elem)?;
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
            Object::Properties => {
                let mut elems = vec![];
                while !elems.is_empty() && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("properties")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
        }
    }
}

/// "String"
#[derive(PartialEq, Debug, Clone)]
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
    Concat(Vec<ExprOrLiteral>),
    /// Returns the input string converted to lowercase. Follows the Unicode Default Case Conversion algorithm and the locale-insensitive case mappings in the Unicode Character Database.
    ///
    ///  - [Change the case of labels](https://maplibre.org/maplibre-gl-js/docs/examples/change-case-of-labels/)
    Downcase(Box<String>),
    /// Returns the feature's simple geometry type: `Point`, `LineString`, or `Polygon`. `MultiPoint`, `MultiLineString`, and `MultiPolygon` are returned as `Point`, `LineString`, and `Polygon`, respectively.
    GeometryType,
    /// Returns a string formed by concatenating the elements of the input array, inserting a separator between each element.
    Join(Box<Array>, Box<String>),
    /// Converts the input number into a string representation using the provided format_options.
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    NumberFormat(
        Box<Number>,
        #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_map))]
        serde_json::Map<std::string::String, serde_json::Value>,
    ),
    /// Returns the IETF language tag of the locale being used by the provided `collator`. This can be used to determine the default system locale, or to determine if a requested locale was successfully loaded.
    ResolvedLocale(Collator),
    /// Returns a subarray from an array or a substring from a string from a specified start index, or between a start index and an end index if set. The return value is inclusive of the start index but not of the end index. In a string, a UTF-16 surrogate pair counts as a single position.
    Slice(Box<String>, Box<Number>, Option<Box<Number>>),
    /// Asserts that the input value is a string. If multiple values are provided, each one is evaluated in order until a string is obtained. If none of the inputs are strings, the expression is an error.
    Op(Vec<ExprOrLiteral>),
    /// Converts the input value to a string. If the input is `null`, the result is `""`. If the input is a boolean, the result is `"true"` or `"false"`. If the input is a number, it is converted to a string as specified by the ["NumberToString" algorithm](https://tc39.github.io/ecma262/#sec-tostring-applied-to-the-number-type) of the ECMAScript Language Specification. If the input is a color, it is converted to a string of the form `"rgba(r,g,b,a)"`, where `r`, `g`, and `b` are numerals ranging from 0 to 255, and `a` ranges from 0 to 1. Otherwise, the input is converted to a string in the format specified by the [`JSON.stringify`](https://tc39.github.io/ecma262/#sec-json.stringify) function of the ECMAScript Language Specification.
    ///
    ///  - [Create a time slider](https://maplibre.org/maplibre-gl-js/docs/examples/create-a-time-slider/)
    To(ExprOrLiteral),
    /// Returns a string describing the type of the given value.
    Typeof(ExprOrLiteral),
    /// Returns the input string converted to uppercase. Follows the Unicode Default Case Conversion algorithm and the locale-insensitive case mappings in the Unicode Character Database.
    ///
    ///  - [Change the case of labels](https://maplibre.org/maplibre-gl-js/docs/examples/change-case-of-labels/)
    Upcase(Box<String>),
    /// A string literal value.
    Literal(StringLiteral),
    /// A polymorphic expression (`case`, `match`, `get`, …) in a string position.
    AnyExpr(Box<Any>),
}

impl<'de> serde::Deserialize<'de> for String {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(StringVisitor)
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

    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(String::Literal(StringLiteral::from(v.to_string())))
    }

    fn visit_string<E: serde::de::Error>(self, v: std::string::String) -> Result<Self::Value, E> {
        Ok(String::Literal(StringLiteral::from(v)))
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
                Ok(String::Concat(inputs))
            }
            "downcase" => {
                let input = visit_seq_field(&mut seq, "input")?;
                Ok(String::Downcase(input))
            }
            "geometry-type" => Ok(String::GeometryType),
            "join" => {
                let input = visit_seq_field(&mut seq, "input")?;
                let separator = visit_seq_field(&mut seq, "separator")?;
                Ok(String::Join(input, separator))
            }
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
                Ok(String::Op(inputs))
            }
            "to-string" => {
                let value = visit_seq_field(&mut seq, "value")?;
                Ok(String::To(value))
            }
            "typeof" => {
                let value = visit_seq_field(&mut seq, "value")?;
                Ok(String::Typeof(value))
            }
            "upcase" => {
                let input = visit_seq_field(&mut seq, "input")?;
                Ok(String::Upcase(input))
            }
            _ => {
                let mut elems = vec![serde_json::Value::String(op)];
                while let Some(v) = seq.next_element::<serde_json::Value>()? {
                    elems.push(v);
                }
                let arr = serde_json::Value::Array(elems);
                let any_expr =
                    serde_json::from_value::<Any>(arr).map_err(serde::de::Error::custom)?;
                Ok(String::AnyExpr(Box::new(any_expr)))
            }
        }
    }
}

impl serde::Serialize for String {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        match self {
            String::Concat(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("concat")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        seq.serialize_element(elem)?;
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
            String::Downcase(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("downcase")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            String::GeometryType => {
                let mut elems = vec![];
                while !elems.is_empty() && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("geometry-type")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            String::Join(f0, f1) => {
                let mut elems = vec![
                    serde_json::to_value(f0).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f1).map_err(serde::ser::Error::custom)?,
                ];
                while elems.len() > 2 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("join")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            String::NumberFormat(f0, f1) => {
                let mut elems = vec![
                    serde_json::to_value(f0).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f1).map_err(serde::ser::Error::custom)?,
                ];
                while elems.len() > 2 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("number-format")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            String::ResolvedLocale(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("resolved-locale")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            String::Slice(f0, f1, f2) => {
                let mut elems = vec![
                    serde_json::to_value(f0).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f1).map_err(serde::ser::Error::custom)?,
                    serde_json::to_value(f2).map_err(serde::ser::Error::custom)?,
                ];
                while elems.len() > 2 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("slice")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            String::Op(inner) => {
                let inner_val = serde_json::to_value(inner).map_err(serde::ser::Error::custom)?;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("string")?;
                if let serde_json::Value::Array(top) = &inner_val {
                    for elem in top {
                        seq.serialize_element(elem)?;
                    }
                } else {
                    seq.serialize_element(&inner_val)?;
                }
                seq.end()
            }
            String::To(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("to-string")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            String::Typeof(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("typeof")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            String::Upcase(f0) => {
                let mut elems = vec![serde_json::to_value(f0).map_err(serde::ser::Error::custom)?];
                while elems.len() > 1 && elems.last().is_some_and(serde_json::Value::is_null) {
                    elems.pop();
                }
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("upcase")?;
                for elem in &elems {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            String::Literal(s) => s.serialize(serializer),
            String::AnyExpr(a) => a.serialize(serializer),
        }
    }
}

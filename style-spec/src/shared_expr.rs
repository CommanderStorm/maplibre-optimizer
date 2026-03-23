//! Shared expression + property inner types for generated spec code.
//!
//! Instead of generating ~70 lines per expression-backed property (expression enum + serde +
//! wrapper enum + serde + default), the generator emits a single macro invocation that wraps
//! one of the shared inner types defined here.

// Expression operator enums are inherently large; boxing would add indirection on every access.
#![allow(clippy::large_enum_variant)]

// ── Shared Expression Enums ─────────────────────────────────────────────────

/// Nested expression: ramp (`interpolate` / …) or regular [`Number`](crate::spec::Number) operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum NumericExpression {
    Number(crate::spec::Number),
    Ramp(crate::spec::NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for NumericExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for NumericExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, String)> = Vec::new();
        match <crate::spec::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Number(v)),
            Err(e) => errors.push(("Number", e.to_string())),
        }
        match <crate::spec::NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }
        let details: Vec<String> = errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "NumericExpression: no variant matched. Expected Number(Number) | Ramp(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Nested expression: ramp (`interpolate-hcl`, …) or [`Color`](crate::spec::Color) operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum ColorExpression {
    Color(crate::spec::Color),
    Ramp(crate::spec::ColorOrArrayOfColor),
}

impl serde::Serialize for ColorExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Color(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for ColorExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, String)> = Vec::new();
        match <crate::spec::Color as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Color(v)),
            Err(e) => errors.push(("Color", e.to_string())),
        }
        match <crate::spec::ColorOrArrayOfColor as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Ramp(v)),
            Err(e) => errors.push(("Ramp", e.to_string())),
        }
        let details: Vec<String> = errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "ColorExpression: no variant matched. Expected Color(Color) | Ramp(ColorOrArrayOfColor). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Nested expression: [`Boolean`](crate::spec::Boolean) operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum BooleanExpression {
    Boolean(crate::spec::Boolean),
}

impl serde::Serialize for BooleanExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Boolean(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for BooleanExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, String)> = Vec::new();
        match <crate::spec::Boolean as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Boolean(v)),
            Err(e) => errors.push(("Boolean", e.to_string())),
        }
        let details: Vec<String> = errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "BooleanExpression: no variant matched. Expected Boolean(Boolean). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Nested expression: [`String`](crate::spec::String) operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum StringExpression {
    String(crate::spec::String),
}

impl serde::Serialize for StringExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::String(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for StringExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, String)> = Vec::new();
        match <crate::spec::String as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::String(v)),
            Err(e) => errors.push(("String", e.to_string())),
        }
        let details: Vec<String> = errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "StringExpression: no variant matched. Expected String(String). Errors: [{}]",
            details.join("; ")
        )))
    }
}

// ── Shared Inner Prop Enums ─────────────────────────────────────────────────

/// Inner representation for numeric expression-backed properties.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum NumericPropInner {
    Expr(Box<NumericExpression>),
    Literal(
        #[cfg_attr(
            feature = "fuzz",
            arbitrary(with = crate::fuzz_helpers::arbitrary_json_number)
        )]
        serde_json::Number,
    ),
}

impl NumericPropInner {
    /// If this is a literal number, return it as `f64`.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Literal(n) => n.as_f64(),
            Self::Expr(_) => None,
        }
    }
}

impl serde::Serialize for NumericPropInner {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for NumericPropInner {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, String)> = Vec::new();
        match <NumericExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }
        let details: Vec<String> = errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "NumericPropInner: no variant matched. Expected Expr(NumericExpression) | Literal(serde_json::Number). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Inner representation for color expression-backed properties.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum ColorPropInner {
    Expr(Box<ColorExpression>),
    Literal(
        #[cfg_attr(
            feature = "fuzz",
            arbitrary(with = crate::fuzz_helpers::arbitrary_json_value)
        )]
        serde_json::Value,
    ),
}

impl serde::Serialize for ColorPropInner {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for ColorPropInner {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, String)> = Vec::new();
        match <ColorExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        // serde_json::Value always succeeds — must be last
        match <serde_json::Value as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }
        let details: Vec<String> = errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "ColorPropInner: no variant matched. Expected Expr(ColorExpression) | Literal(serde_json::Value). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Inner representation for boolean expression-backed properties.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum BooleanPropInner {
    Expr(Box<BooleanExpression>),
    Literal(bool),
}

impl serde::Serialize for BooleanPropInner {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for BooleanPropInner {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, String)> = Vec::new();
        match <BooleanExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <bool as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }
        let details: Vec<String> = errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "BooleanPropInner: no variant matched. Expected Expr(BooleanExpression) | Literal(bool). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Inner representation for array-like expression-backed properties.
/// Uses `serde_json::Value` for the literal branch to accommodate diverse array shapes.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum ArrayPropInner {
    Expr(Box<StringExpression>),
    Literal(
        #[cfg_attr(
            feature = "fuzz",
            arbitrary(with = crate::fuzz_helpers::arbitrary_json_value)
        )]
        serde_json::Value,
    ),
}

impl serde::Serialize for ArrayPropInner {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for ArrayPropInner {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, String)> = Vec::new();
        match <StringExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        // serde_json::Value always succeeds — must be last
        match <serde_json::Value as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }
        let details: Vec<String> = errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "ArrayPropInner: no variant matched. Expected Expr(StringExpression) | Literal(serde_json::Value). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Inner representation for string expression-backed properties.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum StringPropInner {
    Expr(Box<StringExpression>),
    Literal(std::string::String),
}

impl serde::Serialize for StringPropInner {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for StringPropInner {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, String)> = Vec::new();
        match <StringExpression as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <std::string::String as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }
        let details: Vec<String> = errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "StringPropInner: no variant matched. Expected Expr(StringExpression) | Literal(std::string::String). Errors: [{}]",
            details.join("; ")
        )))
    }
}

// ── Shared Visibility Enum ──────────────────────────────────────────────────

/// Whether a layer is displayed.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Visibility {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "visible")]
    #[default]
    Visible,
}

// ── Per-Property Newtype Macros ─────────────────────────────────────────────

/// Stamp out a newtype wrapping [`NumericPropInner`] with optional bounds validation and default.
///
/// Bounds are validated on the `Literal` arm during deserialization.
#[macro_export]
macro_rules! numeric_prop {
    ($name:ident, doc = $doc:expr $(, min = $min:expr)? $(, max = $max:expr)? $(, default = $default:expr)?) => {
        #[doc = $doc]
        #[derive(PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct $name(pub $crate::shared_expr::NumericPropInner);

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.0.serialize(serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let inner = <$crate::shared_expr::NumericPropInner as serde::Deserialize>::deserialize(deserializer)?;
                if let $crate::shared_expr::NumericPropInner::Literal(ref n) = inner {
                    let _v = n.as_f64().unwrap_or(f64::NAN);
                    $(if _v < $min {
                        return Err(serde::de::Error::custom(
                            format!(concat!(stringify!($name), ": {} < minimum {}"), _v, $min)
                        ));
                    })?
                    $(if _v > $max {
                        return Err(serde::de::Error::custom(
                            format!(concat!(stringify!($name), ": {} > maximum {}"), _v, $max)
                        ));
                    })?
                }
                Ok(Self(inner))
            }
        }

        $crate::numeric_prop!(@default $name $(, $default)?);
    };

    // Default impl when a default value is provided
    (@default $name:ident, $default:expr) => {
        impl Default for $name {
            fn default() -> Self {
                Self($crate::shared_expr::NumericPropInner::Literal($default))
            }
        }
    };
    // No default
    (@default $name:ident) => {};
}

/// Stamp out a newtype wrapping [`ColorPropInner`] with optional default.
#[macro_export]
macro_rules! color_prop {
    ($name:ident, doc = $doc:expr $(, default = $default:expr)?) => {
        #[doc = $doc]
        #[derive(PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct $name(pub $crate::shared_expr::ColorPropInner);

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.0.serialize(serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let inner = <$crate::shared_expr::ColorPropInner as serde::Deserialize>::deserialize(deserializer)?;
                Ok(Self(inner))
            }
        }

        $crate::color_prop!(@default $name $(, $default)?);
    };

    (@default $name:ident, $default:expr) => {
        impl Default for $name {
            fn default() -> Self {
                Self($crate::shared_expr::ColorPropInner::Literal($default))
            }
        }
    };
    (@default $name:ident) => {};
}

/// Stamp out a newtype wrapping [`BooleanPropInner`] with optional default.
#[macro_export]
macro_rules! boolean_prop {
    ($name:ident, doc = $doc:expr $(, default = $default:expr)?) => {
        #[doc = $doc]
        #[derive(PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct $name(pub $crate::shared_expr::BooleanPropInner);

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.0.serialize(serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let inner = <$crate::shared_expr::BooleanPropInner as serde::Deserialize>::deserialize(deserializer)?;
                Ok(Self(inner))
            }
        }

        $crate::boolean_prop!(@default $name $(, $default)?);
    };

    (@default $name:ident, $default:expr) => {
        impl Default for $name {
            fn default() -> Self {
                Self($crate::shared_expr::BooleanPropInner::Literal($default))
            }
        }
    };
    (@default $name:ident) => {};
}

/// Stamp out a newtype wrapping [`StringPropInner`] with optional default.
#[macro_export]
macro_rules! string_prop {
    ($name:ident, doc = $doc:expr $(, default = $default:expr)?) => {
        #[doc = $doc]
        #[derive(PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct $name(pub $crate::shared_expr::StringPropInner);

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.0.serialize(serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let inner = <$crate::shared_expr::StringPropInner as serde::Deserialize>::deserialize(deserializer)?;
                Ok(Self(inner))
            }
        }

        $crate::string_prop!(@default $name $(, $default)?);
    };

    (@default $name:ident, $default:expr) => {
        impl Default for $name {
            fn default() -> Self {
                Self($crate::shared_expr::StringPropInner::Literal($default))
            }
        }
    };
    (@default $name:ident) => {};
}

/// Stamp out a newtype wrapping [`ArrayPropInner`] with optional default.
#[macro_export]
macro_rules! array_prop {
    ($name:ident, doc = $doc:expr $(, default = $default:expr)?) => {
        #[doc = $doc]
        #[derive(PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct $name(pub $crate::shared_expr::ArrayPropInner);

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.0.serialize(serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let inner = <$crate::shared_expr::ArrayPropInner as serde::Deserialize>::deserialize(deserializer)?;
                Ok(Self(inner))
            }
        }

        $crate::array_prop!(@default $name $(, $default)?);
    };

    (@default $name:ident, $default:expr) => {
        impl Default for $name {
            fn default() -> Self {
                Self($crate::shared_expr::ArrayPropInner::Literal($default))
            }
        }
    };
    (@default $name:ident) => {};
}

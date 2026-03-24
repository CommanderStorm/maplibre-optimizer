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

impl NumericExpression {
    /// If this represents a literal number, return it as `f64`.
    ///
    /// Handles `Number::Literal(NumberLiteral(n))` for bare JSON numbers.
    pub fn as_f64(&self) -> Option<f64> {
        let v = serde_json::to_value(self).ok()?;
        v.as_f64()
    }

    /// Build a literal `NumericExpression` from an `f64`.
    #[cfg(feature = "fuzz")]
    pub fn from_f64(v: f64) -> Self {
        let n = serde_json::Number::from_f64(v).unwrap_or_else(|| serde_json::Number::from(0));
        Self::Number(crate::spec::Number::Literal(
            crate::spec::NumberLiteral::from(n),
        ))
    }
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

/// Nested expression: `interpolate` / `interpolate-hcl` / `interpolate-lab` ramps,
/// or [`Color`](crate::spec::Color) operators.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum ColorExpression {
    Color(crate::spec::Color),
    Ramp(crate::spec::ColorOrArrayOfColor),
    /// Generic `interpolate` — can produce colors as well as numbers/arrays.
    Interpolate(crate::spec::NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection),
}

impl serde::Serialize for ColorExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Color(v) => v.serialize(serializer),
            Self::Ramp(v) => v.serialize(serializer),
            Self::Interpolate(v) => v.serialize(serializer),
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
        match <crate::spec::NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Interpolate(v)),
            Err(e) => errors.push(("Interpolate", e.to_string())),
        }
        let details: Vec<String> = errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "ColorExpression: no variant matched. Expected Color(Color) | Ramp(ColorOrArrayOfColor) | Interpolate(NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection). Errors: [{}]",
            details.join("; ")
        )))
    }
}

// ── Shared Inner Prop Enums ─────────────────────────────────────────────────

/// Inner representation for array-like expression-backed properties.
/// Uses `serde_json::Value` for the literal branch to accommodate diverse array shapes.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum ArrayPropInner {
    Expr(Box<crate::spec::String>),
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
        match <crate::spec::String as serde::Deserialize>::deserialize(&value) {
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
            "ArrayPropInner: no variant matched. Expected Expr(String) | Literal(serde_json::Value). Errors: [{}]",
            details.join("; ")
        )))
    }
}

/// Inner representation for formatted expression-backed properties.
/// Accepts `Formatted` expressions (e.g. `["format", ...]`), string expressions, or plain string literals.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum FormattedPropInner {
    Formatted(Box<crate::spec::Formatted>),
    Expr(Box<crate::spec::String>),
    Literal(std::string::String),
}

impl serde::Serialize for FormattedPropInner {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Formatted(v) => v.as_ref().serialize(serializer),
            Self::Expr(v) => v.as_ref().serialize(serializer),
            Self::Literal(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for FormattedPropInner {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, String)> = Vec::new();
        match <crate::spec::Formatted as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Formatted(Box::new(v))),
            Err(e) => errors.push(("Formatted", e.to_string())),
        }
        match <crate::spec::String as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Expr(Box::new(v))),
            Err(e) => errors.push(("Expr", e.to_string())),
        }
        match <std::string::String as serde::Deserialize>::deserialize(&value) {
            Ok(v) => return Ok(Self::Literal(v)),
            Err(e) => errors.push(("Literal", e.to_string())),
        }
        let details: Vec<String> = errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
        Err(serde::de::Error::custom(format!(
            "FormattedPropInner: no variant matched. Expected Formatted | Expr(String) | Literal(std::string::String). Errors: [{}]",
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

/// Stamp out a newtype wrapping [`NumericExpression`] with optional bounds validation and default.
///
/// `NumericExpression` already handles bare number literals via `Number::Literal(NumberLiteral)`,
/// so no separate `NumericPropInner` wrapper is needed.
/// Bounds are validated on literal values during deserialization.
#[macro_export]
macro_rules! numeric_prop {
    ($name:ident, doc = $doc:expr $(, min = $min:expr)? $(, max = $max:expr)? $(, default = $default:expr)?) => {
        #[doc = $doc]
        #[derive(PartialEq, Debug, Clone)]
        pub struct $name(pub $crate::shared_expr::NumericExpression);

        #[cfg(feature = "fuzz")]
        impl<'a> arbitrary::Arbitrary<'a> for $name {
            fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
                let inner = $crate::shared_expr::NumericExpression::arbitrary(u)?;
                if let Some(v) = inner.as_f64() {
                    let mut _clamped = v;
                    $(_clamped = _clamped.max($min as f64);)?
                    $(_clamped = _clamped.min($max as f64);)?
                    if _clamped != v {
                        return Ok(Self($crate::shared_expr::NumericExpression::from_f64(_clamped)));
                    }
                }
                Ok(Self(inner))
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.0.serialize(serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let inner = <$crate::shared_expr::NumericExpression as serde::Deserialize>::deserialize(deserializer)?;
                if let Some(_v) = inner.as_f64() {
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
                let value = serde_json::Value::Number($default);
                let expr = serde_json::from_value(value).expect("invalid numeric default");
                Self(expr)
            }
        }
    };
    // No default
    (@default $name:ident) => {};
}

/// Stamp out a newtype wrapping [`ColorExpression`] with optional default.
///
/// `ColorExpression` already handles bare string literals via `Color::Literal(StringLiteral)`,
/// so no separate `ColorPropInner` wrapper is needed.
#[macro_export]
macro_rules! color_prop {
    ($name:ident, doc = $doc:expr $(, default = $default:expr)?) => {
        #[doc = $doc]
        #[derive(PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct $name(pub $crate::shared_expr::ColorExpression);

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.0.serialize(serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let inner = <$crate::shared_expr::ColorExpression as serde::Deserialize>::deserialize(deserializer)?;
                Ok(Self(inner))
            }
        }

        $crate::color_prop!(@default $name $(, $default)?);
    };

    (@default $name:ident, $default:expr) => {
        impl Default for $name {
            fn default() -> Self {
                let value = $default;
                let expr = serde_json::from_value(value).expect("invalid color default");
                Self(expr)
            }
        }
    };
    (@default $name:ident) => {};
}

/// Stamp out a newtype wrapping [`Boolean`](crate::spec::Boolean) with optional default.
///
/// `Boolean` already has `Literal(bool)` and handles bare `true`/`false` via `visit_bool`,
/// so no separate `BooleanPropInner` wrapper is needed.
#[macro_export]
macro_rules! boolean_prop {
    ($name:ident, doc = $doc:expr $(, default = $default:expr)?) => {
        #[doc = $doc]
        #[derive(PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct $name(pub $crate::spec::Boolean);

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.0.serialize(serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let inner = <$crate::spec::Boolean as serde::Deserialize>::deserialize(deserializer)?;
                Ok(Self(inner))
            }
        }

        $crate::boolean_prop!(@default $name $(, $default)?);
    };

    (@default $name:ident, $default:expr) => {
        impl Default for $name {
            fn default() -> Self {
                Self($crate::spec::Boolean::Literal($default))
            }
        }
    };
    (@default $name:ident) => {};
}

/// Stamp out a newtype wrapping [`String`](crate::spec::String) with optional default.
///
/// `spec::String` already has `Literal(StringLiteral)` and handles bare strings via
/// `visit_str`/`visit_string`, so no separate `StringPropInner` wrapper is needed.
#[macro_export]
macro_rules! string_prop {
    ($name:ident, doc = $doc:expr $(, default = $default:expr)?) => {
        #[doc = $doc]
        #[derive(PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct $name(pub $crate::spec::String);

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.0.serialize(serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let inner = <$crate::spec::String as serde::Deserialize>::deserialize(deserializer)?;
                Ok(Self(inner))
            }
        }

        $crate::string_prop!(@default $name $(, $default)?);
    };

    (@default $name:ident, $default:expr) => {
        impl Default for $name {
            fn default() -> Self {
                Self($crate::spec::String::Literal($crate::spec::StringLiteral::from($default)))
            }
        }
    };
    (@default $name:ident) => {};
}

/// Stamp out a newtype wrapping [`FormattedPropInner`] with optional default.
#[macro_export]
macro_rules! formatted_prop {
    ($name:ident, doc = $doc:expr $(, default = $default:expr)?) => {
        #[doc = $doc]
        #[derive(PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct $name(pub $crate::shared_expr::FormattedPropInner);

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.0.serialize(serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let inner = <$crate::shared_expr::FormattedPropInner as serde::Deserialize>::deserialize(deserializer)?;
                Ok(Self(inner))
            }
        }

        $crate::formatted_prop!(@default $name $(, $default)?);
    };

    (@default $name:ident, $default:expr) => {
        impl Default for $name {
            fn default() -> Self {
                Self($crate::shared_expr::FormattedPropInner::Literal($default))
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

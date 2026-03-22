use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::decoder::DecodedParsedItem;
use crate::generator::formatter::to_upper_camel_case;

// ── Shared field metadata ─────────────────────────────────────────────────────

/// Metadata shared by every field variant — only what is truly common to all.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MirFieldMeta {
    /// Original spec name (e.g. `"fill-color"`) — used for `#[serde(rename="...")]`.
    pub spec_name: String,
    /// Pre-computed snake_case Rust identifier (e.g. `"fill_color"`).
    pub rust_name: String,
    pub optional: bool,
    /// Whether this field supports CSS transitions.
    pub transition: bool,
    /// Expression capability if data-driven; `None` means not data-driven.
    pub expression: Option<MirExpressionCapabilities>,
    /// Documentation string — may include range annotations for numeric types.
    pub doc: String,
    pub example: Option<Value>,
    pub units: Option<String>,
}

// ── Algebraic field type ──────────────────────────────────────────────────────

/// Every variant wraps a dedicated struct that carries only data valid for its kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MirField {
    Number(MirNumberField),
    Boolean(MirBooleanField),
    String(MirStringField),
    Color(MirColorField),
    Enum(MirEnumField),
    Array(MirArrayField),
    NumberArray(MirNumberArrayField),
    ColorArray(MirColorArrayField),
    FormattedText(MirFormattedTextField),
    ResolvedImage(MirResolvedImageField),
    Padding(MirPaddingField),
    State(MirStateField),
    ProjectionDefinition(MirProjectionDefinitionField),
    /// "bad spec" types — no type-specific data beyond the shared metadata.
    Sprite(MirFieldMeta),
    PromoteId(MirFieldMeta),
    VariableAnchorOffsetCollection(MirFieldMeta),
    /// Catch-all wildcard type (`*`) from the spec.
    Star(MirFieldMeta),
    /// Reference to a named type in `MirSpec::named_types`.
    Reference(MirReferenceField),
}

impl MirField {
    pub fn meta(&self) -> &MirFieldMeta {
        match self {
            MirField::Number(f) => &f.meta,
            MirField::Boolean(f) => &f.meta,
            MirField::String(f) => &f.meta,
            MirField::Color(f) => &f.meta,
            MirField::Enum(f) => &f.meta,
            MirField::Array(f) => &f.meta,
            MirField::NumberArray(f) => &f.meta,
            MirField::ColorArray(f) => &f.meta,
            MirField::FormattedText(f) => &f.meta,
            MirField::ResolvedImage(f) => &f.meta,
            MirField::Padding(f) => &f.meta,
            MirField::State(f) => &f.meta,
            MirField::ProjectionDefinition(f) => &f.meta,
            MirField::Sprite(m)
            | MirField::PromoteId(m)
            | MirField::VariableAnchorOffsetCollection(m)
            | MirField::Star(m) => m,
            MirField::Reference(f) => &f.meta,
        }
    }
}

// ── Per-type field structs ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirNumberField {
    pub meta: MirFieldMeta,
    /// Default value — `serde_json::Number` preserves int/float distinction.
    pub default: Option<serde_json::Number>,
    /// Minimum bound (for doc annotation only).
    pub min: Option<f64>,
    /// Maximum bound (for doc annotation only).
    pub max: Option<f64>,
    /// Periodicity (for doc annotation only).
    pub period: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirBooleanField {
    pub meta: MirFieldMeta,
    pub default: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirStringField {
    pub meta: MirFieldMeta,
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirColorField {
    pub meta: MirFieldMeta,
    /// May be a CSS string like `"#fff"` or a JSON object.
    pub default: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirColorArrayField {
    pub meta: MirFieldMeta,
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirResolvedImageField {
    pub meta: MirFieldMeta,
    pub tokens: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirFormattedTextField {
    pub meta: MirFieldMeta,
    pub tokens: bool,
    pub default: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirPaddingField {
    pub meta: MirFieldMeta,
    pub default: Vec<serde_json::Number>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirStateField {
    pub meta: MirFieldMeta,
    pub default: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirProjectionDefinitionField {
    pub meta: MirFieldMeta,
    pub default: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirReferenceField {
    pub meta: MirFieldMeta,
    /// Name of the referenced named type in `MirSpec::named_types`.
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirNumberArrayField {
    pub meta: MirFieldMeta,
    pub default: Option<serde_json::Number>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirEnumField {
    pub meta: MirFieldMeta,
    /// Default value (typically a string variant name, kept as `Value` for flexibility).
    pub default: Option<Value>,
    pub variants: MirEnum,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirArrayField {
    pub meta: MirFieldMeta,
    pub default: Option<Vec<Value>>,
    pub element: MirArrayElement,
    pub length: Option<usize>,
}

// ── Enum types ────────────────────────────────────────────────────────────────

/// Algebraic enum — never a struct with a `kind` discriminant field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MirEnum {
    Regular(MirRegularEnum),
    Version(MirVersionEnum),
    Syntax(MirSyntaxEnumMap),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirRegularEnum {
    pub variants: BTreeMap<String, MirRegularVariant>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirRegularVariant {
    pub doc: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirVersionEnum {
    pub versions: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirSyntaxEnumMap {
    pub variants: BTreeMap<String, MirSyntaxVariantDef>,
}

/// One entry in a syntax enum — mirrors `decoder::enum::DecodedSyntaxEnum` without `sdk_support`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirSyntaxVariantDef {
    pub doc: String,
    pub syntax: MirSyntax,
    pub example: Option<Value>,
    pub group: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirSyntax {
    pub overloads: Vec<MirOverload>,
    #[serde(default)]
    pub parameters: Vec<MirParameter>,
}

impl MirSyntax {
    pub fn has_variadic_overload(&self) -> bool {
        self.overloads
            .iter()
            .any(|overload| overload.is_variadic(&self.parameters))
    }

    /// Fixes where v8 JSON types are too narrow for real styles (see `from_decoder`).
    pub fn patch_expression_parameters(operator: &str, parameters: &mut [MirParameter]) {
        if operator == "format" {
            for p in parameters.iter_mut() {
                if p.name == "input_i" {
                    // v8 documents `["string","image"]` as JSON literals + `["image", …]`, but real styles
                    // (and the official example) also use string-producing operators (`upcase`, `get`, …).
                    // `spec::Any` is only the small polymorphic "value" subset — it is not "any expression".
                    p.r#type = MirParameterType::ExpressionAnyOf(vec![
                        MirParameterType::Literal(MirLiteral::String),
                        MirParameterType::Expression(Box::new(MirExpression::String)),
                        MirParameterType::Expression(Box::new(MirExpression::Image)),
                    ]);
                }
            }
        }
        if operator == "match" {
            for p in parameters.iter_mut() {
                if p.name == "input" {
                    // v8 narrows `input` to string|number, but real styles use arbitrary expressions
                    // (e.g. `["get", "building_type"]`). Model as JSON until we have a richer union.
                    p.r#type = MirParameterType::Expression(Box::new(MirExpression::Any));
                }
            }
        }
        if operator == "in" {
            for p in parameters.iter_mut() {
                if p.name == "array" {
                    // Haystack may be a literal array or another expression; `Array` syntax enum is
                    // too narrow (only `array` / `literal` / … operators).
                    p.r#type = MirParameterType::Expression(Box::new(MirExpression::Any));
                }
            }
        }
        if operator == "index-of" {
            for p in parameters.iter_mut() {
                if p.name == "array" {
                    p.r#type = MirParameterType::Expression(Box::new(MirExpression::Any));
                }
            }
        }
        if operator == "array" {
            for p in parameters.iter_mut() {
                if p.name == "type" {
                    p.r#type = MirParameterType::StringEnum(vec![
                        "string".to_string(),
                        "number".to_string(),
                        "boolean".to_string(),
                        "color".to_string(),
                    ]);
                }
            }
        }
        if operator == "length" {
            for p in parameters.iter_mut() {
                if p.name == "array_or_string" {
                    // `length` accepts string literals ("hello"), string expressions, array expressions,
                    // and general expressions like `["get","prop"]` that may return either at runtime.
                    // Including Literal::String triggers the string_literal_with_string_expr flag in
                    // the generator, which automatically adds an Any(Box<Any>) fallback arm and
                    // #[serde(untagged)] — the Any syntax enum only accepts expression arrays, so bare
                    // scalars (e.g. the number 2) are still correctly rejected.
                    p.r#type = MirParameterType::ExpressionAnyOf(vec![
                        MirParameterType::Literal(MirLiteral::String),
                        MirParameterType::Expression(Box::new(MirExpression::Array {
                            r#type: None,
                            length: None,
                        })),
                        MirParameterType::Expression(Box::new(MirExpression::String)),
                    ]);
                }
            }
        }
    }

    /// Lower decoded reference syntax into MIR, applying fixes where the JSON schema is too narrow.
    pub fn from_decoder(operator: &str, syntax: crate::decoder::r#enum::Syntax) -> Self {
        let mut mir = MirSyntax::from(syntax);
        Self::patch_expression_parameters(operator, &mut mir.parameters);
        Self::patch_overloads(operator, &mut mir.overloads);
        mir
    }

    /// Removes overloads that are JSON-indistinguishable from another overload in the same
    /// operator, which would make `#[serde(untagged)]` deserialization non-deterministic.
    fn patch_overloads(operator: &str, overloads: &mut Vec<MirOverload>) {
        if operator == "in" {
            // Both `["in", item, array]` and `["in", substring, string]` serialize to the same
            // two-element JSON array. Keeping both produces an untagged enum where `Item` always
            // wins on deserialization, breaking round-trips for the `Substring` variant. Drop the
            // second overload and let `Item(ExprOrLiteral, ExprOrLiteral)` cover all uses.
            overloads.retain(|o| o.parameters.first().is_none_or(|p| p != "substring"));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirOverload {
    pub parameters: Vec<String>,
    pub output_type: MirParameterType,
}

impl MirOverload {
    pub fn position_of_variadic_separator(&self) -> usize {
        self.parameters
            .iter()
            .position(|p| p == "...")
            .expect("... parameter must be in a variadic list")
    }

    pub fn is_variadic(&self, params: &[MirParameter]) -> bool {
        self.parameters.iter().any(|p| p == "...")
            || !self.parameters.iter().all(|overloaded_param| {
                params.iter().any(|actual_param| {
                    actual_param.matches_overload_parameter_name(overloaded_param)
                })
            })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirParameter {
    pub name: String,
    pub r#type: MirParameterType,
    pub doc: Option<String>,
}

impl MirParameter {
    pub fn matches_overload_parameter_name(&self, overloaded_name: &str) -> bool {
        if let Some(maybe_template) = self.name.strip_suffix("_i") {
            for suffix in &["_1", "_2", "_1?", "_2?"] {
                if let Some(param) = overloaded_name.strip_suffix(suffix) {
                    return maybe_template == param;
                }
            }
            self.name == overloaded_name
        } else if let Some(opt) = overloaded_name.strip_suffix('?') {
            self.name == opt
        } else {
            self.name == overloaded_name
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MirParameterType {
    Literal(MirLiteral),
    LiteralAnyOf(Vec<MirLiteral>),
    Expression(Box<MirExpression>),
    ExpressionAnyOf(Vec<MirParameterType>),
    Object(BTreeMap<String, DecodedParsedItem>),
    Reference(String),
    /// A closed set of valid plain string values (e.g. `"string"`, `"number"`, `"boolean"`).
    /// Generates a unit-variant enum with `#[serde(rename = "...")]` on each arm.
    StringEnum(Vec<String>),
}

impl MirParameterType {
    pub fn to_upper_camel_case(&self) -> String {
        match self {
            MirParameterType::Literal(l) => l.to_upper_camel_case().to_string(),
            MirParameterType::LiteralAnyOf(ls) => ls
                .iter()
                .map(|l| l.to_upper_camel_case())
                .collect::<Vec<_>>()
                .join("Or"),
            MirParameterType::Expression(e) => e.to_upper_camel_case().to_string(),
            MirParameterType::ExpressionAnyOf(es) => es
                .iter()
                .map(|e| e.to_upper_camel_case())
                .collect::<Vec<_>>()
                .join("Or"),
            MirParameterType::Object(_) => "Object".to_string(),
            MirParameterType::Reference(r) if r == "T" => "Any".to_string(),
            MirParameterType::Reference(r) => to_upper_camel_case(r),
            MirParameterType::StringEnum(values) => values
                .iter()
                .map(to_upper_camel_case)
                .collect::<Vec<_>>()
                .join("Or"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum MirLiteral {
    Number,
    String,
    GeoJSONObject,
    JSONObject,
    JSONArray,
}

impl MirLiteral {
    pub fn to_upper_camel_case(&self) -> &'static str {
        match self {
            MirLiteral::Number => "NumberLiteral",
            MirLiteral::String => "StringLiteral",
            MirLiteral::GeoJSONObject => "GeoJSONObjectLiteral",
            MirLiteral::JSONObject => "JSONObjectLiteral",
            MirLiteral::JSONArray => "JSONArrayLiteral",
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum MirExpression {
    Any,
    Boolean,
    Number,
    String,
    Collator,
    Formatted,
    Image,
    Object,
    Color,
    Array {
        r#type: Option<MirParameterType>,
        length: Option<usize>,
    },
}

impl MirExpression {
    pub fn to_upper_camel_case(&self) -> String {
        match self {
            MirExpression::Any => "Any".to_string(),
            MirExpression::Boolean => "Boolean".to_string(),
            MirExpression::Number => "Number".to_string(),
            MirExpression::String => "String".to_string(),
            MirExpression::Collator => "Collator".to_string(),
            MirExpression::Formatted => "Formatted".to_string(),
            MirExpression::Image => "Image".to_string(),
            MirExpression::Object => "Object".to_string(),
            MirExpression::Color => "Color".to_string(),
            MirExpression::Array { r#type, length } => {
                if let Some(length) = length {
                    if let Some(r#type) = r#type {
                        format!(
                            "ArrayLess{}LengthGreater{}",
                            r#type.to_upper_camel_case(),
                            to_upper_camel_case(length.to_string())
                        )
                    } else {
                        format!("ArrayLength{}", to_upper_camel_case(length.to_string()))
                    }
                } else if let Some(r#type) = r#type {
                    // `array<T>` in the reference uses the type variable `T` (e.g. `in`, `index-of`).
                    if matches!(&r#type, MirParameterType::Reference(r) if r == "T") {
                        "Array".to_string()
                    } else {
                        format!("ArrayOf{}", r#type.to_upper_camel_case())
                    }
                } else {
                    "Array".to_string()
                }
            }
        }
    }
}

impl From<crate::decoder::r#enum::Literal> for MirLiteral {
    fn from(value: crate::decoder::r#enum::Literal) -> Self {
        match value {
            crate::decoder::r#enum::Literal::Number => MirLiteral::Number,
            crate::decoder::r#enum::Literal::String => MirLiteral::String,
            crate::decoder::r#enum::Literal::GeoJSONObject => MirLiteral::GeoJSONObject,
            crate::decoder::r#enum::Literal::JSONObject => MirLiteral::JSONObject,
            crate::decoder::r#enum::Literal::JSONArray => MirLiteral::JSONArray,
        }
    }
}

impl From<crate::decoder::r#enum::Expression> for MirExpression {
    fn from(value: crate::decoder::r#enum::Expression) -> Self {
        match value {
            crate::decoder::r#enum::Expression::Any => MirExpression::Any,
            crate::decoder::r#enum::Expression::Boolean => MirExpression::Boolean,
            crate::decoder::r#enum::Expression::Number => MirExpression::Number,
            crate::decoder::r#enum::Expression::String => MirExpression::String,
            crate::decoder::r#enum::Expression::Collator => MirExpression::Collator,
            crate::decoder::r#enum::Expression::Formatted => MirExpression::Formatted,
            crate::decoder::r#enum::Expression::Image => MirExpression::Image,
            crate::decoder::r#enum::Expression::Object => MirExpression::Object,
            crate::decoder::r#enum::Expression::Color => MirExpression::Color,
            crate::decoder::r#enum::Expression::Array { r#type, length } => MirExpression::Array {
                r#type: r#type.map(MirParameterType::from),
                length,
            },
        }
    }
}

impl From<crate::decoder::r#enum::ParameterType> for MirParameterType {
    fn from(value: crate::decoder::r#enum::ParameterType) -> Self {
        match value {
            crate::decoder::r#enum::ParameterType::Literal(l) => {
                MirParameterType::Literal(l.into())
            }
            crate::decoder::r#enum::ParameterType::LiteralAnyOf(ls) => {
                MirParameterType::LiteralAnyOf(ls.into_iter().map(MirLiteral::from).collect())
            }
            crate::decoder::r#enum::ParameterType::Expression(e) => {
                MirParameterType::Expression(Box::new((*e).into()))
            }
            crate::decoder::r#enum::ParameterType::ExpressionAnyOf(pts) => {
                MirParameterType::ExpressionAnyOf(
                    pts.into_iter().map(MirParameterType::from).collect(),
                )
            }
            crate::decoder::r#enum::ParameterType::Object(obj) => MirParameterType::Object(obj),
            crate::decoder::r#enum::ParameterType::Reference(r) => MirParameterType::Reference(r),
        }
    }
}

impl From<crate::decoder::r#enum::Parameter> for MirParameter {
    fn from(value: crate::decoder::r#enum::Parameter) -> Self {
        MirParameter {
            name: value.name,
            r#type: value.r#type.into(),
            doc: value.doc,
        }
    }
}

impl From<crate::decoder::r#enum::DecodedOverload> for MirOverload {
    fn from(value: crate::decoder::r#enum::DecodedOverload) -> Self {
        MirOverload {
            parameters: value.parameters,
            output_type: value.output_type.into(),
        }
    }
}

impl From<crate::decoder::r#enum::Syntax> for MirSyntax {
    fn from(value: crate::decoder::r#enum::Syntax) -> Self {
        MirSyntax {
            overloads: value.overloads.into_iter().map(MirOverload::from).collect(),
            parameters: value
                .parameters
                .into_iter()
                .map(MirParameter::from)
                .collect(),
        }
    }
}

// ── Array element types ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MirArrayElement {
    String,
    Number { min: Option<f64>, max: Option<f64> },
    Boolean,
    Color,
    Enum(MirRegularEnum),
    Star,
    Layer,
    FunctionStop,
    FontFaces,
    ExpressionName,
    InterpolationName,
    Either(Vec<MirArrayElement>),
    Complex(Box<MirField>),
}

// ── Expression capabilities ───────────────────────────────────────────────────

/// Tracks expression-capability of a field. MIR-only — not emitted to `spec.rs`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirExpressionCapabilities {
    pub interpolated: bool,
    pub zoom: bool,
    pub feature: bool,
    pub global_state: bool,
}

// ── MirType / MirArrayElementType ───────────────────────────────────────
// These simpler enums are used by the layer and root preprocessing passes.

/// Clean representation of a field's type, abstracting over the decoder's `DecodedPrimitiveType`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MirType {
    Number {
        min: Option<f64>,
        max: Option<f64>,
    },
    String,
    Boolean,
    Color,
    Enum {
        values: Vec<String>,
    },
    Array {
        element: MirArrayElementType,
        length: Option<usize>,
    },
    Padding,
    Formatted {
        tokens: bool,
    },
    ResolvedImage {
        tokens: bool,
    },
    NumberArray {
        min: Option<f64>,
        max: Option<f64>,
    },
    ColorArray,
    State,
    /// Catch-all open object type (from spec's `*` type)
    AnyObject,
    Sprite,
    PromoteId,
    ProjectionDefinition,
    VariableAnchorOffsetCollection,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MirArrayElementType {
    String,
    Number,
    Color,
    Enum(Vec<String>),
    Layer,
}

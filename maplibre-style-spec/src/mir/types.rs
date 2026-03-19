use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::decoder::r#enum::Syntax;

// ── Shared field metadata ─────────────────────────────────────────────────────

/// Metadata shared by every field variant — only what is truly common to all.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldMeta {
    /// Original spec name (e.g. `"fill-color"`) — used for `#[serde(rename="...")]`.
    pub spec_name: String,
    /// Pre-computed snake_case Rust identifier (e.g. `"fill_color"`).
    pub rust_name: String,
    pub optional: bool,
    /// Whether this field supports CSS transitions.
    pub transition: bool,
    /// Expression capability if data-driven; `None` means not data-driven.
    pub expression: Option<ExpressionCapabilities>,
    /// Documentation string — may include range annotations for numeric types.
    pub doc: String,
    pub example: Option<Value>,
    pub units: Option<String>,
}

impl Default for FieldMeta {
    fn default() -> Self {
        Self {
            spec_name: String::new(),
            rust_name: String::new(),
            optional: false,
            transition: false,
            expression: None,
            doc: String::new(),
            example: None,
            units: None,
        }
    }
}

// ── Algebraic field type ──────────────────────────────────────────────────────

/// Every variant wraps a dedicated struct that carries only data valid for its kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MirField {
    Number(NumberField),
    Boolean(BooleanField),
    String(StringField),
    Color(ColorField),
    Enum(EnumField),
    Array(ArrayField),
    NumberArray(NumberArrayField),
    ColorArray(ColorArrayField),
    FormattedText(FormattedTextField),
    ResolvedImage(ResolvedImageField),
    Padding(PaddingField),
    State(StateField),
    ProjectionDefinition(ProjectionDefinitionField),
    /// "bad spec" types — no type-specific data beyond the shared metadata.
    Sprite(FieldMeta),
    PromoteId(FieldMeta),
    VariableAnchorOffsetCollection(FieldMeta),
    /// Catch-all wildcard type (`*`) from the spec.
    Star(FieldMeta),
    /// Reference to a named type in `IntermediateSpec::named_types`.
    Reference(ReferenceField),
}

impl MirField {
    pub fn meta(&self) -> &FieldMeta {
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
pub struct NumberField {
    pub meta: FieldMeta,
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
pub struct BooleanField {
    pub meta: FieldMeta,
    pub default: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StringField {
    pub meta: FieldMeta,
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorField {
    pub meta: FieldMeta,
    /// May be a CSS string like `"#fff"` or a JSON object.
    pub default: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorArrayField {
    pub meta: FieldMeta,
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolvedImageField {
    pub meta: FieldMeta,
    pub tokens: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormattedTextField {
    pub meta: FieldMeta,
    pub tokens: bool,
    pub default: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaddingField {
    pub meta: FieldMeta,
    pub default: Vec<serde_json::Number>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateField {
    pub meta: FieldMeta,
    pub default: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectionDefinitionField {
    pub meta: FieldMeta,
    pub default: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReferenceField {
    pub meta: FieldMeta,
    /// Name of the referenced named type in `IntermediateSpec::named_types`.
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumberArrayField {
    pub meta: FieldMeta,
    pub default: Option<serde_json::Number>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumField {
    pub meta: FieldMeta,
    /// Default value (typically a string variant name, kept as `Value` for flexibility).
    pub default: Option<Value>,
    pub variants: MirEnum,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArrayField {
    pub meta: FieldMeta,
    pub default: Option<Vec<Value>>,
    pub element: ArrayElement,
    pub length: Option<usize>,
}

// ── Enum types ────────────────────────────────────────────────────────────────

/// Algebraic enum — never a struct with a `kind` discriminant field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MirEnum {
    Regular(RegularEnum),
    Version(VersionEnum),
    Syntax(SyntaxEnumMap),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegularEnum {
    pub variants: BTreeMap<String, RegularVariant>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegularVariant {
    pub doc: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionEnum {
    pub versions: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyntaxEnumMap {
    pub variants: BTreeMap<String, SyntaxVariantDef>,
}

/// One entry in a syntax enum — mirrors `decoder::enum::SyntaxEnum` without `sdk_support`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyntaxVariantDef {
    pub doc: String,
    /// Already well-typed from the decoder; kept as-is.
    pub syntax: Syntax,
    pub example: Option<Value>,
    pub group: Option<String>,
}

// ── Array element types ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArrayElement {
    String,
    Number { min: Option<f64>, max: Option<f64> },
    Boolean,
    Color,
    Enum(RegularEnum),
    Star,
    Layer,
    FunctionStop,
    FontFaces,
    ExpressionName,
    InterpolationName,
    Either(Vec<ArrayElement>),
    Complex(Box<MirField>),
}

// ── Expression capabilities ───────────────────────────────────────────────────

/// Tracks expression-capability of a field. MIR-only — not emitted to `spec.rs`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionCapabilities {
    pub interpolated: bool,
    pub zoom: bool,
    pub feature: bool,
    pub global_state: bool,
}

// ── IntermediateType / ArrayElementType ───────────────────────────────────────
// These simpler enums are used by the layer and root preprocessing passes.

/// Clean representation of a field's type, abstracting over the decoder's `PrimitiveType`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IntermediateType {
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
        element: ArrayElementType,
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
pub enum ArrayElementType {
    String,
    Number,
    Color,
    Enum(Vec<String>),
    Layer,
}

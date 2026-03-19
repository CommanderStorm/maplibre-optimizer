use crate::decoder::array::{ArrayValue, SimpleArrayValue};
use crate::decoder::r#enum::EnumValues;
use crate::decoder::{Fields, ParsedItem, PrimitiveType};
use crate::generator::formatter::to_snake_case;
use crate::mir::types::{
    ArrayElement, ArrayField, BooleanField, ColorArrayField, ColorField, EnumField,
    ExpressionCapabilities, FieldMeta, FormattedTextField, MirEnum, MirField, NumberArrayField,
    NumberField, PaddingField, ProjectionDefinitionField, ReferenceField, RegularEnum,
    RegularVariant, ResolvedImageField, StateField, StringField, SyntaxEnumMap, SyntaxVariantDef,
    VersionEnum,
};

/// The single conversion point: `ParsedItem` (decoder) → `MirField` (MIR).
/// Pre-computes `FieldMeta::rust_name` via `to_snake_case`.
pub fn lower_parsed_item(spec_name: &str, item: ParsedItem) -> MirField {
    let optional = item.optional();
    match item {
        ParsedItem::Primitive(p) => lower_primitive(spec_name, p, optional),
        ParsedItem::Reference { references, common } => MirField::Reference(ReferenceField {
            meta: make_meta(spec_name, &common, optional),
            target: references,
        }),
    }
}

fn lower_primitive(spec_name: &str, p: PrimitiveType, optional: bool) -> MirField {
    match p {
        PrimitiveType::Number {
            common,
            default,
            maximum,
            minimum,
            period,
        } => {
            let min = minimum.as_ref().and_then(|n| n.as_f64());
            let max = maximum.as_ref().and_then(|n| n.as_f64());
            let per = period.as_ref().and_then(|n| n.as_f64());
            let mut meta = make_meta(spec_name, &common, optional);
            meta.doc = doc_with_range(&meta.doc, max, min, per);
            MirField::Number(NumberField {
                meta,
                default,
                min,
                max,
                period: per,
            })
        }

        PrimitiveType::Boolean { common, default } => MirField::Boolean(BooleanField {
            meta: make_meta(spec_name, &common, optional),
            default,
        }),

        PrimitiveType::String { common, default } => MirField::String(StringField {
            meta: make_meta(spec_name, &common, optional),
            default,
        }),

        PrimitiveType::Color { common, default } => MirField::Color(ColorField {
            meta: make_meta(spec_name, &common, optional),
            default,
        }),

        PrimitiveType::Enum {
            common,
            default,
            values,
        } => MirField::Enum(EnumField {
            meta: make_meta(spec_name, &common, optional),
            default,
            variants: lower_enum_values(values),
        }),

        PrimitiveType::Array {
            common,
            default,
            value,
            values,
            minimum,
            maximum,
            length,
        } => {
            let min = minimum.as_ref().and_then(|n| n.as_f64());
            let max = maximum.as_ref().and_then(|n| n.as_f64());
            let element = lower_array_element(value, values);
            let mut meta = make_meta(spec_name, &common, optional);
            // Array docs may also carry range info (for numeric element bounds)
            meta.doc = doc_with_range(&meta.doc, max, min, None);
            MirField::Array(ArrayField {
                meta,
                default,
                element,
                length,
            })
        }

        PrimitiveType::NumberArray {
            common,
            default,
            minimum,
            maximum,
        } => {
            let min = minimum.as_ref().and_then(|n| n.as_f64());
            let max = maximum.as_ref().and_then(|n| n.as_f64());
            let mut meta = make_meta(spec_name, &common, optional);
            meta.doc = doc_with_range(&meta.doc, max, min, None);
            MirField::NumberArray(NumberArrayField {
                meta,
                default,
                min,
                max,
            })
        }

        PrimitiveType::ColorArray { common, default } => MirField::ColorArray(ColorArrayField {
            meta: make_meta(spec_name, &common, optional),
            default,
        }),

        PrimitiveType::Formatted {
            common,
            tokens,
            default,
        } => MirField::FormattedText(FormattedTextField {
            meta: make_meta(spec_name, &common, optional),
            tokens,
            default,
        }),

        PrimitiveType::ResolvedImage { common, tokens } => {
            MirField::ResolvedImage(ResolvedImageField {
                meta: make_meta(spec_name, &common, optional),
                tokens,
            })
        }

        PrimitiveType::Padding { common, default } => MirField::Padding(PaddingField {
            meta: make_meta(spec_name, &common, optional),
            default,
        }),

        PrimitiveType::State { common, default } => MirField::State(StateField {
            meta: make_meta(spec_name, &common, optional),
            default,
        }),

        PrimitiveType::ProjectionDefinition { common, default } => {
            MirField::ProjectionDefinition(ProjectionDefinitionField {
                meta: make_meta(spec_name, &common, optional),
                default,
            })
        }

        PrimitiveType::Sprite(common) => MirField::Sprite(make_meta(spec_name, &common, optional)),

        PrimitiveType::PromoteId(common) => {
            MirField::PromoteId(make_meta(spec_name, &common, optional))
        }

        PrimitiveType::VariableAnchorOffsetCollection(common) => {
            MirField::VariableAnchorOffsetCollection(make_meta(spec_name, &common, optional))
        }

        PrimitiveType::Star(common) => MirField::Star(make_meta(spec_name, &common, optional)),

        // Meta-type: not used for codegen; caller should have filtered this out.
        PrimitiveType::PropertyType(_) => {
            panic!("PropertyType is a meta-type and should be filtered before lowering")
        }
    }
}

// ── Enum value lowering ───────────────────────────────────────────────────────

pub fn lower_enum_values(values: EnumValues) -> MirEnum {
    match values {
        EnumValues::Version(numbers) => MirEnum::Version(VersionEnum {
            versions: numbers
                .iter()
                .map(|n| n.as_u64().expect("version number must be a u64") as u32)
                .collect(),
        }),
        EnumValues::Enum(map) => MirEnum::Regular(RegularEnum {
            variants: map
                .into_iter()
                .map(|(k, v)| (k, RegularVariant { doc: v.doc }))
                .collect(),
        }),
        EnumValues::SyntaxEnum(map) => MirEnum::Syntax(SyntaxEnumMap {
            variants: map
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        SyntaxVariantDef {
                            doc: v.doc,
                            syntax: v.syntax.into(),
                            example: v.example,
                            group: v.group,
                        },
                    )
                })
                .collect(),
        }),
    }
}

// ── Array element lowering ────────────────────────────────────────────────────

fn lower_array_element(
    value: ArrayValue,
    enum_values: Option<crate::decoder::r#enum::EnumValues>,
) -> ArrayElement {
    match value {
        ArrayValue::Simple(s) => lower_simple_array_value(s, enum_values),
        ArrayValue::Either(options) => ArrayElement::Either(
            options
                .into_iter()
                .map(|v| lower_array_element(v, None))
                .collect(),
        ),
        ArrayValue::Complex(item) => {
            ArrayElement::Complex(Box::new(lower_parsed_item("element", *item)))
        }
    }
}

fn lower_simple_array_value(
    value: SimpleArrayValue,
    enum_values: Option<crate::decoder::r#enum::EnumValues>,
) -> ArrayElement {
    match value {
        SimpleArrayValue::String => ArrayElement::String,
        SimpleArrayValue::Number => ArrayElement::Number {
            min: None,
            max: None,
        },
        SimpleArrayValue::Star => ArrayElement::Star,
        SimpleArrayValue::FunctionStop => ArrayElement::FunctionStop,
        SimpleArrayValue::Layer => ArrayElement::Layer,
        SimpleArrayValue::Enum => {
            let values = enum_values.expect("Enum array element requires EnumValues");
            if let MirEnum::Regular(r) = lower_enum_values(values) {
                ArrayElement::Enum(r)
            } else {
                panic!("array enum element must be a regular enum")
            }
        }
        SimpleArrayValue::Color => ArrayElement::Color,
        SimpleArrayValue::FontFaces => ArrayElement::FontFaces,
        SimpleArrayValue::ExpressionName => ArrayElement::ExpressionName,
        SimpleArrayValue::InterpolationName => ArrayElement::InterpolationName,
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_meta(spec_name: &str, common: &Fields, optional: bool) -> FieldMeta {
    FieldMeta {
        spec_name: spec_name.to_string(),
        rust_name: to_snake_case(spec_name),
        optional,
        transition: common.transition.unwrap_or(false),
        expression: common.expression.as_ref().map(lower_expression),
        doc: common.doc.clone(),
        example: common.example.clone(),
        units: common.units.clone(),
    }
}

fn lower_expression(e: &crate::decoder::Expression) -> ExpressionCapabilities {
    ExpressionCapabilities {
        interpolated: e.interpolated,
        zoom: e.parameters.iter().any(|p| p == "zoom"),
        feature: e.parameters.iter().any(|p| p == "feature"),
        global_state: e.parameters.iter().any(|p| p == "global-state"),
    }
}

/// Compute a doc string with optional range annotation — matches the existing
/// `Fields::doc_with_range` format exactly.
pub fn doc_with_range(
    doc: &str,
    max: Option<f64>,
    min: Option<f64>,
    period: Option<f64>,
) -> String {
    if max.is_none() && min.is_none() && period.is_none() {
        return doc.to_string();
    }
    let mut result = doc.to_string();
    if !result.is_empty() {
        result.push_str("\n\n");
    }
    result.push_str("Range: ");
    if min.is_some() || max.is_some() {
        if let Some(min) = min {
            result.push_str(&display_f64(min));
        }
        result.push_str("..");
        if let Some(max) = max {
            result.push('=');
            result.push_str(&display_f64(max));
        }
        if period.is_some() {
            result.push(' ');
        }
    }
    if let Some(period) = period {
        result.push_str(&format!("every {}\n", display_f64(period)));
    }
    result
}

/// Display an f64 as an integer string when it has no fractional part,
/// matching `serde_json::Number::to_string()` for integer-valued numbers.
fn display_f64(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

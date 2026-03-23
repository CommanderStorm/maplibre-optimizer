use crate::decoder::array::{DecodedArrayValue, DecodedSimpleArrayValue};
use crate::decoder::r#enum::DecodedEnumValues;
use crate::decoder::{DecodedFields, DecodedParsedItem, DecodedPrimitiveType};
use crate::generator::formatter::to_snake_case;
use crate::mir::types::{
    MirArrayElement, MirArrayField, MirBooleanField, MirColorArrayField, MirColorField, MirEnum,
    MirEnumField, MirExpressionCapabilities, MirField, MirFieldMeta, MirFormattedTextField,
    MirNumberArrayField, MirNumberField, MirPaddingField, MirProjectionDefinitionField,
    MirReferenceField, MirRegularEnum, MirRegularVariant, MirResolvedImageField, MirStateField,
    MirStringField, MirSyntax, MirSyntaxEnumMap, MirSyntaxVariantDef, MirVersionEnum,
};

/// The single conversion point: `DecodedParsedItem` (decoder) → `MirField` (MIR).
/// Pre-computes `MirFieldMeta::rust_name` via `to_snake_case`.
pub fn lower_parsed_item(spec_name: &str, item: DecodedParsedItem) -> MirField {
    let optional = item.optional();
    match item {
        DecodedParsedItem::Primitive(p) => lower_primitive(spec_name, p, optional),
        DecodedParsedItem::Reference { references, common } => {
            // `$root["font-faces"]` references `type: "fontFaces"` but there is no named `fontFaces`
            // group in `fields` — lower as the concrete map-of-font-faces array shape (see v8.json).
            if references == "fontFaces" {
                return MirField::Array(MirArrayField {
                    meta: make_meta(spec_name, &common, optional),
                    default: None,
                    element: MirArrayElement::FontFaces,
                    length: None,
                });
            }
            MirField::Reference(MirReferenceField {
                meta: make_meta(spec_name, &common, optional),
                target: references,
            })
        }
    }
}

fn lower_primitive(spec_name: &str, p: DecodedPrimitiveType, optional: bool) -> MirField {
    match p {
        DecodedPrimitiveType::Number {
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
            MirField::Number(MirNumberField {
                meta,
                default,
                min,
                max,
                period: per,
            })
        }

        DecodedPrimitiveType::Boolean { common, default } => MirField::Boolean(MirBooleanField {
            meta: make_meta(spec_name, &common, optional),
            default,
        }),

        DecodedPrimitiveType::String { common, default } => MirField::String(MirStringField {
            meta: make_meta(spec_name, &common, optional),
            default,
        }),

        DecodedPrimitiveType::Color { common, default } => MirField::Color(MirColorField {
            meta: make_meta(spec_name, &common, optional),
            default,
        }),

        DecodedPrimitiveType::Enum {
            common,
            default,
            values,
        } => MirField::Enum(MirEnumField {
            meta: make_meta(spec_name, &common, optional),
            default,
            variants: lower_enum_values(values),
        }),

        DecodedPrimitiveType::Array {
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
            MirField::Array(MirArrayField {
                meta,
                default,
                element,
                length,
            })
        }

        DecodedPrimitiveType::NumberArray {
            common,
            default,
            minimum,
            maximum,
        } => {
            let min = minimum.as_ref().and_then(|n| n.as_f64());
            let max = maximum.as_ref().and_then(|n| n.as_f64());
            let mut meta = make_meta(spec_name, &common, optional);
            meta.doc = doc_with_range(&meta.doc, max, min, None);
            MirField::NumberArray(MirNumberArrayField {
                meta,
                default,
                min,
                max,
            })
        }

        DecodedPrimitiveType::ColorArray { common, default } => {
            MirField::ColorArray(MirColorArrayField {
                meta: make_meta(spec_name, &common, optional),
                default,
            })
        }

        DecodedPrimitiveType::Formatted {
            common,
            tokens,
            default,
        } => MirField::FormattedText(MirFormattedTextField {
            meta: make_meta(spec_name, &common, optional),
            tokens,
            default,
        }),

        DecodedPrimitiveType::ResolvedImage { common, tokens } => {
            MirField::ResolvedImage(MirResolvedImageField {
                meta: make_meta(spec_name, &common, optional),
                tokens,
            })
        }

        DecodedPrimitiveType::Padding { common, default } => MirField::Padding(MirPaddingField {
            meta: make_meta(spec_name, &common, optional),
            default,
        }),

        DecodedPrimitiveType::State { common, default } => MirField::State(MirStateField {
            meta: make_meta(spec_name, &common, optional),
            default,
        }),

        DecodedPrimitiveType::ProjectionDefinition { common, default } => {
            MirField::ProjectionDefinition(MirProjectionDefinitionField {
                meta: make_meta(spec_name, &common, optional),
                default,
            })
        }

        DecodedPrimitiveType::Sprite(common) => {
            MirField::Sprite(make_meta(spec_name, &common, optional))
        }

        DecodedPrimitiveType::PromoteId(common) => {
            MirField::PromoteId(make_meta(spec_name, &common, optional))
        }

        DecodedPrimitiveType::VariableAnchorOffsetCollection(common) => {
            MirField::VariableAnchorOffsetCollection(make_meta(spec_name, &common, optional))
        }

        DecodedPrimitiveType::Star(common) => {
            MirField::Star(make_meta(spec_name, &common, optional))
        }

        // Meta-type: not used for codegen; caller should have filtered this out.
        DecodedPrimitiveType::PropertyType(_) => {
            panic!("DecodedPropertyType is a meta-type and should be filtered before lowering")
        }
    }
}

// ── Enum value lowering ───────────────────────────────────────────────────────

pub fn lower_enum_values(values: DecodedEnumValues) -> MirEnum {
    match values {
        DecodedEnumValues::Version(numbers) => MirEnum::Version(MirVersionEnum {
            versions: numbers
                .iter()
                .map(|n| n.as_u64().expect("version number must be a u64") as u32)
                .collect(),
        }),
        DecodedEnumValues::Enum(map) => MirEnum::Regular(MirRegularEnum {
            variants: map
                .into_iter()
                .map(|(k, v)| (k, MirRegularVariant { doc: v.doc }))
                .collect(),
        }),
        DecodedEnumValues::SyntaxEnum(map) => MirEnum::Syntax(MirSyntaxEnumMap {
            variants: map
                .into_iter()
                .map(|(k, v)| {
                    let syntax = MirSyntax::from_decoder(&k, v.syntax);
                    (
                        k,
                        MirSyntaxVariantDef {
                            doc: v.doc,
                            syntax,
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
    value: DecodedArrayValue,
    enum_values: Option<crate::decoder::r#enum::DecodedEnumValues>,
) -> MirArrayElement {
    match value {
        DecodedArrayValue::Simple(s) => lower_simple_array_value(s, enum_values),
        DecodedArrayValue::Either(options) => MirArrayElement::Either(
            options
                .into_iter()
                .map(|v| lower_array_element(v, None))
                .collect(),
        ),
        DecodedArrayValue::Complex(item) => {
            MirArrayElement::Complex(Box::new(lower_parsed_item("element", *item)))
        }
    }
}

fn lower_simple_array_value(
    value: DecodedSimpleArrayValue,
    enum_values: Option<crate::decoder::r#enum::DecodedEnumValues>,
) -> MirArrayElement {
    match value {
        DecodedSimpleArrayValue::String => MirArrayElement::String,
        DecodedSimpleArrayValue::Number => MirArrayElement::Number {
            min: None,
            max: None,
        },
        DecodedSimpleArrayValue::Star => MirArrayElement::Star,
        DecodedSimpleArrayValue::FunctionStop => MirArrayElement::FunctionStop,
        DecodedSimpleArrayValue::Layer => MirArrayElement::Layer,
        DecodedSimpleArrayValue::Enum => {
            let values = enum_values.expect("Enum array element requires DecodedEnumValues");
            if let MirEnum::Regular(r) = lower_enum_values(values) {
                MirArrayElement::Enum(r)
            } else {
                panic!("array enum element must be a regular enum")
            }
        }
        DecodedSimpleArrayValue::Color => MirArrayElement::Color,
        DecodedSimpleArrayValue::FontFaces => MirArrayElement::FontFaces,
        DecodedSimpleArrayValue::ExpressionName => MirArrayElement::ExpressionName,
        DecodedSimpleArrayValue::InterpolationName => MirArrayElement::InterpolationName,
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_meta(spec_name: &str, common: &DecodedFields, optional: bool) -> MirFieldMeta {
    MirFieldMeta {
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

fn lower_expression(e: &crate::decoder::Expression) -> MirExpressionCapabilities {
    MirExpressionCapabilities {
        interpolated: e.interpolated,
        zoom: e.parameters.iter().any(|p| p == "zoom"),
        feature: e.parameters.iter().any(|p| p == "feature"),
        global_state: e.parameters.iter().any(|p| p == "global-state"),
    }
}

/// Compute a doc string with optional range annotation — matches the existing
/// `DecodedFields::doc_with_range` format exactly.
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

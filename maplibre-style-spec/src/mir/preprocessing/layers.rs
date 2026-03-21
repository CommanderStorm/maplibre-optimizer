use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::decoder;
use crate::decoder::array::{ArrayValue, SimpleArrayValue};
use crate::decoder::r#enum::EnumValues;
use crate::decoder::{ParsedItem, PrimitiveType};
use crate::mir::layers::{IntermediateLayerField, IntermediateLayerType, IntermediateLayers};
use crate::mir::preprocessing::pop_one_of_as_group;
use crate::mir::types::{ArrayElementType, ExpressionCapabilities, IntermediateType};

fn parsed_items_to_layer_fields(
    map: BTreeMap<String, ParsedItem>,
) -> BTreeMap<String, IntermediateLayerField> {
    map.into_iter()
        .filter(|(k, _)| k != "*")
        .map(|(k, v)| (k, parsed_item_to_layer_field(v)))
        .collect()
}

fn lower_expression_caps(e: &crate::decoder::Expression) -> ExpressionCapabilities {
    ExpressionCapabilities {
        interpolated: e.interpolated,
        zoom: e.parameters.iter().any(|p| p == "zoom"),
        feature: e.parameters.iter().any(|p| p == "feature"),
        global_state: e.parameters.iter().any(|p| p == "global-state"),
    }
}

fn parsed_item_to_layer_field(item: ParsedItem) -> IntermediateLayerField {
    match item {
        ParsedItem::Reference { references, common } => IntermediateLayerField {
            r#type: reference_to_intermediate_type(&references),
            default: None,
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        ParsedItem::Primitive(p) => primitive_to_layer_field(p),
    }
}

fn primitive_to_layer_field(p: PrimitiveType) -> IntermediateLayerField {
    match p {
        PrimitiveType::Number {
            common,
            default,
            maximum,
            minimum,
            period: _,
        } => IntermediateLayerField {
            r#type: IntermediateType::Number {
                min: minimum.as_ref().and_then(|n| n.as_f64()),
                max: maximum.as_ref().and_then(|n| n.as_f64()),
            },
            default: default.map(Value::Number),
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        PrimitiveType::Boolean { common, default } => IntermediateLayerField {
            r#type: IntermediateType::Boolean,
            default: default.map(Value::Bool),
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        PrimitiveType::String { common, default } => IntermediateLayerField {
            r#type: IntermediateType::String,
            default: default.map(Value::String),
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        PrimitiveType::Color { common, default } => IntermediateLayerField {
            r#type: IntermediateType::Color,
            default,
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        PrimitiveType::Enum {
            common,
            default,
            values,
        } => IntermediateLayerField {
            r#type: IntermediateType::Enum {
                values: enum_values_to_strings(&values),
            },
            default,
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        PrimitiveType::Array {
            common,
            default,
            value,
            values,
            minimum: _,
            maximum: _,
            length,
        } => {
            let element = array_value_to_element_type(value, values);
            IntermediateLayerField {
                r#type: IntermediateType::Array { element, length },
                default: default.map(Value::Array),
                doc: common.doc,
                required: common.required.unwrap_or(false),
                expression: common.expression.as_ref().map(lower_expression_caps),
            }
        }
        PrimitiveType::NumberArray {
            common,
            default,
            minimum,
            maximum,
        } => IntermediateLayerField {
            r#type: IntermediateType::NumberArray {
                min: minimum.as_ref().and_then(|n| n.as_f64()),
                max: maximum.as_ref().and_then(|n| n.as_f64()),
            },
            default: default.map(Value::Number),
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        PrimitiveType::ColorArray { common, default } => IntermediateLayerField {
            r#type: IntermediateType::ColorArray,
            default: default.map(Value::String),
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        PrimitiveType::Padding { common, default } => IntermediateLayerField {
            r#type: IntermediateType::Padding,
            default: Some(Value::Array(
                default.into_iter().map(Value::Number).collect(),
            )),
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        PrimitiveType::Formatted {
            common,
            tokens,
            default,
        } => IntermediateLayerField {
            r#type: IntermediateType::Formatted { tokens },
            default: Some(Value::String(default)),
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        PrimitiveType::ResolvedImage { common, tokens } => IntermediateLayerField {
            r#type: IntermediateType::ResolvedImage {
                tokens: tokens.unwrap_or(false),
            },
            default: None,
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        PrimitiveType::State { common, default } => IntermediateLayerField {
            r#type: IntermediateType::State,
            default: if default == Value::Null {
                None
            } else {
                Some(default)
            },
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        PrimitiveType::ProjectionDefinition { common, default } => IntermediateLayerField {
            r#type: IntermediateType::ProjectionDefinition,
            default: Some(Value::String(default)),
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        PrimitiveType::Star(common) => IntermediateLayerField {
            r#type: IntermediateType::AnyObject,
            default: None,
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        PrimitiveType::Sprite(common) => IntermediateLayerField {
            r#type: IntermediateType::Sprite,
            default: None,
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        PrimitiveType::PromoteId(common) => IntermediateLayerField {
            r#type: IntermediateType::PromoteId,
            default: None,
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        PrimitiveType::VariableAnchorOffsetCollection(common) => IntermediateLayerField {
            r#type: IntermediateType::VariableAnchorOffsetCollection,
            default: None,
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        PrimitiveType::PropertyType(_) => {
            panic!("PropertyType is a meta-type and should be filtered before lowering")
        }
    }
}

fn enum_values_to_strings(values: &EnumValues) -> Vec<String> {
    match values {
        EnumValues::Enum(map) => map.keys().cloned().collect(),
        EnumValues::Version(numbers) => numbers.iter().map(|n| n.to_string()).collect(),
        EnumValues::SyntaxEnum(map) => map.keys().cloned().collect(),
    }
}

fn array_value_to_element_type(
    value: ArrayValue,
    enum_values: Option<EnumValues>,
) -> ArrayElementType {
    match value {
        ArrayValue::Simple(s) => match s {
            SimpleArrayValue::String => ArrayElementType::String,
            SimpleArrayValue::Number => ArrayElementType::Number,
            SimpleArrayValue::Color => ArrayElementType::Color,
            SimpleArrayValue::Layer => ArrayElementType::Layer,
            SimpleArrayValue::Enum => {
                let variants = match enum_values {
                    Some(EnumValues::Enum(map)) => map.keys().cloned().collect(),
                    _ => vec![],
                };
                ArrayElementType::Enum(variants)
            }
            // Fall back to String for uncommon/meta types
            SimpleArrayValue::Star
            | SimpleArrayValue::FunctionStop
            | SimpleArrayValue::FontFaces
            | SimpleArrayValue::ExpressionName
            | SimpleArrayValue::InterpolationName => ArrayElementType::String,
        },
        ArrayValue::Either(_) | ArrayValue::Complex(_) => ArrayElementType::String,
    }
}

fn reference_to_intermediate_type(references: &str) -> IntermediateType {
    match references {
        "color" => IntermediateType::Color,
        "string" => IntermediateType::String,
        "number" => IntermediateType::Number {
            min: None,
            max: None,
        },
        "boolean" => IntermediateType::Boolean,
        "enum" => IntermediateType::Enum { values: vec![] },
        "array" => IntermediateType::Array {
            element: ArrayElementType::String,
            length: None,
        },
        "formatted" => IntermediateType::Formatted { tokens: false },
        "resolvedImage" => IntermediateType::ResolvedImage { tokens: false },
        "padding" => IntermediateType::Padding,
        "variableAnchorOffsetCollection" => IntermediateType::VariableAnchorOffsetCollection,
        "projectionDefinition" => IntermediateType::ProjectionDefinition,
        _ => IntermediateType::String,
    }
}

pub fn preprocess_layers(reference: &mut decoder::StyleReference) -> IntermediateLayers {
    let Some(layers_item) = reference.root.remove("layers") else {
        return IntermediateLayers {
            common_fields: BTreeMap::new(),
            layer_types: BTreeMap::new(),
        };
    };
    let decoder::PrimitiveType::Array { .. } = layers_item.as_primitive() else {
        panic!("layers must be an array");
    };

    let mut layer = reference.fields.remove("layer").unwrap().as_group().clone();

    let layer_type_item = layer.remove("type").unwrap();
    let (layer_type_values, _layer_type_common, _layer_type_default) =
        layer_type_item.as_primitive().as_enum();
    let layer_type_keys: Vec<String> = layer_type_values.as_enum().keys().cloned().collect();

    layer.remove("layout");
    let mut layout = pop_one_of_as_group(&mut reference.fields, "layout");
    layer.remove("paint");
    let mut paint = pop_one_of_as_group(&mut reference.fields, "paint");

    assert_eq!(
        paint.keys().collect::<BTreeSet<_>>(),
        layout.keys().collect::<BTreeSet<_>>(),
        "paint and layout must have the same keys"
    );

    let mut common_fields = parsed_items_to_layer_fields(layer);

    // Remove `filter` from auto-generated common fields — the generator emits
    // the `Layer` struct with a hand-written `filter: Option<LayerFilter>` field
    // and a dedicated `LayerFilter` type.
    common_fields.remove("filter");

    let mut layer_types = BTreeMap::new();
    for key in layer_type_keys {
        let layout = parsed_items_to_layer_fields(layout.remove(&key).unwrap_or_default());
        let paint = parsed_items_to_layer_fields(paint.remove(&key).unwrap_or_default());
        layer_types.insert(key, IntermediateLayerType { layout, paint });
    }

    IntermediateLayers {
        common_fields,
        layer_types,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder;

    #[test]
    fn test_decode_top_level() {
        let mut reference: decoder::StyleReference =
            serde_json::from_str(include_str!("../../../../upstream/src/reference/v8.json"))
                .unwrap();
        let layers = preprocess_layers(&mut reference);
        assert!(
            !layers.layer_types.is_empty(),
            "must have at least one layer type"
        );
        assert!(
            !layers.common_fields.is_empty(),
            "layers must have common fields"
        );
        insta::assert_json_snapshot!("decode_top_level", layers);
    }
}

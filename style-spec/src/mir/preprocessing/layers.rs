use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::decoder;
use crate::decoder::array::{DecodedArrayValue, DecodedSimpleArrayValue};
use crate::decoder::r#enum::DecodedEnumValues;
use crate::decoder::{DecodedParsedItem, DecodedPrimitiveType};
use crate::mir::layers::{MirLayerField, MirLayerType, MirLayers};
use crate::mir::preprocessing::pop_one_of_as_group;
use crate::mir::types::{MirArrayElementType, MirExpressionCapabilities, MirType};

fn parsed_items_to_layer_fields(
    map: BTreeMap<String, DecodedParsedItem>,
) -> BTreeMap<String, MirLayerField> {
    map.into_iter()
        .filter(|(k, _)| k != "*")
        .map(|(k, v)| (k, parsed_item_to_layer_field(v)))
        .collect()
}

fn lower_expression_caps(e: &crate::decoder::Expression) -> MirExpressionCapabilities {
    MirExpressionCapabilities {
        interpolated: e.interpolated,
        zoom: e.parameters.iter().any(|p| p == "zoom"),
        feature: e.parameters.iter().any(|p| p == "feature"),
        global_state: e.parameters.iter().any(|p| p == "global-state"),
    }
}

fn parsed_item_to_layer_field(item: DecodedParsedItem) -> MirLayerField {
    match item {
        DecodedParsedItem::Reference { references, common } => MirLayerField {
            r#type: reference_to_intermediate_type(&references),
            default: None,
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        DecodedParsedItem::Primitive(p) => primitive_to_layer_field(p),
    }
}

fn primitive_to_layer_field(p: DecodedPrimitiveType) -> MirLayerField {
    match p {
        DecodedPrimitiveType::Number {
            common,
            default,
            maximum,
            minimum,
            period: _,
        } => MirLayerField {
            r#type: MirType::Number {
                min: minimum.as_ref().and_then(|n| n.as_f64()),
                max: maximum.as_ref().and_then(|n| n.as_f64()),
            },
            default: default.map(Value::Number),
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        DecodedPrimitiveType::Boolean { common, default } => MirLayerField {
            r#type: MirType::Boolean,
            default: default.map(Value::Bool),
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        DecodedPrimitiveType::String { common, default } => MirLayerField {
            r#type: MirType::String,
            default: default.map(Value::String),
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        DecodedPrimitiveType::Color { common, default } => MirLayerField {
            r#type: MirType::Color,
            default,
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        DecodedPrimitiveType::Enum {
            common,
            default,
            values,
        } => MirLayerField {
            r#type: MirType::Enum {
                values: enum_values_to_strings(&values),
            },
            default,
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        DecodedPrimitiveType::Array {
            common,
            default,
            value,
            values,
            minimum: _,
            maximum: _,
            length,
        } => {
            let element = array_value_to_element_type(value, values);
            MirLayerField {
                r#type: MirType::Array { element, length },
                default: default.map(Value::Array),
                doc: common.doc,
                required: common.required.unwrap_or(false),
                expression: common.expression.as_ref().map(lower_expression_caps),
            }
        }
        DecodedPrimitiveType::NumberArray {
            common,
            default,
            minimum,
            maximum,
        } => MirLayerField {
            r#type: MirType::NumberArray {
                min: minimum.as_ref().and_then(|n| n.as_f64()),
                max: maximum.as_ref().and_then(|n| n.as_f64()),
            },
            default: default.map(Value::Number),
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        DecodedPrimitiveType::ColorArray { common, default } => MirLayerField {
            r#type: MirType::ColorArray,
            default: default.map(Value::String),
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        DecodedPrimitiveType::Padding { common, default } => MirLayerField {
            r#type: MirType::Padding,
            default: Some(Value::Array(
                default.into_iter().map(Value::Number).collect(),
            )),
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        DecodedPrimitiveType::Formatted {
            common,
            tokens,
            default,
        } => MirLayerField {
            r#type: MirType::Formatted { tokens },
            default: Some(Value::String(default)),
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        DecodedPrimitiveType::ResolvedImage { common, tokens } => MirLayerField {
            r#type: MirType::ResolvedImage {
                tokens: tokens.unwrap_or(false),
            },
            default: None,
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        DecodedPrimitiveType::State { common, default } => MirLayerField {
            r#type: MirType::State,
            default: if default == Value::Null {
                None
            } else {
                Some(default)
            },
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        DecodedPrimitiveType::ProjectionDefinition { common, default } => MirLayerField {
            r#type: MirType::ProjectionDefinition,
            default: Some(Value::String(default)),
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        DecodedPrimitiveType::Star(common) => MirLayerField {
            r#type: MirType::AnyObject,
            default: None,
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        DecodedPrimitiveType::Sprite(common) => MirLayerField {
            r#type: MirType::Sprite,
            default: None,
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        DecodedPrimitiveType::PromoteId(common) => MirLayerField {
            r#type: MirType::PromoteId,
            default: None,
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        DecodedPrimitiveType::VariableAnchorOffsetCollection(common) => MirLayerField {
            r#type: MirType::VariableAnchorOffsetCollection,
            default: None,
            doc: common.doc,
            required: common.required.unwrap_or(false),
            expression: common.expression.as_ref().map(lower_expression_caps),
        },
        DecodedPrimitiveType::PropertyType(_) => {
            panic!("DecodedPropertyType is a meta-type and should be filtered before lowering")
        }
    }
}

fn enum_values_to_strings(values: &DecodedEnumValues) -> Vec<String> {
    match values {
        DecodedEnumValues::Enum(map) => map.keys().cloned().collect(),
        DecodedEnumValues::Version(numbers) => numbers.iter().map(|n| n.to_string()).collect(),
        DecodedEnumValues::SyntaxEnum(map) => map.keys().cloned().collect(),
    }
}

fn array_value_to_element_type(
    value: DecodedArrayValue,
    enum_values: Option<DecodedEnumValues>,
) -> MirArrayElementType {
    match value {
        DecodedArrayValue::Simple(s) => match s {
            DecodedSimpleArrayValue::String => MirArrayElementType::String,
            DecodedSimpleArrayValue::Number => MirArrayElementType::Number,
            DecodedSimpleArrayValue::Color => MirArrayElementType::Color,
            DecodedSimpleArrayValue::Layer => MirArrayElementType::Layer,
            DecodedSimpleArrayValue::Enum => {
                let variants = match enum_values {
                    Some(DecodedEnumValues::Enum(map)) => map.keys().cloned().collect(),
                    _ => vec![],
                };
                MirArrayElementType::Enum(variants)
            }
            // Fall back to String for uncommon/meta types
            DecodedSimpleArrayValue::Star
            | DecodedSimpleArrayValue::FunctionStop
            | DecodedSimpleArrayValue::FontFaces
            | DecodedSimpleArrayValue::ExpressionName
            | DecodedSimpleArrayValue::InterpolationName => MirArrayElementType::String,
        },
        DecodedArrayValue::Either(_) | DecodedArrayValue::Complex(_) => MirArrayElementType::String,
    }
}

fn reference_to_intermediate_type(references: &str) -> MirType {
    match references {
        "color" => MirType::Color,
        "string" => MirType::String,
        "number" => MirType::Number {
            min: None,
            max: None,
        },
        "boolean" => MirType::Boolean,
        "enum" => MirType::Enum { values: vec![] },
        "array" => MirType::Array {
            element: MirArrayElementType::String,
            length: None,
        },
        "formatted" => MirType::Formatted { tokens: false },
        "resolvedImage" => MirType::ResolvedImage { tokens: false },
        "padding" => MirType::Padding,
        "variableAnchorOffsetCollection" => MirType::VariableAnchorOffsetCollection,
        "projectionDefinition" => MirType::ProjectionDefinition,
        _ => MirType::String,
    }
}

pub fn preprocess_layers(reference: &mut decoder::StyleReference) -> MirLayers {
    let Some(layers_item) = reference.root.remove("layers") else {
        return MirLayers {
            common_fields: BTreeMap::new(),
            layer_types: BTreeMap::new(),
        };
    };
    let decoder::DecodedPrimitiveType::Array { .. } = layers_item.as_primitive() else {
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
        layer_types.insert(key, MirLayerType { layout, paint });
    }

    MirLayers {
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

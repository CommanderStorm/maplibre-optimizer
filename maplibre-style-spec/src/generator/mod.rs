use std::collections::BTreeMap;

use codegen2::Scope;

use crate::generator::formatter::{to_snake_case, to_upper_camel_case};
use crate::generator::literals::generate_literals;
use crate::mir::types::{
    ArrayElement, ArrayElementType, BooleanField, ColorArrayField, ColorField, EnumField,
    FieldMeta, FormattedTextField, IntermediateType, MirEnum, MirField, NumberArrayField,
    NumberField, PaddingField, ProjectionDefinitionField, RegularEnum, RegularVariant,
    ResolvedImageField, StateField, StringField,
};
use crate::mir::{
    Expressions, IntermediateLayerField, IntermediateNamedType, IntermediateOneOf,
    IntermediateSpec, Layers, Sources,
};

mod autotest;
pub mod formatter;
mod items;
mod literals;

/// Generate Rust source from the semantic MIR.
/// This is the sole entry point; it never touches decoder types.
pub fn generate_spec_scope(spec: &IntermediateSpec) -> String {
    let mut scope = Scope::new();

    generate_root_struct(&mut scope, spec);
    generate_literals(&mut scope);

    // Named types (groups, type aliases, OneOf enums)
    for (key, named_type) in &spec.named_types {
        let name = to_upper_camel_case(key);
        generate_named_type(&mut scope, &name, named_type);
    }

    // Expression syntax enums (per-output-type)
    generate_expression_types(&mut scope, &spec.expressions);

    // Source struct types
    generate_source_types(&mut scope, &spec.sources);

    // Layer struct types
    generate_layer_types(&mut scope, &spec.layers);

    scope
        .get_or_new_module("test")
        .attr("cfg(test)")
        .import("super", "*");

    scope.to_string()
}

// ── Root struct ───────────────────────────────────────────────────────────────

fn generate_root_struct(scope: &mut Scope, spec: &IntermediateSpec) {
    let s = scope
        .new_struct("MaplibreStyleSpecification")
        .doc("This is a Maplibre Style Specification")
        .vis("pub")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone");

    for (key, field) in &spec.root.0 {
        let meta = field.meta();
        let type_name = to_upper_camel_case(&format!("root {key}"));
        let mut field_type = type_name.clone();
        if meta.optional {
            field_type = format!("Option<{field_type}>");
        }
        let sf = s
            .new_field(&meta.rust_name, field_type)
            .vis("pub")
            .doc(&meta.doc);
        if &meta.rust_name != key {
            sf.annotation(format!("#[serde(rename=\"{key}\")]"));
        }
    }

    // Generate subtypes for each root field
    for (key, field) in &spec.root.0 {
        let type_name = to_upper_camel_case(&format!("root {key}"));
        generate_mir_type(scope, &type_name, field);
    }
}

// ── Named types ───────────────────────────────────────────────────────────────

fn generate_named_type(scope: &mut Scope, name: &str, named_type: &IntermediateNamedType) {
    match named_type {
        IntermediateNamedType::Struct(fields) => generate_struct_from_fields(scope, name, fields),
        IntermediateNamedType::TypeDef(field) => generate_mir_type(scope, name, field),
        IntermediateNamedType::OneOf(one_of) => generate_oneof(scope, name, one_of),
    }
}

/// Generate a named struct from a slice of MIR fields.
/// Handles the single-star (`*`) wildcard field as a BTreeMap wrapper.
fn generate_struct_from_fields(scope: &mut Scope, name: &str, fields: &[MirField]) {
    // Special case: single-star field → BTreeMap wrapper
    if fields.len() == 1 {
        if let MirField::Star(meta) = &fields[0] {
            let inner_name = to_upper_camel_case(&format!("Inner {name}"));
            scope
                .new_struct(name)
                .vis("pub")
                .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
                .tuple_field(format!("std::collections::BTreeMap<String,{inner_name}>"));
            items::star::generate(scope, &inner_name, meta);
            return;
        }
    }

    let s = scope
        .new_struct(name)
        .vis("pub")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone");

    for field in fields {
        let meta = field.meta();
        let field_type_name = to_upper_camel_case(&format!("{name} {}", meta.spec_name));
        let mut field_type = if meta.spec_name == "*" {
            format!("std::collections::BTreeMap<String,{field_type_name}>")
        } else {
            field_type_name.clone()
        };
        if meta.optional {
            field_type = format!("Option<{field_type}>");
        }
        let sf = s
            .new_field(&meta.rust_name, field_type)
            .vis("pub")
            .doc(&meta.doc);
        if meta.spec_name == "*" {
            sf.annotation("#[serde(flatten)]");
        } else if &meta.rust_name != meta.spec_name.as_str() {
            sf.annotation(format!("#[serde(rename=\"{}\")]", meta.spec_name));
        }
    }

    // Generate subtypes for each field
    for field in fields {
        let meta = field.meta();
        if meta.spec_name == "*" {
            // Already handled above via star::generate
            continue;
        }
        let field_type_name = to_upper_camel_case(&format!("{name} {}", meta.spec_name));
        generate_mir_type(scope, &field_type_name, field);
    }
}

/// Generate a `#[serde(tag)]` or `#[serde(untagged)]` sum-type enum.
fn generate_oneof(scope: &mut Scope, name: &str, one_of: &IntermediateOneOf) {
    let enu = scope
        .new_enum(name)
        .vis("pub")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone");

    if let Some(tag) = &one_of.tag {
        enu.attr(format!("serde(tag=\"{tag}\")"));
    } else {
        enu.attr("serde(untagged)");
    }

    for variant_key in &one_of.variants {
        let var_name = to_upper_camel_case(variant_key);
        let var = enu.new_variant(&var_name).tuple(&var_name);
        if let Some(rename) = one_of.renames.get(&var_name) {
            var.annotation(format!("#[serde(rename=\"{rename}\")]"));
        }
    }
}

// ── Expression types ──────────────────────────────────────────────────────────

fn generate_expression_types(scope: &mut Scope, expressions: &Expressions) {
    for (output_type_name, group) in &expressions.by_output_type {
        items::r#enum::syntax::generate_syntax_enum(
            scope,
            output_type_name,
            &format!("{output_type_name:?}"),
            &group.variants,
        );
    }
}

// ── Source types ──────────────────────────────────────────────────────────────

fn generate_source_types(scope: &mut Scope, sources: &Sources) {
    if sources.source_types.is_empty() {
        return;
    }

    // Generate a struct per source type
    for (type_name, def) in &sources.source_types {
        let struct_name = to_upper_camel_case(&format!("{type_name} source"));
        generate_struct_from_fields(scope, &struct_name, &def.fields);
    }

    // Generate the Source sum type
    let variant_keys: Vec<String> = sources
        .source_types
        .keys()
        .map(|k| format!("{k}_source"))
        .collect();

    // Detect common tag field ("type") and build renames
    let tag = if sources
        .source_types
        .values()
        .all(|d| d.discriminant_value.is_some())
    {
        Some("type".to_string())
    } else {
        None
    };

    let renames: BTreeMap<String, String> = sources
        .source_types
        .iter()
        .filter_map(|(k, d)| {
            d.discriminant_value
                .as_ref()
                .map(|v| (to_upper_camel_case(&format!("{k}_source")), v.clone()))
        })
        .collect();

    generate_oneof(
        scope,
        "Source",
        &IntermediateOneOf {
            variants: variant_keys,
            tag,
            renames,
        },
    );
}

// ── Layer types ───────────────────────────────────────────────────────────────

fn generate_layer_types(scope: &mut Scope, layers: &Layers) {
    if layers.common_fields.is_empty() && layers.layer_types.is_empty() {
        return;
    }

    // Common `Layer` struct
    let common_mir: Vec<MirField> = layer_fields_to_mir(&layers.common_fields);
    generate_struct_from_fields(scope, "Layer", &common_mir);

    // Per-type layout and paint structs
    for (type_key, layer_type) in &layers.layer_types {
        let layout_name = to_upper_camel_case(&format!("{type_key} layout layer"));
        let paint_name = to_upper_camel_case(&format!("{type_key} paint layer"));
        let layout_mir: Vec<MirField> = layer_fields_to_mir(&layer_type.layout);
        let paint_mir: Vec<MirField> = layer_fields_to_mir(&layer_type.paint);
        generate_struct_from_fields(scope, &layout_name, &layout_mir);
        generate_struct_from_fields(scope, &paint_name, &paint_mir);
    }
}

fn layer_fields_to_mir(
    fields: &std::collections::BTreeMap<String, IntermediateLayerField>,
) -> Vec<MirField> {
    fields
        .iter()
        .map(|(name, f)| layer_field_to_mir(name, f))
        .collect()
}

fn layer_field_to_mir(spec_name: &str, f: &IntermediateLayerField) -> MirField {
    let meta = FieldMeta {
        spec_name: spec_name.to_string(),
        rust_name: to_snake_case(spec_name),
        optional: !f.required,
        transition: false,
        expression: f.expression.clone(),
        doc: f.doc.clone(),
        example: None,
        units: None,
    };

    match &f.r#type {
        IntermediateType::Number { min, max } => MirField::Number(NumberField {
            meta,
            default: f
                .default
                .as_ref()
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            min: *min,
            max: *max,
            period: None,
        }),
        IntermediateType::String => MirField::String(StringField {
            meta,
            default: f
                .default
                .as_ref()
                .and_then(|v| v.as_str().map(|s| s.to_string())),
        }),
        IntermediateType::Boolean => MirField::Boolean(BooleanField {
            meta,
            default: f.default.as_ref().and_then(|v| v.as_bool()),
        }),
        IntermediateType::Color => MirField::Color(ColorField {
            meta,
            default: f.default.clone(),
        }),
        IntermediateType::Enum { values } => MirField::Enum(EnumField {
            meta,
            default: f.default.clone(),
            variants: MirEnum::Regular(RegularEnum {
                variants: values
                    .iter()
                    .map(|v| (v.clone(), RegularVariant { doc: String::new() }))
                    .collect(),
            }),
        }),
        IntermediateType::Array { element, length } => {
            let mir_element = array_element_type_to_mir(element);
            MirField::Array(crate::mir::types::ArrayField {
                meta,
                default: f
                    .default
                    .as_ref()
                    .and_then(|v| serde_json::from_value(v.clone()).ok()),
                element: mir_element,
                length: *length,
            })
        }
        IntermediateType::Padding => MirField::Padding(PaddingField {
            meta,
            default: match &f.default {
                Some(serde_json::Value::Array(arr)) => arr
                    .iter()
                    .filter_map(|v| serde_json::from_value(v.clone()).ok())
                    .collect(),
                _ => vec![],
            },
        }),
        IntermediateType::Formatted { tokens } => MirField::FormattedText(FormattedTextField {
            meta,
            tokens: *tokens,
            default: f
                .default
                .as_ref()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_default(),
        }),
        IntermediateType::ResolvedImage { tokens } => MirField::ResolvedImage(ResolvedImageField {
            meta,
            tokens: Some(*tokens),
        }),
        IntermediateType::NumberArray { min, max } => MirField::NumberArray(NumberArrayField {
            meta,
            default: f
                .default
                .as_ref()
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            min: *min,
            max: *max,
        }),
        IntermediateType::ColorArray => MirField::ColorArray(ColorArrayField {
            meta,
            default: f
                .default
                .as_ref()
                .and_then(|v| v.as_str().map(|s| s.to_string())),
        }),
        IntermediateType::State => MirField::State(StateField {
            meta,
            default: f.default.clone().unwrap_or(serde_json::Value::Null),
        }),
        IntermediateType::AnyObject => MirField::Star(meta),
        IntermediateType::Sprite => MirField::Sprite(meta),
        IntermediateType::PromoteId => MirField::PromoteId(meta),
        IntermediateType::ProjectionDefinition => {
            MirField::ProjectionDefinition(ProjectionDefinitionField {
                meta,
                default: f
                    .default
                    .as_ref()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_default(),
            })
        }
        IntermediateType::VariableAnchorOffsetCollection => {
            MirField::VariableAnchorOffsetCollection(meta)
        }
    }
}

fn array_element_type_to_mir(element: &ArrayElementType) -> ArrayElement {
    match element {
        ArrayElementType::String => ArrayElement::String,
        ArrayElementType::Number => ArrayElement::Number {
            min: None,
            max: None,
        },
        ArrayElementType::Color => ArrayElement::Color,
        ArrayElementType::Enum(values) => ArrayElement::Enum(RegularEnum {
            variants: values
                .iter()
                .map(|v| (v.clone(), RegularVariant { doc: String::new() }))
                .collect(),
        }),
        ArrayElementType::Layer => ArrayElement::Layer,
    }
}

// ── MirField dispatch ─────────────────────────────────────────────────────────

/// Dispatch a `MirField` to the appropriate item generator.
/// Called both from this module and from `items/array.rs` (for `Complex` elements).
pub fn generate_mir_type(scope: &mut Scope, name: &str, field: &MirField) {
    match field {
        MirField::Number(f) => items::number::generate(scope, name, f),
        MirField::Boolean(f) => items::boolean::generate(scope, name, f),
        MirField::String(f) => items::string::generate(scope, name, f),
        MirField::Color(f) => items::color::generate(scope, name, f),
        MirField::Enum(f) => items::r#enum::generate_mir(scope, name, f),
        MirField::Array(f) => items::array::generate(scope, name, f),
        MirField::NumberArray(f) => items::number_array::generate(scope, name, f),
        MirField::ColorArray(f) => items::color_array::generate(scope, name, f),
        MirField::FormattedText(f) => items::formatted::generate(scope, name, f),
        MirField::ResolvedImage(f) => items::resolved_image::generate(scope, name, f),
        MirField::Padding(f) => items::padding::generate(scope, name, f),
        MirField::State(f) => items::state::generate(scope, name, f),
        MirField::ProjectionDefinition(f) => items::projection_definition::generate(scope, name, f),
        MirField::Sprite(m) => items::sprite::generate(scope, name, m),
        MirField::PromoteId(m) => items::promote_id::generate(scope, name, m),
        MirField::VariableAnchorOffsetCollection(m) => {
            items::variable_anchor_offset_collection::generate(scope, name, m)
        }
        MirField::Star(m) => items::star::generate(scope, name, m),
        MirField::Reference(f) => items::reference::generate(scope, name, f),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::decoder::StyleReference;
    use crate::mir::IntermediateSpec;

    #[test]
    fn test_generate_spec_items() {
        let reference = json!({
            "$version": 8,
            "$root": {},
            "number_one": {
              "doc": "A number between 0 and 10.",
              "type": "number",
              "default": 0
            }
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        let spec = IntermediateSpec::from(reference);
        insta::assert_snapshot!(generate_spec_scope(&spec));
    }

    #[test]
    fn test_generate_spec_groups() {
        let reference = json!({
            "$version": 8,
            "$root": {},
            "names": {
              "name_one": {
                "type": "number",
                "doc": "A number between 0 and 10.",
                "default": 1.0
              }
            }
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        let spec = IntermediateSpec::from(reference);
        insta::assert_snapshot!(generate_spec_scope(&spec));
    }

    #[test]
    fn test_generate_spec_oneof() {
        let reference = json!({
            "$version": 8,
            "$root": {},
            "number_one": {
              "type": "number",
              "doc": "A number between 0 and 20.",
              "default": 1.0,
              "minimum": 0.0,
              "maximum": 10.0
            },
            "number_two": {
              "type": "number",
              "doc": "Another number"
            },
            "numbers": ["number_one", "number_two"]
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        let spec = IntermediateSpec::from(reference);
        insta::assert_snapshot!(generate_spec_scope(&spec));
    }
}

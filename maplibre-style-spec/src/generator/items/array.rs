use codegen2::Scope;
use serde_json::Value;

use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::formatter::to_upper_camel_case;
use crate::generator::items::number::generate_number_default;
use crate::mir::types::{ArrayElement, ArrayField, RegularEnum};

pub fn generate(scope: &mut Scope, name: &str, field: &ArrayField) {
    let element_type_name = to_upper_camel_case(&format!("{name} Value"));
    let rust_element_type = generate_array_element(scope, &element_type_name, &field.element);

    let field_type = if is_direct_element(&field.element) {
        rust_element_type
    } else if let Some(length) = field.length {
        format!("Box<[{rust_element_type}; {length}]>")
    } else {
        format!("Vec<{rust_element_type}>")
    };

    scope
        .new_struct(name)
        .doc(&field.meta.doc)
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .tuple_field(field_type);

    if let Some(default) = &field.default {
        let mut default_expr = String::new();
        generate_value_array_default(&mut default_expr, default, field.length.as_ref());
        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(format!("Self({default_expr})"));
    }
    generate_test_from_example_if_present(scope, name, field.meta.example.as_ref());
}

/// Whether this element type should be used directly as the field type
/// rather than being wrapped in `Vec<...>`.
fn is_direct_element(element: &ArrayElement) -> bool {
    matches!(
        element,
        ArrayElement::FontFaces | ArrayElement::ExpressionName | ArrayElement::InterpolationName
    )
}

/// Returns the Rust type name for the array element, generating any necessary
/// helper types into `scope`.
fn generate_array_element(scope: &mut Scope, name: &str, element: &ArrayElement) -> String {
    match element {
        ArrayElement::String => "String".to_string(),
        ArrayElement::Number { .. } => "serde_json::Number".to_string(),
        ArrayElement::Boolean => "bool".to_string(),
        ArrayElement::Color => "color::DynamicColor".to_string(),
        ArrayElement::Star => "serde_json::Value".to_string(),
        ArrayElement::Layer => "Layer".to_string(),
        ArrayElement::FunctionStop => "FunctionStop".to_string(),
        ArrayElement::ExpressionName => "ExpressionName".to_string(),
        ArrayElement::InterpolationName => "InterpolationName".to_string(),

        ArrayElement::Enum(r) => {
            generate_inline_enum(scope, name, r);
            name.to_string()
        }

        ArrayElement::FontFaces => {
            generate_font_faces(scope);
            "std::collections::BTreeMap<String,FontFace>".to_string()
        }

        ArrayElement::Either(options) => {
            let mut variant_types = Vec::with_capacity(options.len());
            for (i, option) in options.iter().enumerate() {
                let enum_variant_name = to_upper_camel_case(&i.to_string());
                let variant_type_name = to_upper_camel_case(&format!("{name} {enum_variant_name}"));
                variant_types.push((
                    enum_variant_name,
                    generate_array_element(scope, &variant_type_name, option),
                ));
            }

            let enu = scope
                .new_enum(name)
                .doc(format!("{name} Values"))
                .attr("serde(untagged)")
                .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
                .vis("pub");
            for (variant_name, variant_type) in variant_types {
                enu.new_variant(variant_name).tuple(variant_type);
            }
            name.to_string()
        }

        ArrayElement::Complex(inner_field) => {
            // Delegate to the central MIR dispatch in the parent generator module.
            crate::generator::generate_mir_type(scope, name, inner_field);
            name.to_string()
        }
    }
}

fn generate_inline_enum(scope: &mut Scope, name: &str, r: &RegularEnum) {
    crate::generator::items::r#enum::generate_regular(scope, name, "", r, None);
}

fn generate_font_faces(scope: &mut Scope) {
    let font_with_range = scope
        .new_struct("FontWithRange")
        .vis("pub")
        .doc("Font file URL and the unicode-range at which it can be used")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone");
    font_with_range
        .new_field("url", "url::Url")
        .vis("pub")
        .doc("URL the font can retrieved under");
    font_with_range
        .new_field("unicode_range", "String")
        .vis("pub")
        .doc("Unicode characters where this font should be used")
        .annotation("#[serde(rename=\"unicode-range\")]");

    let enu = scope
        .new_enum("FontFace")
        .vis("pub")
        .attr("serde(untagged)")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone");
    enu.new_variant("Url")
        .doc("A single global font file URL")
        .tuple("url::Url");
    enu.new_variant("FontRange")
        .doc("Load different fonts depending on the unicode range")
        .tuple("Vec<FontWithRange>");
}

fn generate_value_array_default(buffer: &mut String, items: &[Value], length: Option<&usize>) {
    if length.is_some() {
        buffer.push_str("Box::new([");
    } else {
        buffer.push_str("Vec::from([");
    }
    let mut needs_separator = false;
    for item in items {
        if needs_separator {
            buffer.push_str(", ");
        }
        generate_value_default(buffer, item);
        needs_separator = true;
    }
    buffer.push_str("])");
}

fn generate_value_default(buffer: &mut String, item: &Value) {
    match item {
        Value::Null => buffer.push_str("None"),
        Value::Bool(b) => buffer.push_str(&b.to_string()),
        Value::Number(n) => buffer.push_str(&generate_number_default(n)),
        Value::String(s) => {
            buffer.push('"');
            buffer.push_str(s);
            buffer.push_str("\".to_string()");
        }
        Value::Array(a) => generate_value_array_default(buffer, a, None),
        Value::Object(o) => {
            let json = serde_json::to_string(o).expect("serializing json object must succeed");
            buffer.push_str(&format!(
                "serde_json::from_str::<serde_json::Value>({json:?}).expect(\"object default must be valid json\")"
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::StyleReference;
    use crate::mir::types::FieldMeta;

    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(
            &mut scope,
            "Foo",
            &ArrayField {
                meta: FieldMeta::default(),
                default: None,
                element: ArrayElement::Star,
                length: None,
            },
        );
        insta::assert_snapshot!(scope.to_string(), @r"
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        struct Foo(Vec<serde_json::Value>);
        ")
    }

    #[test]
    fn test_generate_spec() {
        let reference = serde_json::json!({
        "$version": 8,
        "$root": {},
        "position": {
            "type": "array",
            "default": [
                1.15,
                210,
                30
            ],
            "length": 3,
            "value": "number",
            "property-type": "data-constant",
            "transition": true,
            "expression": {
                "interpolated": true,
                "parameters": [
                  "zoom"
                ]
            },
            "doc": "Position of the light source relative to lit (extruded) geometries, in [r radial coordinate, a azimuthal angle, p polar angle] where r indicates the distance from the center of the base of an object to its light, a indicates the position of the light relative to 0° (0° when `light.anchor` is set to `viewport` corresponds to the top of the viewport, or 0° when `light.anchor` is set to `map` corresponds to due north, and degrees proceed clockwise), and p indicates the height of the light (from 0°, directly above, to 180°, directly below).",
            "example": [
                1.5,
                90,
                80
            ],
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        let spec = crate::mir::IntermediateSpec::from(reference);
        insta::assert_snapshot!(crate::generator::generate_spec_scope(&spec), @r#"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        /// Position of the light source relative to lit (extruded) geometries, in [r radial coordinate, a azimuthal angle, p polar angle] where r indicates the distance from the center of the base of an object to its light, a indicates the position of the light relative to 0° (0° when `light.anchor` is set to `viewport` corresponds to the top of the viewport, or 0° when `light.anchor` is set to `map` corresponds to due north, and degrees proceed clockwise), and p indicates the height of the light (from 0°, directly above, to 180°, directly below).
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        struct Position(Box<[serde_json::Number; 3]>);

        impl Default for Position {
            fn default() -> Self {
                Self(Box::new([serde_json::Number::from_f64(1.15).expect("the number is serialised from a number and is thus always valid"), serde_json::Number::from_i128(210).expect("the number is serialised from a number and is thus always valid"), serde_json::Number::from_i128(30).expect("the number is serialised from a number and is thus always valid")]))
            }
        }

        #[cfg(test)]
        mod test {
            use super::*;

            #[test]
            fn test_example_position_decodes() {
                let example = serde_json::json!([1.5,90,80]);
                let _ = serde_json::from_value::<Position>(example).expect("example should decode");
            }
        }
        "#);
    }

    #[test]
    fn test_generate_spec_layers() {
        let reference = serde_json::json!({
            "$version": 8,
            "$root": {},
            "layers": {
                "required": true,
                "type": "array",
                "value": "layer",
                "doc": "A style's `layers` property lists all the layers available in that style.",
                "example": [
                    {
                        "id": "coastline",
                        "source": "maplibre",
                        "source-layer": "countries",
                        "type": "line",
                        "paint": {
                            "line-color": "#198EC8"
                        }
                    }
                ]
            }
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        let spec = crate::mir::IntermediateSpec::from(reference);
        insta::assert_snapshot!(crate::generator::generate_spec_scope(&spec), @r##"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        /// A style's `layers` property lists all the layers available in that style.
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        struct Layers(Vec<Layer>);

        #[cfg(test)]
        mod test {
            use super::*;

            #[test]
            fn test_example_layers_decodes() {
                let example = serde_json::json!([{"id":"coastline","paint":{"line-color":"#198EC8"},"source":"maplibre","source-layer":"countries","type":"line"}]);
                let _ = serde_json::from_value::<Layers>(example).expect("example should decode");
            }
        }
        "##);
    }

    #[test]
    fn test_generate_spec_interpolation() {
        let reference = serde_json::json!({
            "$version": 8,
            "$root": {},
            "interpolation": {
              "type": "array",
              "value": "interpolation_name",
              "minimum": 1,
              "doc": "An interpolation defines how to transition between items. The first element of an interpolation array is a string naming the interpolation operator, e.g. `\"linear\"` or `\"exponential\"`. Elements that follow (if any) are the _arguments_ to the interpolation."
            },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        let spec = crate::mir::IntermediateSpec::from(reference);
        insta::assert_snapshot!(crate::generator::generate_spec_scope(&spec), @r#"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        /// An interpolation defines how to transition between items. The first element of an interpolation array is a string naming the interpolation operator, e.g. `"linear"` or `"exponential"`. Elements that follow (if any) are the _arguments_ to the interpolation.
        ///
        /// Range: 1..
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        struct Interpolation(InterpolationName);

        #[cfg(test)]
        mod test {
            use super::*;

        }
        "#);
    }
}

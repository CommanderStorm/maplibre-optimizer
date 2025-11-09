use codegen::Scope;
use serde_json::{Number, Value};

use crate::decoder::{ArrayValue, EnumValues, Fields, SimpleArrayValue};
use crate::generator::formatter::to_upper_camel_case;
use crate::generator::generate_parsed_item;
use crate::generator::items::number::generate_number_default;

#[allow(clippy::too_many_arguments)]
pub fn generate(
    scope: &mut Scope,
    name: &str,
    common: &Fields,
    default: Option<&Vec<Value>>,
    value: &ArrayValue,
    // if value is an enum
    values: Option<&EnumValues>,
    // if value is a number
    min: Option<&Number>,
    max: Option<&Number>,
    length: Option<&usize>,
) {
    // some arrays might require a new type name
    let new_type = to_upper_camel_case(&format!("{name} Value"));
    let type_name = generate_array_value(scope, &new_type, common, value, values);

    let field = if let Some(length) = length {
        format!("Box<[{type_name}; {length}]>")
    } else {
        format!("Vec<{type_name}>")
    };
    scope
        .new_struct(name)
        .doc(&common.doc_with_range(max, min, None))
        .derive("serde::Deserialize, PartialEq, Debug, Clone")
        .tuple_field(field);

    if let Some(default) = default {
        let mut default_value = String::new();
        generate_value_array_default(&mut default_value, default, length);
        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(format!("Self({default_value})"));
    }
}

fn generate_value_array_default(buffer: &mut String, items: &Vec<Value>, length: Option<&usize>) {
    if length.is_some() {
        buffer.push_str("Box::new([")
    } else {
        buffer.push_str("Vec::from([")
    };
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
        Value::Number(n) => {
            buffer.push_str(&generate_number_default(n));
        }
        Value::String(s) => {
            buffer.push('"');
            buffer.push_str(&s);
            buffer.push_str("\".to_string()");
        }
        Value::Array(a) => generate_value_array_default(buffer, a, None),
        Value::Object(o) => unimplemented!("Object in default value.."),
    }
}

fn generate_array_value(
    scope: &mut Scope,
    name: &str,
    common: &Fields,
    value: &ArrayValue,
    values: Option<&EnumValues>,
) -> String {
    match value {
        ArrayValue::Simple(v) => match v {
            SimpleArrayValue::String => "String".to_string(),
            SimpleArrayValue::Number => "serde_json::Number".to_string(),
            SimpleArrayValue::Star => "serde_json::Value".to_string(),
            SimpleArrayValue::FontFaces => "FontFaces".to_string(),
            SimpleArrayValue::FunctionStop => "FunctionStop".to_string(),
            SimpleArrayValue::Layer => "Layer".to_string(),
            SimpleArrayValue::Enum => {
                crate::generator::items::r#enum::generate(
                    scope,
                    name,
                    common,
                    None,
                    values.expect("EnumValues is required for SimpleArrayValue::Enum"),
                );
                name.to_string()
            }
            SimpleArrayValue::Color => "color::DynamicColor".to_string(),
        },
        ArrayValue::Either(options) => {
            let mut variant_types = Vec::with_capacity(options.len());
            for (i, option) in options.iter().enumerate() {
                let enum_variant_name = to_upper_camel_case(&i.to_string());
                let new_variant_type_name =
                    to_upper_camel_case(&format!("{name} {enum_variant_name}"));
                variant_types.push((
                    enum_variant_name,
                    generate_array_value(scope, &new_variant_type_name, common, option, values),
                ));
            }

            let enu = scope
                .new_enum(&name)
                .doc(format!("{name} Values"))
                .attr("serde(untagged)")
                .derive("serde::Deserialize, PartialEq, Debug, Clone")
                .vis("pub");

            for (enum_variant_name, variant_type) in variant_types {
                enu.new_variant(enum_variant_name).tuple(variant_type);
            }
            name.to_string()
        }
        ArrayValue::Complex(c) => {
            generate_parsed_item(scope, c, &name);
            name.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::decoder::{SimpleArrayValue, StyleReference};
    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(
            &mut scope,
            "Foo",
            &Fields::default(),
            None,
            &ArrayValue::Simple(SimpleArrayValue::Star),
            None,
            None,
            None,
            None,
        );
        insta::assert_snapshot!(scope.to_string(), @r"
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        struct Foo(Vec<serde_json::Value>);
        ")
    }

    #[test]
    fn test_generate_spec_number() {
        let reference = json!({
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
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r#"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        /// Position of the light source relative to lit (extruded) geometries, in [r radial coordinate, a azimuthal angle, p polar angle] where r indicates the distance from the center of the base of an object to its light, a indicates the position of the light relative to 0° (0° when `light.anchor` is set to `viewport` corresponds to the top of the viewport, or 0° when `light.anchor` is set to `map` corresponds to due north, and degrees proceed clockwise), and p indicates the height of the light (from 0°, directly above, to 180°, directly below).
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        struct Position(Box<[serde_json::Number; 3]>);

        impl Default for Position {
            fn default() -> Self {
                Self(Box::new([serde_json::Number::from_f64(1.15).expect("the number is serialised from a number and is thus always valid"), serde_json::Number::from_i128(210).expect("the number is serialised from a number and is thus always valid"), serde_json::Number::from_i128(30).expect("the number is serialised from a number and is thus always valid")]))
            }
        }
        "#);
    }

    #[test]
    fn test_generate_spec_layers() {
        let reference = json!({
            "$version": 8,
            "$root": {},
            "layers": {
                "required": true,
                "type": "array",
                "value": "layer",
                "doc": "A style's `layers` property lists all the layers available in that style. The type of layer is specified by the `type` property, and must be one of `background`, `fill`, `line`, `symbol`, `raster`, `circle`, `fill-extrusion`, `heatmap`, `hillshade`, `color-relief`.\n\nExcept for layers of the `background` type, each layer needs to refer to a source. Layers take the data that they get from a source, optionally filter features, and then define how those features are styled.",
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
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        /// A style's `layers` property lists all the layers available in that style. The type of layer is specified by the `type` property, and must be one of `background`, `fill`, `line`, `symbol`, `raster`, `circle`, `fill-extrusion`, `heatmap`, `hillshade`, `color-relief`.
        ///
        /// Except for layers of the `background` type, each layer needs to refer to a source. Layers take the data that they get from a source, optionally filter features, and then define how those features are styled.
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        struct Layers(Vec<Layer>);
        ");
    }
}

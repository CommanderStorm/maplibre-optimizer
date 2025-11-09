use codegen::Scope;
use serde_json::{Number, Value};

use crate::decoder::{ArrayValue, EnumValues, Fields};

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
    let field = if let Some(length) = length {
        format!("Box<[serde_json::Value; {}]>", length)
    } else {
        "Vec<serde_json::Value>".to_string()
    };
    scope
        .new_struct(name)
        .doc(&common.doc_with_range(max, min, None))
        .attr("deprecated = \"not_implemented\"")
        .derive("serde::Deserialize, PartialEq, Debug, Clone")
        .tuple_field(field);

    if let Some(default) = default {
        let mut items = if length.is_some() {
            "Box::new([".to_string()
        } else {
            "Vec::from([".to_string()
        };
        let mut needs_separator = false;
        for item in default {
            if needs_separator {
                items.push_str(", ");
            }
            items.push_str("serde_json::Value::from("); // todo: remove this
            items.push_str(&item.to_string());
            items.push(')'); // todo: remove this
            needs_separator = true;
        }
        items.push_str("])");

        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(format!("Self({items})"));
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
        insta::assert_snapshot!(scope.to_string(), @r##"
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        #[deprecated = "not_implemented"]
        struct Foo(Vec<serde_json::Value>);
        "##)
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
        #[deprecated = "not_implemented"]
        struct Position(Box<[serde_json::Value; 3]>);

        impl Default for Position {
            fn default() -> Self {
                Self(Box::new([serde_json::Value::from(1.15), serde_json::Value::from(210), serde_json::Value::from(30)]))
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
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r#"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        /// A style's `layers` property lists all the layers available in that style. The type of layer is specified by the `type` property, and must be one of `background`, `fill`, `line`, `symbol`, `raster`, `circle`, `fill-extrusion`, `heatmap`, `hillshade`, `color-relief`.
        ///
        /// Except for layers of the `background` type, each layer needs to refer to a source. Layers take the data that they get from a source, optionally filter features, and then define how those features are styled.
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        #[deprecated = "not_implemented"]
        struct Layers(Vec<serde_json::Value>);
        "#);
    }
}

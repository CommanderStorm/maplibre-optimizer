use codegen2::Scope;
use serde_json::Number;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;

pub fn generate(
    scope: &mut Scope,
    name: &str,
    common: &Fields,
    default: Option<&Number>,
    max: Option<&Number>,
    min: Option<&Number>,
    period: Option<&Number>,
) {
    scope
        .new_struct(name)
        .doc(&common.doc_with_range(max, min, period))
        .vis("pub")
        .derive("serde::Deserialize, PartialEq, Debug, Clone")
        .tuple_field("serde_json::Number");
    if let Some(default) = default {
        let default = generate_number_default(default);
        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(format!("Self({default})"));
    }
    generate_test_from_example_if_present(scope, name, common.example.as_ref());
}

pub fn generate_number_default(n: &Number) -> String {
    let underlying_datatype = if n.is_f64() {
        "f64"
    } else if n.is_i64() {
        "i128"
    } else {
        "u128"
    };
    format!(
        "serde_json::Number::from_{underlying_datatype}({n}).expect(\"the number is serialised from a number and is thus always valid\")"
    )
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::decoder::StyleReference;

    #[test]
    fn generate_number_empty() {
        let mut scope = Scope::new();
        generate(
            &mut scope,
            "Foo",
            &Fields::default(),
            None,
            None,
            None,
            None,
        );
        insta::assert_snapshot!(scope.to_string(), @r"
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct Foo(serde_json::Number);
        ")
    }

    #[test]
    fn generate_number_min_max_period() {
        let mut scope = Scope::new();
        generate(
            &mut scope,
            "Foo",
            &Fields::default(),
            None,
            Some(&1.into()),
            Some(&360.into()),
            Some(&360.into()),
        );
        insta::assert_snapshot!(scope.to_string(), @r"
        /// Range: 360..=1 every 360
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct Foo(serde_json::Number);
        ")
    }

    #[test]
    fn generate_number_with_default() {
        let mut scope = Scope::new();
        generate(
            &mut scope,
            "Foo",
            &Fields::default(),
            Some(&42.into()),
            None,
            None,
            None,
        );
        insta::assert_snapshot!(scope.to_string(), @r##"
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct Foo(serde_json::Number);

        impl Default for Foo {
            fn default() -> Self {
                Self(serde_json::Number::from_i128(42).expect("the number is serialised from a number and is thus always valid"))
            }
        }
        "##)
    }

    #[test]
    fn test_generate_spec() {
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
}

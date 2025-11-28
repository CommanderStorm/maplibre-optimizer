use codegen2::Scope;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::formatter::to_upper_camel_case;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields, default: &str) {
    let enu = scope
        .new_enum("AvailableProjections")
        .vis("pub")
        .doc("Available Projections")
        .derive("serde::Deserialize, PartialEq, Eq, Debug, Clone");
    enu.new_variant("Mercator")
        .annotation("#[serde(rename=\"mercator\")]")
        .doc("[Web Mercator projection](https://en.wikipedia.org/wiki/Web_Mercator_projection)");
    enu.new_variant("VerticalPerspective").annotation("#[serde(rename=\"vertical-perspective\")]").doc("[Vertical Perspective projection](https://en.wikipedia.org/wiki/General_Perspective_projection)");

    let enu = scope
        .new_enum(name)
        .vis("pub")
        .doc(&common.doc)
        .attr("serde(untagged)")
        .derive("serde::Deserialize, PartialEq, Eq, Debug, Clone");
    enu.new_variant("Globe")
        .annotation("#[serde(rename = \"globe\")]")
        .doc("Preset for the Globe projection");
    enu.new_variant("Raw")
        .doc("Preset for the Globe projection")
        .tuple("AvailableProjections");
    enu.new_variant("CameraExpression")
        .doc("Preset for the Equirectangular projection")
        .tuple("Vec<CameraExpression>");

    scope
        .new_impl(name)
        .impl_trait("Default")
        .new_fn("default")
        .ret("Self")
        .line(format!(
            "Self::Raw(AvailableProjections::{})",
            to_upper_camel_case(default)
        ));
    generate_test_from_example_if_present(scope, name, common.example.as_ref());
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::decoder::StyleReference;
    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(&mut scope, "Foo", &Fields::default(), "mercator");
        insta::assert_snapshot!(scope.to_string(), @r#"
        /// Available Projections
        #[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone)]
        pub enum AvailableProjections {
            /// [Web Mercator projection](https://en.wikipedia.org/wiki/Web_Mercator_projection)
            #[serde(rename="mercator")]
            Mercator,
            /// [Vertical Perspective projection](https://en.wikipedia.org/wiki/General_Perspective_projection)
            #[serde(rename="vertical-perspective")]
            VerticalPerspective,
        }

        #[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone)]
        #[serde(untagged)]
        pub enum Foo {
            /// Preset for the Globe projection
            #[serde(rename = "globe")]
            Globe,
            /// Preset for the Globe projection
            Raw(AvailableProjections),
            /// Preset for the Equirectangular projection
            CameraExpression(Vec<CameraExpression>),
        }

        impl Default for Foo {
            fn default() -> Self {
                Self::Raw(AvailableProjections::Mercator)
            }
        }
        "#)
    }

    #[test]
    fn test_generate_spec() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "projection": {
          "type": {
            "type": "projectionDefinition",
            "doc": "The projection definition type. Can be specified as a string, a transition state, or an expression.",
            "default": "mercator",
            "property-type": "data-constant",
            "transition": false,
            "expression": {
              "interpolated": true,
              "parameters": [
                "zoom"
              ]
            }
          }
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r#"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct Projection {
            /// The projection definition type. Can be specified as a string, a transition state, or an expression.
            #[serde(rename="type")]
            pub r#type: Option<ProjectionType>,
        }

        /// Available Projections
        #[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone)]
        pub enum AvailableProjections {
            /// [Web Mercator projection](https://en.wikipedia.org/wiki/Web_Mercator_projection)
            #[serde(rename="mercator")]
            Mercator,
            /// [Vertical Perspective projection](https://en.wikipedia.org/wiki/General_Perspective_projection)
            #[serde(rename="vertical-perspective")]
            VerticalPerspective,
        }

        /// The projection definition type. Can be specified as a string, a transition state, or an expression.
        #[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone)]
        #[serde(untagged)]
        pub enum ProjectionType {
            /// Preset for the Globe projection
            #[serde(rename = "globe")]
            Globe,
            /// Preset for the Globe projection
            Raw(AvailableProjections),
            /// Preset for the Equirectangular projection
            CameraExpression(Vec<CameraExpression>),
        }

        impl Default for ProjectionType {
            fn default() -> Self {
                Self::Raw(AvailableProjections::Mercator)
            }
        }

        #[cfg(test)]
        mod test {
            use super::*;

        }
        "#);
    }
}

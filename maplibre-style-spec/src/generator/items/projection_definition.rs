use codegen::Scope;

use crate::decoder::Fields;
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
        .tuple("Vec<CameraExpression<AvailableProjections>>");

    scope
        .new_impl(name)
        .impl_trait("Default")
        .new_fn("default")
        .ret("Self")
        .line(format!(
            "Self::Raw(AvailableProjections::{})",
            to_upper_camel_case(default)
        ));
}

#[cfg(test)]
mod tests {
    use super::*;
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
            CameraExpression(Vec<CameraExpression<AvailableProjections>>),
        }

        impl Default for Foo {
            fn default() -> Self {
                Self::Raw(AvailableProjections::Mercator)
            }
        }
        "#)
    }
}

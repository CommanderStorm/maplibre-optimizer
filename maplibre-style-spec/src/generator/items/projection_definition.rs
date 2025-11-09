use codegen::Scope;

use crate::decoder::Fields;
use crate::generator::formatter::to_upper_camel_case;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields, default: &str) {
    let enu = scope
        .new_enum(name)
        .doc(&common.doc)
        .attr("serde(untagged)")
        .derive("serde::Deserialize, PartialEq, Eq, Debug, Clone");
    enu.new_variant("Globe")
        .annotation("#[serde(rename = \"globe\")]")
        .doc("Preset for the Globe projection");
    enu.new_variant("CameraExpression")
        .doc("Preset for the Equirectangular projection")
        .tuple("Vec<CameraExpression>");

    scope
        .new_impl(name)
        .impl_trait("Default")
        .new_fn("default")
        .ret("Self")
        .line(format!("Self::{}", to_upper_camel_case(default)));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(&mut scope, "Foo", &Fields::default(), "mercator");
        insta::assert_snapshot!(scope.to_string(), @r##"
        #[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone)]
        #[serde(untagged)]
        enum Foo {
            /// Preset for the Globe projection
            #[serde(rename = "globe")]
            Globe,
            /// Preset for the Equirectangular projection
            CameraExpression(Vec<CameraExpression>),
        }

        impl Default for Foo {
            fn default() -> Self {
                Self::Mercator
            }
        }
        "##)
    }
}

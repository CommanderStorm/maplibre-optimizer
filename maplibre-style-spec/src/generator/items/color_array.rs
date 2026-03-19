use codegen2::Scope;

use crate::generator::autotest::generate_test_from_example_if_present;
use crate::mir::types::ColorArrayField;

pub fn generate(scope: &mut Scope, name: &str, field: &ColorArrayField) {
    let enu = scope
        .new_enum(name)
        .vis("pub")
        .attr("serde(untagged)")
        .doc(&field.meta.doc)
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone");
    enu.new_variant("One")
        .tuple("color::DynamicColor")
        .doc("A color");
    enu.new_variant("Multiple")
        .tuple("Vec<color::DynamicColor>")
        .doc("A set of colors");

    if let Some(default) = &field.default {
        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(format!("Self::One(color::parse_color(\"{default}\").expect(\"Invalid color specified as the default value\"))"));
    }
    generate_test_from_example_if_present(scope, name, field.meta.example.as_ref());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::types::FieldMeta;

    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(
            &mut scope,
            "Foo",
            &ColorArrayField {
                meta: FieldMeta::default(),
                default: None,
            },
        );
        insta::assert_snapshot!(scope.to_string(), @r"
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[serde(untagged)]
        pub enum Foo {
            /// A color
            One(color::DynamicColor),
            /// A set of colors
            Multiple(Vec<color::DynamicColor>),
        }
        ")
    }
}

use codegen2::Scope;

use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::fuzz;
use crate::mir::types::ColorArrayField;

pub fn generate(scope: &mut Scope, name: &str, field: &ColorArrayField) {
    let enu = scope
        .new_enum(name)
        .vis("pub")
        .attr("serde(untagged)")
        .doc(&field.meta.doc)
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY);
    enu.new_variant("One")
        .tuple_with_attrs([fuzz::ARB_DYNAMIC_COLOR], "color::DynamicColor")
        .doc("A color");
    enu.new_variant("Multiple")
        .tuple_with_attrs([fuzz::ARB_VEC_DYNAMIC_COLOR], "Vec<color::DynamicColor>")
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
        insta::assert_snapshot!(scope.to_string(), @r#"
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[serde(untagged)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub enum Foo {
            /// A color
            One(
                #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_dynamic_color))]
                 color::DynamicColor,
            ),
            /// A set of colors
            Multiple(
                #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_vec_dynamic_color))]
                 Vec<color::DynamicColor>,
            ),
        }
        "#)
    }
}

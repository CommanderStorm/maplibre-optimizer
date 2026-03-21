use codegen2::Scope;

use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::fuzz;
use crate::mir::types::PaddingField;

pub fn generate(scope: &mut Scope, name: &str, field: &PaddingField) {
    let enu = scope
        .new_enum(name)
        .vis("pub")
        .attr("serde(untagged)")
        .doc(&field.meta.doc)
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY);
    enu.new_variant("One")
        .doc("A single value applies to all four sides")
        .tuple_with_attrs(
            [fuzz::ARB_BOX_1_JSON_NUMBER],
            "Box<[serde_json::Number; 1]>",
        );
    enu.new_variant("Two")
        .doc("two values apply to `[top/bottom, left/right]`")
        .tuple_with_attrs(
            [fuzz::ARB_BOX_2_JSON_NUMBER],
            "Box<[serde_json::Number; 2]>",
        );
    enu.new_variant("Three")
        .doc("three values apply to `[top, left/right, bottom]`")
        .tuple_with_attrs(
            [fuzz::ARB_BOX_3_JSON_NUMBER],
            "Box<[serde_json::Number; 3]>",
        );
    enu.new_variant("Four")
        .doc("four values apply to `[top, right, bottom, left]`")
        .tuple_with_attrs(
            [fuzz::ARB_BOX_4_JSON_NUMBER],
            "Box<[serde_json::Number; 4]>",
        );

    let mut items = String::from("Box::new([");
    let mut needs_separator = false;
    for item in &field.default {
        if needs_separator {
            items.push_str(", ");
        }
        items.push_str(&item.to_string());
        items.push_str(".into()");
        needs_separator = true;
    }
    items.push_str("])");

    let enum_variant_name = match field.default.len() {
        1 => "One",
        2 => "Two",
        3 => "Three",
        4 => "Four",
        _ => panic!("invalid padding default length"),
    };

    scope
        .new_impl(name)
        .impl_trait("Default")
        .new_fn("default")
        .ret("Self")
        .line(format!("Self::{enum_variant_name}({items})"));
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
            &PaddingField {
                meta: FieldMeta::default(),
                default: vec![2.into()],
            },
        );
        insta::assert_snapshot!(scope.to_string(), @r#"
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[serde(untagged)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub enum Foo {
            /// A single value applies to all four sides
            One(
                #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_1_json_number))]
                 Box<[serde_json::Number; 1]>,
            ),
            /// two values apply to `[top/bottom, left/right]`
            Two(
                #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_2_json_number))]
                 Box<[serde_json::Number; 2]>,
            ),
            /// three values apply to `[top, left/right, bottom]`
            Three(
                #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_3_json_number))]
                 Box<[serde_json::Number; 3]>,
            ),
            /// four values apply to `[top, right, bottom, left]`
            Four(
                #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_4_json_number))]
                 Box<[serde_json::Number; 4]>,
            ),
        }

        impl Default for Foo {
            fn default() -> Self {
                Self::One(Box::new([2.into()]))
            }
        }
        "#)
    }
}

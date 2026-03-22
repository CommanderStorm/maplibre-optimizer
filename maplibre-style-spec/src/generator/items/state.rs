use codegen2::Scope;

use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::fuzz;
use crate::mir::types::MirStateField;

pub fn generate(scope: &mut Scope, name: &str, field: &MirStateField) {
    scope
        .new_struct(name)
        .vis("pub")
        .doc(&field.meta.doc)
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY)
        .tuple_field_with_attrs([fuzz::ARB_JSON_VALUE], "serde_json::Value");

    let default = &field.default;
    scope
        .new_impl(name)
        .impl_trait("Default")
        .new_fn("default")
        .ret("Self")
        .line(format!("Self(serde_json::json!({default}))"));
    generate_test_from_example_if_present(scope, name, field.meta.example.as_ref());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::types::MirFieldMeta;

    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(
            &mut scope,
            "Foo",
            &MirStateField {
                meta: MirFieldMeta::default(),
                default: serde_json::json!(null),
            },
        );
        insta::assert_snapshot!(scope.to_string(), @r#"
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct Foo(
            #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
            serde_json::Value,
        );

        impl Default for Foo {
            fn default() -> Self {
                Self(serde_json::json!(null))
            }
        }
        "#)
    }
}

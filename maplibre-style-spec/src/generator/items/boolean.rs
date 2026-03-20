use codegen2::Scope;

use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::fuzz;
use crate::mir::types::BooleanField;

pub fn generate(scope: &mut Scope, name: &str, field: &BooleanField) {
    // `clippy::derivable_impls`: for `Default` implementations that are always `false`, prefer
    // `#[derive(Default)]` and avoid hand-written `impl Default`.
    let derives = if field.default == Some(false) {
        "serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy, Default"
    } else {
        "serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy"
    };

    scope
        .new_struct(name)
        .vis("pub")
        .doc(&field.meta.doc)
        .derive(derives)
        .attr(fuzz::CFG_DERIVE_ARBITRARY)
        .tuple_field("bool");

    if field.default == Some(true) {
        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line("Self(true)");
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
            &BooleanField {
                meta: FieldMeta::default(),
                default: None,
            },
        );
        insta::assert_snapshot!(scope.to_string(), @r#"
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct Foo(bool);
        "#)
    }
}

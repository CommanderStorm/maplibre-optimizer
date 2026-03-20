use codegen2::Scope;

use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::fuzz;
use crate::mir::types::FormattedTextField;

pub fn generate(scope: &mut Scope, name: &str, field: &FormattedTextField) {
    scope
        .new_struct(name)
        .vis("pub")
        .doc(&field.meta.doc)
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY)
        .tuple_field("std::string::String");

    let default = &field.default;
    scope
        .new_impl(name)
        .impl_trait("Default")
        .new_fn("default")
        .ret("Self")
        .line(format!("Self(\"{default}\".to_string())"));
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
            &FormattedTextField {
                meta: FieldMeta::default(),
                tokens: false,
                default: String::new(),
            },
        );
        insta::assert_snapshot!(scope.to_string(), @r#"
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct Foo(std::string::String);

        impl Default for Foo {
            fn default() -> Self {
                Self("".to_string())
            }
        }
        "#)
    }
}

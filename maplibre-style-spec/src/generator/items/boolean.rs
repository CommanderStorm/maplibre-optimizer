use codegen2::Scope;

use crate::generator::autotest::generate_test_from_example_if_present;
use crate::mir::types::BooleanField;

pub fn generate(scope: &mut Scope, name: &str, field: &BooleanField) {
    scope
        .new_struct(name)
        .doc(&field.meta.doc)
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy")
        .tuple_field("bool");

    if let Some(default) = &field.default {
        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(format!("Self({default})"));
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
        insta::assert_snapshot!(scope.to_string(), @r"
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
        struct Foo(bool);
        ")
    }
}

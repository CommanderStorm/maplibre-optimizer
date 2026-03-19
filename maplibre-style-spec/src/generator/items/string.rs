use codegen2::Scope;

use crate::generator::autotest::generate_test_from_example_if_present;
use crate::mir::types::StringField;

pub fn generate(scope: &mut Scope, name: &str, field: &StringField) {
    scope
        .new_struct(name)
        .vis("pub")
        .doc(&field.meta.doc)
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .tuple_field("String");

    if let Some(default) = &field.default {
        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(format!("Self(\"{default}\".to_string())"));
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
            &StringField {
                meta: FieldMeta::default(),
                default: None,
            },
        );
        insta::assert_snapshot!(scope.to_string(), @r"
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        pub struct Foo(String);
        ")
    }
}

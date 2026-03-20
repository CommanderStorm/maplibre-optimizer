use codegen2::Scope;

use crate::generator::autotest::generate_test_from_example_if_present;
use crate::mir::types::ResolvedImageField;

pub fn generate(scope: &mut Scope, name: &str, field: &ResolvedImageField) {
    scope
        .new_struct(name)
        .doc(&field.meta.doc)
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone")
        .tuple_field("std::string::String");
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
            &ResolvedImageField {
                meta: FieldMeta::default(),
                tokens: None,
            },
        );
        insta::assert_snapshot!(scope.to_string(), @r"
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone)]
        struct Foo(std::string::String);
        ")
    }
}

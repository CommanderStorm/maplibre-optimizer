use codegen2::Scope;

use crate::generator::autotest::generate_test_from_example_if_present;
use crate::mir::types::FieldMeta;

pub fn generate(scope: &mut Scope, name: &str, meta: &FieldMeta) {
    scope
        .new_struct(name)
        .vis("pub")
        .doc(&meta.doc)
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .tuple_field("std::string::String");
    generate_test_from_example_if_present(scope, name, meta.example.as_ref());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(&mut scope, "Foo", &FieldMeta::default());
        insta::assert_snapshot!(scope.to_string(), @r"
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        pub struct Foo(String);
        ")
    }
}

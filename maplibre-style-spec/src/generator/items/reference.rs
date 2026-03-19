use codegen2::Scope;

use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::formatter::to_upper_camel_case;
use crate::mir::types::ReferenceField;

pub fn generate(scope: &mut Scope, name: &str, field: &ReferenceField) {
    scope
        .new_struct(name)
        .doc(&field.meta.doc)
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .tuple_field(to_upper_camel_case(&field.target));

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
            &ReferenceField {
                meta: FieldMeta::default(),
                target: "sky".to_string(),
            },
        );
        insta::assert_snapshot!(scope.to_string(), @r"
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        struct Foo(Sky);
        ")
    }
}

use codegen2::Scope;

use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::formatter::to_upper_camel_case;
use crate::mir::types::ReferenceField;

pub fn generate(scope: &mut Scope, name: &str, field: &ReferenceField) {
    let rust_inner = match field.target.as_str() {
        // `$root.sources` uses `type: "sources"` but sources are modeled as `Source` plus a map.
        "sources" => "std::collections::BTreeMap<std::string::String, Source>".to_string(),
        // Named typedef `expression` is the spec's "expression array" shape; the syntax enum is `Any`.
        "expression" => "Any".to_string(),
        other => to_upper_camel_case(other),
    };
    scope
        .new_struct(name)
        .vis("pub")
        .doc(&field.meta.doc)
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .tuple_field(rust_inner);

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
        pub struct Foo(Sky);
        ")
    }
}

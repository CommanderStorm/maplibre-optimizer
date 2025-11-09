use codegen::Scope;
use serde_json::Value;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields, default: &Value) {
    scope
        .new_struct(name)
        .doc(&common.doc)
        .derive("serde::Deserialize, PartialEq, Debug, Clone")
        .tuple_field("serde_json::Value");

    scope
        .new_impl(name)
        .impl_trait("Default")
        .new_fn("default")
        .ret("Self")
        .line(format!("serde_json::json!({default})"));
    generate_test_from_example_if_present(scope, name, common);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(
            &mut scope,
            "Foo",
            &Fields::default(),
            &Value::String("hello_world".to_string()),
        );
        insta::assert_snapshot!(scope.to_string(), @r#"
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        struct Foo(serde_json::Value);

        impl Default for Foo {
            fn default() -> Self {
                serde_json::json!("hello_world")
            }
        }
        "#)
    }
}

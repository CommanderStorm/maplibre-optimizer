use codegen::Scope;
use serde_json::Value;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields, default: Option<&Value>) {
    scope
        .new_struct(name)
        .doc(&common.doc)
        .derive("serde::Deserialize, PartialEq, Debug, Clone")
        .tuple_field("color::DynamicColor");

    if let Some(default) = default {
        let fun = scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self");
        if let Value::String(default) = default {
            fun.line(format!("Self(color::parse_color(\"{default}\").expect(\"Invalid color specified as the default value\"))"));
        } else {
            fun.line("todo!(\"needs expressions to be expressed in the style spec\")");
        }
    }
    generate_test_from_example_if_present(scope, name, common);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(&mut scope, "Foo", &Fields::default(), None);
        insta::assert_snapshot!(scope.to_string(), @r"
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        struct Foo(color::DynamicColor);
        ")
    }
}

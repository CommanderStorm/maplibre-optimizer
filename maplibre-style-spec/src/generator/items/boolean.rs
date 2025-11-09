use codegen::Scope;

use crate::decoder::Fields;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields, default: Option<&bool>) {
    scope
        .new_struct(name)
        .doc(&common.doc)
        .derive("serde::Deserialize, PartialEq, Debug, Clone, Copy")
        .tuple_field("bool");

    if let Some(default) = default {
        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(format!("Self({default})"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(&mut scope, "Foo", &Fields::default(), None);
        insta::assert_snapshot!(scope.to_string(), @r##"
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        #[deprecated = "not_implemented"]
        struct Foo(serde_json::Value);
        "##)
    }
}

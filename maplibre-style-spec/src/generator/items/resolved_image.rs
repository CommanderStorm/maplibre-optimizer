use codegen::Scope;

use crate::decoder::Fields;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields, tokens: Option<bool>) {
    scope
        .new_struct(name)
        .doc(&common.doc)
        .attr("deprecated = \"not_implemented\"")
        .derive("serde::Deserialize, PartialEq, Debug, Clone")
        .tuple_field("serde_json::Value");
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

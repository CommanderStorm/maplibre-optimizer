use codegen::Scope;

use crate::decoder::Fields;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields, default: &str, tokens: bool) {
    scope
        .new_struct(name)
        .doc(&common.doc)
        .attr("deprecated = \"not_implemented\"")
        .tuple_field("serde_json::Value");

    scope
        .new_impl(&name)
        .impl_trait("Default")
        .new_fn("default")
        .line(default);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(&mut scope, "Foo", &Fields::default(), "some", false);
        insta::assert_snapshot!(scope.to_string(), @"")
    }
}

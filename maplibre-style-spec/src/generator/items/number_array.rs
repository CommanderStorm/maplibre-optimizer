use codegen::Scope;
use serde_json::{Number};

use crate::decoder::Fields;

pub fn generate(
    scope: &mut Scope,
    name: &str,
    common: &Fields,
    default: Option<&Number>,
    min: Option<&Number>,
    max: Option<&Number>,
) {
    scope
        .new_struct(name)
        .doc(&common.doc)
        .attr("deprecated = \"not_implemented\"")
        .tuple_field("serde_json::Value");

    if let Some(default) = default {
        scope
            .new_impl(&name)
            .impl_trait("Default")
            .new_fn("default")
            .line(default);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(&mut scope, "Foo", &Fields::default(), None, None, None);
        insta::assert_snapshot!(scope.to_string(), @"")
    }
}

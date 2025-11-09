use codegen::Scope;
use serde_json::Number;

use crate::decoder::Fields;
use crate::generator::items::number::generate_number_default;

pub fn generate(
    scope: &mut Scope,
    name: &str,
    common: &Fields,
    default: Option<&Number>,
    min: Option<&Number>,
    max: Option<&Number>,
) {
    let enu = scope
        .new_enum(name)
        .vis("pub")
        .attr("serde(untagged)")
        .doc(&common.doc_with_range(max, min, None))
        .derive("serde::Deserialize, PartialEq, Debug, Clone");
    enu.new_variant("One").tuple("serde_json::Number");
    enu.new_variant("Many").tuple("Vec<serde_json::Number>");

    if let Some(default) = default {
        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(format!("Self::One({})", generate_number_default(default)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(&mut scope, "Foo", &Fields::default(), None, None, None);
        insta::assert_snapshot!(scope.to_string(), @r"
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        #[serde(untagged)]
        pub enum Foo {
            One(serde_json::Number),
            Many(Vec<serde_json::Number>),
        }
        ")
    }
}

use codegen2::Scope;

use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::items::number::generate_number_default;
use crate::mir::types::NumberArrayField;

pub fn generate(scope: &mut Scope, name: &str, field: &NumberArrayField) {
    let enu = scope
        .new_enum(name)
        .vis("pub")
        .attr("serde(untagged)")
        .doc(&field.meta.doc)
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone");
    enu.new_variant("One").tuple("serde_json::Number");
    enu.new_variant("Many").tuple("Vec<serde_json::Number>");

    if let Some(default) = &field.default {
        let default_expr = generate_number_default(default);
        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(format!("Self::One({default_expr})"));
    }
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
            &NumberArrayField {
                meta: FieldMeta::default(),
                default: None,
                min: None,
                max: None,
            },
        );
        insta::assert_snapshot!(scope.to_string(), @r"
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[serde(untagged)]
        pub enum Foo {
            One(serde_json::Number),
            Many(Vec<serde_json::Number>),
        }
        ")
    }
}

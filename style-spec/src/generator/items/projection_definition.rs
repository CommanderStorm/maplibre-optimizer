use codegen2::Scope;

use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::fuzz;
use crate::generator::untagged::{self, Variant};
use crate::mir::types::MirProjectionDefinitionField;

pub fn generate(scope: &mut Scope, name: &str, field: &MirProjectionDefinitionField) {
    if field.meta.expression.is_some() {
        let enu = scope
            .new_enum(name)
            .vis("pub")
            .doc(&field.meta.doc)
            .derive("PartialEq, Debug, Clone")
            .attr(fuzz::CFG_DERIVE_ARBITRARY);
        enu.new_variant("Expr")
            .tuple("NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection");
        enu.new_variant("Literal").tuple("std::string::String");
        untagged::emit_untagged_serde(
            scope,
            name,
            &[
                Variant {
                    name: "Expr".into(),
                    inner_type: "NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection".into(),
                    is_boxed: false,
                    is_unit: false,
                    skip_when: None,
                },
                Variant {
                    name: "Literal".into(),
                    inner_type: "std::string::String".into(),
                    is_boxed: false,
                    is_unit: false,
                    skip_when: None,
                },
            ],
        );
    } else {
        scope
            .new_struct(name)
            .vis("pub")
            .doc(&field.meta.doc)
            .derive("serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone")
            .attr(fuzz::CFG_DERIVE_ARBITRARY)
            .tuple_field("std::string::String");
    }

    let default = &field.default;
    let default_line = if field.meta.expression.is_some() {
        format!("Self::Literal(\"{default}\".to_string())")
    } else {
        format!("Self(\"{default}\".to_string())")
    };
    scope
        .new_impl(name)
        .impl_trait("Default")
        .new_fn("default")
        .ret("Self")
        .line(default_line);
    generate_test_from_example_if_present(scope, name, field.meta.example.as_ref());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::types::MirFieldMeta;

    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(
            &mut scope,
            "Foo",
            &MirProjectionDefinitionField {
                meta: MirFieldMeta::default(),
                default: "mercator".to_string(),
            },
        );
        insta::assert_snapshot!(scope.to_string(), @r#"
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct Foo(std::string::String);

        impl Default for Foo {
            fn default() -> Self {
                Self("mercator".to_string())
            }
        }
        "#)
    }
}

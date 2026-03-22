use codegen2::Scope;

use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::fuzz;
use crate::generator::untagged::{self, Variant};
use crate::mir::types::MirBooleanField;

pub fn generate(scope: &mut Scope, name: &str, field: &MirBooleanField) {
    if field.meta.expression.is_some() {
        let expr_name = format!("{name}Expression");
        let expr = scope
            .new_enum(&expr_name)
            .doc("Nested expression: [`Boolean`] operators.")
            .vis("pub")
            .derive("PartialEq, Debug, Clone")
            .attr(fuzz::CFG_DERIVE_ARBITRARY);
        expr.new_variant("Boolean").tuple("Boolean");
        untagged::emit_untagged_serde(
            scope,
            &expr_name,
            &[Variant {
                name: "Boolean".into(),
                inner_type: "Boolean".into(),
                is_boxed: false,
                is_unit: false,
                skip_when: None,
            }],
        );

        let enu = scope
            .new_enum(name)
            .doc(&field.meta.doc)
            .vis("pub")
            .derive("PartialEq, Debug, Clone")
            .attr(fuzz::CFG_DERIVE_ARBITRARY);
        enu.new_variant("Expr").tuple(format!("Box<{expr_name}>"));
        enu.new_variant("Literal").tuple("bool");
        untagged::emit_untagged_serde(
            scope,
            name,
            &[
                Variant {
                    name: "Expr".into(),
                    inner_type: expr_name.clone(),
                    is_boxed: true,
                    is_unit: false,
                    skip_when: None,
                },
                Variant {
                    name: "Literal".into(),
                    inner_type: "bool".into(),
                    is_boxed: false,
                    is_unit: false,
                    skip_when: None,
                },
            ],
        );
    } else {
        // `clippy::derivable_impls`: for `Default` implementations that are always `false`, prefer
        // `#[derive(Default)]` and avoid hand-written `impl Default`.
        let derives = if field.default == Some(false) {
            "serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy, Default"
        } else {
            "serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy"
        };

        scope
            .new_struct(name)
            .vis("pub")
            .doc(&field.meta.doc)
            .derive(derives)
            .attr(fuzz::CFG_DERIVE_ARBITRARY)
            .tuple_field("bool");
    }

    if let Some(true) = field.default {
        let default_body = if field.meta.expression.is_some() {
            "Self::Literal(true)"
        } else {
            "Self(true)"
        };
        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(default_body);
    } else if field.default == Some(false) && field.meta.expression.is_some() {
        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line("Self::Literal(false)");
    }
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
            &MirBooleanField {
                meta: MirFieldMeta::default(),
                default: None,
            },
        );
        insta::assert_snapshot!(scope.to_string(), @r#"
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct Foo(bool);
        "#)
    }
}

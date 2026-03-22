use codegen2::Scope;
use serde_json::Value;

use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::fuzz;
use crate::generator::untagged::{self, Variant};
use crate::mir::types::ColorField;

pub fn generate(scope: &mut Scope, name: &str, field: &ColorField) {
    if field.meta.expression.is_some() {
        let expr_name = format!("{name}Expression");
        let expr = scope
            .new_enum(&expr_name)
            .doc("Nested expression: ramp (`interpolate-hcl`, …) or [`Color`] operators.")
            .vis("pub")
            .derive("PartialEq, Debug, Clone")
            .attr(fuzz::CFG_DERIVE_ARBITRARY);
        expr.new_variant("Color").tuple("Color");
        expr.new_variant("Ramp").tuple("ColorOrArrayOfColor");
        untagged::emit_untagged_serde(
            scope,
            &expr_name,
            &[
                Variant {
                    name: "Color".into(),
                    inner_type: "Color".into(),
                    is_boxed: false,
                    is_unit: false,
                    skip_when: None,
                },
                Variant {
                    name: "Ramp".into(),
                    inner_type: "ColorOrArrayOfColor".into(),
                    is_boxed: false,
                    is_unit: false,
                    skip_when: None,
                },
            ],
        );

        let enu = scope
            .new_enum(name)
            .vis("pub")
            .doc(&field.meta.doc)
            .derive("PartialEq, Debug, Clone")
            .attr(fuzz::CFG_DERIVE_ARBITRARY);
        enu.new_variant("Expr").tuple(format!("Box<{expr_name}>"));
        // CSS / JSON color literals (`\"#fff\"`, arrays, …) are not always accepted by
        // `color::DynamicColor`'s serde impl — accept raw JSON here; validation can tighten.
        enu.new_variant("Literal")
            .tuple_with_attrs([fuzz::ARB_JSON_VALUE], "serde_json::Value");
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
                    inner_type: "serde_json::Value".into(),
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
            .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
            .attr(fuzz::CFG_DERIVE_ARBITRARY)
            .tuple_field_with_attrs([fuzz::ARB_DYNAMIC_COLOR], "color::DynamicColor");
    }

    if let Some(default) = &field.default {
        let fun = scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self");
        if field.meta.expression.is_some() {
            fun.line(format!("Self::Literal(serde_json::json!({default}))"));
        } else if let Value::String(default) = default {
            fun.line(format!("Self(color::parse_color(\"{default}\").expect(\"Invalid color specified as the default value\"))"));
        } else {
            fun.line(format!("let default = serde_json::json!({default});"));
            fun.line("let default = serde_json::from_value(default).expect(\"Invalid color specified as the default value\");");
            fun.line("Self(default)");
        }
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
            &ColorField {
                meta: FieldMeta::default(),
                default: None,
            },
        );
        insta::assert_snapshot!(scope.to_string(), @r#"
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct Foo(
            #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_dynamic_color))]
            color::DynamicColor,
        );
        "#)
    }
}

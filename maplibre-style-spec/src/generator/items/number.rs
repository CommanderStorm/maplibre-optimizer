use codegen2::Scope;

use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::fuzz;
use crate::mir::types::NumberField;

pub fn generate(scope: &mut Scope, name: &str, field: &NumberField) {
    if field.meta.expression.is_some() {
        // `interpolate` / `step` for numeric properties use the combined ramp enum, not [`Number`].
        let expr_name = format!("{name}Expression");
        let expr = scope
            .new_enum(&expr_name)
            .doc("Nested expression: ramp (`interpolate` / …) or regular [`Number`] operators.")
            .vis("pub")
            .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
            .attr(fuzz::CFG_DERIVE_ARBITRARY)
            .attr("serde(untagged)");
        // Try [`Number`] first so `["+", …]` / `["*", …]` decode without trying the ramp enum
        // (which only accepts `interpolate`).
        expr.new_variant("Number").tuple("Number");
        expr.new_variant("Ramp")
            .tuple("NumberOrArrayOfNumberOrColorOrArrayOfColorOrProjection");

        let enu = scope
            .new_enum(name)
            .doc(&field.meta.doc)
            .vis("pub")
            .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
            .attr(fuzz::CFG_DERIVE_ARBITRARY)
            .attr("serde(untagged)");
        enu.new_variant("Expr").tuple(&expr_name);
        enu.new_variant("Literal")
            .tuple_with_attrs([fuzz::ARB_JSON_NUMBER], "serde_json::Number");
    } else {
        scope
            .new_struct(name)
            .doc(&field.meta.doc)
            .vis("pub")
            .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
            .attr(fuzz::CFG_DERIVE_ARBITRARY)
            .tuple_field_with_attrs([fuzz::ARB_JSON_NUMBER], "serde_json::Number");
    }
    if let Some(default) = &field.default {
        let default_expr = generate_number_default(default);
        let default_body = if field.meta.expression.is_some() {
            format!("Self::Literal({default_expr})") // Literal arm remains the numeric default
        } else {
            format!("Self({default_expr})")
        };
        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(default_body);
    }
    generate_test_from_example_if_present(scope, name, field.meta.example.as_ref());
}

pub fn generate_number_default(n: &serde_json::Number) -> String {
    let underlying_datatype = if n.is_f64() {
        "f64"
    } else if n.is_i64() {
        "i128"
    } else {
        "u128"
    };
    format!(
        "serde_json::Number::from_{underlying_datatype}({n}).expect(\"the number is serialised from a number and is thus always valid\")"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::types::FieldMeta;

    #[test]
    fn generate_number_empty() {
        let mut scope = Scope::new();
        generate(
            &mut scope,
            "Foo",
            &NumberField {
                meta: FieldMeta::default(),
                default: None,
                min: None,
                max: None,
                period: None,
            },
        );
        insta::assert_snapshot!(scope.to_string(), @r#"
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct Foo(
            #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
            serde_json::Number,
        );
        "#)
    }

    #[test]
    fn generate_number_min_max_period() {
        use crate::mir::lower::doc_with_range;
        let mut scope = Scope::new();
        let doc = doc_with_range("", Some(1.0), Some(360.0), Some(360.0));
        generate(
            &mut scope,
            "Foo",
            &NumberField {
                meta: FieldMeta {
                    doc,
                    ..FieldMeta::default()
                },
                default: None,
                min: Some(360.0),
                max: Some(1.0),
                period: Some(360.0),
            },
        );
        insta::assert_snapshot!(scope.to_string(), @r#"
        /// Range: 360..=1 every 360
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct Foo(
            #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
            serde_json::Number,
        );
        "#)
    }

    #[test]
    fn generate_number_with_default() {
        let mut scope = Scope::new();
        generate(
            &mut scope,
            "Foo",
            &NumberField {
                meta: FieldMeta::default(),
                default: Some(42.into()),
                min: None,
                max: None,
                period: None,
            },
        );
        insta::assert_snapshot!(scope.to_string(), @r#"
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct Foo(
            #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
            serde_json::Number,
        );

        impl Default for Foo {
            fn default() -> Self {
                Self(serde_json::Number::from_i128(42).expect("the number is serialised from a number and is thus always valid"))
            }
        }
        "#)
    }
}

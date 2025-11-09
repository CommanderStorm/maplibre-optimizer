use codegen::Scope;
use serde_json::Number;

use crate::decoder::Fields;

pub fn generate(
    scope: &mut Scope,
    name: &str,
    common: &Fields,
    default: Option<&Number>,
    max: Option<&Number>,
    min: Option<&Number>,
    period: Option<&Number>,
) {
    scope
        .new_struct(name)
        .doc(&common.doc_with_range(max, min, period))
        .vis("pub")
        .derive("serde::Deserialize, PartialEq, Debug, Clone")
        .tuple_field("serde_json::Number");
    if let Some(default) = default {
        let underlying_datatype = if default.is_f64() {
            "f64"
        } else if default.is_i64() {
            "i128"
        } else {
            "u128"
        };
        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(format!(
                "Self(serde_json::Number::from_{underlying_datatype}({default}).expect(\"the number is serialised from a number and is thus always valid\"))"
            ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generate_number_empty() {
        let mut scope = Scope::new();
        generate(
            &mut scope,
            "Foo",
            &Fields::default(),
            None,
            None,
            None,
            None,
        );
        insta::assert_snapshot!(scope.to_string(), @r"
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct Foo(serde_json::Number);
        ")
    }
    #[test]
    fn generate_number_min_max_period() {
        let mut scope = Scope::new();
        generate(
            &mut scope,
            "Foo",
            &Fields::default(),
            None,
            Some(&1.into()),
            Some(&360.into()),
            Some(&360.into()),
        );
        insta::assert_snapshot!(scope.to_string(), @r"
        ///
        ///
        /// Range: 360..=1 every 360
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct Foo(serde_json::Number);
        ")
    }
    #[test]
    fn generate_number_with_default() {
        let mut scope = Scope::new();
        generate(
            &mut scope,
            "Foo",
            &Fields::default(),
            Some(&42.into()),
            None,
            None,
            None,
        );
        insta::assert_snapshot!(scope.to_string(), @r"
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct Foo(serde_json::Number);

        impl Default for Foo {
            fn default() -> Self {
                Self(serde_json::Number::from_i128(42))
            }
        }
        ")
    }
}

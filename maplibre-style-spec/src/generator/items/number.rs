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
    let mut doc = common.doc.clone();
    if max.is_some() || min.is_some() || period.is_some() {
        doc.push_str("\n\n# Range\n");
        if let Some(max) = max {
            doc.push_str(&format!("- Maximum: {}\n", max))
        }
        if let Some(min) = min {
            doc.push_str(&format!("- Minimum: {}\n", min))
        }
        if let Some(period) = period {
            doc.push_str(&format!("- Period: {}\n", period))
        }
    }

    scope
        .new_struct(&name)
        .doc(&doc)
        .vis("pub")
        .derive("serde::Deserialize, PartialEq, Debug, Clone")
        .tuple_field("serde_json::Number");
    if let Some(default) = default {
        scope
            .new_impl(&name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(format!("Self({default})"));
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
        /// # Range
        /// - Maximum: 1
        /// - Minimum: 360
        /// - Period: 360
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
            fn default() {
                42
            }
        }
        ")
    }
}

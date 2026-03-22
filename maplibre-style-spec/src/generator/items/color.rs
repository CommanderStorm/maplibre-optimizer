use codegen2::Scope;
use serde_json::Value;

use super::escape_doc_for_macro;
use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::fuzz;
use crate::mir::types::MirColorField;

pub fn generate(scope: &mut Scope, name: &str, field: &MirColorField) {
    if field.meta.expression.is_some() {
        let doc = escape_doc_for_macro(&field.meta.doc);
        let mut args = format!("{name}, doc = \"{doc}\"");
        if let Some(default) = &field.default {
            args.push_str(&format!(", default = serde_json::json!({default})"));
        }
        scope.raw(format!("color_prop!({args});"));
    } else {
        scope
            .new_struct(name)
            .vis("pub")
            .doc(&field.meta.doc)
            .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
            .attr(fuzz::CFG_DERIVE_ARBITRARY)
            .tuple_field_with_attrs([fuzz::ARB_DYNAMIC_COLOR], "color::DynamicColor");

        if let Some(default) = &field.default {
            let fun = scope
                .new_impl(name)
                .impl_trait("Default")
                .new_fn("default")
                .ret("Self");
            if let Value::String(default) = default {
                fun.line(format!("Self(color::parse_color(\"{default}\").expect(\"Invalid color specified as the default value\"))"));
            } else {
                fun.line(format!("let default = serde_json::json!({default});"));
                fun.line("let default = serde_json::from_value(default).expect(\"Invalid color specified as the default value\");");
                fun.line("Self(default)");
            }
        }
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
            &MirColorField {
                meta: MirFieldMeta::default(),
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

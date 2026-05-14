use codegen2::Scope;

use super::escape_doc_for_macro;
use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::fuzz;
use crate::mir::types::MirStringField;

pub fn generate(scope: &mut Scope, name: &str, field: &MirStringField) {
    if field.meta.expression.is_some() {
        let doc = escape_doc_for_macro(&field.meta.doc);
        let mut args = format!("{name}, doc = \"{doc}\"");
        if let Some(default) = &field.default {
            let escaped = default.replace('\\', "\\\\").replace('"', "\\\"");
            args.push_str(&format!(", default = \"{escaped}\".to_string()"));
        }
        scope.raw(format!("string_prop!({args});"));
    } else {
        scope
            .new_struct(name)
            .vis("pub")
            .doc(&field.meta.doc)
            .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
            .attr(fuzz::CFG_DERIVE_ARBITRARY)
            .tuple_field("std::string::String");

        if let Some(default) = &field.default {
            scope
                .new_impl(name)
                .impl_trait("Default")
                .new_fn("default")
                .ret("Self")
                .line(format!("Self(\"{default}\".to_string())"));
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
            &MirStringField {
                meta: MirFieldMeta::default(),
                default: None,
            },
        );
        insta::assert_snapshot!(scope.to_string(), @r#"
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct Foo(std::string::String);
        "#)
    }
}

use codegen2::Scope;

use super::escape_doc_for_macro;
use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::fuzz;
use crate::mir::types::MirResolvedImageField;

pub fn generate(scope: &mut Scope, name: &str, field: &MirResolvedImageField) {
    if field.meta.expression.is_some() {
        let doc = escape_doc_for_macro(&field.meta.doc);
        let args = format!("{name}, doc = \"{doc}\"");
        scope.raw(format!("string_prop!({args});"));
    } else {
        scope
            .new_struct(name)
            .vis("pub")
            .doc(&field.meta.doc)
            .derive("serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone")
            .attr(fuzz::CFG_DERIVE_ARBITRARY)
            .tuple_field("std::string::String");
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
            &MirResolvedImageField {
                meta: MirFieldMeta::default(),
                tokens: None,
            },
        );
        insta::assert_snapshot!(scope.to_string(), @r#"
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct Foo(std::string::String);
        "#)
    }
}
